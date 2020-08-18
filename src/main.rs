extern crate confy;
#[macro_use]
extern crate log;

use bme280::{Measurements, BME280};
use confy::ConfyError;
use core::fmt;
use env_logger::Env;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use serde::export::Formatter;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let result = load_config()
        .and_then(|config| -> Result<Measurements<_>, Box<dyn Error>> {
            read_bme280(config.i2c_bus_path)
        })
        .and_then(send_to_mqtt);

    match result {
        Ok(_) => info!("GREAT SUCCESS"),
        Err(e) => error!("{}", e),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    i2c_bus_path: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            i2c_bus_path: "/dev/i2c-1".into(),
        }
    }
}

/*
The bme280 crate doesn't implement Error for their errors, so we have to wrap
 */
#[derive(Debug)]
struct BME280ErrorWrapper(bme280::Error<LinuxI2CError>);
impl fmt::Display for BME280ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            bme280::Error::InvalidData => write!(f, "Invalid Data"),
            _ => write!(f, "Erm"),
        }
    }
}
impl Error for BME280ErrorWrapper {}

fn read_bme280(i2c_bus_path: String) -> Result<Measurements<LinuxI2CError>, Box<dyn Error>> {
    debug!("Reading i2c bus at {}", i2c_bus_path.clone());

    let i2c_bus =
        I2cdev::new(i2c_bus_path).map_err(|e| BME280ErrorWrapper(bme280::Error::I2c(e)))?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init().map_err(BME280ErrorWrapper)?;
    let m = bme280.measure().map_err(BME280ErrorWrapper)?;
    Ok(m)
}

fn send_to_mqtt(measurements: Measurements<LinuxI2CError>) -> Result<(), Box<dyn Error>> {
    Err("Not implemented".into())
}

fn load_config() -> Result<Configuration, Box<dyn Error>> {
    let config: Configuration = confy::load("sensor_mqtt")?;
    Ok(config)
}
