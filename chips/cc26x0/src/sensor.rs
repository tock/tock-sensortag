use i2c;

pub const BUFFER_SIZE: usize = 32;

#[derive(Copy, Clone)]
pub struct Sensor {
    interface: i2c::I2cInterface,
    address: u8,
}

impl Sensor {
    pub const fn new(interface: i2c::I2cInterface, address: u8) -> Sensor {
        Sensor {
            interface,
            address,
        }
    }

    pub unsafe fn select(&self) {
        i2c::I2C0.select(self.interface, self.address);
    }

    pub unsafe fn deselect(&self) {
        i2c::I2C0.select(i2c::I2cInterface::Interface0, 0);
    }

    pub unsafe fn read(&self, buf: &mut [u8], len: u8) -> bool {
        i2c::I2C0.read(buf, len)
    }

    pub unsafe fn write(&self, buf: & [u8], len: u8) -> bool {
        i2c::I2C0.write(buf, len)
    }

    pub unsafe fn read_from_reg(&self, addr: u8, buf: &mut [u8], len: u8) -> bool {
        buf[0] = addr;
        i2c::I2C0.write_read(buf, 1, len)
    }

    pub unsafe fn write_to_reg(&self, addr: u8, buf: &mut [u8], len: u8) -> bool {
        if len == 0 {
            self.write_reg_address(addr)
        } else {
            let mut local_buf = [0; BUFFER_SIZE];
            local_buf[0] = addr;
            for i in 0..len {
                local_buf[(i + 1) as usize] = buf[i as usize];
            }
            i2c::I2C0.write(&mut local_buf, len + 1)
        }
    }

    pub unsafe fn write_reg_address(&self, addr: u8) -> bool {
        i2c::I2C0.write_single(addr)
    }

}
