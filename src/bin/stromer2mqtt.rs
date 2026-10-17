#![warn(clippy::unwrap_used)]
use core::time;
use std::path::PathBuf;

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

    /// Enable MQTT TLS using a PEM CA certificate file
    #[clap(
        long = "mqtt-ca-cert",
        env = "S2M_MQTT_CA_CERT",
        conflicts_with = "mqtt_insecure"
    )]
    mqtt_ca_cert: Option<PathBuf>,

    /// Enable MQTT TLS without validating the server certificate (insecure)
    #[clap(long = "mqtt-insecure", env = "S2M_MQTT_INSECURE")]
    mqtt_insecure: bool,

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
            ca_cert: opt.mqtt_ca_cert,
            insecure: opt.mqtt_insecure,
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, FromArgMatches};

    fn parse_options(extra: &[&str]) -> Result<Opt, clap::Error> {
        let mut args = vec![
            "stromer2mqtt",
            "--stromer-username",
            "test",
            "--stromer-password",
            "test",
            "--stromer-client-id",
            "test",
            "--mqtt-addr",
            "localhost",
        ];
        args.extend_from_slice(extra);
        // Test CLI behavior independently of the user's environment.
        let matches = Opt::command()
            .mut_args(|arg| arg.env(None::<&str>))
            .try_get_matches_from(args)?;
        Opt::from_arg_matches(&matches)
    }

    #[test]
    fn mqtt_defaults_to_plaintext() {
        let opt = parse_options(&[]).expect("default options");
        assert!(opt.mqtt_ca_cert.is_none());
        assert!(!opt.mqtt_insecure);
        assert_eq!(opt.mqtt_port, 1883);
    }

    #[test]
    fn mqtt_certificate_option() {
        let opt = parse_options(&["--mqtt-ca-cert", "ca.pem", "--mqtt-port", "8883"])
            .expect("certificate options");
        assert_eq!(opt.mqtt_ca_cert, Some(PathBuf::from("ca.pem")));
        assert!(!opt.mqtt_insecure);
        assert_eq!(opt.mqtt_port, 8883);
    }

    #[test]
    fn mqtt_insecure_option() {
        let opt = parse_options(&["--mqtt-insecure"]).expect("insecure option");
        assert!(opt.mqtt_insecure);
        assert!(opt.mqtt_ca_cert.is_none());
    }

    #[test]
    fn mqtt_tls_options_conflict() {
        let err = parse_options(&["--mqtt-ca-cert", "ca.pem", "--mqtt-insecure"])
            .expect_err("conflicting TLS options");
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
