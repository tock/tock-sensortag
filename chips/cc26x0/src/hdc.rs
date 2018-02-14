use core::cell::Cell;
use i2c::I2cInterface;
use sensor::Sensor;
use kernel;

pub struct HDC {
    sensor: Cell<Sensor>,
    client: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
}

impl HDC {
    pub fn new(interface: I2cInterface, address: u8) -> HDC {
        HDC {
          sensor: Cell::new(Sensor::new(interface, address)),
          client: Cell::new(None),
        }
    }
}

impl kernel::hil::sensors::TemperatureDriver for HDC {
    fn read_temperature(&self) -> kernel::ReturnCode {
        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.client.set(Some(client));
    }
}
