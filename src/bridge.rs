use crate::Result;
use crate::bike::StromerBike;
use crate::hassdiscovery;
use crate::mqtt::{MqttConfig, MqttConnection, MqttData};
use crate::stromerapi::{MaintenanceInfo, StromerApi};
use anyhow::anyhow;
use std::time::Duration;
use tokio::time;

pub struct StromerMqttBridgeConfig {
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub bike_id: Option<String>,
    pub mqtt_config: MqttConfig,
    pub hass_discovery: bool,
    pub poll_interval: Duration,
}

pub struct StromerMqttBridge {
    mqtt: MqttConnection,
    api: StromerApi,
    bike: Option<StromerBike>,
    bike_id: String,
    bike_name: String,
    bike_model: String,
    maintenance: Option<MaintenanceInfo>,
    mqtt_base_topic: String,
    hass_discovery: bool,
    poll_interval: Duration,
    discovery_published: bool,
}

impl StromerMqttBridge {
    pub async fn new(cfg: StromerMqttBridgeConfig) -> Result<StromerMqttBridge> {
        let mqtt_base_topic = format!("{}/", cfg.mqtt_config.base_topic);

        let mut api = StromerApi::new(cfg.username, cfg.password, cfg.client_id, cfg.client_secret);

        // Authenticate
        api.stromer_connect().await?;

        // Determine bike_id
        let (bike_id, bike_name, bike_model, maintenance) = if let Some(id) = cfg.bike_id {
            log::info!("Using configured bike ID: {}", id);
            (id, String::new(), String::new(), None)
        } else {
            let bikes = api.stromer_detect().await?;
            if bikes.is_empty() {
                return Err(anyhow!("No bikes detected on the Stromer account"));
            }
            let first = &bikes[0];
            let id = first.bike_id_string();
            log::info!(
                "Auto-detected bike: {} (id: {}, type: {})",
                first.nickname,
                id,
                first.bike_type
            );
            (
                id,
                first.nickname.clone(),
                first
                    .bike_model
                    .clone()
                    .unwrap_or_else(|| first.bike_type.clone()),
                first.maintenance_feature.clone(),
            )
        };

        Ok(StromerMqttBridge {
            mqtt: MqttConnection::new(cfg.mqtt_config),
            api,
            bike: None,
            bike_id,
            bike_name,
            bike_model,
            maintenance,
            mqtt_base_topic,
            hass_discovery: cfg.hass_discovery,
            poll_interval: cfg.poll_interval,
            discovery_published: false,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut interval = time::interval(self.poll_interval);
        log::debug!("Poll interval: {:?}", self.poll_interval);

        loop {
            tokio::select! {
                mqtt_msg = self.mqtt.poll() => {
                    match mqtt_msg {
                        Ok(Some(msg)) => {
                            log::info!("MQTT cmnd: {} {}", msg.topic, msg.payload);
                            if let Err(err) = self.handle_command(msg).await {
                                log::error!("Failed to process command: {:#}", err);
                            }
                        }
                        Ok(None) => {
                            // No command message, just MQTT housekeeping (keepalive, etc.)
                        }
                        Err(err) => {
                            // Reconnect by continuing the loop
                            // The MQTT library will handle reconnection automatically
                            log::error!("MQTT connection error: {:#}", err);
                        }
                    }
                }
                _ = interval.tick() => {
                    log::debug!("Polling Stromer API for updates");
                    match self.poll_stromer().await {
                        Ok(()) => {
                            if let Err(err) = self.mqtt.publish_online().await {
                                log::error!("Failed to publish online state: {:#}", err);
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to update Stromer status: {:#}", err);
                            self.reset_status();
                            if let Err(err) = self.mqtt.publish_offline().await {
                                log::error!("Failed to publish offline state: {:#}", err);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn poll_stromer(&mut self) -> Result<()> {
        log::debug!("Fetching Stromer status and position");

        let (status, position) = self.api.stromer_update(&self.bike_id).await?;

        log::debug!("Status: {:?}", status);
        log::debug!("Position: {:?}", position);

        let first_poll = self.bike.is_none();

        let bike = self.bike.get_or_insert_with(|| {
            StromerBike::new(
                self.bike_id.clone(),
                self.bike_name.clone(),
                self.bike_model.clone(),
                self.maintenance.clone(),
            )
        });

        bike.update_status(status);
        bike.update_position(position.clone());

        // Publish Home Assistant discovery messages on first successful poll
        if first_poll && self.hass_discovery && !self.discovery_published {
            log::info!("Publishing Home Assistant discovery messages");
            match hassdiscovery::create_discovery_messages(bike, &self.mqtt_base_topic) {
                Ok(discovery_data) => {
                    if let Err(err) = self.mqtt.publish_multiple(discovery_data).await {
                        log::error!("Failed to publish HA discovery messages: {:#}", err);
                    } else {
                        self.discovery_published = true;
                    }
                }
                Err(err) => {
                    log::error!("Failed to create HA discovery messages: {:#}", err);
                }
            }
        }

        // Publish position as a JSON object for the device tracker
        if let Some(ref bike) = self.bike {
            if let (Some(lat), Some(lng)) = (position.latitude, position.longitude) {
                let mut position_map = serde_json::json!({
                    "latitude": lat,
                    "longitude": lng,
                });
                if let Some(alt) = position.altitude {
                    position_map["altitude"] = serde_json::json!(alt);
                }
                let position_json = position_map;
                let position_topic = format!(
                    "{}stromer_{}/position",
                    self.mqtt_base_topic,
                    bike.bike_id()
                );
                if let Err(err) = self
                    .mqtt
                    .publish(MqttData {
                        topic: position_topic,
                        payload: position_json.to_string(),
                    })
                    .await
                {
                    log::error!("Failed to publish position: {:#}", err);
                }
            }
        }

        // Publish changed topics
        if let Some(ref mut bike) = self.bike {
            for mut mqtt_data in bike.topics_that_need_updating() {
                mqtt_data.topic = format!("{}{}", self.mqtt_base_topic, mqtt_data.topic);
                log::info!("{}: {}", mqtt_data.topic, mqtt_data.payload);
                if let Err(err) = self.mqtt.publish(mqtt_data).await {
                    log::error!("Failed to publish MQTT data: {:#}", err);
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, msg: MqttData) -> Result<()> {
        let path = msg
            .topic
            .strip_prefix(self.mqtt_base_topic.as_str())
            .ok_or_else(|| anyhow!("Unexpected command path: {}", msg.topic))?;

        // Expected format: stromer_{bike_id}/cmnd/{action}
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 3 || parts[1] != "cmnd" {
            return Err(anyhow!("Invalid command topic format: {}", path));
        }

        let action = parts[2];
        let payload = msg.payload.trim().to_uppercase();

        match action {
            "lock" => {
                let state = match payload.as_str() {
                    "ON" => true,
                    "OFF" => false,
                    _ => return Err(anyhow!("Invalid lock payload: {}", payload)),
                };
                log::info!("Setting lock to {}", state);
                self.api.stromer_call_lock(&self.bike_id, state).await?;
            }
            "light" => {
                let mode = match payload.as_str() {
                    "ON" => "on",
                    "OFF" => "off",
                    _ => return Err(anyhow!("Invalid light payload: {}", payload)),
                };
                log::info!("Setting light to {}", mode);
                self.api.stromer_call_light(&self.bike_id, mode).await?;
            }
            _ => {
                return Err(anyhow!("Unknown command action: {}", action));
            }
        }

        // Trigger an immediate poll to update state after command
        if let Err(err) = self.poll_stromer().await {
            log::error!("Failed to poll after command: {:#}", err);
        }

        Ok(())
    }

    fn reset_status(&mut self) {
        if let Some(ref mut bike) = self.bike {
            bike.reset();
        }
    }
}
