use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;

use peripheral_manager::PowerClient;
use chip::SleepMode;

const TMP_INTERFACE: I2cInterface = I2cInterface::Interface0;
const TMP_ADDRESS: u8 = 0x44;
const TMP_CONF_REG: u8 = 0x02;

pub static mut TMP007_SENSOR: TMP = TMP::new();

pub struct TMP {
    sensor: Cell<Sensor>,
}

impl TMP {
    pub const fn new() -> TMP {
        TMP {
            sensor: Cell::new(Sensor::new(TMP_INTERFACE, TMP_ADDRESS)),
        }
    }

    pub unsafe fn disable_sensor(&self) {
        self.sensor.get().select();
        let mut buf = [0; 2];
        self.sensor.get().write_to_reg(TMP_CONF_REG, &mut buf, 2);
        self.sensor.get().deselect();
    }
}

impl PowerClient for TMP {
    fn before_sleep(&self, _sleep_mode: u32) {
        // Ensure that the sensor is disabled, since it
        // draws quite a lot of current by default.
        unsafe {
            self.disable_sensor();
        }
    }

    fn after_wakeup(&self, _sleep_mode: u32) {
        // We do not need to enable it at the moment
    }

    fn lowest_sleep_mode(&self) -> u32 {
        SleepMode::DeepSleep as u32
    }
}
