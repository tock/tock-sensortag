/// Power management
///
/// This is the power management module for the CC26X0. It wraps up
/// a power manager and connects it to each peripheral on the chip.
///
/// To request power for a certain peripheral
///     power::request(power::Peripherals::<peripheral>)
///
/// Then release access to it once you're done
///     power::release(power::Peripherals::<peripheral>)
///
/// This works multiple times, and it will toggle
/// the power as long as it is not used elsewhere at the same time.

use uart;
use gpio;
use power_manager::{PowerManager, PoweredPeripheral};

pub static mut PM: PowerManager<'static> = PowerManager::new();

#[repr(u32)]
pub enum Peripherals {
    UART = 0,
    GPIO = 1,
}

static mut UART0_PERIPHERAL: PoweredPeripheral<'static> = unsafe {
    PoweredPeripheral::new(&uart::UART0)
};
static mut GPIO_PERIPHERAL: PoweredPeripheral<'static> = PoweredPeripheral::new(&gpio::GPIO);

pub unsafe fn init() {
    PM.register(&GPIO_PERIPHERAL);
    PM.register(&UART0_PERIPHERAL);
}

pub fn request(peripheral: Peripherals) {
    unsafe {
        PM.request(peripheral as u32);
    }
}

pub fn release(peripheral: Peripherals) {
    unsafe {
        PM.release(peripheral as u32);
    }
}
