use kernel::common::VolatileCell;

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

pub const GPT0_BASE: *mut Registers = 0x4001_0000 as *mut Registers;
pub const GPT1_BASE: *mut Registers = 0x4001_1000 as *mut Registers;
pub const GPT2_BASE: *mut Registers = 0x4001_2000 as *mut Registers;
pub const GPT3_BASE: *mut Registers = 0x4001_3000 as *mut Registers;

pub struct Timer {
   regs: *mut Registers,
}
