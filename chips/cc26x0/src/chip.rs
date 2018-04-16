use cortexm3::{self, nvic};
use cc26xx::gpio;
use cc26xx::peripheral_interrupts::*;
use kernel::common::regs::ReadWrite;

const X0_RF_CPE1: u32 = 2;
const X0_RF_CPE0: u32 = 9;
const X0_RF_CMD_ACK: u32 = 11;

use radio;
use timer;
use uart;
use kernel;
use rtc;
use prcm;
use kernel::support;
use peripherals;
use aux;

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

pub struct SystemControlRegisters {
    scr: ReadWrite<u32, SystemControl::Register>,
}

register_bitfields![
    u32,
    SystemControl [
        SLEEP_ON_EXIT   OFFSET(1)   NUMBITS(1) [], // Go to sleep after ISR
        SLEEP_DEEP      OFFSET(1)   NUMBITS(1) [], // Enable deep sleep
        SEVONPEND       OFFSET(1)   NUMBITS(1) []  // Wake up on all events (even disabled interrupts)
    ]
];

pub struct Cc26x0 {
    mpu: (),
    systick: cortexm3::systick::SysTick,
    sys_ctrl_regs: *const SystemControlRegisters,
}

const SYS_CTRL_BASE: u32 = 0xE000ED10;

impl Cc26x0 {
    pub unsafe fn new() -> Cc26x0 {
        Cc26x0 {
            mpu: (),
            // The systick clocks with 48MHz by default
            systick: cortexm3::systick::SysTick::new_with_calibration(48 * 1000000),
            sys_ctrl_regs: SYS_CTRL_BASE as *const SystemControlRegisters,
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
        unsafe {
            let sleep_mode: SleepMode = SleepMode::from(peripherals::M.lowest_sleep_mode());
            let regs = &*self.sys_ctrl_regs;

            match sleep_mode {
                SleepMode::DeepSleep => {
                    peripherals::M.before_sleep(sleep_mode as u32);

                    // Power down the AUX power
                    aux::AUX_CTL.power_down();

                    // Power down the MCU, it is no longer needed
                    // until we wake up again.
                    prcm::Power::disable_domain(prcm::PowerDomain::CPU);
                    prcm::mcu_power_down();

                    // Set the deep sleep bit
                    regs.scr.modify(SystemControl::SLEEP_DEEP::SET);

                    // TODO(cpluss): perform power down of regions & clocks
                    // TODO(cpluss): check VIMS mode, it needs to be OFF
                },
                _ => ()
            }

            support::wfi();

            match sleep_mode {
                SleepMode::DeepSleep => {
                    peripherals::M.after_wakeup(sleep_mode as u32);

                    // Clear the deep sleep bit
                    regs.scr.modify(SystemControl::SLEEP_DEEP::CLEAR);

                    prcm::Power::enable_domain(prcm::PowerDomain::CPU);

                    // TODO(cpluss): perform power up of regions & clocks
                },
                _ => ()
            }
        }
    }
}
