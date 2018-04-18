//! Always On Module (AON) management
//!
//! AON is a set of peripherals which is _always on_ (eg. the RTC, MCU, etc).
//!
//! The current configuration disables all wake-up selectors, since the
//! MCU never go to sleep and is always active.

use kernel::common::VolatileCell;
use kernel::common::regs::{ReadOnly,ReadWrite};

#[repr(C)]
pub struct AonEventRegisters {
    mcu_wu_sel: VolatileCell<u32>,       // MCU Wake-up selector
    aux_wu_sel: VolatileCell<u32>,       // AUX Wake-up selector
    event_to_mcu_sel: VolatileCell<u32>, // Event selector for MCU Events
    rtc_sel: VolatileCell<u32>,          // RTC Capture event selector for AON_RTC
}

#[repr(C)]
struct AonWucRegisters {
    mcu_clk: ReadWrite<u32, McuClk::Register>,
    aux_clk: ReadWrite<u32, AuxClk::Register>,
    mcu_cfg: VolatileCell<u32>,
    aux_cfg: VolatileCell<u32>,
    aux_ctl: VolatileCell<u32>,
    pwr_stat: VolatileCell<u32>,
    _shutdown: VolatileCell<u32>,

    _reserved0: VolatileCell<u32>,

    ctl0: VolatileCell<u32>,
    _ctl1: VolatileCell<u32>,

    _reserved1: [VolatileCell<u8>; 0x18],

    jtag_cfg: VolatileCell<u32>,
}

register_bitfields![
    u32,
    McuClk [
        PWR_DWN_SRC OFFSET(0) NUMBITS(2) [
            NO_CLOCK = 0b00,
            SCLK_LF = 0b01
        ],
        RCOSC_HF_CAL_DONE   OFFSET(2) NUMBITS(1) []
    ],
    AuxClk [
        SRC     OFFSET(0) NUMBITS(3) [
            SCLK_HF = 0x01,
            SCLK_LF = 0x04
        ],
        SCLK_HF_DIV OFFSET(8) NUMBITS(3) [],
        PWR_DWN_SRC OFFSET(11) NUMBITS(2) [
            NO_CLOCK = 0b00,
            SCLK_LF = 0b01
        ]
    ],
    McuCfg [

    ]
];


pub struct AonEvent {
    event_regs: *const AonEventRegisters,
    aon_wuc_regs: *const AonWucRegisters,
}

pub static mut AON_EVENT: AonEvent = AonEvent::new();

impl AonEvent {
    const fn new() -> AonEvent {
        AonEvent {
            event_regs: 0x4009_3000 as *const AonEventRegisters,
            aon_wuc_regs: 0x4009_1000 as *const AonWucRegisters,
        }
    }

    pub fn setup(&self) {
        let regs: &AonEventRegisters = unsafe { &*self.event_regs };

        // Default to no events at all
        regs.aux_wu_sel.set(0x3F3F3F3F);

        // Set RTC CH1 as a wakeup source by default
        regs.mcu_wu_sel.set(0x3F3F3F24);

        // Disable RTC combined event
        regs.rtc_sel.set(0x0000003F);

        // The default reset value is 0x002B2B2B. However, 0x2b for each
        // programmable event corresponds to a JTAG event; which is fired
        // *all* the time during debugging through JTAG. It is better to
        // ignore it in this case.
        //      NOTE: the aon programmable interrupt will still be fired
        //            once a debugger is attached through JTAG.
        regs.event_to_mcu_sel.set(0x003F3F3F);
    }

    pub fn aux_disable_sram_retention(&self) {

    }
}
