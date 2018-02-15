use cortexm3::{self, nvic};
use cc26xx::{gpio,rtc,timer};
use cc26xx::peripheral_interrupts::*;
use radio;

use uart;
use kernel;

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

                    RF_CMD_ACK => radio::RFC.handle_interrupt(radio::RfcInterrupt::CmdAck),

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
}
