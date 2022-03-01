/*
The bme280 crate doesn't implement Error for their errors, so we have to wrap
 */
use crate::configuration::Configuration;
use crate::MessageToPublish;
use bme280::{Measurements, BME280};
use core::fmt;

use anyhow::Result;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use std::error::Error;
use std::fmt::Formatter;

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

pub fn read_bme280(i2c_bus_path: &str) -> Result<Measurements<LinuxI2CError>> {
    debug!("Reading i2c bus at {}", i2c_bus_path.clone());
    let i2c_bus =
        I2cdev::new(i2c_bus_path).map_err(|e| BME280ErrorWrapper(bme280::Error::I2c(e)))?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init().map_err(BME280ErrorWrapper)?;
    let m = bme280.measure().map_err(BME280ErrorWrapper)?;
    Ok(m)
}

pub fn measurements_to_messages(
    measurements: Measurements<LinuxI2CError>,
    config: &Configuration,
) -> Result<Vec<MessageToPublish>> {
    let topic = format!(
        "{topic_base}/{hostname}/state",
        topic_base = config.mqtt_topic_base.as_str(),
        hostname = whoami::hostname()
    );
    let payload = serde_json::to_string(&measurements)?;
    Ok(vec![MessageToPublish {
        topic,
        payload,
        retain: false,
    }])
}
