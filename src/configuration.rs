use anyhow::Result;
use clap::Parser;
use config::Config;

#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct Args {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "/etc/sensor_mqtt/sensor_mqtt.toml")]
    config: String,

    /// Enable debug logging
    #[clap(short, long)]
    debug: bool,
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
        dbg!(&args);
        Config::builder()
            .add_source(config::File::with_name(&args.config).required(false))
            .build()
            .and_then(|c: Config| c.try_deserialize())
            .map(|mut c: Configuration| {
                if args.debug {
                    c.debug_log = true
                }
                c
            })
            .map_err(|e| e.into())
    }
}
