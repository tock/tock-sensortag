//! Oscillator control
//!
//!

use kernel::common::VolatileCell;

use aux;

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

pub const HF_STAT0_MASK: u32 = 0x10000000;
pub const LF_STAT0_MASK: u32 = 0x60000000;

struct DdiRegisters {
    ctl0: VolatileCell<u32>,
    _ctl1: VolatileCell<u32>,

    _radc_ext_cfg: VolatileCell<u32>,
    _amp_comp_ctl: VolatileCell<u32>,
    _amp_comp_th1: VolatileCell<u32>,
    _amp_comp_th2: VolatileCell<u32>,

    _ana_bypass_val1: VolatileCell<u32>,
    _ana_bypass_val2: VolatileCell<u32>,

    _analog_test_ctl: VolatileCell<u32>,
    _adc_doubler_nanoamp_ctl: VolatileCell<u32>,

    _xosc_hf_ctl: VolatileCell<u32>,
    _lf_osc_ctl: VolatileCell<u32>,
    _rco_sc_hf_ctl: VolatileCell<u32>,

    stat0: VolatileCell<u32>,
    _stat1: VolatileCell<u32>,
    _stat2: VolatileCell<u32>,
}

pub struct Oscillator {
    regs: *const DdiRegisters,
}

pub const OSCILLATOR_CONTROL: Oscillator = Oscillator::new();

impl Oscillator {
    pub const fn new() -> Oscillator {
        Oscillator {
            regs: 0x400C_A000 as *const DdiRegisters,
        }
    }

    pub fn configure(&self) {
        aux::AUX_CTL.activate_clock(aux::AuxClock::OscillatorControl);
        aux::AUX_CTL.activate_clock(aux::AuxClock::Semaphores);
    }

    pub fn switch_to_hf_xosc(&self) {
        self.configure();

        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            self.clock_source_set(ClockType::HF, HF_XOSC);
        }
    }

    pub fn clock_source_get(&self, clock: ClockType) -> u8 {
        let regs: &DdiRegisters = unsafe { &*self.regs };
        match clock {
            ClockType::LF => {
                ((regs.stat0.get() & LF_STAT0_MASK) >> 29) as u8
            },
            ClockType::HF => {
                ((regs.stat0.get() & HF_STAT0_MASK) >> 28) as u8
            },
        }
    }

    pub fn clock_source_set(&self, clock: ClockType, src: u8) {
        let regs: &DdiRegisters = unsafe { &*self.regs };
        match clock {
            ClockType::LF => {
                // Reset
                regs.ctl0.set(regs.ctl0.get() & !0xC);
                // Set
                let mask = ((src & 0x03) << 2) as u32;
                regs.ctl0.set(regs.ctl0.get() | mask);
            },
            ClockType::HF => {
                // Reset
                regs.ctl0.set(regs.ctl0.get() & !0x1);
                // Set
                let mask = (src & 0x01) as u32;
                regs.ctl0.set(regs.ctl0.get() | mask);
            }
        }
    }
}
