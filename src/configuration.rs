use config::{Config, ConfigError};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub i2c_bus_path: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub mqtt_topic_base: String,
    pub device_name: String,
    pub enable_homeassistant_discovery: bool,
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
            device_name: "".to_string(),
            enable_homeassistant_discovery: true,
        }
    }
}

impl Configuration {
    pub(crate) fn new(path: &str) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        let result = config.merge(config::File::with_name(path));
        if result.is_err() {
            warn!("Unable to load config from {}", path);
        }
        config.try_into()
    }
}
