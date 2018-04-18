//! AUX management
//!
//! NOTE: as of now, the aux controller can only be used by one process at a time.

use kernel::common::VolatileCell;
use aon;

struct AuxWucRegisters {
    mod_clk_en0: VolatileCell<u32>,
    pwr_off_req: VolatileCell<u32>,
    _pwr_dwn_req: VolatileCell<u32>,
    _pwr_dwn_ack: VolatileCell<u32>,

    _clk_lf_req: VolatileCell<u32>,
    _clk_lf_ack: VolatileCell<u32>,

    _res0: [u8; 0x10],

    _wu_evflags: VolatileCell<u32>,
    _wu_evclr: VolatileCell<u32>,

    _adc_clk_ctl: VolatileCell<u32>,
    _tdc_clk_ctl: VolatileCell<u32>,
    _ref_clk_ctl: VolatileCell<u32>,

    _rtc_subsec_inc0: VolatileCell<u32>,
    _rtc_subsec_inc1: VolatileCell<u32>,
    _rtc_subsec_inc_ctl: VolatileCell<u32>,

    mcu_bus_ctl: VolatileCell<u32>,
    _mcu_bus_stat: VolatileCell<u32>,

    _aon_ctl_stat: VolatileCell<u32>,
    _aux_io_latch: VolatileCell<u32>,

    _res1: VolatileCell<u32>,

    _mod_clk_en1: VolatileCell<u32>,
}

pub struct Aux {
    aux_regs: *const AuxWucRegisters,
}

#[derive(PartialEq)]
pub enum WakeupMode {
    AllowSleep = 0x00,
    WakeUp = 0x01,
}

pub enum AuxClock {
    OscillatorControl = 0x01,
    Semaphores = 0x02,
}

pub const AUX_CTL: Aux = Aux::new();

impl Aux {
    pub const fn new() -> Aux {
        Aux {
            aux_regs: 0x400C_6000 as *const AuxWucRegisters,
        }
    }

    pub fn activate_clock(&self, clock: AuxClock) {
        self.power_up();

        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };
        match clock {
            AuxClock::OscillatorControl => {
                aux_regs.mod_clk_en0.set(aux_regs.mod_clk_en0.get() | 0x40);
            },
            AuxClock::Semaphores => {
                aux_regs.mod_clk_en0.set(aux_regs.mod_clk_en0.get() | 0x1);
            }
        }
    }

    pub fn clock_is_active(&self, clock: AuxClock) -> bool {
        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };
        match clock {
            AuxClock::OscillatorControl => {
                (aux_regs.mod_clk_en0.get() & 0x40) != 0
            },
            AuxClock::Semaphores => {
                (aux_regs.mod_clk_en0.get() & 0x1) != 0
            }
        }
    }

    pub fn power_up(&self) {
        if self.power_status() == WakeupMode::WakeUp {
            return
        }

        // Force the AUX to wake up
        self.wakeup_event(WakeupMode::WakeUp);
        // Wait for it to power up
        while self.power_status() != WakeupMode::WakeUp { }
    }

    pub fn power_down(&self) {
        if self.power_status() == WakeupMode::AllowSleep {
            return
        }

        // Disable SRAM retention of the aux
        aon::AON.aux_set_ram_retention(false);

        self.wakeup_event(WakeupMode::AllowSleep);

        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };

        // Make a power off request and disconnect the bus
        aux_regs.pwr_off_req.set(1);
        aux_regs.mcu_bus_ctl.set(1);

        while self.power_status() != WakeupMode::AllowSleep { }
    }

    pub fn wakeup_event(&self, mode: WakeupMode) {
        match mode {
            WakeupMode::AllowSleep => aon::AON.aux_wakeup(false),
            WakeupMode::WakeUp => aon::AON.aux_wakeup(true)
        }
    }

    pub fn power_status(&self) -> WakeupMode {
        if aon::AON.aux_is_on() {
            WakeupMode::WakeUp
        } else {
            WakeupMode::AllowSleep
        }
    }
}
