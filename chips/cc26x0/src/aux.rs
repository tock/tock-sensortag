//! AUX management
//!
//! NOTE: as of now, the aux controller can only be used by one process at a time.

use kernel::common::regs::{ReadOnly,ReadWrite,WriteOnly};
use aon;

struct AuxWucRegisters {
    mod_clk_en0: ReadWrite<u32, ModClkEn0::Register>,
    pwr_off_req: WriteOnly<u32, PwrOffReq::Register>,
    pwr_dwn_req: WriteOnly<u32, PwrDwnReq::Register>,
    _pwr_dwn_ack: ReadOnly<u32>,

    _clk_lf_req: ReadOnly<u32>,
    _clk_lf_ack: ReadOnly<u32>,

    _res0: [u8; 0x10],

    _wu_evflags: ReadOnly<u32>,
    _wu_evclr: ReadOnly<u32>,

    _adc_clk_ctl: ReadOnly<u32>,
    _tdc_clk_ctl: ReadOnly<u32>,
    _ref_clk_ctl: ReadOnly<u32>,

    _rtc_subsec_inc0: ReadOnly<u32>,
    _rtc_subsec_inc1: ReadOnly<u32>,
    _rtc_subsec_inc_ctl: ReadOnly<k32>,

    mcu_bus_ctl: WriteOnly<u32, McuBusCtl::Register>,
    _mcu_bus_stat: ReadOnly<u32>,

    _aon_ctl_stat: ReadOnly<u32>,
    _aux_io_latch: ReadOnly<u32>,

    _res1: ReadOnly<u32>,

    _mod_clk_en1: ReadOnly<u32>,
}

register_bitfields![
    u32,
    ModClkEn0 [
        AUX_ADI4        OFFSET(7) NUMBITS(1) [], // Clock gate for AUX_ADI4
        AUX_DDI0_OSC    OFFSET(6) NUMBITS(1) [], // Clock gate for DDI0_OSC (Oscillator control)
        TDC             OFFSET(5) NUMBITS(1) [], // Clock gate for AUX_TDCIF
        ANAIF           OFFSET(4) NUMBITS(1) [], // Clock gate for AUX_ANAIF
        TIMER           OFFSET(3) NUMBITS(1) [], // Clock gate for AUX_TIMER
        AIODIO1         OFFSET(2) NUMBITS(1) [], // Clock gate for AUX_AIODIO1
        AIODIO0         OFFSET(1) NUMBITS(1) [], // Clock gate for AUX_AIODIO0
        SMPH            OFFSET(0) NUMBITS(1) []  // Clock gate for AUX_SMPH (Semaphore)
    ],
    PwrOffReq [
        REQ OFFSET(0) NUMBITS(1) []
    ],
    PwrDwnReq [
        REQ OFFSET(0) NUMBITS(1) []
    ],
    McuBusCtl [
        DISCONNECT_REQ OFFSET(0) NUMBITS(1) []
    ]
];

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
                aux_regs.mod_clk_en0.modify(ModClkEn0::AUX_DDI0_OSC::SET);
            },
            AuxClock::Semaphores => {
                aux_regs.mod_clk_en0.modify(ModClkEn0::SMPH::SET);
            }
        }
    }

    pub fn clock_is_active(&self, clock: AuxClock) -> bool {
        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };
        match clock {
            AuxClock::OscillatorControl => {
                aux_regs.mod_clk_en0.is_set(ModClkEn0::AUX_DDI0_OSC)
            },
            AuxClock::Semaphores => {
                aux_regs.mod_clk_en0.is_set(ModClkEn0::SMPH)
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

        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };
        // Make a power down request
        aux_regs.pwr_dwn_req.write(PwrDwnReq::REQ::SET);
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

    pub fn disconnect_bus(&self) {
        let aux_regs: &AuxWucRegisters = unsafe { &*self.aux_regs };
        // Request a bus disconnection
        aux_regs.mcu_bus_ctl.write(McuBusCtl::DISCONNECT_REQ::SET);
    }
}
