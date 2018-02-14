use i2c;
use core::cell::Cell;

static mut BUFFER: [u8; 32] = [0; 32];

pub struct Sensor {
    interface: Cell<i2c::I2cInterface>,
    address: Cell<u8>,
}

impl Sensor {
    pub const fn new(interface: i2c::I2cInterface, address: u8) -> Sensor {
        Sensor {
            interface: Cell::new(interface),
            address: Cell::new(address),
        }
    }

    pub fn select(&self) {
        unsafe { i2c::I2C0.select(self.interface.get(), self.address.get()); }
    }

    pub fn deselect(&self) {
        unsafe { i2c::I2C0.select(i2c::I2cInterface::Interface0, 0); }
    }

    pub fn read_reg(addr: u8, buf: &'static mut [u8], len: u8) -> bool {
        buf[0] = addr;
        unsafe { i2c::I2C0.write_read(buf, 1, len) }
    }

    pub fn write_reg(addr: u8, buf: &'static mut [u8], len: u8) -> bool {
        unsafe {
            BUFFER[0] = addr;
            for i in 1..len {
                BUFFER[i as usize] = buf[i as usize];
            }
            i2c::I2C0.write(&mut BUFFER, len + 1)
        }
    }
}


