extern crate config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::process::exit;

use anyhow::Result;
use futures::TryFutureExt;
use log::LevelFilter;
use rumqttc::Outgoing::PubAck;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use simplelog::{ColorChoice, Config, TerminalMode};

use crate::bme280::{measurements_to_messages, read_bme280};
use crate::configuration::Configuration;

mod bme280;
mod configuration;

pub struct MessageToPublish {
    topic: String,
    payload: String,
    retain: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = Configuration::new().unwrap_or_else(|e| {
        error!("Unable to load config: {e}");
        exit(2)
    });
    simplelog::TermLogger::init(
        if configuration.debug_log {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )?;
    debug!("Debug logging enabled");
    debug!("Configuration is {:#?}", configuration);
    read_bme280((&configuration.i2c_bus_path).as_ref())
        .and_then(|measurements| measurements_to_messages(measurements, &configuration))
        .and_then(|measurement_messages| async {
            debug!("{} measurements received", measurement_messages.len());
            get_homeassistant_discovery_messages(&configuration).map(|mut messages| {
                messages.extend(measurement_messages);
                messages
            })
        })
        .and_then(|messages_to_publish| {
            send_measurements_to_mqtt(messages_to_publish, &configuration)
        })
        .and_then(|_| async {
            info!("Publish complete");
            Ok(())
        })
        .await
}

fn get_homeassistant_discovery_messages(
    this_config: &Configuration,
) -> Result<Vec<MessageToPublish>> {
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
            "object_id": format!("{hostname}_temperature", hostname = whoami::hostname().replace("-", "_")), // Becomes HA entity_id
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
            "object_id": format!("{hostname}_pressure", hostname = whoami::hostname().replace("-", "_")), // Becomes HA entity_id
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
            "object_id": format!("{hostname}_humidity", hostname = whoami::hostname().replace("-", "_")), // Becomes HA entity_id
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

async fn send_measurements_to_mqtt(
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

    let (client, mut event_loop) = AsyncClient::new(mqtt_options, 10);

    let mut pending_messages = messages_to_publish.len();
    debug!("Publishing {pending_messages} messages");

    for x in messages_to_publish {
        client
            .publish(x.topic, QoS::AtLeastOnce, x.retain, x.payload.as_bytes())
            .await?;
    }

    loop {
        let notification = event_loop.poll().await?;
        match notification {
            Event::Outgoing(outgoing) => match outgoing {
                PubAck(p) => debug!("Publishing MQTT... id={p}"),
                outgoing => debug!("MQTT: Sent outgoing {:?}", outgoing),
            },
            Event::Incoming(incoming) => match incoming {
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
        }
    }

    debug!("Disconnecting from MQTT");
    client.disconnect().await?;
    Ok(())
}
