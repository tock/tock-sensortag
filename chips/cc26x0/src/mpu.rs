use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;

pub const MPU_INTERFACE: I2cInterface = I2cInterface::Interface1;
pub const MPU_ADDRESS: u8 = 0x68;

// MPU registers
pub const MPU_SELF_TEST_X_GYRO: u8 =    0x00;
pub const MPU_SELF_TEST_Y_GYRO: u8 =    0x01;
pub const MPU_SELF_TEST_Z_GYRO: u8 =    0x02;
pub const MPU_SELF_TEST_X_ACCEL: u8 =   0x0D;
pub const MPU_SELF_TEST_Z_ACCEL: u8 =   0x0E;
pub const MPU_SELF_TEST_Y_ACCEL: u8 =   0x0F;
pub const MPU_XG_OFFSET_H: u8 =         0x13;
pub const MPU_XG_OFFSET_L: u8 =         0x14;
pub const MPU_YG_OFFSET_H: u8 =         0x15;
pub const MPU_YG_OFFSET_L: u8 =         0x16;
pub const MPU_ZG_OFFSET_H: u8 =         0x17;
pub const MPU_ZG_OFFSET_L: u8 =         0x18;
pub const MPU_SMPLRT_DIV: u8 =          0x19;
pub const MPU_CONFIG: u8 =              0x1A;
pub const MPU_GYRO_CONFIG: u8 =         0x1B;
pub const MPU_ACCEL_CONFIG: u8 =        0x1C;
pub const MPU_ACCEL_CONFIG_2: u8 =      0x1D;
pub const MPU_LP_ACCEL_ODR: u8 =        0x1E;
pub const MPU_WOM_THR: u8 =             0x1F;
pub const MPU_FIFO_EN: u8 =             0x23;
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
pub const MPU_SIGNAL_PATH_RESET: u8 =   0x68;
pub const MPU_ACCEL_INTEL_CTRL: u8 =    0x69;
pub const MPU_USER_CTRL: u8 =           0x6A;
pub const MPU_PWR_MGMT_1: u8 =          0x6B;
pub const MPU_PWR_MGMT_2: u8 =          0x6C;
pub const MPU_FIFO_COUNT_H: u8 =        0x72;
pub const MPU_FIFO_COUNT_L: u8 =        0x73;
pub const MPU_FIFO_R_W: u8 =            0x74;
pub const MPU_WHO_AM_I: u8 =            0x75;

pub struct MPU {
    sensor: Cell<Sensor>,
}

impl MPU {
    fn new() -> MPU {
        MPU {
            sensor: Cell::new(Sensor::new(MPU_INTERFACE, MPU_ADDRESS)),
        }
    }

    unsafe fn int_status(&self) -> u8 {
        self.sensor.get().select();
        let mut buf = [0, 1];
        self.sensor.get().read_from_reg(MPU_INT_STATUS, &mut buf, 1);
        self.sensor.get().deselect();
        buf[0]
    }
}