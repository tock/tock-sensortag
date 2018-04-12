use uart;
use gpio;
use power_manager::{Manager, PoweredPeripheral};

pub static mut PM: Manager<'static> = Manager::new();

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
