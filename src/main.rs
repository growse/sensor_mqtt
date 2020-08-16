extern crate confy;
#[macro_use]
extern crate log;

use bme280::{Measurements, BME280};
use confy::ConfyError;
use env_logger::Env;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use serde_derive::{Deserialize, Serialize};

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let maybe_config = load_config();

    if maybe_config.is_err() {
        error!(
            "Unable to load configuration: {}",
            maybe_config
                .err()
                .map_or("Nope".to_string(), |e: ConfyError| -> String {
                    e.to_string()
                })
        );
        return;
    }
    let config = maybe_config.unwrap();
    let measurement = read_bme280(config.i2c_bus_path);
    match measurement {
        Ok(temp) => info!("Temperature: {}", temp.temperature),
        Err(_) => error!("Could not read BME280"),
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

fn read_bme280(
    i2c_bus_path: String,
) -> Result<Measurements<LinuxI2CError>, bme280::Error<LinuxI2CError>> {
    debug!("Reading i2c bus at {}", i2c_bus_path.clone());
    let i2c_bus = I2cdev::new(i2c_bus_path).map_err(bme280::Error::I2c)?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init()?;
    let m = bme280.measure()?;
    Ok(m)
}

fn send_to_mqtt() -> Result<(), &'static str> {
    Err("Not implemented")
}

fn load_config() -> Result<Configuration, ConfyError> {
    let config: Configuration = confy::load("sensor_mqtt")?;
    Ok(config)
}
