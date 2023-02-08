use anyhow::Result;
use clap::Parser;
use config::Config;

#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct Args {
    /// Sets a custom config file
    #[clap(short, long, default_value = "/etc/sensor_mqtt/sensor_mqtt.toml")]
    config: String,

    /// Enable debug logging
    #[clap(short, long)]
    debug: bool,

    /// Specify the I2C bus path
    #[clap(long)]
    i2c_bus_path: Option<String>,

    /// Specify the MQTT host to connect to
    #[clap(long)]
    mqtt_host: Option<String>,

    /// Specify the MQTT port to connect to
    #[clap(long)]
    mqtt_port: Option<u16>,

    /// Specify the MQTT authentication username
    #[clap(long)]
    mqtt_user: Option<String>,

    /// Specify the MQTT authentication password
    #[clap(long)]
    mqtt_password: Option<String>,

    /// Specify the base topic used to publish MQTT messages on
    #[clap(long)]
    mqtt_topic_base: Option<String>,

    /// Specify the reported device name
    #[clap(long)]
    device_name: Option<String>,

    /// Enable HomeAssistant discovery messages to be sent
    #[clap(short, long)]
    enable_homeassistant_discovery: bool,
}

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
    pub debug_log: bool,
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
            debug_log: false,
        }
    }
}

impl Configuration {
    pub(crate) fn new() -> Result<Self> {
        let args: Args = Args::parse();
        Config::builder()
            .add_source(config::File::with_name(&args.config).required(false))
            .build()
            .and_then(|c: Config| c.try_deserialize())
            .map(|mut c: Configuration| {
                if args.debug {
                    c.debug_log = true
                }
                if args.enable_homeassistant_discovery {
                    c.enable_homeassistant_discovery = true
                }

                if let Some(bus_path) = args.i2c_bus_path { c.i2c_bus_path = bus_path };
                if let Some(mqtt_host) = args.mqtt_host { c.mqtt_host = mqtt_host }
                if let Some(mqtt_port) = args.mqtt_port { c.mqtt_port = mqtt_port }
                if let Some(mqtt_user) = args.mqtt_user { c.mqtt_username = mqtt_user }
                if let Some(mqtt_password) = args.mqtt_password { c.mqtt_password = mqtt_password }
                if let Some(mqtt_topic_base) = args.mqtt_topic_base { c.mqtt_topic_base = mqtt_topic_base }
                if let Some(device_name) = args.device_name { c.device_name = device_name }
                c
            })
            .map_err(|e| e.into())
    }
}
