extern crate config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::process::exit;

use clap::Clap;
use env_logger::Env;
use serde_json::json;

use crate::bme280::{measurements_to_messages, read_bme280};
use crate::configuration::Configuration;
use rumqttc::Outgoing::PubAck;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let this_config = Configuration::new(&opts.config).unwrap_or_else(|e| {
        error!("Unable to load config: {}", e);
        exit(2)
    });

    let result = read_bme280(&this_config.i2c_bus_path)
        .and_then(|measurements| measurements_to_messages(measurements, &this_config))
        .and_then(|measurement_messages| {
            get_homeassistant_discovery_messages(&this_config).map(|mut messages| {
                messages.extend(measurement_messages);
                return messages;
            })
        })
        .and_then(|messages_to_publish| {
            send_measurements_to_mqtt(messages_to_publish, &this_config)
        });

    match result {
        Ok(_) => info!("GREAT SUCCESS"),
        Err(e) => error!("{}", e),
    }
}

fn get_homeassistant_discovery_messages(
    this_config: &Configuration,
) -> Result<Vec<MessageToPublish>, Box<dyn Error>> {
    if !this_config.enable_homeassistant_discovery {
        return Ok(vec![]);
    }
    let state_topic = format!(
        "{topic_base}/{hostname}/state",
        topic_base = this_config.mqtt_topic_base,
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
            "name": format!("{} Temperature",this_config.device_name),
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
            "name": format!("{} Pressure",this_config.device_name),
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
            "name": format!("{} Humidity",this_config.device_name),
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
    this_config: &Configuration,
) -> Result<(), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new(
        format!("sensor-mqtt-client-{}", this_config.device_name),
        this_config.mqtt_host.as_str(),
        this_config.mqtt_port,
    );
    mqtt_options.set_credentials(
        this_config.mqtt_username.as_str(),
        this_config.mqtt_password.as_str(),
    );
    let (mut client, mut connection) = Client::new(mqtt_options, 10);

    let mut pending_messages = messages_to_publish.len();
    debug!("Publishing {} messages", pending_messages);

    for x in messages_to_publish {
        client.publish(x.topic, QoS::AtLeastOnce, x.retain, x.payload.as_bytes())?;
    }

    for (_i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Event::Outgoing(outgoing)) => match outgoing {
                PubAck(p) => debug!("Publishing MQTT... id={}", p),
                outgoing => debug!("MQTT: Sent outgoing {:?}", outgoing),
            },
            Ok(Event::Incoming(incoming)) => match incoming {
                Packet::ConnAck(_) => debug!("MQTT Connected"),
                Packet::PubAck(pub_ack) => {
                    debug!("MQTT published id={:?}", pub_ack.pkid);
                    pending_messages -= 1;
                    if pending_messages == 0 {
                        break;
                    }
                }
                incoming => debug!("MQTT: Received incoming {:?}", incoming),
            },
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }
    client.disconnect()?;
    Ok(())
}
