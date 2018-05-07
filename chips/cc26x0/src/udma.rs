///
/// # Micro Direct Memory Access for the TI CC26x0 Microcontroller
///

use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::ReturnCode;
use prcm;

pub const UDMA_BASE: usize = 0x4002_0000;

#[repr(align(1024))]
pub struct ControlTable {
    config_array: [DMAConfig; 32]
}

#[derive(Copy, Clone)]
struct DMAConfig {
    source_ptr: usize,
    dest_ptr: usize,
    control: ReadWrite<u32,DMATableControl::Register>,
    _unused: usize,
}

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


static mut DMACTRLTAB: ControlTable = ControlTable::new();

impl ControlTable {
    const fn new() -> ControlTable {
        ControlTable {
            config_array: [DMAConfig {
                                source_ptr: 0, 
                                  dest_ptr: 0,
                                   control: ReadWrite::new(0),
                                   _unused: 0
                            }; 32]
        }
    }
}

#[repr(C)]
struct DMARegisters {
    status: ReadOnly<u32, Status::Register>,
    cfg: WriteOnly<u32, Config::Register>,
    ctrl: ReadWrite<u32>,
    alt_ctrl: ReadOnly<u32>,
    wait_on_req: ReadWrite<u32>,
    soft_req: ReadWrite<u32>,
    set_burst: ReadWrite<u32>,
    clear_burst: ReadWrite<u32>,
    set_req_mask: ReadWrite<u32>,
    clear_req_mask: ReadWrite<u32>,
    set_channel_en: ReadWrite<u32>,
    clear_channel_en: ReadWrite<u32>,
    set_chnl_pri_alt: ReadWrite<u32>,
    clear_chnl_pri_alt: ReadWrite<u32>,
    set_chnl_priority: ReadWrite<u32>,
    clear_chnl_priority: ReadWrite<u32>,
    error: ReadWrite<u32>,
    req_done: ReadWrite<u32>,
    done_mask: ReadWrite<u32>
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
            INC16   = 17,
            INC32   = 34
        ]
    ]
];

pub struct Udma {
    regs: *const DMARegisters,
}

pub static mut UDMA: Udma = Udma::new();

impl Udma {
    /// Constructor 
    pub const fn new() -> Udma {
        Udma {
            regs: UDMA_BASE as *const DMARegisters,
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
            regs.ctrl.set(&mut DMACTRLTAB as *mut ControlTable as u32)
        }
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}
        prcm::Clock::enable_dma();
    }

    fn set_dest_pointer(
        &self, 
        channel: DMAPeripheral,
        dest_address: usize,
    ) { 
        unsafe{
            DMACTRLTAB.config_array[channel as usize].dest_ptr = dest_address;
        }
    }

    fn set_source_pointer(
        &self, 
        channel: DMAPeripheral,
        source_address: usize,
    ) { 
        unsafe{
            DMACTRLTAB.config_array[channel as usize].source_ptr = source_address;
        }
    }
}