use i2c::I2cInterface;
use core::cell::Cell;
use gpio;
use ioc;
use sensor::Sensor;
use kernel;

const TMP_CONF_REG: u8 = 0x02;
const TMP_RDY_PIN: u8 = 0x01;

const TMP_CONFIG_OFF: u16 = 0x0000;
const TMP_CONFIG_ON: u16 = 0x1000;

const TMP_INTERFACE: I2cInterface = I2cInterface::Interface0;
const TMP_ADDRESS: u8 = 0x44;

pub struct TMP {
    sensor: Cell<Sensor>,
}

impl TMP {
    pub unsafe fn new() -> TMP {
        TMP {
            sensor: Cell::new(Sensor::new(TMP_INTERFACE, TMP_ADDRESS)),
        }
    }

    pub unsafe fn init_hardware(&self) -> bool {
        gpio::PORT[TMP_RDY_PIN as usize].conf_as_input();

        ioc::IOCFG[TMP_RDY_PIN as usize].set_input_mode(kernel::hil::gpio::InputMode::PullUp);

        ioc::IOCFG[TMP_RDY_PIN as usize].set_hyst(true);

        self.enable_sensor(false)
    }

    pub unsafe fn enable_sensor(&self, enable: bool) -> bool {
        self.sensor.get().select();

        let val;
        if enable {
            val = TMP_CONFIG_ON;
        }
        else {
            val = TMP_CONFIG_OFF;
        }

        let mut buf = [0; 2];
        buf[0] = ((val & 0xFF00) >> 8) as u8;
        buf[1] = (val & 0xFF) as u8;

        let success = self.sensor.get().write_to_reg(TMP_CONF_REG, &mut buf, 2);

        self.sensor.get().deselect();

        success
    }

}
