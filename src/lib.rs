#![warn(clippy::unwrap_used)]
use thiserror::Error;

pub mod bridge;
pub mod mqtt;

mod bike;
mod hassdiscovery;
mod stromerapi;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Network Error {0}")]
    Network(#[from] std::io::Error),
    #[error("Error {0}")]
    Runtime(String),
    #[error("MQTT error {0}")]
    MqttClientError(#[from] rumqttc::v5::ClientError),
    #[error("MQTT error {0}")]
    MqttConnectionError(#[from] rumqttc::v5::ConnectionError),
    #[error("Serialization error {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Request error {0}")]
    RequestError(#[from] reqwest::Error),
}

pub type Result<T> = anyhow::Result<T>;
