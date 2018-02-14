use i2c;

pub static mut BUFFER: [u8; 32] = [0; 32];

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

