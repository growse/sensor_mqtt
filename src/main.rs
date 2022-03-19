extern crate config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use anyhow::Result;

mod configuration;
mod homeassistant;
mod samplers;
mod sensors;
//
// #[derive(Parser, Debug)]
// #[clap(version, author)]
// struct Args {
//     /// Sets a custom config file. Could have been an Option<T> with no default too
//     #[clap(short, long, default_value = "/etc/sensor_mqtt/sensor_mqtt.toml")]
//     config: String,
// }
//
// pub struct MessageToPublish {
//     topic: String,
//     payload: String,
//     retain: bool,
// }

#[tokio::main]
async fn main() -> Result<()> {
    // let opts = Args::parse();
    // simplelog::TermLogger::init(
    //     LevelFilter::Info,
    //     Config::default(),
    //     TerminalMode::Mixed,
    //     ColorChoice::Auto,
    // )?;
    // let _this_config = Configuration::new(&opts.config).unwrap_or_else(|e| {
    //     error!("Unable to load config: {}", e);
    //     exit(2)
    // });
    //
    Ok(())

    // Can we even read the sensor?
    // read_bme280(&this_config.i2c_bus_path)?;

    // get_homeassistant_discovery_messages(&this_config)
    //     .and_then(|messages| send_messages_via_mqtt(messages, &this_config))?;
    //
    // let sampler = FixedPeriodSampler::new(60);
    // let (mqtt_client, eventloop) = get_mqtt_client(&this_config);
    // info!("Client: {mqtt_client:?}");
    // loop {
    //     read_measurements(&this_config.i2c_bus_path)
    //         .and_then(|measurements| measurements_to_messages(measurements, &this_config))?;
    //
    //     tokio::time::sleep(Duration::from_secs(sampler.next())).await;
    // }
}

// pub fn measurements_to_messages(
//     measurements: Measurements<LinuxI2CError>,
//     config: &Configuration,
// ) -> Result<Vec<MessageToPublish>> {
//     let topic = format!(
//         "{topic_base}/{hostname}/state",
//         topic_base = config.mqtt_topic_base.as_str(),
//         hostname = whoami::hostname()
//     );
//     let payload = serde_json::to_string(&measurements)?;
//     Ok(vec![MessageToPublish {
//         topic,
//         payload,
//         retain: false,
//     }])
// }
//
// fn get_mqtt_client(this_config: &Configuration) -> (AsyncClient, EventLoop) {
//     let mut mqtt_options = MqttOptions::new(
//         format!("sensor-mqtt-client-{}", this_config.device_name),
//         this_config.mqtt_host.as_str(),
//         this_config.mqtt_port,
//     );
//     mqtt_options.set_credentials(
//         this_config.mqtt_username.as_str(),
//         this_config.mqtt_password.as_str(),
//     );
//     AsyncClient::new(mqtt_options, 10)
// }
//
// fn send_messages_via_mqtt(
//     messages_to_publish: Vec<MessageToPublish>,
//     this_config: &Configuration,
// ) -> Result<()> {
//     // let (mut client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
//
//     // let mut pending_messages = messages_to_publish.len();
//     // debug!("Publishing {} messages", pending_messages);
//     //
//     // for x in messages_to_publish {
//     //     client.publish(x.topic, QoS::AtLeastOnce, x.retain, x.payload.as_bytes())?;
//     // }
//     //
//     // for (_i, notification) in connection.iter().enumerate() {
//     //     match notification {
//     //         Ok(Event::Outgoing(outgoing)) => match outgoing {
//     //             PubAck(p) => debug!("Publishing MQTT... id={}", p),
//     //             outgoing => debug!("MQTT: Sent outgoing {:?}", outgoing),
//     //         },
//     //         Ok(Event::Incoming(incoming)) => match incoming {
//     //             Packet::ConnAck(_) => debug!("MQTT Connected"),
//     //             Packet::PubAck(pub_ack) => {
//     //                 debug!("MQTT published id={:?}", pub_ack.pkid);
//     //                 pending_messages -= 1;
//     //                 if pending_messages == 0 {
//     //                     break;
//     //                 }
//     //             }
//     //             incoming => debug!("MQTT: Received incoming {:?}", incoming),
//     //         },
//     //         Err(e) => return Err(anyhow!(e)),
//     //     }
//     // }
//     // client.disconnect()?;
//     Ok(())
// }
