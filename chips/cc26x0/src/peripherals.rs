use uart;
use tmp;
use radio;
use peripheral_manager::{Peripheral, PeripheralManager};

pub static mut M: PeripheralManager = PeripheralManager::new();

static mut UART_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&uart::UART0) };

static mut TMP007_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&tmp::TMP007_SENSOR) };

static mut BLE_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&radio::BLE) };

pub unsafe fn init() {
    let peripherals = [&UART_PERIPHERAL, &TMP007_PERIPHERAL, &BLE_PERIPHERAL];

    for peripheral in peripherals.iter() {
        M.register_peripheral(peripheral);
    }
}
