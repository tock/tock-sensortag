use cortexm3::{self, nvic};
use cc26xx::gpio;
use cc26xx::peripheral_interrupts::*;

const X0_RF_CPE1: u32 = 2;
const X0_RF_CPE0: u32 = 9;
const X0_RF_CMD_ACK: u32 = 11;

use radio;
use timer;
use uart;
use kernel;
use rtc;
use kernel::support;
use peripherals;
use power;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum SleepMode {
    DeepSleep = 0,
    Sleep = 1,
    Active = 2,
}

impl From<u32> for SleepMode {
    fn from(n: u32) -> Self {
        match n {
            0 => SleepMode::DeepSleep,
            1 => SleepMode::Sleep,
            2 => SleepMode::Active,
            _ => unimplemented!()
        }
    }
}

pub struct Cc26x0 {
    mpu: (),
    systick: cortexm3::systick::SysTick,
}

impl Cc26x0 {
    pub unsafe fn new() -> Cc26x0 {
        Cc26x0 {
            mpu: (),
            // The systick clocks with 48MHz by default
            systick: cortexm3::systick::SysTick::new_with_calibration(48 * 1000000),
        }
    }
}

impl kernel::Chip for Cc26x0 {
    type MPU = ();
    type SysTick = cortexm3::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                match interrupt {
                    GPIO => gpio::PORT.handle_interrupt(),
                    AON_RTC => rtc::RTC.handle_interrupt(),

                    UART0 => uart::UART0.handle_interrupt(),

                    GPT0A => timer::GPT0.handle_interrupt(),
                    GPT0B => timer::GPT0.handle_interrupt(),
                    GPT1A => timer::GPT1.handle_interrupt(),
                    GPT1B => timer::GPT1.handle_interrupt(),
                    GPT2A => timer::GPT2.handle_interrupt(),
                    GPT2B => timer::GPT2.handle_interrupt(),
                    GPT3A => timer::GPT3.handle_interrupt(),
                    GPT3B => timer::GPT3.handle_interrupt(),

                    X0_RF_CMD_ACK => radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::CmdAck),
                    X0_RF_CPE0 => radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe0),
                    X0_RF_CPE1 => radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe1),

                    // AON Programmable interrupt
                    // We need to ignore JTAG events since some debuggers emit these
                    AON_PROG => (),
                    _ => panic!("unhandled interrupt {}", interrupt),
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {
        let sleep_mode: SleepMode = SleepMode::from(unsafe {
            peripherals::M.lowest_sleep_mode()
        });

        match sleep_mode {
            SleepMode::DeepSleep => unsafe {
                peripherals::M.before_sleep(sleep_mode as u32);
                power::prepare_deep_sleep();
            },
            _ => ()
        }

        unsafe { support::wfi() }

        match sleep_mode {
            SleepMode::DeepSleep => unsafe {
                power::prepare_wakeup();
                peripherals::M.after_wakeup(sleep_mode as u32);
            },
            _ => ()
        }
    }
}

