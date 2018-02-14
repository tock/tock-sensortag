use kernel::common::take_cell::TakeCell;
use i2c::I2cInterface;
use core::cell::Cell;
use sensor::Sensor;
use kernel;

pub const HDC_TEMP_REG: u32 = 0x00;
pub const HDC_CONF_REG: u32 = 0x02;

pub const HDC_CONFIG: u32 = 0x1000; // 14 bit resolution

pub const HDC_INTERFACE: I2cInterface = I2cInterface::Interface0;
pub const HDC_ADDRESS: u8 = 0x43;

static mut HDC_BUFFER: [u8; 32] = [0; 32];
static mut SENSOR: Sensor = Sensor::new(HDC_INTERFACE, HDC_ADDRESS);

pub struct HDC {
    sensor: TakeCell<'static, Sensor>,
    client: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
}

impl HDC {
    pub unsafe fn new() -> HDC {
        HDC {
          sensor: TakeCell::new(&mut SENSOR),
          client: Cell::new(None),
        }
    }

    fn convert_to_celsius(&self, raw_temp: u32) -> u32 {
        raw_temp * 165 / 65536 - 40
    }
}

impl kernel::hil::sensors::TemperatureDriver for HDC {
    fn read_temperature(&self) -> kernel::ReturnCode {
        unsafe {
            self.sensor.map(|sensor| {
                sensor.select();
            });

            // Write config to peripheral
            HDC_BUFFER[0] = ((HDC_CONFIG & 0xFF00) >> 8) as u8;
            HDC_BUFFER[1] = (HDC_CONFIG & 0xFF) as u8;
            self.sensor.map(|sensor| {
                sensor.write_reg(HDC_CONF_REG as u8, &mut HDC_BUFFER, 2);
            });

            // Start measurement by selecting temperature register
            self.sensor.map(|sensor| {
                sensor.write_reg(HDC_TEMP_REG as u8, &mut HDC_BUFFER, 0);
            });

            // Delay to make sure the value is ready when reading
            for _ in 0..0xFFFFFF { asm!("NOP"); }

            // Read the temperature
            self.sensor.map(|sensor| {
                sensor.read_reg(HDC_TEMP_REG as u8, &mut HDC_BUFFER, 2);
            });

            let raw_temp = (HDC_BUFFER[0] as u32) << 8 | (HDC_BUFFER[1] as u32);
            let temp = self.convert_to_celsius(raw_temp);

            self.client
                .get()
                .map(|client| client.callback(temp as usize));

            self.sensor.map(|sensor| {
                sensor.deselect();
            });
        }

        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.client.set(Some(client));
    }
}
