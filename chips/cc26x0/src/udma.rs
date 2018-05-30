///
/// # Micro Direct Memory Access for the TI CC26x0 Microcontroller
///

use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use prcm;

pub const UDMA_BASE: usize = 0x4002_0000;

#[repr(C)]
struct DMARegisters {
    status: ReadOnly<u32, Status::Register>,
    cfg: WriteOnly<u32, Config::Register>,
    ctrl: ReadWrite<u32>,
    alt_ctrl: ReadOnly<u32>,
    wait_on_req: ReadOnly<u32, DMAChannelSelect::Register>,
    soft_req: WriteOnly<u32, DMAChannelSelect::Register>,
    set_burst: ReadWrite<u32, DMAChannelSelect::Register>,
    clear_burst: WriteOnly<u32, DMAChannelSelect::Register>,
    set_req_mask: ReadWrite<u32, DMAChannelSelect::Register>,
    clear_req_mask: WriteOnly<u32, DMAChannelSelect::Register>,
    set_channel_en: ReadWrite<u32, DMAChannelSelect::Register>,
    clear_channel_en: WriteOnly<u32, DMAChannelSelect::Register>,
    set_chnl_pri_alt: ReadWrite<u32, DMAChannelSelect::Register>,
    clear_chnl_pri_alt: WriteOnly<u32, DMAChannelSelect::Register>,
    set_chnl_priority: ReadWrite<u32, DMAChannelSelect::Register>,
    clear_chnl_priority: WriteOnly<u32, DMAChannelSelect::Register>,
    _reserved0: [u8; 0xC],
    error: ReadWrite<u32>,
    _reserved1: [u8; 0x4B4],
    req_done: ReadWrite<u32, DMAChannelSelect::Register>,
    done_mask: ReadWrite<u32, DMAChannelSelect::Register>
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
    
    DMAChannelSelect [
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
        MODE OFFSET(0) NUMBITS(3)[
                          STOP = 0,
                         BASIC = 1, 
                          AUTO = 2, 
                      PINGPONG = 3, 
            MEM_SCATTER_GATHER = 4, 
            PER_SCATTER_GATHER = 6
        ],
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

#[derive(Copy, Clone)]
#[repr(u8)]
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

/*
pub struct DMAChannel {
    channel: DMAPeripheral,
    client: Cell<Option<&'static DMAClient>>,
    width: Cell<DMAWidth>,
    enabled: Cell<bool>,
    transfer_type: Cell<DMATransferType>
}
*/

/*
pub trait DMAClient {
    fn xfer_done(&self, pid: DMAPeripheral);
}
*/

pub static mut UDMA: Udma = Udma::new();

static mut DMACTRLTAB: DMAChannelControlTable = DMAChannelControlTable::new();

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

    pub fn initialize_channel(
        &self,
        dma_channel: DMAPeripheral, 
        width: DMAWidth, 
        transfer_type: DMATransferType,
        base_loc: u32,
    ) {

        // -----------------------
        // Channel Control section
        // -----------------------

        //use `channel` to determine which Channel Control Table row we want to use
        let channel = unsafe{&mut DMACTRLTAB.config_array[dma_channel as usize]};
        //use `width` to determine the 
        channel.control.modify(DMATableControl::PSIZE.val(width as u32));
        
        //per-byte arbitration we'll keep as the default for now
        channel.control.modify(DMATableControl::ARB.val(0));

        match transfer_type {
            //a data transfer means the pointer being passed here is a destination
            //and that we're going to increment
            // TODO: variable source/destination increments, right now it's just
            //       the same as the transfer width.
            DMATransferType::DataCopy => {
                channel.dest_ptr  = base_loc as usize;
                channel.control.modify(DMATableControl::SRCINC.val(width as u32));
                channel.control.modify(DMATableControl::DSTINC.val(width as u32));
                         },
            //a tx transfer means the data is going FROM a buffer TO a point in
            //memory (such as a peripheral's TX register)
            //This means that the pointer is the destination, and the destination
            //pointer should not be incremented
            DMATransferType::DataTx   => {
                channel.dest_ptr  = base_loc as usize;
                channel.control.modify(DMATableControl::SRCINC.val(width as u32));
                channel.control.modify(DMATableControl::DSTINC.val(3));
                         },
            //a rx transfer means the data is going FROM a point in memory (such
            //as a peripheral's RX register) TO a buffer
            //This means that the supplied pointer is the source, and this source
            //pointer should not be incremented.
            DMATransferType::DataRx   => {
                channel.source_ptr  = base_loc as usize;
                channel.control.modify(DMATableControl::SRCINC.val(3));
                channel.control.modify(DMATableControl::DSTINC.val(width as u32));
                         },    
        };

        // ------------------------
        // UDMA Register Section
        // ------------------------

        let registers: &DMARegisters = unsafe {&*self.regs};

        // Set Bit (Channel N) of CLEARCHNLPRIORITY to set normal priority
        registers.clear_chnl_priority.set(1<<(dma_channel as u32));
        // Set Bit (Channel N) of CLEARCHNLPRIALT to use primary channel control
        registers.clear_chnl_pri_alt.set(1<<(dma_channel as u32));
        // Set Bit (Channel N) of CLEARBURST to use either single or burst requests
        registers.clear_burst.set(1<<(dma_channel as u32));
        // Set Bit (Channel N) of CLEARREQMASK to recognize requests to channel
        registers.clear_req_mask.set(1<<(dma_channel as u32));
    }

    
    pub fn start_transfer(
        &self,
        dma_channel: DMAPeripheral,
    ) {
        let registers: &DMARegisters = unsafe { &*UDMA.regs };
        let channel = unsafe{&mut DMACTRLTAB.config_array[dma_channel as usize]};
        
        channel.control.modify(DMATableControl::MODE::BASIC);
        registers.set_channel_en.set(1<<(dma_channel as u32));
    }

    pub fn prepare_transfer(
        &self, 
        dma_channel: DMAPeripheral, 
        bufptr: usize,
        transfer_type: DMATransferType,
        len: usize
    ) {
        //let registers: &DMARegisters = unsafe { &*UDMA.regs };
        let channel = unsafe{&mut DMACTRLTAB.config_array[dma_channel as usize]};

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

        //write the length of the transfer to the channel config
        channel.control.modify(DMATableControl::TRANSFERSIZE.val(len as u32));

        match transfer_type {
            DMATransferType::DataCopy => {channel.source_ptr   = bufptr;},
            DMATransferType::DataTx   => {channel.source_ptr   = bufptr;},
            DMATransferType::DataRx   => {channel.dest_ptr     = bufptr;},    
        };


        //Enable the transfer complete interrupt
        //only if this is a software DMA transfer (not necessary if a peripheral transfer)

    }

    pub fn do_transfer(
        &self, 
        dma_channel: DMAPeripheral, 
        bufptr: usize,
        transfer_type: DMATransferType,
        len: usize
    ){
        self.prepare_transfer(dma_channel, bufptr, transfer_type, len);
        self.start_transfer(dma_channel);
    }

    /// Take the current channel, and check if the REQDONE register has a Bit set
    ///
    pub fn transfer_complete(
        &self,
        dma_channel: DMAPeripheral
    ) -> bool{
        let registers: &DMARegisters = unsafe {&*self.regs};

        let reqdone: u32 = registers.req_done.get();

        //ugly but better than the match interface let's be honest
        return ((reqdone >> (dma_channel as u32)) & 0x1) == 0x1 ; 

        /*
        //there must be a better way to do this with the register interface
        match dma_channel {
            DMAPeripheral::UART0_RX => registers.req_done.is_set(DMAChannelSelect::UART0_RX),
            DMAPeripheral::UART0_TX => registers.req_done.is_set(DMAChannelSelect::UART0_TX),
            _ => false,
        }   
        */
    }

    pub fn clear_transfer(
        &self,
        dma_channel: DMAPeripheral
    ){
        let registers: &DMARegisters = unsafe {&*self.regs};
        registers.clear_channel_en.set(1<<(dma_channel as u32));
        registers.req_done.set(1<<(dma_channel as u32));
    }
}

