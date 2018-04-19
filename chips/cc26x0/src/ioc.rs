//! IO Configuration (IOC)
//!
//! For details see p. 984 in the cc2650 technical reference manual.
//!
//! Required to setup and configure IO pins to different modes - all depending on
//! usage (eg. UART, GPIO, etc). It is used internally.

use kernel::common::regs::ReadWrite;
use kernel::hil;

pub const IOC_PULL_CTL: u8 = 13;
pub const IOC_IE: u8 = 29;
pub const IOC_EDGE_DET: u8 = 16;
pub const IOC_EDGE_IRQ_EN: u8 = 18;

pub const IOC_UART0_RX_ID: u32 = 0xF;
pub const IOC_UART0_TX_ID: u32 = 0x10;
pub const IOC_I2C_MSSDA: u32 = 0xD;
pub const IOC_I2C_MSSCL: u32 = 0xE;

pub const IOC_IOMODE_OPEN_DRAIN_NORMAL: u32 = 0x4000000;
pub const IOC_HYST_ENABLE: u32 = 0x40000000;
pub const IOC_IOPULL_UP: u32 = 0x4000;

pub const IOC_CURRENT_MODE: u32 = 0xC00;
pub const IOC_DRIVE_STRENGTH: u32 = 0x300;

pub enum CurrentMode {
    Current2mA = 0x0,
    Current4mA = 0x400,
    Current8mA = 0x800,
}

pub enum DriveStrength {
    Auto = 0x0,
    Max = 0x300,
    Med = 0x200,
    Min = 0x100,
}



#[repr(C)]
pub struct IocRegisters {
    iocfg: [ReadWrite<u32, IoConfiguration::Register>; 32],
}

register_bitfields![
    u32,
    IoConfiguration [
        IE          OFFSET(29) NUMBITS(1) [], // Input Enable
        IO_MODE     OFFSET(24) NUMBITS(3) [],
        EDGE_IRQ_EN OFFSET(18) NUMBITS(1) [], // Interrupt enable
        EDGE_DET    OFFSET(16) NUMBITS(2) [
            None            = 0b00,
            NegativeEdge    = 0b01,
            PositiveEdge    = 0b10,
            EitherEdge      = 0b11
        ],
        PULL_CTL    OFFSET(13) NUMBITS(2) [
            PullDown = 0b01,
            PullUp   = 0b10,
            PullNone = 0b11
        ],
        PORT_ID     OFFSET(0) NUMBITS(6) [
            GPIO = 0x00
            // Add more as needed from datasheet p.1028
        ]
    ]
];

const IOC_BASE: *mut IocRegisters = 0x4008_1000 as *mut IocRegisters;

pub struct IocfgPin {
    pin: usize,
}

impl IocfgPin {
    const fn new(pin: u8) -> IocfgPin {
        IocfgPin { pin: pin as usize }
    }

    pub fn enable_gpio(&self) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        // In order to configure the pin for GPIO we need to clear
        // the lower 6 bits.
        pin_ioc.write(IoConfiguration::PORT_ID::GPIO);
    }

    pub fn enable_uart_rx(&self) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        pin_ioc.set(pin_ioc.get() | IOC_UART0_RX_ID);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_input();
    }

    pub fn enable_uart_tx(&self) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        pin_ioc.set(pin_ioc.get() | IOC_UART0_TX_ID);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_output();
    }

    pub fn enable_i2c_sda(&self) {
        self.set_input_mode(hil::gpio::InputMode::PullNone);

        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        pin_ioc.set(IOC_I2C_MSSDA | IOC_IOMODE_OPEN_DRAIN_NORMAL | IOC_IOPULL_UP); // This will reset previous config
        self.enable_input();
    }

    pub fn enable_i2c_scl(&self) {
        self.set_input_mode(hil::gpio::InputMode::PullNone);

        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        pin_ioc.set(IOC_I2C_MSSCL | IOC_IOMODE_OPEN_DRAIN_NORMAL | IOC_IOPULL_UP); // This will reset previous config
        self.enable_input();
    }

    pub fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        let field = match mode {
            hil::gpio::InputMode::PullDown => IoConfiguration::PULL_CTL::PullDown,
            hil::gpio::InputMode::PullUp => IoConfiguration::PULL_CTL::PullUp,
            hil::gpio::InputMode::PullNone => IoConfiguration::PULL_CTL::PullNone,
        };

        pin_ioc.modify(field);
    }

    pub fn set_hyst(&self, enable: bool) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        if enable {
            pin_ioc.set(pin_ioc.get() | IOC_HYST_ENABLE);
        }
        else {
            pin_ioc.set(pin_ioc.get() & !IOC_HYST_ENABLE);
        }
    }

    pub fn set_drv_strength(&self, current: CurrentMode, strength: DriveStrength) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        pin_ioc.set(pin_ioc.get() & !IOC_DRIVE_STRENGTH & !IOC_CURRENT_MODE);
        pin_ioc.set(pin_ioc.get() | (current as u32) | (strength as u32));
    }

    pub fn enable_output(&self) {
        // Enable by disabling input
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];
        pin_ioc.modify(IoConfiguration::IE::CLEAR);
    }

    pub fn enable_input(&self) {
        // Set IE (Input Enable) bit
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];
        pin_ioc.modify(IoConfiguration::IE::SET);
    }

    pub fn enable_interrupt(&self, mode: hil::gpio::InterruptMode) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];

        let ioc_edge_mode = match mode {
            hil::gpio::InterruptMode::FallingEdge => IoConfiguration::EDGE_DET::NegativeEdge,
            hil::gpio::InterruptMode::RisingEdge => IoConfiguration::EDGE_DET::PositiveEdge,
            hil::gpio::InterruptMode::EitherEdge => IoConfiguration::EDGE_DET::EitherEdge,
        };

        pin_ioc.modify(ioc_edge_mode + IoConfiguration::EDGE_IRQ_EN::SET);
    }

    pub fn disable_interrupt(&self) {
        let regs: &IocRegisters = unsafe { &*IOC_BASE };
        let pin_ioc = &regs.iocfg[self.pin];
        pin_ioc.modify(IoConfiguration::EDGE_IRQ_EN::CLEAR);
    }
}

pub static IOCFG: [IocfgPin; 32] = [
    IocfgPin::new(0),
    IocfgPin::new(1),
    IocfgPin::new(2),
    IocfgPin::new(3),
    IocfgPin::new(4),
    IocfgPin::new(5),
    IocfgPin::new(6),
    IocfgPin::new(7),
    IocfgPin::new(8),
    IocfgPin::new(9),
    IocfgPin::new(10),
    IocfgPin::new(11),
    IocfgPin::new(12),
    IocfgPin::new(13),
    IocfgPin::new(14),
    IocfgPin::new(15),
    IocfgPin::new(16),
    IocfgPin::new(17),
    IocfgPin::new(18),
    IocfgPin::new(19),
    IocfgPin::new(20),
    IocfgPin::new(21),
    IocfgPin::new(22),
    IocfgPin::new(23),
    IocfgPin::new(24),
    IocfgPin::new(25),
    IocfgPin::new(26),
    IocfgPin::new(27),
    IocfgPin::new(28),
    IocfgPin::new(29),
    IocfgPin::new(30),
    IocfgPin::new(31),
];
