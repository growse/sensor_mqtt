/*
The bme280 crate doesn't implement Error for their errors, so we have to wrap
 */
use core::fmt;
use std::error::Error;
use std::fmt::Formatter;

use anyhow::Result;
use bme280::{Measurements, BME280};
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};

use crate::configuration::Configuration;
use crate::sensors::sensor::Sensor;
use crate::MessageToPublish;

#[derive(Debug)]
pub struct BME280ErrorWrapper(pub bme280::Error<LinuxI2CError>);

impl fmt::Display for BME280ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            bme280::Error::InvalidData => write!(f, "Invalid Data"),
            bme280::Error::CompensationFailed => write!(f, "Compensation Failed"),
            bme280::Error::NoCalibrationData => write!(f, "No Calibration Data"),
            bme280::Error::UnsupportedChip => write!(f, "Unsupported Chip"),
            bme280::Error::I2c(e) => write!(f, "I2c error {}", e),
        }
    }
}

impl Error for BME280ErrorWrapper {}

struct BME280Sensor {
    i2c_bus_path: String,
}

impl Sensor for BME280Sensor {
    fn measure(&self) -> Result<crate::sensors::sensor::Measurements> {
        debug!("Reading i2c bus at {}", i2c_bus_path.clone());
        let i2c_bus =
            I2cdev::new(i2c_bus_path).map_err(|e| BME280ErrorWrapper(bme280::Error::I2c(e)))?;
        let mut bme280 = BME280::new_primary(i2c_bus, Delay);
        bme280.init().map_err(BME280ErrorWrapper)?;
        let m = bme280.measure().map_err(BME280ErrorWrapper)?;
        Ok(m)
    }
}

pub fn read_measurements(i2c_bus_path: &str) -> Result<Measurements<LinuxI2CError>> {}
