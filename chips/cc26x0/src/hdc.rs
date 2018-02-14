use core::cell::Cell;
use kernel;

pub struct HDC {
    client: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
}

impl HDC {

}

impl kernel::hil::sensors::TemperatureDriver for HDC {
    fn read_temperature(&self) -> kernel::ReturnCode {
        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.client.set(Some(client));
    }
}
