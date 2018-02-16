use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;
use kernel;

pub const HDC_TEMP_REG: u32 = 0x00;
pub const HDC_CONF_REG: u32 = 0x02;

pub const HDC_CONFIG: u32 = 0x1000; // 14 bit resolution

pub const HDC_INTERFACE: I2cInterface = I2cInterface::Interface0;
pub const HDC_ADDRESS: u8 = 0x43;

pub struct HDC {
    sensor: Cell<Sensor>,
    client: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
}

impl HDC {
    pub unsafe fn new() -> HDC {
        HDC {
            sensor: Cell::new(Sensor::new(HDC_INTERFACE, HDC_ADDRESS)),
            client: Cell::new(None),
        }
    }

    pub unsafe fn read_temp(&self) -> u32 {
        self.sensor.get().select();
        let mut buf = [0; 2];

        // Write config to peripheral
        buf[0] = ((HDC_CONFIG & 0xFF00) >> 8) as u8;
        buf[1] = (HDC_CONFIG & 0xFF) as u8;
        self.sensor.get().write_to_reg(HDC_CONF_REG as u8, &mut buf, 2);

        // Start measurement by selecting temperature register
        self.sensor.get().write_reg_address(HDC_TEMP_REG as u8);

        // Delay to make sure the value is ready when reading
        for _ in 0..0xFFFFFF { asm!("NOP"); }

        // Read the temperature
        self.sensor.get().read(&mut buf, 2);

        let raw_temp = (buf[0] as u32) << 8 | (buf[1] as u32);
        self.convert_to_celsius(raw_temp)
    }

    fn convert_to_celsius(&self, raw_temp: u32) -> u32 {
        raw_temp * 165 / 65536 - 40
    }
}

impl kernel::hil::sensors::TemperatureDriver for HDC {
    fn read_temperature(&self) -> kernel::ReturnCode {
        unsafe {
            let temp = self.read_temp();
            self.client
                .get()
                .map(|client| client.callback(temp as usize));
            self.sensor.get().deselect();
        }
        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.client.set(Some(client));
    }
}
