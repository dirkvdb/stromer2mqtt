#![warn(clippy::unwrap_used)]
use core::time;

use clap::Parser;
use clap_verbosity_flag::DebugLevel;
use env_logger::Env;
use stromer2mqtt::{
    bridge::{StromerMqttBridge, StromerMqttBridgeConfig},
    mqtt::MqttConfig,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[clap(
    name = "stromer2mqtt",
    about = "Interface between Stromer e-bike cloud API and MQTT"
)]
struct Opt {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<DebugLevel>,

    #[clap(long = "stromer-username", env = "S2M_STROMER_USERNAME")]
    stromer_username: String,

    #[clap(long = "stromer-password", env = "S2M_STROMER_PASSWORD")]
    stromer_password: String,

    /// Stromer API client ID
    #[clap(long = "stromer-client-id", env = "S2M_STROMER_CLIENT_ID")]
    stromer_client_id: String,

    /// Stromer API client secret (enables v3 API)
    #[clap(long = "stromer-client-secret", env = "S2M_STROMER_CLIENT_SECRET")]
    stromer_client_secret: Option<String>,

    /// Stromer bike ID (auto-detected if not provided)
    #[clap(long = "stromer-bike-id", env = "S2M_STROMER_BIKE_ID")]
    stromer_bike_id: Option<String>,

    #[clap(
        long = "poll-interval",
        env = "S2M_POLL_INTERVAL",
        default_value_t = 600
    )]
    poll_interval: u64,

    #[clap(long = "mqtt-addr", env = "S2M_MQTT_ADDRESS")]
    mqtt_addr: String,

    #[clap(long = "mqtt-port", env = "S2M_MQTT_PORT", default_value_t = 1883)]
    mqtt_port: u16,

    #[clap(long = "mqtt-user", env = "S2M_MQTT_USER")]
    mqtt_user: Option<String>,

    #[clap(long = "mqtt-pass", env = "S2M_MQTT_PASS")]
    mqtt_password: Option<String>,

    #[clap(long = "mqtt-client-id", env = "S2M_CLIENT_ID", default_value_t = String::from("stromer2mqtt"))]
    mqtt_client_id: String,

    #[clap(long = "mqtt-base-topic", env = "S2M_MQTT_BASE_TOPIC", default_value_t = String::from("stromer"))]
    mqtt_base_topic: String,

    #[clap(
        long = "hass-discovery",
        env = "S2M_HASS_DISCOVERY",
        default_value_t = false
    )]
    hass_discovery: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    env_logger::Builder::from_env(
        Env::default().default_filter_or(opt.verbose.log_level_filter().to_string()),
    )
    .format_timestamp(None)
    .init();

    log::info!("{} version {}", PACKAGE, VERSION);

    let cfg = StromerMqttBridgeConfig {
        username: opt.stromer_username,
        password: opt.stromer_password,
        client_id: opt.stromer_client_id,
        client_secret: opt.stromer_client_secret,
        bike_id: opt.stromer_bike_id,
        poll_interval: time::Duration::from_secs(opt.poll_interval),
        mqtt_config: MqttConfig {
            server: opt.mqtt_addr,
            port: opt.mqtt_port,
            client_id: opt.mqtt_client_id,
            user: opt.mqtt_user.unwrap_or_default(),
            password: opt.mqtt_password.unwrap_or_default(),
            base_topic: opt.mqtt_base_topic,
        },
        hass_discovery: opt.hass_discovery,
    };

    match StromerMqttBridge::new(cfg).await {
        Ok(bridge) => {
            bridge.run().await.expect("Failed to run bridge");
        }
        Err(err) => {
            log::error!("Failed to initialize bridge: {:#}", err);
            std::process::exit(1);
        }
    }
}
