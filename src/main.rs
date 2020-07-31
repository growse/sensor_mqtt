#[macro_use]
extern crate log;

use bme280::{Measurements, BME280};
use env_logger::Env;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let measurement = read_bme280();
    match measurement {
        Ok(temp) => info!("Temperature: {}", temp.temperature),
        Err(_) => error!("Could not read BME280"),
    }
}

fn read_bme280() -> Result<Measurements<LinuxI2CError>, bme280::Error<LinuxI2CError>> {
    let i2c_bus = I2cdev::new("/dev/i2c-1").map_err(bme280::Error::I2c)?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init()?;
    let m = bme280.measure()?;
    Ok(m)
}
