use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil::gpio::Pin;
use kernel::hil::uart;
use kernel;
use cc26xx::{prcm,gpio,peripheral_interrupts};
use cortexm3::nvic;

pub const UART_CTL_UARTEN: u32 = 1;
pub const UART_CTL_TXE: u32 = 1 << 8;
pub const UART_CTL_RXE: u32 = 1 << 9;
pub const UART_LCRH_FEN: u32 = 1 << 4;
pub const UART_FR_BUSY: u32 = 1 << 3;
pub const UART_INT_ALL: u32 = 0x7F2;
pub const UART_INT_RX: u32 = 0x010;
pub const UART_INT_RT: u32 = 0x040;
pub const UART_FIFO_TX7_8: u32 = 0x04;          // Transmit interrupt at 7/8 Full
pub const UART_FIFO_RX4_8: u32 = 0x10;          // Receive interrupt at 1/2 Full
pub const UART_FR_TXFF: u32 = 0x20;
pub const UART_CONF_WLEN_8: u32 = 0x60;
pub const UART_CONF_BAUD_RATE: u32 = 115200;

pub const MCU_CLOCK: u32 = 48_000_000;

pub const UART_BASE: usize = 0x4000_1000;

#[repr(C)]
pub struct Registers {
    pub dr: VolatileCell<u32>,
    pub rsr_ecr: VolatileCell<u32>,
    _reserved0: [VolatileCell<u8>; 0x10],
    pub fr: VolatileCell<u32>,
    _reserved1: [VolatileCell<u8>; 0x8],
    pub ibrd: VolatileCell<u32>,
    pub fbrd: VolatileCell<u32>,
    pub lcrh: VolatileCell<u32>,
    pub ctl: VolatileCell<u32>,
    pub ifls: VolatileCell<u32>,
    pub imsc: VolatileCell<u32>,
    pub ris: VolatileCell<u32>,
    pub mis: VolatileCell<u32>,
    pub icr: VolatileCell<u32>,
    pub dmactl: VolatileCell<u32>,
}

pub struct UART {
    regs: *const Registers,
    client: Cell<Option<&'static uart::Client>>,
    tx_pin: Option<&'static gpio::GPIOPin>,
    rx_pin: Option<&'static gpio::GPIOPin>,
}

pub static mut UART0: UART = UART::new();

impl UART {
    pub const fn new() -> UART {
        UART {
            regs: UART_BASE as *mut Registers,
            client: Cell::new(None),
            tx_pin: None,
            rx_pin: None,
        }
    }

    pub fn set_pins(&mut self, tx_pin: &'static gpio::GPIOPin, rx_pin: &'static gpio::GPIOPin) {
        self.tx_pin = Some(tx_pin);
        self.rx_pin = Some(rx_pin);
    }

    pub fn configure(&self, params: kernel::hil::uart::UARTParams) {
        let ctl_val = UART_CTL_UARTEN | UART_CTL_TXE | UART_CTL_RXE;

        /*
        * Make sure the TX pin is output / high before assigning it to UART control
        * to avoid falling edge glitches
        */
        self.tx_pin.unwrap().make_output();
        self.tx_pin.unwrap().set();

        // Map UART signals to IO pin
        self.tx_pin.unwrap().iocfg().enable_uart_tx();
        self.rx_pin.unwrap().iocfg().enable_uart_rx();

        // Disable the UART before configuring
        self.disable();

        self.set_baud_rate(params.baud_rate);

        // Set word length
        let regs = unsafe { &*self.regs };
        regs.lcrh.set(UART_CONF_WLEN_8);

        // Set fifo interrupt level
        regs.ifls.set(UART_FIFO_TX7_8 | UART_FIFO_RX4_8);
        self.fifo_enable();

        // Enable, TX, RT and UART
        regs.ctl.set(ctl_val);
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) { };
        prcm::Clock::enable_uart_run();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;

        // Set the baud rate
        let regs = unsafe { &*self.regs };
        regs.ibrd.set(div / 64);
        regs.fbrd.set(div % 64);
    }

    fn fifo_enable(&self) {
        let regs = unsafe { &*self.regs };
        regs.lcrh.set(regs.lcrh.get() | UART_LCRH_FEN);
    }

    fn fifo_disable(&self) {
        let regs = unsafe { &*self.regs };
        regs.lcrh.set(regs.lcrh.get() & !UART_LCRH_FEN);
    }

    pub fn disable(&self) {
        self.fifo_disable();
        let regs = unsafe { &*self.regs };
        regs.ctl.set(regs.ctl.get() & !(UART_CTL_RXE | UART_CTL_TXE | UART_CTL_UARTEN));
    }

    pub fn disable_interrupts(&self) {
        unsafe {
            let uart0_int = nvic::Nvic::new(peripheral_interrupts::UART0);
            uart0_int.disable();
        }

        // Disable all UART module interrupts
        let regs = unsafe { &*self.regs };
        regs.imsc.set(regs.imsc.get() & !UART_INT_ALL);

        // Clear all UART interrupts
        regs.icr.set(UART_INT_ALL);
    }

    pub fn enable_interrupts(&self) {
        // Clear all UART interrupts
        let regs = unsafe { &*self.regs };
        regs.icr.set(UART_INT_ALL);

        // We don't care about TX interrupts
        regs.imsc.set(regs.imsc.get() | UART_INT_RT | UART_INT_RX);

        unsafe {
            let uart0_int = nvic::Nvic::new(peripheral_interrupts::UART0);
            uart0_int.enable();
        }
    }

    pub fn send_byte(&self, c: u8) {
        // Wait for space
        while !self.tx_ready() {}

        let regs = unsafe { &*self.regs };
        regs.dr.set(c as u32);
    }

    pub fn tx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.fr.get() & UART_FR_TXFF == 0
    }

    pub fn handle_interrupt(&self) {
        self.power_and_clock();

        // Get status bits
        let regs = unsafe { &*self.regs };
        #[allow(unused)]
        let flags: u32 = regs.fr.get();

        // Clear interrupts
        regs.icr.set(UART_INT_ALL);
    }
}

impl kernel::hil::uart::UART for UART {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: kernel::hil::uart::UARTParams) {
        self.power_and_clock();
        self.disable();
        self.disable_interrupts();
        self.configure(params);
        self.enable_interrupts();
    }

    #[allow(unused)]
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        if tx_len == 0 { return; }

        for i in 0..tx_len {
            self.send_byte(tx_data[i]);
        }
    }

    #[allow(unused)]
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {}
}
