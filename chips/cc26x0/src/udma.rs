///
/// # Micro Direct Memory Access for the TI CC26x0 Microcontroller
///

use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use prcm;

pub const UDMA_BASE: usize = 0x4002_0000;

#[repr(align(1024))]
pub struct ControlTable {
    config_array: [DMAConfig; 32]
}


#[derive(Clone, Copy)]
struct DMAConfig {
    source_ptr: usize,
    dest_ptr: usize,
    control: usize,
    _unused: usize,
}

static mut DMACTRLTAB: ControlTable = ControlTable::new();

impl ControlTable {
    const fn new() -> ControlTable {
        ControlTable {
            config_array: [ DMAConfig {
                                source_ptr: 0, 
                                  dest_ptr: 0,
                                   control: 0,
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
}