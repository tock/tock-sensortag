#![feature(asm, concat_idents, const_fn, const_cell_new, try_from)]
#![no_std]
#![crate_name = "cc26x0"]
#![crate_type = "rlib"]
extern crate cc26xx;
extern crate cortexm3;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;

#[macro_use]
extern crate bitfield;

pub mod aon;
pub mod chip;
pub mod crt1;
pub mod uart;
pub mod i2c;
pub mod sensor;
pub mod hdc;
pub mod aux_wuc;
pub mod radio;
pub mod timer;
pub mod osc;
pub mod ioc;
pub mod prcm;
pub mod rtc;
pub mod gpio;
pub mod udma;
pub mod tmp;

pub mod power;
pub mod peripherals;
pub mod power_manager;
pub mod peripheral_manager;

// Since the setup code is converted from C -> Rust, we
// ignore side effects from the conversion (unused vars & muts).
#[allow(unused, unused_mut)]
mod setup;

pub use crt1::init;
