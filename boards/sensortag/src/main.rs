#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib, asm)]

extern crate capsules;

extern crate cc26x0;
extern crate cc26xx;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

use cc26xx::{trng};
use cc26x0::{aon, radio, rtc, uart, gpio, power};

#[macro_use]
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
static mut PROCESSES: [Option<&'static mut kernel::Process<'static>>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        radio::ble::Ble,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, uart::UART>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
    >,
    rng: &'static capsules::rng::SimpleRng<'static, trng::Trng>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x0::init();

    // Setup AON event defaults
    aon::AON_EVENT.setup();

    // Setup power management and register all resources to be used
    power::init();

    // Power on peripheral domain and gpio clocks
    gpio::power_on_gpio();

    // LEDs
    let led_pins = static_init!(
        [(&'static gpio::GPIOPin, capsules::led::ActivationMode); 2],
        [
            (&gpio::PORT[10], capsules::led::ActivationMode::ActiveHigh), // Red
            (&gpio::PORT[15], capsules::led::ActivationMode::ActiveHigh)  // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (&gpio::PORT[0], capsules::button::GpioMode::LowWhenPressed), // Button 2
            (&gpio::PORT[4], capsules::button::GpioMode::LowWhenPressed)  // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    uart::UART0.set_pins(29, 28);
    let console = static_init!(
        capsules::console::Console<uart::UART>,
        capsules::console::Console::new(
            &uart::UART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    kernel::hil::uart::UART::set_client(&uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(console), kc);

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static gpio::GPIOPin; 26],
        [
            &gpio::PORT[1],
            &gpio::PORT[2],
            &gpio::PORT[3],
            &gpio::PORT[5],
            &gpio::PORT[6],
            &gpio::PORT[7],
            &gpio::PORT[8],
            &gpio::PORT[9],
            &gpio::PORT[11],
            &gpio::PORT[12],
            &gpio::PORT[13],
            &gpio::PORT[14],
            &gpio::PORT[16],
            &gpio::PORT[17],
            &gpio::PORT[18],
            &gpio::PORT[19],
            &gpio::PORT[20],
            &gpio::PORT[21],
            &gpio::PORT[22],
            &gpio::PORT[23],
            &gpio::PORT[24],
            &gpio::PORT[25],
            &gpio::PORT[26],
            &gpio::PORT[27],
            &gpio::PORT[30],
            &gpio::PORT[31]
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
    );
    virtual_alarm1.set_client(alarm);
    let ble_radio_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    trng::TRNG.enable();
    let rng = static_init!(
        capsules::rng::SimpleRng<'static, trng::Trng>,
        capsules::rng::SimpleRng::new(&trng::TRNG, kernel::Grant::create())
    );
    trng::TRNG.set_client(rng);

    // Use BLE
    radio::RFC.set_client(&radio::BLE);
    let ble_radio = static_init!(
        capsules::ble_advertising_driver::BLE<
            'static,
            radio::ble::Ble,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        >,
        capsules::ble_advertising_driver::BLE::new(
            &mut radio::BLE,
            kernel::Grant::create(),
            &mut capsules::ble_advertising_driver::BUF,
            ble_radio_virtual_alarm
        )
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
        &radio::BLE,
        ble_radio,
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
        &radio::BLE,
        ble_radio,
    );
    ble_radio_virtual_alarm.set_client(ble_radio);

    let sensortag = Platform {
        ble_radio,
        gpio,
        led,
        button,
        console,
        alarm,
        rng,
    };

    let mut chip = cc26x0::chip::Cc26x0::new();

    debug!("Initialization complete. Entering main loop\r");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    kernel::main(
        &sensortag,
        &mut chip,
        &mut PROCESSES,
        &kernel::ipc::IPC::new(),
    );
}
