extern crate config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::process::exit;

use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Env;
use rumqttc::Outgoing::PubAck;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use crate::bme280::{measurements_to_messages, read_bme280};
use crate::configuration::Configuration;
use crate::homeassistant::get_homeassistant_discovery_messages;

mod bme280;
mod configuration;
mod homeassistant;

#[derive(Parser, Debug)]
#[clap(version, author)]
struct Args {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "/etc/sensor_mqtt/sensor_mqtt.toml")]
    config: String,
}

pub struct MessageToPublish {
    topic: String,
    payload: String,
    retain: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let this_config = Configuration::new(&opts.config).unwrap_or_else(|e| {
        error!("Unable to load config: {}", e);
        exit(2)
    });

    read_bme280(&this_config.i2c_bus_path)
        .and_then(|measurements| measurements_to_messages(measurements, &this_config))
        .and_then(|measurement_messages| {
            get_homeassistant_discovery_messages(&this_config).map(|mut messages| {
                messages.extend(measurement_messages);
                return messages;
            })
        })
        .and_then(|messages_to_publish| {
            send_measurements_to_mqtt(messages_to_publish, &this_config)
        })
}

fn send_measurements_to_mqtt(
    messages_to_publish: Vec<MessageToPublish>,
    this_config: &Configuration,
) -> Result<()> {
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
            Err(e) => return Err(anyhow!(e)),
        }
    }
    client.disconnect()?;
    Ok(())
}
