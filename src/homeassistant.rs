use anyhow::Result;
use serde_json::json;

use crate::{Configuration, MessageToPublish};

pub(crate) fn get_homeassistant_discovery_messages(
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
