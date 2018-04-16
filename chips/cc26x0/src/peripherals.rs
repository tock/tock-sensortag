use uart;
use peripheral_manager::{Peripheral,PeripheralManager};

pub static mut M: PeripheralManager = PeripheralManager::new();

static mut UART_PERIPHERAL: Peripheral<'static> = unsafe {
    Peripheral::new(&uart::UART0)
};

pub unsafe fn init() {
    M.register_peripheral(&UART_PERIPHERAL);
}