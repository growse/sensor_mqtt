extern crate confy;
#[macro_use]
extern crate log;

use core::fmt;
use serde_json::json;
use std::error::Error;
use std::process::exit;

use bme280::{Measurements, BME280};
use env_logger::Env;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use rumqttc::{Client, Incoming, MqttOptions, Outgoing, QoS};
use serde::export::Formatter;
use serde_derive::{Deserialize, Serialize};

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

struct MessageToPublish {
    topic: String,
    payload: String,
    retain: bool,
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = load_config().unwrap_or_else(|e| {
        error!("Unable to load config: {}", e);
        exit(2)
    });

    let result = read_bme280(&config.i2c_bus_path)
        .and_then(|measurements| map_measurements_to_messages(measurements, &config))
        .and_then(|measurement_messages| {
            get_homeassistant_discovery_messages().map(|mut messages| {
                messages.extend(measurement_messages);
                return messages;
            })
        })
        .and_then(|messages_to_publish| send_measurements_to_mqtt(messages_to_publish, &config));

    match result {
        Ok(_) => info!("GREAT SUCCESS"),
        Err(e) => error!("{:?}", e),
    }
}

fn map_measurements_to_messages(
    measurements: Measurements<LinuxI2CError>,
    config: &Configuration,
) -> Result<Vec<MessageToPublish>, Box<dyn Error>> {
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

fn read_bme280(i2c_bus_path: &str) -> Result<Measurements<LinuxI2CError>, Box<dyn Error>> {
    debug!("Reading i2c bus at {}", i2c_bus_path.clone());

    let i2c_bus =
        I2cdev::new(i2c_bus_path).map_err(|e| BME280ErrorWrapper(bme280::Error::I2c(e)))?;
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init().map_err(BME280ErrorWrapper)?;
    let m = bme280.measure().map_err(BME280ErrorWrapper)?;
    Ok(m)
}

fn get_homeassistant_discovery_messages() -> Result<Vec<MessageToPublish>, Box<dyn Error>> {
    let state_topic = format!("homeassistant/sensor/{}/state", whoami::hostname());
    Ok(vec![
        MessageToPublish {
            topic: format!(
                "homeassistant/sensor/{}_temperature/config",
                whoami::hostname()
            ),
            payload: json!({
            "device_class": "temperature",
            "name": "Temperature",
            "state_topic": state_topic,
            "unit_of_measurement": "°C",
            "value_template": "{{ value_json.temperature}}"
            })
            .to_string(),
            retain: true,
        },
        MessageToPublish {
            topic: format!(
                "homeassistant/sensor/{}_pressure/config",
                whoami::hostname()
            ),
            payload: json!({
            "device_class": "pressure",
            "name": "Pressure",
            "state_topic": state_topic,
            "unit_of_measurement": "hPa",
            "value_template": "{{ value_json.pressure}}"
            })
            .to_string(),
            retain: true,
        },
        MessageToPublish {
            topic: format!(
                "homeassistant/sensor/{}_humidity/config",
                whoami::hostname()
            ),
            payload: json!({
            "device_class": "humidity",
            "name": "Humidity",
            "state_topic": state_topic,
            "unit_of_measurement": "%",
            "value_template": "{{ value_json.humidity}}"
            })
            .to_string(),
            retain: true,
        },
    ])
}

fn send_measurements_to_mqtt(
    messages_to_publish: Vec<MessageToPublish>,
    config: &Configuration,
) -> Result<(), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new(
        "sensor-mqtt-client",
        config.mqtt_host.as_str(),
        config.mqtt_port,
    );
    mqtt_options.set_credentials(config.mqtt_username.as_str(), config.mqtt_password.as_str());
    let (mut client, mut connection) = Client::new(mqtt_options, 10);

    let mut pending_messages = messages_to_publish.len();
    debug!("Publishing {:?} messages", pending_messages);

    for x in messages_to_publish {
        client.publish(x.topic, QoS::AtLeastOnce, x.retain, x.payload.as_bytes())?;
    }

    for (_i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(success_notification) => match success_notification {
                (None, Some(Outgoing::Publish(p))) => println!("Publishing MQTT... id={:?}", p),
                (Some(Incoming::Connected), None) => println!("MQTT Connected"),
                (Some(Incoming::PubAck(pub_ack)), None) => {
                    debug!("MQTT published id={:?}", pub_ack.pkid);
                    pending_messages -= 1;
                    if pending_messages == 0 {
                        break;
                    }
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
