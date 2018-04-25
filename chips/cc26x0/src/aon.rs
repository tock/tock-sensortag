//! Always On Module (AON) management
//!
//! AON is a set of peripherals which is _always on_ (eg. the RTC, MCU, etc).
//!
//! The current configuration disables all wake-up selectors, since the
//! MCU never go to sleep and is always active.

use kernel::common::VolatileCell;
use kernel::common::regs::{ReadOnly,ReadWrite};
use rtc;

#[repr(C)]
pub struct AonIocRegisters {
    _reserved0: [u32; 3],
    ioc_latch: ReadWrite<u32, IocLatch::Register>,
}

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
    mcu_cfg: ReadWrite<u32, McuCfg::Register>,
    aux_cfg: ReadWrite<u32, AuxCfg::Register>,
    aux_ctl: ReadWrite<u32, AuxCtl::Register>,
    pwr_stat: ReadOnly<u32, PwrStat::Register>,
    _shutdown: ReadOnly<u32>,

    _reserved0: ReadOnly<u32>,

    ctl0: ReadWrite<u32, Ctl0::Register>,
    _ctl1: ReadOnly<u32>,

    _reserved1: [ReadOnly<u8>; 0x18],

    jtag_cfg: ReadWrite<u32, JtagCfg::Register>,
}

#[repr(C)]
struct AonSysctlRegisters {
    pwrtcl: ReadWrite<u32, PwrCtl::Register>,
    resetctl: ReadOnly<u32>,
    sleepctl: ReadOnly<u32>
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
        VIRT_OFF    OFFSET(17) NUMBITS(1) [],
        FIXED_WU_EN OFFSET(16) NUMBITS(1) [],

        // SRAM Retention enabled
        //  0x00 - Retention disabled
        //  0x01 - Retention enabled for BANK0
        //  0x03 - Retention enabled for BANK0, BANK1
        //  0x07 - Retention enabled for BANK0, BANK1, BANK2
        //  0x0F - Retention enabled for BANK0, BANK1, BANK2, BANK3
        SRAM_RET_EN OFFSET(0)  NUMBITS(4) [
            OFF = 0x00,
            ON = 0x0F   // Default to enable retention in all banks
        ]
    ],
    AuxCfg [
        RAM_RET_EN OFFSET(0) NUMBITS(1) []
    ],
    AuxCtl [
        RESET_REQ    OFFSET(31) NUMBITS(1) [],
        AUX_FORCE_ON OFFSET(0) NUMBITS(1) []
    ],
    PwrStat [
        AUX_PWR_DWN OFFSET(9) NUMBITS(1) [],
        JTAG_PD_ON  OFFSET(6) NUMBITS(1) [],
        AUX_PD_ON   OFFSET(5) NUMBITS(1) [],
        MCU_PD_ON   OFFSET(4) NUMBITS(1) [],
        AUX_BUS_CONNECTED OFFSET(2) NUMBITS(1) [],
        AUX_RESET_DONE OFFSET(1) NUMBITS(1) []
    ],
    Ctl0 [
        // Controls whether MCU & AUX requesting to be powered off
        // will enable a transition to powerdown (0 = Enabled, 1 = Disabled)
        PWR_DWN_DIS     OFFSET(8) NUMBITS(1) []
    ],
    JtagCfg [
        JTAG_PD_FORCE_ON    OFFSET(8) NUMBITS(1) []
    ],
    IocLatch [
        EN  OFFSET(0) NUMBITS(1) []
    ],

    // PwrCtl controls the power configuration to supply the VDDR to the
    // entire chip (that is, the power source).
    //      GLDO = Global LDO, uses higher current
    //      DCDC = Regulated LDO, uses lower current (to conserve energy)
    //      EXT  = Use an external power source
    //  *NOTE*: DCDC_ACTIVE and DCDC_EN should always have the same value
    PwrCtl [
        // 0 = use GLDO in active mode, 1 = use DCDC in active mode
        DCDC_ACTIVE  OFFSET(2) NUMBITS(1) [],
        // 0 = DCDC/GLDO are used, 1 = DCDC/GLDO are bypassed and using a external regulater
        EXT_REG_MODE OFFSET(1) NUMBITS(1) [],
        // 0 = use GDLO for recharge, 1 = use DCDC for recharge
        DCDC_EN      OFFSET(0) NUMBITS(1) []
    ]
];


pub struct Aon {
    event_regs: *const AonEventRegisters,
    aon_wuc_regs: *const AonWucRegisters,
    aon_ioc_regs: *const AonIocRegisters,
    aon_sysctl_regs: *const AonSysctlRegisters,
}

pub const AON: Aon = Aon::new();

impl Aon {
    const fn new() -> Aon {
        Aon {
            event_regs: 0x4009_3000 as *const AonEventRegisters,
            aon_wuc_regs: 0x4009_1000 as *const AonWucRegisters,
            aon_ioc_regs:  0x4009_4000 as *const AonIocRegisters,
            aon_sysctl_regs: 0x4009_0000 as *const AonSysctlRegisters,
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

    pub fn set_dcdc_enabled(&self, enabled: bool) {
        let regs: &AonSysctlRegisters = unsafe { &*self.aon_sysctl_regs };
        if enabled {
            regs.pwrtcl.modify(
                PwrCtl::DCDC_ACTIVE::SET
                    + PwrCtl::DCDC_EN::SET
            );
        } else {
            regs.pwrtcl.modify(
                PwrCtl::DCDC_ACTIVE::CLEAR
                    + PwrCtl::DCDC_EN::CLEAR
            );
        }
    }

    pub fn lock_io_pins(&self, lock: bool) {
        let regs: &AonIocRegisters = unsafe { &*self.aon_ioc_regs };
        if lock {
            regs.ioc_latch.write(IocLatch::EN::CLEAR);
        }
        else {
            regs.ioc_latch.write(IocLatch::EN::SET);
        }
    }

    pub fn aux_set_ram_retention(&self, enabled: bool) {
        let regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        regs.aux_cfg.modify({
            if enabled { AuxCfg::RAM_RET_EN::SET } else { AuxCfg::RAM_RET_EN::CLEAR }
        });
    }

    pub fn aux_wakeup(&self, wakeup: bool) {
        let regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        regs.aux_ctl.modify({
            if wakeup { AuxCtl::AUX_FORCE_ON::SET } else { AuxCtl::AUX_FORCE_ON::CLEAR }
        });
    }

    pub fn aux_is_on(&self) -> bool {
        let regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        regs.pwr_stat.is_set(PwrStat::AUX_PD_ON)
    }

    pub fn jtag_set_enabled(&self, enabled: bool) {
        let regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        regs.jtag_cfg.modify({
            if enabled { JtagCfg::JTAG_PD_FORCE_ON::SET } else { JtagCfg::JTAG_PD_FORCE_ON::CLEAR }
        });
    }

    pub fn mcu_set_ram_retention(&self, on: bool) {
        let regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        regs.mcu_cfg.modify({
            if on { McuCfg::SRAM_RET_EN::ON } else { McuCfg::SRAM_RET_EN::OFF }
        });
    }

    pub fn mcu_disable_power_down_clock(&self) {
        let aon_regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        // Disable the clock
        aon_regs.mcu_clk.modify(
            McuClk::PWR_DWN_SRC::NO_CLOCK
        );
    }

    pub fn aux_disable_power_down_clock(&self) {
        let aon_regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        aon_regs.aux_clk.modify(
            AuxClk::PWR_DWN_SRC::NO_CLOCK
        );
    }

    pub fn mcu_power_down_enable(&self) {
        let aon_regs: &AonWucRegisters = unsafe { &*self.aon_wuc_regs };
        // Enable power down of the MCU
        aon_regs.ctl0.modify(
            Ctl0::PWR_DWN_DIS::CLEAR
        );
    }

    /// Await a cycle of the AON domain in order
    /// to sync with it.
    pub fn sync(&self) {
        rtc::RTC.sync();
    }
}
