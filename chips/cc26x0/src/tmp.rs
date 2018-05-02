use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;

const TMP_INTERFACE: I2cInterface = I2cInterface::Interface0;
const TMP_ADDRESS: u8 = 0x44;
const TMP_CONF_REG: u8 = 0x02;

pub struct TMP {
    sensor: Cell<Sensor>,
}

impl TMP {
    pub unsafe fn new() -> TMP {
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
