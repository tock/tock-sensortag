//! Radio module for CC2650 SensorTag
//!
//! The radio works by communication to an external Cortex-M0 Radio MCU which handles
//! all logic and transmissions. The radio mcu has the capability to wake-up the main chip
//! when traffic is detected, among other things.

pub mod rfc;

pub const RFC: rfc::RFCore = rfc::RFCore::new();

