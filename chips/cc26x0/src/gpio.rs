//! General Purpose Input Output (GPIO)
//!
//! For details see p.987 in the cc2650 technical reference manual.
//!
//! Configures the GPIO pins, and interfaces with the HIL for gpio.

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::regs::{ReadWrite, WriteOnly};
use kernel::hil::gpio::Pin;
use kernel::hil;
use prcm;
use ioc;

const NUM_PINS: usize = 32;
const GPIO_BASE: *const GpioRegisters = 0x4002_2000 as *const GpioRegisters;

#[repr(C)]
pub struct GpioRegisters {
    _reserved0: [u8; 0x80],
    pub dout_31_0: ReadWrite<u32>,
    _reserved1: [u8; 0xC],
    pub dout_set: WriteOnly<u32>,
    _reserved2: [u8; 0xC],
    pub dout_clr: WriteOnly<u32>,
    _reserved3: [u8; 0xC],
    pub dout_tgl: WriteOnly<u32>,
    _reserved4: [u8; 0xC],
    pub din: ReadWrite<u32>,
    _reserved5: [u8; 0xC],
    pub doe: ReadWrite<u32>,
    _reserved6: [u8; 0xC],
    pub evflags: ReadWrite<u32>,
}

use kernel::hil::gpio::PinCtl;
pub unsafe fn power_on_gpio() {
    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);
    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}
    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();
    // Set all pins in a low-power state
    set_pins_to_default_conf();
}

pub unsafe fn set_pins_to_default_conf() {
    const MIC_POWER: usize = 13;
    const AUDIO_DI: usize = 2;
    const AUDIO_CLK: usize = 11;
    const DP2: usize = 23;
    const DP1: usize = 24;
    const DP0: usize = 25;
    const DP3: usize = 27;
    const DEVPK_ID: usize = 30;
    const UART_TX: usize = 29;
    const SDA_HP: usize = 8;
    const SCL_HP: usize = 9;
    const SDA: usize = 5;
    const SCL: usize = 6;
    const SPI_MOSI: usize = 19;
    const SPI_CLK_FLASH: usize = 17;
    const TMP_RDY: usize = 1;

    for pin in PORT.pins.iter() {
        pin.disable();
    }

    PORT[TMP_RDY].make_input();
    PORT[TMP_RDY].set_input_mode(hil::gpio::InputMode::PullUp);

    PORT[SPI_MOSI].make_input();
    PORT[SPI_MOSI].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[SPI_CLK_FLASH].make_input();
    PORT[SPI_CLK_FLASH].set_input_mode(hil::gpio::InputMode::PullDown);

    PORT[SDA_HP].make_input();
    PORT[SDA_HP].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[SCL_HP].make_input();
    PORT[SCL_HP].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[SDA].make_input();
    PORT[SDA].set_input_mode(hil::gpio::InputMode::PullUp);
    PORT[SCL].make_input();
    PORT[SCL].set_input_mode(hil::gpio::InputMode::PullUp);

    PORT[DP0].make_input();
    PORT[DP0].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[DP1].make_input();
    PORT[DP1].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[DP2].make_input();
    PORT[DP2].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[DP3].make_input();
    PORT[DP3].set_input_mode(hil::gpio::InputMode::PullDown);

    PORT[DEVPK_ID].make_input();
    PORT[DEVPK_ID].set_input_mode(hil::gpio::InputMode::PullUp);

    PORT[MIC_POWER].make_output();
    PORT[MIC_POWER].clear();
    PORT[AUDIO_DI].make_input();
    PORT[AUDIO_DI].set_input_mode(hil::gpio::InputMode::PullDown);
    PORT[AUDIO_CLK].make_input();
    PORT[AUDIO_CLK].set_input_mode(hil::gpio::InputMode::PullDown);

    PORT[UART_TX].make_input();
    PORT[UART_TX].set_input_mode(hil::gpio::InputMode::PullDown);
}

pub struct GPIOPin {
    regs: *const GpioRegisters,
    pin: usize,
    pin_mask: u32,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(pin: usize) -> GPIOPin {
        GPIOPin {
            regs: GPIO_BASE,
            pin: pin,
            pin_mask: 1 << (pin % NUM_PINS),
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    fn enable_gpio(&self) {
        ioc::IOCFG[self.pin].enable_gpio();
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }

    pub fn iocfg(&self) -> &ioc::IocfgPin {
        &ioc::IOCFG[self.pin]
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        ioc::IOCFG[self.pin].set_input_mode(mode);
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        self.enable_gpio();
        // Disable input in the io configuration
        ioc::IOCFG[self.pin].enable_output();
        // Enable data output
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.doe.set(regs.doe.get() | self.pin_mask);
    }

    fn make_input(&self) {
        self.enable_gpio();
        ioc::IOCFG[self.pin].enable_input();
        // Disable data output
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.doe.set(regs.doe.get() & !self.pin_mask);
    }

    fn disable(&self) {
        ioc::IOCFG[self.pin].low_leakage_mode();
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.doe.set(regs.doe.get() & !self.pin_mask);
    }

    fn set(&self) {
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.dout_set.set(self.pin_mask);
    }

    fn clear(&self) {
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.dout_clr.set(self.pin_mask);
    }

    fn toggle(&self) {
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.dout_tgl.set(self.pin_mask);
    }

    fn read(&self) -> bool {
        let regs: &GpioRegisters = unsafe { &*self.regs };
        regs.din.get() & self.pin_mask != 0
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        self.client_data.set(client_data);
        ioc::IOCFG[self.pin].enable_interrupt(mode);
    }

    fn disable_interrupt(&self) {
        ioc::IOCFG[self.pin].disable_interrupt();
    }
}

pub struct Port {
    pins: [GPIOPin; NUM_PINS],
}

impl Index<usize> for Port {
    type Output = GPIOPin;

    fn index(&self, index: usize) -> &GPIOPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GPIOPin {
        &mut self.pins[index]
    }
}

impl Port {
    pub fn handle_interrupt(&self) {
        let regs: &GpioRegisters = unsafe { &*GPIO_BASE };
        let evflags = regs.evflags.get();
        // Clear all interrupts by setting their bits to 1 in evflags
        regs.evflags.set(evflags);

        // evflags indicate which pins has triggered an interrupt,
        // we need to call the respective handler for positive bit in evflags.
        let mut pin: usize = usize::max_value();
        while pin < self.pins.len() {
            pin = evflags.trailing_zeros() as usize;
            if pin >= self.pins.len() {
                break;
            }

            self.pins[pin].handle_interrupt();
        }
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GPIOPin::new(0),
        GPIOPin::new(1),
        GPIOPin::new(2),
        GPIOPin::new(3),
        GPIOPin::new(4),
        GPIOPin::new(5),
        GPIOPin::new(6),
        GPIOPin::new(7),
        GPIOPin::new(8),
        GPIOPin::new(9),
        GPIOPin::new(10),
        GPIOPin::new(11),
        GPIOPin::new(12),
        GPIOPin::new(13),
        GPIOPin::new(14),
        GPIOPin::new(15),
        GPIOPin::new(16),
        GPIOPin::new(17),
        GPIOPin::new(18),
        GPIOPin::new(19),
        GPIOPin::new(20),
        GPIOPin::new(21),
        GPIOPin::new(22),
        GPIOPin::new(23),
        GPIOPin::new(24),
        GPIOPin::new(25),
        GPIOPin::new(26),
        GPIOPin::new(27),
        GPIOPin::new(28),
        GPIOPin::new(29),
        GPIOPin::new(30),
        GPIOPin::new(31),
    ],
};
