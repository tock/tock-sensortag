//! UART driver, cc26xx family
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::hil::gpio::Pin;
use kernel::hil::uart;
use core::cell::Cell;
use core::cmp::min;
use kernel;
use prcm;
use cc26xx::gpio;
use ioc;
use udma;
use power::PM;
use chip;
use peripheral_manager;

pub const UART_BASE: usize = 0x4000_1000;
pub const MCU_CLOCK: u32 = 48_000_000;

const UART_MAX_BUFFER_SIZE: u32 = 0x3ff;

const TX_DMA: udma::DMAPeripheral = udma::DMAPeripheral::UART0_TX;
const RX_DMA: udma::DMAPeripheral = udma::DMAPeripheral::UART0_RX;


#[repr(C)]
struct Registers {
    dr: ReadWrite<u32, Data::Register>,
    rsr_ecr: ReadWrite<u32, Errors::Register>,
    _reserved0: [u8; 0x10],
    fr: ReadOnly<u32, Flags::Register>,
    _reserved1: [u8; 0x8],
    ibrd: ReadWrite<u32, IntDivisor::Register>,
    fbrd: ReadWrite<u32, FracDivisor::Register>,
    lcrh: ReadWrite<u32, LineControl::Register>,
    ctl: ReadWrite<u32, Control::Register>,
    ifls: ReadWrite<u32>,
    imsc: ReadWrite<u32, Interrupts::Register>,
    ris: ReadOnly<u32, Interrupts::Register>,
    mis: ReadOnly<u32, Interrupts::Register>,
    icr: WriteOnly<u32, Interrupts::Register>,
    dmactl: ReadWrite<u32, DMACtl::Register>,
}

register_bitfields![
    u32,
    Data [
        DATA OFFSET(0) NUMBITS(8)
    ],
    Errors[
        ALL_ERRORS     OFFSET(0) NUMBITS(4),
        FRAMING_ERROR  OFFSET(0) NUMBITS(1),
        PARITY_ERROR   OFFSET(1) NUMBITS(1),
        BREAK_ERROR    OFFSET(2) NUMBITS(1),
        OVERFLOW_ERROR OFFSET(3) NUMBITS(1)
    ],
    Control [
        UART_ENABLE OFFSET(0) NUMBITS(1) [],
        TX_ENABLE OFFSET(8) NUMBITS(1) [],
        RX_ENABLE OFFSET(9) NUMBITS(1) []
    ],
    LineControl [
        FIFO_ENABLE OFFSET(4) NUMBITS(1) [],
        WORD_LENGTH OFFSET(5) NUMBITS(2) [
            Len5 = 0x0,
            Len6 = 0x1,
            Len7 = 0x2,
            Len8 = 0x3
        ]
    ],
    IntDivisor [
        DIVISOR OFFSET(0) NUMBITS(16) []
    ],
    FracDivisor [
        DIVISOR OFFSET(0) NUMBITS(6) []
    ],
    Flags [
        RX_FIFO_EMPTY OFFSET(4) NUMBITS(1) [],
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        UART_BUSY OFFSET(3) NUMBITS(1) []
    ],
    Interrupts [
        ALL_INTERRUPTS OFFSET(0) NUMBITS(12) [],
        CTSM OFFSET(1)  NUMBITS(1)[],
          RX OFFSET(4)  NUMBITS(1)[],
          TX OFFSET(5)  NUMBITS(1)[],
          RT OFFSET(6)  NUMBITS(1)[],
          FE OFFSET(7)  NUMBITS(1)[],
          PE OFFSET(8)  NUMBITS(1)[],
          BE OFFSET(9)  NUMBITS(1)[],
          OE OFFSET(10) NUMBITS(1)[]
    ],

    DMACtl [ 
        DMAONERR OFFSET (2) NUMBITS(1) [],
        TXDMAE OFFSET(1) NUMBITS(1) [],
        RXDMAE OFFSET(0) NUMBITS(1) []
    ]
];

pub struct UART {
    regs: *const Registers,

    client: Cell<Option<&'static uart::Client>>,

    rx_buffer: kernel::common::take_cell::TakeCell<'static, [u8]>,
    rx_remaining_bytes: Cell<usize>,

    tx_buffer: kernel::common::take_cell::TakeCell<'static, [u8]>,
    tx_remaining_bytes: Cell<usize>,

    tx_pin: Cell<Option<u8>>,
    rx_pin: Cell<Option<u8>>,

    params: Cell<Option<kernel::hil::uart::UARTParams>>,
}

pub static mut UART0: UART = UART::new();

impl UART {
    pub const fn new() -> UART {
        UART {
            regs: UART_BASE as *mut Registers,

            client: Cell::new(None),
            
            rx_buffer: kernel::common::take_cell::TakeCell::empty(),
            tx_buffer: kernel::common::take_cell::TakeCell::empty(),
            
            rx_remaining_bytes: Cell::new(0),
            tx_remaining_bytes: Cell::new(0),

            rx_pin: Cell::new(None),
            tx_pin: Cell::new(None),

            params: Cell::new(None),
        }
    }

    pub unsafe fn configure_dma(&self) {
        udma::UDMA.initialize_channel(
            TX_DMA,
            udma::DMAWidth::Width8Bit,
            udma::DMATransferType::DataTx,
            UART_BASE as u32
        );

        udma::UDMA.initialize_channel(
            RX_DMA,
            udma::DMAWidth::Width8Bit,
            udma::DMATransferType::DataRx,
            UART_BASE as u32
        );   
    }

    pub fn set_pins(&self, tx_pin: u8, rx_pin: u8) {
        self.tx_pin.set(Some(tx_pin));
        self.rx_pin.set(Some(rx_pin));
    }

    pub fn configure(&self) {
        let tx_pin = match self.tx_pin.get() {
            Some(pin) => pin,
            None => panic!("Tx pin not configured for UART"),
        };

        let rx_pin = match self.rx_pin.get() {
            Some(pin) => pin,
            None => panic!("Rx pin not configured for UART"),
        };

        let params = self.params.get().expect("No params supplied to uart.");

        unsafe {
            /*
             * Make sure the TX pin is output/high before assigning it to UART control
             * to avoid falling edge glitches
             */
            gpio::PORT[tx_pin as usize].make_output();
            gpio::PORT[tx_pin as usize].set();

            // Map UART signals to IO pin
            ioc::IOCFG[tx_pin as usize].enable_uart_tx();
            ioc::IOCFG[rx_pin as usize].enable_uart_rx();
        }

        // Disable the UART before configuring
        self.disable();

        self.set_baud_rate(params.baud_rate);

        // Set word length
        let regs = unsafe { &*self.regs };
        regs.lcrh.write(LineControl::WORD_LENGTH::Len8);

        self.fifo_enable();

        unsafe{self.configure_dma()};

        // Enable UART, RX and TX
        regs.ctl
            .write(Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET);
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;
        // Set the baud rate
        let regs = unsafe { &*self.regs };
        regs.ibrd.write(IntDivisor::DIVISOR.val(div / 64));
        regs.fbrd.write(FracDivisor::DIVISOR.val(div % 64));
    }

    fn fifo_enable(&self) {
        let regs = unsafe { &*self.regs };
        regs.lcrh.modify(LineControl::FIFO_ENABLE::SET);
    }

    fn fifo_disable(&self) {
        let regs = unsafe { &*self.regs };
        regs.lcrh.modify(LineControl::FIFO_ENABLE::CLEAR);
    }

    fn busy(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.fr.is_set(Flags::UART_BUSY)
    }

    pub fn disable(&self) {
        self.fifo_disable();
        let regs = unsafe { &*self.regs };
        regs.ctl.modify(
            Control::UART_ENABLE::CLEAR + Control::TX_ENABLE::CLEAR + Control::RX_ENABLE::CLEAR,
        );
    }

    pub fn disable_interrupts(&self) {
        // Disable all UART interrupts
        let regs = unsafe { &*self.regs };
        regs.imsc.modify(Interrupts::ALL_INTERRUPTS::CLEAR);
        // Clear all UART interrupts
        regs.icr.write(Interrupts::ALL_INTERRUPTS::SET);
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        // Clear all UART interrupts
        regs.icr.write(Interrupts::ALL_INTERRUPTS::SET);
        // Enable all UART interrupts        
        regs.imsc.write(Interrupts::CTSM::SET + 
                        Interrupts::RX::SET   + 
                        Interrupts::TX::SET   + 
                        Interrupts::RT::SET   + 
                        Interrupts::FE::SET   + 
                        Interrupts::PE::SET   + 
                        Interrupts::BE::SET   + 
                        Interrupts::OE::SET);
    }


    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        // Get status bits
        #[allow(unused)]
        let flags: u32 = regs.fr.get();

        // Clear interrupts
        regs.icr.write(Interrupts::ALL_INTERRUPTS::SET);
        regs.rsr_ecr.write(Errors::ALL_ERRORS::SET);

        self.disable_interrupts();

        let tx_transfer_complete = unsafe{udma::UDMA.transfer_complete(TX_DMA)};
        
        if tx_transfer_complete {
            regs.dmactl.modify(DMACtl::TXDMAE::CLEAR);
            //clear the transfer flag
            unsafe{udma::UDMA.clear_transfer(TX_DMA)};

            self.client.get().map(|client| {
                self.tx_buffer.take().map(|tx_buffer| {
                    client.transmit_complete(
                        tx_buffer,
                        kernel::hil::uart::Error::CommandComplete,
                    );
                });
            });
        }

        let rx_transfer_complete = unsafe{udma::UDMA.transfer_complete(RX_DMA)};

        if rx_transfer_complete {
            //clear the receive flag
            unsafe{udma::UDMA.clear_transfer(RX_DMA);};
            regs.dmactl.modify(DMACtl::RXDMAE::CLEAR); 

            self.client.get().map(|client| {
                self.rx_buffer.take().map(|rx_buffer| {
                    client.receive_complete(
                        rx_buffer,
                        self.rx_remaining_bytes.get(),
                        kernel::hil::uart::Error::CommandComplete,
                    );
                });
            });
            self.rx_remaining_bytes.set(0);
        }

        self.enable_interrupts();
    }

    /// Transmits a single byte if the hardware is ready.
    pub fn send_byte(&self, c: u8) {
        // Wait for space in FIFO
        while !self.tx_ready() {}
        // Put byte in data register
        let regs = unsafe { &*self.regs };
        regs.dr.set(c as u32);
    }

    /// Checks if there is space in the transmit fifo queue.
    pub fn tx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        !regs.fr.is_set(Flags::TX_FIFO_FULL)
    }

    pub fn set_tx_dma_to_buffer(&self){
        //we use (self.tx_remaining_bytes.get()-1) which, if it's 0, could be awkward...
        if(self.tx_remaining_bytes.get() > 0){
            self.tx_buffer.map(|tx_buffer| {
                unsafe{udma::UDMA.prepare_transfer(
                           TX_DMA, 
                           tx_buffer[(self.tx_remaining_bytes.get()-1)..].as_ptr() as usize,
                           udma::DMATransferType::DataTx,
                           self.tx_remaining_bytes.get()-1)
                };
            });
        }
    }

    pub fn start_tx(&self){
        //configure the DMA controller to start
        unsafe{udma::UDMA.start_transfer(TX_DMA)};
        //enable the DMA-UART module connection
        let regs = unsafe { &*self.regs };
        regs.dmactl.modify(DMACtl::TXDMAE::SET);
    }

    pub fn set_rx_dma_to_buffer(&self){
        //you have to pass a pointer to the last element of the buffer array, not the first 
        //I'm using `self.tx_remaining_bytes.get()-1`, which, if it's 0, could be awkward...
        if(self.rx_remaining_bytes.get() > 0){
            self.rx_buffer.map(|rx_buffer| {
                unsafe{udma::UDMA.prepare_transfer(
                           RX_DMA, 
                           rx_buffer[(self.rx_remaining_bytes.get()-1)..].as_ptr() as usize,
                           udma::DMATransferType::DataRx,
                           self.rx_remaining_bytes.get()-1)
                };
            });
        }
    }

    pub fn start_rx(&self){
        //configure the DMA controller to start
        unsafe{udma::UDMA.start_transfer(RX_DMA)};
        //enable the DMA-UART module connection
        let regs = unsafe { &*self.regs };
        regs.dmactl.modify(DMACtl::RXDMAE::SET);
    }

    pub fn set_params(&self, params: kernel::hil::uart::UARTParams) {
        self.params.set(Some(params));
    }
}

impl kernel::hil::uart::UART for UART {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: kernel::hil::uart::UARTParams) {
        unsafe {
            PM.request_resource(prcm::PowerDomain::Serial as u32);
        }
        prcm::Clock::enable_uart_run();

        self.disable_interrupts();
        self.set_params(params);
        self.configure(); 
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let truncated_len = min(tx_data.len(), tx_len);
        
        if truncated_len == 0 {
            return;
        }
        //disable the interrupts until we're ready
        //self.disable_interrupts();

        //initialize the UART with the passed settings, and then transfer them
        //to the DMA controller
        self.tx_remaining_bytes.set(tx_len);
        self.tx_buffer.replace(tx_data);
        self.set_tx_dma_to_buffer();

        //After calling this function, the DMA controller will be primed to listen 
        //for the interrupt signals coming from the UART that normally get routed to the handler
        //(I think)
        self.start_tx();

        //enable interrupts to start the transfer for real
        self.enable_interrupts();
    }

    fn receive(&self, rx_data: &'static mut [u8], rx_len: usize) {
        let truncated_len = min(rx_data.len(), rx_len);

        if truncated_len == 0 {
            return;
        }

        //self.disable_interrupts();

        self.rx_remaining_bytes.set(rx_len);
        self.rx_buffer.replace(rx_data);
        self.set_rx_dma_to_buffer();

        self.start_rx();

        self.enable_interrupts();
    } 
}

impl peripheral_manager::PowerClient for UART {
    fn before_sleep(&self, _sleep_mode: u32) {
        // Wait for all transmissions to occur
        while self.busy() {}

        unsafe {
            // Disable the TX & RX pins in order to avoid current leakage
            self.tx_pin.get().map(|pin| {
                gpio::PORT[pin as usize].disable();
            });
            self.rx_pin.get().map(|pin| {
                gpio::PORT[pin as usize].disable();
            });

            PM.release_resource(prcm::PowerDomain::Serial as u32);
        }

        prcm::Clock::disable_uart_run();
    }

    fn after_wakeup(&self, _sleep_mode: u32) {
        unsafe {
            PM.request_resource(prcm::PowerDomain::Serial as u32);
        }
        prcm::Clock::enable_uart_run();
        self.configure();
    }

    fn lowest_sleep_mode(&self) -> u32 {
        if self.rx_remaining_bytes.get() == 0 {
            chip::SleepMode::DeepSleep as u32
        } else {
            chip::SleepMode::Sleep as u32
        }
    }
}
