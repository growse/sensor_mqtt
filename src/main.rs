extern crate config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::process::exit;

use clap::Clap;
use env_logger::Env;
use rumqttc::{Client, Incoming, MqttOptions, Outgoing, QoS};
use serde_json::json;

use crate::bme280::{measurements_to_messages, read_bme280};
use crate::configuration::Configuration;

mod bme280;
mod configuration;

#[derive(Clap)]
#[clap(version = "0.2.0", about = "Publishes BM280 / BLE metrics over MQTT.")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "/etc/sensor_mqtt/sensor_mqtt.toml")]
    config: String,
}

pub struct MessageToPublish {
    topic: String,
    payload: String,
    retain: bool,
}

fn main() {
    let opts = Opts::parse();
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = Configuration::new(&opts.config).unwrap_or_else(|e| {
        error!("Unable to load config: {}", e);
        exit(2)
    });

    let result = read_bme280(&config.i2c_bus_path)
        .and_then(|measurements| measurements_to_messages(measurements, &config))
        .and_then(|measurement_messages| {
            get_homeassistant_discovery_messages(&config).map(|mut messages| {
                messages.extend(measurement_messages);
                return messages;
            })
        })
        .and_then(|messages_to_publish| send_measurements_to_mqtt(messages_to_publish, &config));

    match result {
        Ok(_) => info!("GREAT SUCCESS"),
        Err(e) => error!("{}", e),
    }
}
fn get_homeassistant_discovery_messages(
    config: &Configuration,
) -> Result<Vec<MessageToPublish>, Box<dyn Error>> {
    if !config.enable_homeassistant_discovery {
        return Ok(vec![]);
    }
    let state_topic = format!(
        "{topic_base}/{hostname}/state",
        topic_base = config.mqtt_topic_base,
        hostname = whoami::hostname()
    );
    Ok(vec![
        MessageToPublish {
            topic: format!(
                "homeassistant/sensor/{}_temperature/config",
                whoami::hostname()
            ),
            payload: json!({
            "device_class": "temperature",
            "name": format!("{} Temperature",config.device_name),
            "state_topic": state_topic,
            "unit_of_measurement": "°C",
            "value_template": "{{ value_json.temperature}}",
            "expire_after": 300
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
            "name": format!("{} Pressure",config.device_name),
            "state_topic": state_topic,
            "unit_of_measurement": "Pa",
            "value_template": "{{ value_json.pressure}}",
            "expire_after": 300
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
            "name": format!("{} Humidity",config.device_name),
            "state_topic": state_topic,
            "unit_of_measurement": "%",
            "value_template": "{{ value_json.humidity}}",
            "expire_after": 300
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
    debug!("Publishing {} messages", pending_messages);

    for x in messages_to_publish {
        client.publish(x.topic, QoS::AtLeastOnce, x.retain, x.payload.as_bytes())?;
    }

    for (_i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(success_notification) => match success_notification {
                (None, Some(Outgoing::Publish(p))) => debug!("Publishing MQTT... id={}", p),
                (Some(Incoming::Connected), None) => debug!("MQTT Connected"),
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
