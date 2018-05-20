///
/// # Micro Direct Memory Access for the TI CC26x0 Microcontroller
///

//use core::{cmp};
use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
//use kernel::common::take_cell::TakeCell;
use kernel::ReturnCode;
use prcm;

pub const UDMA_BASE: usize = 0x4002_0000;

#[repr(C)]
struct DMARegisters {
    status: ReadOnly<u32, Status::Register>,
    cfg: WriteOnly<u32, Config::Register>,
    ctrl: ReadWrite<u32>,
    alt_ctrl: ReadOnly<u32>,
    wait_on_req: ReadOnly<u32,DMAChannelBitfield::Register>,
    soft_req: WriteOnly<u32,DMAChannelBitfield::Register>,
    set_burst: ReadWrite<u32,DMAChannelBitfield::Register>,
    clear_burst: WriteOnly<u32,DMAChannelBitfield::Register>,
    set_req_mask: ReadWrite<u32,DMAChannelBitfield::Register>,
    clear_req_mask: WriteOnly<u32,DMAChannelBitfield::Register>,
    set_channel_en: ReadWrite<u32,DMAChannelBitfield::Register>,
    clear_channel_en: WriteOnly<u32,DMAChannelBitfield::Register>,
    set_chnl_pri_alt: ReadWrite<u32,DMAChannelBitfield::Register>,
    clear_chnl_pri_alt: WriteOnly<u32,DMAChannelBitfield::Register>,
    set_chnl_priority: ReadWrite<u32,DMAChannelBitfield::Register>,
    clear_chnl_priority: WriteOnly<u32,DMAChannelBitfield::Register>,
    error: ReadWrite<u32>,
    req_done: ReadWrite<u32,DMAChannelBitfield::Register>,
    done_mask: ReadWrite<u32,DMAChannelBitfield::Register>
}

register_bitfields! [u32,
    Status [
        MASTERENABLE OFFSET(0) NUMBITS(1),
        STATE OFFSET(4) NUMBITS(4),
        TOTALCHANNELS OFFSET(16) NUMBITS(5),
        TEST OFFSET(28) NUMBITS(4)
    ],
    Config [
        MASTERENABLE OFFSET(0) NUMBITS(1),
        PRTOCTRL OFFSET(5) NUMBITS(3)
    ],
    Control [
        BASEPTR OFFSET(10) NUMBITS(22)
    ], 
    DMAChannelBitfield [
        SOFTWARE_0 OFFSET(0)  NUMBITS(1),
        UART0_RX   OFFSET(1)  NUMBITS(1),
        UART0_TX   OFFSET(2)  NUMBITS(1),
        SSP0_RX    OFFSET(3)  NUMBITS(1),
        SSP0_TX    OFFSET(4)  NUMBITS(1),
        AUX_ADC    OFFSET(7)  NUMBITS(1),
        AUX_SW     OFFSET(8)  NUMBITS(1),
        GPT0_A     OFFSET(9)  NUMBITS(1),
        GPT0_B     OFFSET(10) NUMBITS(1),
        GPT1_A     OFFSET(11) NUMBITS(1),
        GPT1_B     OFFSET(12) NUMBITS(1),
        AON_PROG2  OFFSET(13) NUMBITS(1),
        DMA_PROG   OFFSET(14) NUMBITS(1),
        AON_RTC    OFFSET(15) NUMBITS(1),
        SSP1_RX    OFFSET(16) NUMBITS(1),
        SSP1_TX    OFFSET(17) NUMBITS(1),
        SOFTWARE_1 OFFSET(18) NUMBITS(1),
        SOFTWARE_2 OFFSET(19) NUMBITS(1),
        SOFTWARE_3 OFFSET(20) NUMBITS(1)
    ],

    DMATableControl [
        TRANSFERSIZE OFFSET(4) NUMBITS(10) [],
        ARB OFFSET(14) NUMBITS(4) [
            ARB1    = 0,
            ARB2    = 1,
            ARB4    = 2, 
            ARB8    = 3,
            ARB16   = 4,
            ARB32   = 5,
            ARB64   = 6,
            ARB128  = 7,
            ARB256  = 8,
            ARB512  = 9,
            ARB1024 = 10
        ],
        PSIZE OFFSET(24) NUMBITS(6) [
            PSIZE8  = 0,
            PSIZE16 = 1,
            PSIZE32 = 2
        ],
        SRCINC OFFSET(26) NUMBITS(2) [
            INC8    = 0,
            INC16   = 1,
            INC32   = 2,
            INC0    = 3
        ],
        DSTINC OFFSET(30) NUMBITS(2) [
            INC8    = 0,
            INC16   = 1,
            INC32   = 2,
            INC0    = 3
        ]
    ]
];

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DMAPeripheral {
    SOFTWARE_0 = 0,
    UART0_RX = 1,
    UART0_TX = 2,
    SSP0_RX = 3,
    SSP0_TX = 4,
    AUX_ADC = 7,
    AUX_SW = 8,
    GPT0_A = 9,
    GPT0_B = 10,
    GPT1_A = 11,
    GPT1_B = 12,
    AON_PROG2 = 13,
    DMA_PROG = 14,
    AON_RTC = 15,
    SSP1_RX = 16,
    SSP1_TX = 17,
    SOFTWARE_1 = 18,
    SOFTWARE_2 = 19,
    SOFTWARE_3 = 20,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum DMAWidth {
    Width8Bit = 0,
    Width16Bit = 1,
    Width32Bit = 2,
}

pub enum DMATransferType{
    DataCopy = 0,
    DataTx = 1,
    DataRx = 2
}

pub struct Udma {
    regs: *const DMARegisters,
}


//
//
//
#[repr(align(1024))]
pub struct DMAChannelControlTable {
    config_array: [DMAChannelControl; 32]
}

struct DMAChannelControl {
    source_ptr: usize,
    dest_ptr: usize,
    control: ReadWrite<u32,DMATableControl::Register>,
    _unused: usize,
}

pub struct DMAChannel {
    channel: DMAPeripheral,
    client: Cell<Option<&'static DMAClient>>,
    width: Cell<DMAWidth>,
    enabled: Cell<bool>,
    xfer_type: Cell<Option<DMATransferType>>
}

pub trait DMAClient {
    fn xfer_done(&self, pid: DMAPeripheral);
}

pub static mut UDMA: Udma = Udma::new();

static mut DMACTRLTAB: DMAChannelControlTable = DMAChannelControlTable::new();

pub static mut DMA_CHANNELS: [DMAChannel; 21] = [
    DMAChannel::new(DMAPeripheral::SOFTWARE_0),
    DMAChannel::new(DMAPeripheral::UART0_RX),
    DMAChannel::new(DMAPeripheral::UART0_TX),
    DMAChannel::new(DMAPeripheral::SSP0_RX),
    DMAChannel::new(DMAPeripheral::SSP0_TX),
    DMAChannel::new(DMAPeripheral::SSP0_TX), // These two are unused, just here
    DMAChannel::new(DMAPeripheral::SSP0_TX), // for filler
    DMAChannel::new(DMAPeripheral::AUX_ADC),
    DMAChannel::new(DMAPeripheral::AUX_SW),
    DMAChannel::new(DMAPeripheral::GPT0_A),
    DMAChannel::new(DMAPeripheral::GPT0_B),
    DMAChannel::new(DMAPeripheral::GPT1_A),
    DMAChannel::new(DMAPeripheral::GPT1_B),
    DMAChannel::new(DMAPeripheral::AON_PROG2),
    DMAChannel::new(DMAPeripheral::DMA_PROG),
    DMAChannel::new(DMAPeripheral::AON_RTC),
    DMAChannel::new(DMAPeripheral::SSP1_RX),
    DMAChannel::new(DMAPeripheral::SSP1_TX),
    DMAChannel::new(DMAPeripheral::SOFTWARE_1),
    DMAChannel::new(DMAPeripheral::SOFTWARE_2),
    DMAChannel::new(DMAPeripheral::SOFTWARE_3)
];



impl DMAChannelControlTable{
    const fn new() -> DMAChannelControlTable{
        DMAChannelControlTable{
            config_array: [ 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(), 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(),
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(), 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(),
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(), 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(),
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(), 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(),
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(), 
                DMAChannelControl::new(), DMAChannelControl::new(), DMAChannelControl::new(),
                DMAChannelControl::new(), DMAChannelControl::new()
            ]
        }
    }
}

impl DMAChannelControl {
    const fn new() -> DMAChannelControl {
        DMAChannelControl {
            source_ptr: 0, 
            dest_ptr: 0, 
            control: ReadWrite::new(0), 
            _unused: 0 
        }
    }
}

impl Udma {
    /// Constructor 
    pub const fn new() -> Udma {
        Udma {
            regs: UDMA_BASE as *const DMARegisters
        }
    }

    /// Enable Function
    /// Sets up the power domain and the clocks
    /// It then sets MASTERNEABLE in the CFG register and writes the location of
    /// the control table in the CTRL register.
    pub fn enable(
        &self,
    ) {
        let regs = unsafe{&*self.regs};
        self.power_and_clock();
        regs.cfg.write(Config::MASTERENABLE::SET);
        unsafe{
            regs.ctrl.set(&mut DMACTRLTAB as *mut DMAChannelControlTable as u32)
        }
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}
        prcm::Clock::enable_dma();
    }
}

impl DMAChannel {
    const fn new(channel: DMAPeripheral) -> DMAChannel {
        DMAChannel {
            channel: channel,
            width: Cell::new(DMAWidth::Width8Bit),
            client: Cell::new(None),
            enabled: Cell::new(false),
            xfer_type: Cell::new(None)
        }
    }
    pub fn enable(
        &mut self,
    ) {
        let regs = unsafe{&*UDMA.regs};
        //Other stuff
    }

    /// Initialize()
    /// Initialize sets up the DMA Channel with the parts of the transfer that
    /// do not include client information. 
    /// Pass in the following values:
    ///   1. `width`: the width of the data block- 8, 16, or 32 bits
    ///   2. `xfer_type`: the type of transfer, either RX, TX, or data
    ///   3. `ptr`: the pointer to the data register that's unchanging. This
    ///      depends on the transfer type:
    ///     1. `data_xfer`: means that the destination is unchanging, and the
    ///        buffer that will be passed in is the source
    ///     2. `data_tx`: means the data is going FROM a buffer to a non-
    ///        incrementing register
    ///
    pub fn initialize(
        &self, 
        width: DMAWidth, 
        xfer_type: DMATransferType,
        ptr: u32,
    ) {

        // -----------------------
        // Channel Control section
        // -----------------------

        //use `channel` to determine which Channel Control Table row we want to use
        let channel = unsafe{&mut DMACTRLTAB.config_array[self.channel as usize]};
        //use `width` to determine the 
        channel.control.modify(DMATableControl::PSIZE.val(width as u32));
        
        //per-byte arbitration we'll keep as the default for now
        channel.control.modify(DMATableControl::ARB.val(0));

        match &xfer_type {
            //a data transfer means the pointer being passed here is a destination
            //and that we're going to increment
            // TODO: variable source/destination increments, right now it's just
            //       the same as the transfer width.
            data_xfer => {
                channel.dest_ptr  = ptr as usize;
                channel.control.modify(DMATableControl::SRCINC.val(width as u32));
                channel.control.modify(DMATableControl::DSTINC.val(width as u32));
                         },
            //a tx transfer means the data is going FROM a buffer TO a point in
            //memory (such as a peripheral's TX register)
            //This means that the pointer is the destination, and the destination
            //pointer should not be incremented
            data_tx   => {
                channel.dest_ptr  = ptr as usize;
                channel.control.modify(DMATableControl::SRCINC.val(width as u32));
                channel.control.modify(DMATableControl::DSTINC.val(3));
                         },
            //a rx transfer means the data is going FROM a point in memory (such
            //as a peripheral's RX register) TO a buffer
            //This means that the supplied pointer is the source, and this source
            //pointer should not be incremented.
            data_rx   => {
                channel.source_ptr  = ptr as usize;
                channel.control.modify(DMATableControl::SRCINC.val(3));
                channel.control.modify(DMATableControl::DSTINC.val(width as u32));
                         },    
        };

        // ------------------------
        // UDMA Register Section
        // ------------------------

        let registers: &DMARegisters = unsafe { &*UDMA.regs };

        // Set Bit (Channel N) of CLEARCHNLPRIORITY to set normal priority

        // Set Bit (Channel N) of CLEARCHNLPRIALT to use primary channel control

        // Set Bit (Channel N) of CLEARBURST to use either single or burst requests

        // Set Bit (Channel N) of CLEARREQMASK to recognize requests to channel

        

        // ------------------------
        // Local Struct Mod Section
        // ------------------------

        self.width.set(width);
        self.xfer_type.set(Some(xfer_type));
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn handle_interrupt(&mut self) {
        //let registers: &DMARegisters = unsafe { &*UDMA.regs };
        
        //Disable interrupts

        //Call xfer_done for the client connected to the channel
        self.client.get().as_mut().map(|client| {
            client.xfer_done(self.channel);
        });
    }

    pub fn prepare_xfer(
        &self, 
        bufptr: usize, 
        len: usize
    ) {
        let registers: &DMARegisters = unsafe { &*UDMA.regs };
        let channel = unsafe{&mut DMACTRLTAB.config_array[self.channel as usize]};

        //make sure `len` isn't longer than the buffer length
        //find the maximum value `len` could have for the buffer...
        //let maxlen = buf.len() / match self.width.get() {
        //        DMAWidth::Width8Bit /*  DMA is acting on bytes     */ => 1,
        //        DMAWidth::Width16Bit /* DMA is acting on halfwords */ => 2,
        //        DMAWidth::Width32Bit /* DMA is acting on words     */ => 4,
        //    };

        //...and set it to this if `len` is larger than the max value
        //len = cmp::min(len, maxlen);

        //write either the source or destination pointers depending on transfer
        //type 

        match &self.xfer_type {
            data_xfer => channel.source_ptr   = bufptr,
            data_tx   => channel.source_ptr   = bufptr,
            data_rx   => channel.dest_ptr     = bufptr,    
        };

        //write the length of the transfer to the channel config
        channel.control.modify(DMATableControl::TRANSFERSIZE.val(len as u32));

        //Enable the transfer complete interrupt
        //only if this is a software DMA transfer (not necessary if hardware)

        // Store the buffer reference in the TakeCell so it can be returned to
        // the caller in `handle_interrupt`
        //self.buffer.replace(buf);

    }

    /// Take the current channel, and check if the REQDONE register has a Bit set
    ///
    pub fn transfer_complete(&self) -> bool{
        let registers: &DMARegisters = unsafe { &*UDMA.regs };
        //match &self.channel {
        //    UART0_RX => registers.req_done.is_set(DMAChannelBitfield::UART0_RX),
        //    UART0_TX => registers.req_done.is_set(DMAChannelBitfield::UART0_TX),
        //    _ => false,
        //}   
        false 
    }

    pub fn clear_transfer_flag(&self){

    }
}


