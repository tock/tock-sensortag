use kernel::common::VolatileCell;
use core::cell::Cell;
use prcm;

#[repr(C)]
pub struct Registers {
   pub cfg: VolatileCell<u32>,
   pub tamr: VolatileCell<u32>,
   pub tbmr: VolatileCell<u32>,
   pub ctl: VolatileCell<u32>,
   pub sync: VolatileCell<u32>,

   _reserved0: [u8; 0x4],

   pub imr: VolatileCell<u32>,
   pub ris: VolatileCell<u32>,
   pub mis: VolatileCell<u32>,
   pub iclr: VolatileCell<u32>,
   pub tailr: VolatileCell<u32>,
   pub tbilr: VolatileCell<u32>,
   pub tamatchr: VolatileCell<u32>,
   pub tbmatchr: VolatileCell<u32>,
   pub tapr: VolatileCell<u32>,
   pub tbpr: VolatileCell<u32>,
   pub tapmr: VolatileCell<u32>,
   pub tbpmr: VolatileCell<u32>,
   pub tar: VolatileCell<u32>,
   pub tbr: VolatileCell<u32>,
   pub tav: VolatileCell<u32>,
   pub tbv: VolatileCell<u32>,

   _reserved1: [u8; 0x4],

   pub taps: VolatileCell<u32>,
   pub tbps: VolatileCell<u32>,
   pub tapv: VolatileCell<u32>,
   pub tbpv: VolatileCell<u32>,
   pub dmaev: VolatileCell<u32>,

   _reserved2: [u8; 0xF40],

   pub version: VolatileCell<u32>,
   pub andccp: VolatileCell<u32>,
}

pub const GPT_ONE_SHOT: u32 = 0x1;
pub const GPT_REG_BIT: u32 = 0x1;

pub struct Timer {
   regs: *const Registers,
   reg_bit: u32,
   client: Cell<Option<&'static TimerClient>>,
}

trait TimerClient {
   fn fired(&self);
}

impl Timer {
   pub const fn new(gpt_base: u32) -> Timer {
      Timer {
         regs: gpt_base as *const Registers,
         reg_bit: GPT_REG_BIT,
         client: Cell::new(None),
      }
   }

   pub fn init(&self) {
      prcm::Power::enable_domain(prcm::PowerDomain::Serial);
      while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) { }
      prcm::Clock::enable_gpt();
   }

   pub fn one_shot(&self, value: u32) {
      let regs: &Registers = unsafe { &*self.regs };
      regs.ctl.set(regs.ctl.get() & !self.reg_bit);
      regs.cfg.set(0);

      regs.tamr.set(GPT_ONE_SHOT);
      regs.tailr.set(value);

      regs.ctl.set(regs.ctl.get() | self.reg_bit);
   }

   pub fn has_fired(&self) -> bool {
     let regs: &Registers = unsafe { &*self.regs };
     (regs.ris.get() & self.reg_bit) != 0
   }

   pub fn handle_interrupt(&self) {
      let regs: &Registers = unsafe { &*self.regs };
      regs.iclr.set(regs.iclr.get() | self.reg_bit);
      self.client.get().map(|client| {
         client.fired();
      });
   }
}
