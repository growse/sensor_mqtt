extern crate confy;
#[macro_use]
extern crate log;

use core::fmt;
use std::error::Error;

use bme280::{Measurements, BME280};
use env_logger::Env;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};

use rumqttc::{Client, Incoming, MqttOptions, Outgoing, QoS};
use serde::export::Formatter;
use serde_derive::{Deserialize, Serialize};
use std::process::exit;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Configuration {
    i2c_bus_path: String,
    mqtt_host: String,
    mqtt_port: u16,
    mqtt_username: String,
    mqtt_password: String,
    mqtt_topic_base: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            i2c_bus_path: "/dev/i2c-1".to_string(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_username: "".to_string(),
            mqtt_password: "".to_string(),
            mqtt_topic_base: "sensors".to_string(),
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

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = load_config().unwrap_or_else(|e| {
        error!("Unable to load config: {}", e);
        exit(2)
    });

    let result = read_bme280(&config.i2c_bus_path)
        .and_then(|measurements| send_measurements_to_mqtt(measurements, &config));

    match result {
        Ok(_) => info!("GREAT SUCCESS"),
        Err(e) => error!("{:?}", e),
    }
}

fn read_bme280(i2c_bus_path: &str) -> Result<Measurements<LinuxI2CError>, Box<dyn Error>> {
    debug!("Reading i2c bus at {}", i2c_bus_path.clone());

    let i2c_bus =
        I2cdev::new(i2c_bus_path).map_err(|e| BME280ErrorWrapper(bme280::Error::I2c(e)))?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init().map_err(BME280ErrorWrapper)?;
    let m = bme280.measure().map_err(BME280ErrorWrapper)?;
    Ok(m)
}

fn send_measurements_to_mqtt(
    measurements: Measurements<LinuxI2CError>,
    config: &Configuration,
) -> Result<(), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new(
        "sensor-mqtt-client",
        config.mqtt_host.as_str(),
        config.mqtt_port,
    );

    mqtt_options.set_credentials(config.mqtt_username.as_str(), config.mqtt_password.as_str());

    let (mut client, mut connection) = Client::new(mqtt_options, 10);

    let topic = format!(
        "{topic_base}/{hostname}/state",
        topic_base = config.mqtt_topic_base.as_str(),
        hostname = whoami::hostname()
    );
    let payload = serde_json::to_string(&measurements)?;

    client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())?;

    for (_i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(success_notification) => match success_notification {
                (None, Some(Outgoing::Publish(p))) => println!("Publishing MQTT... id={:?}", p),
                (Some(Incoming::Connected), None) => println!("MQTT Connected"),
                (Some(Incoming::PubAck(pub_ack)), None) => {
                    println!("MQTT published id={:?}", pub_ack.pkid);
                    break;
                }
                (None, Some(outgoing)) => debug!("MQTT: Sent outgoing {:?}", outgoing),
                (Some(incoming), None) => debug!("MQTT: Received incoming {:?}", incoming),
                (incoming, outgoing) => debug!("MQTT: Unknown {:?} {:?}", incoming, outgoing),
            },
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}

fn load_config() -> Result<Configuration, Box<dyn Error>> {
    let config: Configuration = confy::load("sensor_mqtt")?;
    Ok(config)
}
