use core::fmt::{write, Arguments, Write};
use kernel::hil::uart::{self, UART};
use kernel::hil::gpio::Pin;
use cc26xx;
use cc26x0;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut cc26x0::uart::UART0 };
        if !self.initialized {
            self.initialized = true;
            uart.init(uart::UARTParams {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
        }
        for c in s.bytes() {
            //uart.send_byte(c);
            //while !uart.tx_ready() {}
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
        ($($arg:tt)*) => (
            {
                use core::fmt::write;
                let writer = &mut $crate::io::WRITER;
                let _ = write(writer, format_args!($($arg)*));
            }
        );
}

#[macro_export]
macro_rules! println {
        ($fmt:expr) => (print!(concat!($fmt, "\n")));
            ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(
    _args: Arguments,
    _file: &'static str,
    _line: usize,
) -> ! {
    let writer = &mut WRITER;
    let _ = writer.write_fmt(format_args!(
        "\r\nKernel panic at {}:{}:\r\n\t\"",
        _file, _line
    ));
    let _ = write(writer, _args);
    let _ = writer.write_str("\"\r\n");

    let led0 = &cc26xx::gpio::PORT[10]; // Red led
    let led1 = &cc26xx::gpio::PORT[15]; // Green led

    led0.make_output();
    led1.make_output();
    loop {
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..100000 {
            led0.set();
            led1.set();
        }
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..500000 {
            led0.set();
            led1.set();
        }
    }
}
