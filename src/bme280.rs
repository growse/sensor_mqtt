/*
The bme280 crate doesn't implement Error for their errors, so we have to wrap
 */
use crate::configuration::Configuration;
use crate::MessageToPublish;
use core::fmt;

use anyhow::{anyhow, Result};
use bme280::i2c::BME280;
use bme280::Measurements;
use linux_embedded_hal::{Delay, I2CError, I2cdev};
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct BME280ErrorWrapper(pub bme280::Error<I2CError>);

impl fmt::Display for BME280ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            bme280::Error::InvalidData => write!(f, "Invalid Data"),
            bme280::Error::CompensationFailed => write!(f, "Compensation Failed"),
            bme280::Error::NoCalibrationData => write!(f, "No Calibration Data"),
            bme280::Error::UnsupportedChip => write!(f, "Unsupported Chip"),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl Error for BME280ErrorWrapper {}

pub async fn read_bme280(i2c_bus_path: &str) -> Result<Measurements<I2CError>> {
    debug!("Reading i2c bus at {i2c_bus_path}");
    let i2c_bus = I2cdev::new(i2c_bus_path)?;
    let mut bme280 = BME280::new_primary(i2c_bus);
    let mut delay = Delay {};
    bme280.init(&mut delay).map_err(BME280ErrorWrapper)?;
    let m = bme280.measure(&mut delay).map_err(BME280ErrorWrapper)?;
    Ok(m)
}

pub async fn measurements_to_messages(
    measurements: Measurements<I2CError>,
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
