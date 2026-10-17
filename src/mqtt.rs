use crate::Result;
use anyhow::{Context, anyhow, ensure};
use bytes::Bytes;
use std::{path::PathBuf, sync::Arc, time::Duration};

use rumqttc::{
    Transport,
    tokio_rustls::rustls::{
        self, ClientConfig, DigitallySignedStruct, SignatureScheme,
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
        crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
        pki_types::{CertificateDer, ServerName, UnixTime},
    },
};

use rumqttc::v5::{
    AsyncClient, Event, EventLoop, MqttOptions,
    mqttbytes::{
        QoS,
        v5::{ConnectReturnCode, LastWill, Packet},
    },
};

#[derive(Clone)]
pub struct MqttConfig {
    pub server: String,
    pub port: u16,
    pub ca_cert: Option<PathBuf>,
    pub insecure: bool,
    pub client_id: String,
    pub user: String,
    pub password: String,
    pub base_topic: String,
}

impl MqttConfig {
    fn transport(&self) -> Result<Transport> {
        ensure!(
            self.ca_cert.is_none() || !self.insecure,
            "MQTT CA certificate and insecure TLS options are mutually exclusive"
        );

        if let Some(path) = &self.ca_cert {
            let ca = std::fs::read(path).with_context(|| {
                format!("Failed to read MQTT CA certificate {}", path.display())
            })?;
            Ok(Transport::tls(ca, None, None))
        } else if self.insecure {
            log::warn!(
                "MQTT TLS certificate validation is disabled; the connection is vulnerable to man-in-the-middle attacks"
            );
            let builder = ClientConfig::builder();
            let verifier = NoCertificateVerification(builder.crypto_provider().clone());
            let config = builder
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(verifier))
                .with_no_client_auth();
            Ok(Transport::tls_with_config(config.into()))
        } else {
            Ok(Transport::tcp())
        }
    }
}

#[derive(Debug)]
struct NoCertificateVerification(Arc<CryptoProvider>);

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    // Only certificate trust and hostname validation are skipped, not handshake signatures.
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MqttData {
    pub topic: String,
    pub payload: String,
}

impl MqttData {
    pub fn new<T: AsRef<str>>(topic: T, payload: T) -> MqttData {
        MqttData {
            topic: String::from(topic.as_ref()),
            payload: String::from(payload.as_ref()),
        }
    }
}

impl PartialOrd for MqttData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.topic.cmp(&other.topic))
    }
}

impl Ord for MqttData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.topic.cmp(&other.topic)
    }
}

pub struct MqttConnection {
    client: AsyncClient,
    eventloop: EventLoop,
    base_topic: String,
}

fn from_mqtt_string(stream: &Bytes) -> Result<String> {
    match String::from_utf8(stream.to_vec()) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow!("Mqtt string conversion error: {}", e)),
    }
}

const OFFLINE_PAYLOAD: &str = "offline";
const ONLINE_PAYLOAD: &str = "online";

fn state_topic(base_topic: &String) -> String {
    format!("{}/state", base_topic)
}

impl MqttConnection {
    pub fn new(cfg: MqttConfig) -> Result<MqttConnection> {
        let transport = cfg.transport()?;
        let mut mqttoptions = MqttOptions::new(cfg.client_id, cfg.server, cfg.port);
        mqttoptions.set_transport(transport);
        mqttoptions.set_clean_start(true);
        mqttoptions.set_keep_alive(Duration::from_secs(180));
        mqttoptions.set_last_will(LastWill::new(
            state_topic(&cfg.base_topic),
            OFFLINE_PAYLOAD,
            QoS::AtLeastOnce,
            true,
            None,
        ));

        if !cfg.user.is_empty() {
            mqttoptions.set_credentials(cfg.user, cfg.password);
        }

        let (client, eventloop) = AsyncClient::new(mqttoptions, 1000);

        log::info!("MQTT connection created");
        Ok(MqttConnection {
            client,
            eventloop,
            base_topic: cfg.base_topic,
        })
    }

    pub async fn poll(&mut self) -> Result<Option<MqttData>> {
        let msg = self.eventloop.poll().await?;
        self.handle_mqtt_message(msg).await
    }

    pub async fn publish(&mut self, data: MqttData) -> Result<()> {
        Ok(self
            .client
            .publish(data.topic, QoS::AtLeastOnce, true, data.payload)
            .await?)
    }

    pub async fn publish_multiple(&mut self, data: Vec<MqttData>) -> Result<()> {
        for d in data {
            self.publish(d).await?;
        }

        Ok(())
    }

    async fn subscribe_to_commands(&mut self) -> Result<()> {
        let cmd_subscription_topic = format!("{}/+/cmnd/+", self.base_topic);
        self.client
            .subscribe(cmd_subscription_topic, QoS::ExactlyOnce)
            .await?;
        Ok(())
    }

    pub async fn publish_online(&mut self) -> Result<()> {
        self.client
            .publish(
                state_topic(&self.base_topic),
                QoS::AtLeastOnce,
                true,
                ONLINE_PAYLOAD,
            )
            .await?;

        Ok(())
    }

    pub async fn publish_offline(&mut self) -> Result<()> {
        self.client
            .publish(
                state_topic(&self.base_topic),
                QoS::AtLeastOnce,
                true,
                OFFLINE_PAYLOAD,
            )
            .await?;
        Ok(())
    }

    async fn handle_mqtt_message(&mut self, ev: Event) -> Result<Option<MqttData>> {
        match ev {
            Event::Incoming(event) => match event {
                Packet::ConnAck(data) => {
                    if data.code == ConnectReturnCode::Success {
                        if !data.session_present {
                            log::info!("Subscribe to mqtt commands");
                            self.subscribe_to_commands().await?;
                        } else {
                            log::debug!("Session still active, no need to resubscribe");
                        }
                    } else {
                        log::error!("MQTT connection refused: {:?}", data.code);
                    }
                    Ok(None)
                }
                Packet::Publish(publ) => Ok(Some(MqttData {
                    topic: from_mqtt_string(&publ.topic)?,
                    payload: from_mqtt_string(&publ.payload)?,
                })),
                _ => Ok(None),
            },
            Event::Outgoing(_) => {
                // Handle outgoing events (keepalive, etc.) - just continue
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rumqttc::TlsConfiguration;

    fn config() -> MqttConfig {
        MqttConfig {
            server: "localhost".into(),
            port: 1883,
            ca_cert: None,
            insecure: false,
            client_id: "test".into(),
            user: String::new(),
            password: String::new(),
            base_topic: "stromer".into(),
        }
    }

    #[test]
    fn default_transport_is_plaintext() {
        assert!(matches!(
            config().transport().expect("transport"),
            Transport::Tcp
        ));
    }

    #[test]
    fn certificate_file_is_passed_to_rumqttc() {
        // Parsing belongs to rumqttc; any file suffices to test forwarding its bytes.
        let cfg = MqttConfig {
            ca_cert: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")),
            ..config()
        };
        let Transport::Tls(TlsConfiguration::Simple { ca, .. }) =
            cfg.transport().expect("TLS transport")
        else {
            panic!("expected certificate-based TLS");
        };
        assert_eq!(ca, include_bytes!("../Cargo.toml"));
    }

    #[test]
    fn insecure_mode_enables_tls() {
        let cfg = MqttConfig {
            insecure: true,
            ..config()
        };
        assert!(matches!(
            cfg.transport().expect("TLS transport"),
            Transport::Tls(_)
        ));
    }

    #[test]
    fn certificate_and_insecure_mode_are_mutually_exclusive() {
        let cfg = MqttConfig {
            ca_cert: Some("ca.pem".into()),
            insecure: true,
            ..config()
        };
        assert!(
            cfg.transport()
                .err()
                .expect("conflicting options should fail")
                .to_string()
                .contains("mutually exclusive")
        );
    }
}
