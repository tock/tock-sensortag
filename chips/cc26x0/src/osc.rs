//! Oscillator control
//!
//!

use aux;
use setup::oscfh;
use kernel::common::regs::{ReadOnly, ReadWrite};

/*
    The cc26xx chips have two clock sources:
        * 24MHz LF clock (low frequency)
        * 48MHz HF clock (high frequency)
    Depending on which peripheral to communicate with, you need to select
    different sources.

    See page 421 in the datasheet for more details.
*/
pub enum ClockType {
    LF,
    HF,
}

/*
    There can be many types of clock sources for both the HF and LF clocks.
    HF:
        HF RCOSC = 0x00
        HF XOSC  = 0x01
    LF:
        LF Derived RCOSC = 0x00
        LF Derived XOSC  = 0x01
        LF RCOSC         = 0x02
        LF XOSC          = 0x03
*/
pub const HF_RCOSC: u8 = 0x00;
pub const HF_XOSC: u8 = 0x01;

pub const LF_DERIVED_RCOSC: u8 = 0x00;
pub const LF_DERIVED_XOSC: u8 = 0x01;
pub const LF_RCOSC: u8 = 0x02;
pub const LF_XOSC: u8 = 0x03;

struct DdiRegisters {
    ctl0: ReadWrite<u32, Ctl0::Register>,
    _ctl1: ReadOnly<u32>,

    _radc_ext_cfg: ReadOnly<u32>,
    _amp_comp_ctl: ReadOnly<u32>,
    _amp_comp_th1: ReadOnly<u32>,
    _amp_comp_th2: ReadOnly<u32>,

    _ana_bypass_val1: ReadOnly<u32>,
    _ana_bypass_val2: ReadOnly<u32>,

    _analog_test_ctl: ReadOnly<u32>,
    _adc_doubler_nanoamp_ctl: ReadOnly<u32>,

    _xosc_hf_ctl: ReadOnly<u32>,
    _lf_osc_ctl: ReadOnly<u32>,
    _rco_sc_hf_ctl: ReadOnly<u32>,

    stat0: ReadOnly<u32, Stat0::Register>,
    _stat1: ReadOnly<u32>,
    _stat2: ReadOnly<u32>,
}

register_bitfields![
    u32,
    Ctl0 [
        XTAL_IS_24M              OFFSET(31) NUMBITS(1) [],
        BYPASS_XOSC_LF_CLK_QUAL  OFFSET(29) NUMBITS(1) [],
        BYPASS_RCOSC_LF_CLK_QUAL OFFSET(28) NUMBITS(1) [],
        DOUBLER_START_DURATION   OFFSET(26) NUMBITS(2) [],
        DOUBLER_RESET_DURATION   OFFSET(25) NUMBITS(1) [],

        FORCE_KICKSTART_EN       OFFSET(22) NUMBITS(1) [],

        ALLOW_SCLK_HF_SWITCHING  OFFSET(16) NUMBITS(1) [],

        HPOSC_MODE_ON            OFFSET(14) NUMBITS(1) [],
        RCOSC_LF_TRIMMED         OFFSET(12) NUMBITS(1) [],
        XOSC_HF_POWER_MODE       OFFSET(11) NUMBITS(1) [],
        XOSC_LF_DIG_BYPASS       OFFSET(10) NUMBITS(1) [],

        CLK_LOSS_EN              OFFSET(9) NUMBITS(1) [],
        ACLK_TDC_SRC_SEL         OFFSET(7) NUMBITS(2) [],
        ACLK_REF_SRC_SEL         OFFSET(5) NUMBITS(2) [],

        SCLK_LF_SRC_SEL          OFFSET(2) NUMBITS(2) [
            RCOSC_HF_DERIVED = 0b00,
            XOSC_HF_DERIVED  = 0b01,
            RCOSC_LF         = 0b10,
            XOSC_LF          = 0b11
        ],
        SCLK_MF_SRC_SEL OFFSET(1) NUMBITS(1) [],
        SCLK_HF_SRC_SEL OFFSET(0) NUMBITS(1) [
            RCOSC_HF = 0b00,
            XOSC_HF  = 0b01
        ]
    ],
    Stat0 [
        SCLK_LF_SRC     OFFSET(29) NUMBITS(2) [
            RCOSC_HF_DERIVED = 0b00,
            XOSC_HF_DERIVED  = 0b01,
            RCOSC_LF         = 0b10,
            XOSC_LF          = 0b11
        ],
        SCLK_HF_SRC     OFFSET(28) NUMBITS(1) [
            RCOSC_HF = 0b00,
            XOSC_HF  = 0b01
        ],
        RCOSC_HF_EN      OFFSET(22) NUMBITS(1) [],
        RCOSC_LF_EN      OFFSET(21) NUMBITS(1) [],
        XOSC_LF_EN       OFFSET(20) NUMBITS(1) [],
        CLK_DCDC_RDY     OFFSET(19) NUMBITS(1) [],
        CLK_DCDC_RDY_ACK OFFSET(18) NUMBITS(1) [],

        SCLK_HF_LOSS     OFFSET(17) NUMBITS(1) [],
        SCLK_LF_LOSS     OFFSET(16) NUMBITS(1) [],
        XOSC_HF_EN       OFFSET(15) NUMBITS(1) [],

        // Is the 48MHz clock from the DOUBLER enabled?
        // It will be enabled if 24 or 48MHz crystal is in use
        XB_48M_CLK_EN    OFFSET(13) NUMBITS(1) [],

        XOSC_HF_LP_BUF_EN OFFSET(11) NUMBITS(1) [],
        XOSC_HF_HP_BUF_EN OFFSET(10) NUMBITS(1) [],

        ADC_THMET       OFFSET(8) NUMBITS(1) [],
        ADC_DATA_READY  OFFSET(7) NUMBITS(1) [],
        ADC_DATA        OFFSET(1) NUMBITS(6) [],

        PENDING_SCLK_HF_SWITCHING OFFSET(0) NUMBITS(1) []
    ]
];

pub struct Oscillator {
    r_regs: *const DdiRegisters,
    wr_regs: *const DdiRegisters,
}

pub const OSC: Oscillator = Oscillator::new();

impl Oscillator {
    pub const fn new() -> Oscillator {
        Oscillator {
            r_regs: 0x400C_A000 as *const DdiRegisters,
            wr_regs: 0x400C_A040 as *const DdiRegisters,
        }
    }

    pub fn configure(&self) {
        aux::AUX_CTL.activate_clock(aux::AuxClock::OscillatorControl);
        aux::AUX_CTL.activate_clock(aux::AuxClock::Semaphores);
    }

    #[allow(unused)]
    pub fn set_xtal_to_24mhz(&self) {
        self.configure();

        let regs: &DdiRegisters = unsafe { &*self.r_regs };
        let wr_regs: &DdiRegisters = unsafe { &*self.wr_regs };
        wr_regs.ctl0.modify(Ctl0::XTAL_IS_24M::SET);
    }

    /// Disable the LF clock qualifiers
    ///     The LF clock qualifiers can disrupt sleep procedures,
    ///     so it's safest to just disable them.
    ///
    /// *Note*: this may be blocking until the LF source has been
    ///         stabilized to RCOSC
    pub fn disable_lf_clock_qualifiers(&self) {
        // Wait until the clock source has been set & stabilised
        while self.clock_source_get(ClockType::LF) != LF_RCOSC {}

        let regs: &DdiRegisters = unsafe { &*self.r_regs };

        // Disable the LF clock qualifiers as they are known to prevent
        // standby modes (deep sleep w/o MCU power).
        regs.ctl0
            .modify(Ctl0::BYPASS_XOSC_LF_CLK_QUAL::SET + Ctl0::BYPASS_RCOSC_LF_CLK_QUAL::SET);
    }

    pub fn request_switch_to_hf_xosc(&self) {
        self.configure();

        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            self.clock_source_set(ClockType::HF, HF_XOSC);
        }
    }

    pub fn switch_to_hf_xosc(&self) {
        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            // Wait for it to stabilize
            let regs: &DdiRegisters = unsafe { &*self.r_regs };
            while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}

            self.perform_switch();
        }
    }

    pub fn switch_to_hf_rcosc(&self) {
        self.clock_source_set(ClockType::HF, HF_RCOSC);

        let regs: &DdiRegisters = unsafe { &*self.r_regs };
        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}

        if self.clock_source_get(ClockType::HF) != HF_RCOSC {
            self.perform_switch();
        }
    }

    pub fn perform_switch(&self) {
        unsafe {
            oscfh::source_switch();
        }
    }

    pub fn clock_source_get(&self, clock: ClockType) -> u8 {
        let regs: &DdiRegisters = unsafe { &*self.r_regs };
        match clock {
            ClockType::LF => regs.stat0.read(Stat0::SCLK_LF_SRC) as u8,
            ClockType::HF => regs.stat0.read(Stat0::SCLK_HF_SRC) as u8,
        }
    }

    pub fn clock_source_set(&self, clock: ClockType, src: u8) {
        let wr_regs: &DdiRegisters = unsafe { &*self.r_regs };
        match clock {
            ClockType::LF => {
                wr_regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL.val(src as u32));
            }
            ClockType::HF => {
                // We need to keep MF & HF clocks in sync
                wr_regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL.val(src as u32));
                wr_regs.ctl0.modify(Ctl0::SCLK_MF_SRC_SEL.val(src as u32));
            }
        }
    }
}
