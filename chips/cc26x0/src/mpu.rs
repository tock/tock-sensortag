use kernel::hil::gpio::Pin;
use kernel::hil;
use ioc;
use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;
use cc26xx::gpio;

pub const MPU_INTERFACE: I2cInterface = I2cInterface::Interface1;
pub const MPU_ADDRESS: u8 = 0x68;

pub const MPU_POWER_IOID: u32 = 0xC;
pub const MPU_INT_IOID: u32 = 0x7;

pub const MPU_DEFAULT_ACC_RANGE: u8 = 0x11;
pub const MPU_DATA_SIZE: usize = 6;
pub const MPU_NUM_AXES: usize = 3;

// MPU registers
pub const MPU_CONFIG: u8 =              0x1A;
pub const MPU_GYRO_CONFIG: u8 =         0x1B;
pub const MPU_ACCEL_CONFIG: u8 =        0x1C;
pub const MPU_ACCEL_CONFIG_2: u8 =      0x1D;
pub const MPU_INT_PIN_CFG: u8 =         0x37;
pub const MPU_INT_ENABLE: u8 =          0x38;
pub const MPU_INT_STATUS: u8 =          0x3A;
pub const MPU_ACCEL_XOUT_H: u8 =        0x3B;
pub const MPU_ACCEL_XOUT_L: u8 =        0x3C;
pub const MPU_ACCEL_YOUT_H: u8 =        0x3D;
pub const MPU_ACCEL_YOUT_L: u8 =        0x3E;
pub const MPU_ACCEL_ZOUT_H: u8 =        0x3F;
pub const MPU_ACCEL_ZOUT_L: u8 =        0x40;
pub const MPU_TEMP_OUT_H: u8 =          0x41;
pub const MPU_TEMP_OUT_L: u8 =          0x42;
pub const MPU_GYRO_XOUT_H: u8 =         0x43;
pub const MPU_GYRO_XOUT_L: u8 =         0x44;
pub const MPU_GYRO_YOUT_H: u8 =         0x45;
pub const MPU_GYRO_YOUT_L: u8 =         0x46;
pub const MPU_GYRO_ZOUT_H: u8 =         0x47;
pub const MPU_GYRO_ZOUT_L: u8 =         0x48;
pub const MPU_USER_CTRL: u8 =           0x6A;
pub const MPU_PWR_MGMT_1: u8 =          0x6B;
pub const MPU_PWR_MGMT_2: u8 =          0x6C;

unsafe fn delay() {
    for _ in 0..0xFFFFF {
        asm!("NOP");
    }
}

pub struct MPU {
    sensor: Cell<Sensor>,
}

impl MPU {
    pub const fn new() -> MPU {
        MPU {
            sensor: Cell::new(Sensor::new(MPU_INTERFACE, MPU_ADDRESS)),
        }
    }

    pub unsafe fn data_ready(&self) -> bool {
        self.sensor.get().select();
        let mut buf = [0];
        self.sensor.get().read_from_reg(MPU_INT_STATUS, &mut buf, 1);
        self.sensor.get().deselect();
        buf[0] & 1 != 0
    }

    unsafe fn power_up(&self) {
        gpio::PORT[MPU_POWER_IOID as usize].set();
        delay();
    }

    pub unsafe fn read_from_acc(&self) -> [i16; MPU_NUM_AXES]{
        self.sensor.get().select();
        let mut buf = [0; MPU_DATA_SIZE];
        self.sensor.get().read_from_reg(MPU_ACCEL_XOUT_H, &mut buf, MPU_DATA_SIZE as u8);
        self.sensor.get().deselect();
        self.convert_vals(buf, 2048)
    }

    pub unsafe fn read_from_gyro(&self) -> [i16; MPU_NUM_AXES] {
        self.sensor.get().select();
        let mut buf = [0; MPU_DATA_SIZE];
        self.sensor.get().read_from_reg(MPU_GYRO_XOUT_H, &mut buf, MPU_DATA_SIZE as u8);
        self.sensor.get().deselect();
        self.convert_vals(buf, 131)
    }

    fn convert_vals(&self, vals: [u8; MPU_DATA_SIZE], div: i16) -> [i16; MPU_NUM_AXES]{
        let mut converted_vals = [0; MPU_NUM_AXES];
        for i in 0..MPU_NUM_AXES {
            let index = i * 2;
            let raw = (vals[index+1] as i16) << 8 | vals[index] as i16;
            converted_vals[i] = raw / div;
        }
        converted_vals
    }

    unsafe fn clear_interrupts(&self) {
        let mut buf = [0];
        self.sensor.get().read_from_reg(MPU_INT_STATUS, &mut buf, 1);
    }

    unsafe fn configure_pins(&self) {
        // Configure interrupt pin
        gpio::PORT[MPU_INT_IOID as usize].make_input();
        ioc::IOCFG[MPU_INT_IOID as usize].set_input_mode(hil::gpio::InputMode::PullDown);
        ioc::IOCFG[MPU_INT_IOID as usize].set_hyst(true);
        // Configure power pin
        gpio::PORT[MPU_POWER_IOID as usize].make_output();
        ioc::IOCFG[MPU_POWER_IOID as usize].set_drv_strength(ioc::CurrentMode::Current4mA, ioc::DriveStrength::Max);
        gpio::PORT[MPU_POWER_IOID as usize].clear();

    }

    pub unsafe fn configure(&self) {
        self.sensor.get().select();
        self.configure_pins();
        self.power_up();
        self.clear_interrupts();
        self.sensor.get().deselect();
    }
}