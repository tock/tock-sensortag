use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite};
use prcm;

#[repr(C)]
pub struct Registers {
    pub cfg: ReadWrite<u32, Configuration::Register>,
    pub tamr: ReadWrite<u32, TimerAMode::Register>,
    pub tbmr: ReadWrite<u32>,
    pub ctl: ReadWrite<u32, Control::Register>,
    pub sync: ReadWrite<u32>,

    _reserved0: [u8; 0x4],

    pub imr: ReadWrite<u32, InterruptMask::Register>,
    pub ris: ReadOnly<u32>,
    pub mis: ReadOnly<u32, MaskedInterruptStatus::Register>,
    pub iclr: ReadWrite<u32, InterruptClear::Register>,
    pub tailr: ReadWrite<u32>,
    pub tbilr: ReadWrite<u32>,
    pub tamatchr: ReadWrite<u32>,
    pub tbmatchr: ReadWrite<u32>,
    pub tapr: ReadWrite<u32>,
    pub tbpr: ReadWrite<u32>,
    pub tapmr: ReadWrite<u32>,
    pub tbpmr: ReadWrite<u32>,
    pub tar: ReadOnly<u32>,
    pub tbr: ReadOnly<u32>,
    pub tav: ReadWrite<u32>,
    pub tbv: ReadWrite<u32>,

    _reserved1: [u8; 0x4],

    pub taps: ReadOnly<u32>,
    pub tbps: ReadOnly<u32>,
    pub tapv: ReadOnly<u32>,
    pub tbpv: ReadOnly<u32>,
    pub dmaev: ReadWrite<u32>,

    _reserved2: [u8; 0xF40],

    pub version: ReadOnly<u32>,
    pub andccp: ReadWrite<u32>,
}

register_bitfields![
    u32,
    Control [
        TAEN OFFSET(0) NUMBITS(1) [],
        TBEN OFFSET(8) NUMBITS(1) []
    ],
    Configuration [
        CFG OFFSET(0) NUMBITS(3) [
            timer32Bit = 0x0,
            timer64Bit = 0x4
        ]
    ],
    TimerAMode [
        TAMR OFFSET(0) NUMBITS(2) [
            OneShot = 0x1,
            Periodic = 0x2,
            Capture = 0x3
        ]
    ],
    InterruptMask [
        TATOIM OFFSET(0) NUMBITS(1) [],
        TBTOIM OFFSET(8) NUMBITS(1) []
    ],
    MaskedInterruptStatus [
        TATOMIS OFFSET(0) NUMBITS(1) [],
        TBTOMIS OFFSET(8) NUMBITS(1) []
    ],
    InterruptClear [
        TATOCINT OFFSET(0) NUMBITS(1) [],
        TBTOCINT OFFSET(8) NUMBITS(1) []
    ]
];

pub const GPT_CFG_32_BIT: u32 = 0x0;
pub const GPT_ONE_SHOT: u32 = 0x1;
pub const GPT_REG_BIT: u32 = 0x1;

#[derive(Copy, Clone, PartialEq)]
pub enum TimerBase {
    GPT0 = 0x4001_0000,
    GPT1 = 0x4001_1000,
    GPT2 = 0x4001_2000,
    GPT3 = 0x4001_3000,
}

pub static mut GPT0: Timer = Timer::new(TimerBase::GPT0);
pub static mut GPT1: Timer = Timer::new(TimerBase::GPT1);
pub static mut GPT2: Timer = Timer::new(TimerBase::GPT2);
pub static mut GPT3: Timer = Timer::new(TimerBase::GPT3);

pub struct Timer {
    regs: *const Registers,
    client: Cell<Option<&'static TimerClient>>,
}

trait TimerClient {
    fn fired(&self);
}

pub fn power_on_timers() {
    prcm::Power::enable_domain(prcm::PowerDomain::Serial);
    while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
    prcm::Clock::enable_gpt();
}

impl Timer {
    const fn new(gpt_base: TimerBase) -> Timer {
        Timer {
            regs: (gpt_base as u32) as *const Registers,
            client: Cell::new(None),
        }
    }

    pub fn one_shot(&self, value: u32) {
        let regs: &Registers = unsafe { &*self.regs };

        // Disable timer before configuration
        regs.ctl.modify(Control::TAEN::CLEAR);
        regs.cfg.write(Configuration::CFG::timer64Bit);

        // Set type and initial timer value
        regs.tamr.write(TimerAMode::TAMR::OneShot);
        regs.tailr.set(value);

        // Enable interrupts and start the timer
        regs.imr.write(InterruptMask::TATOIM::SET);
        regs.ctl.modify(Control::TAEN::SET);
    }

    pub fn has_fired(&self) -> bool {
        let regs: &Registers = unsafe { &*self.regs };
        regs.mis.is_set(MaskedInterruptStatus::TATOMIS)
    }

    pub fn handle_interrupt(&self) {
        let regs: &Registers = unsafe { &*self.regs };
        regs.iclr.modify(InterruptClear::TATOCINT::SET);
        self.client.get().map(|client| {
            client.fired();
        });
    }
}
