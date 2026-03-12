use crate::Result;
use crate::bike::StromerBike;
use crate::mqtt::MqttData;
use serde::Serialize;

const HASS_DISCOVERY_TOPIC: &str = "homeassistant";

#[derive(Serialize)]
struct Origin {
    name: String,
    sw: String,
    url: String,
}

impl Origin {
    fn stromer2mqtt() -> Origin {
        Origin {
            name: String::from(env!("CARGO_PKG_NAME")),
            sw: String::from(env!("CARGO_PKG_VERSION")),
            url: String::from("https://github.com/dirkvdb/stromer2mqtt"),
        }
    }
}

#[derive(Serialize)]
struct Device {
    name: String,
    identifiers: Vec<String>,
    model: String,
    manufacturer: String,
}

#[derive(Serialize)]
struct Sensor {
    origin: Origin,
    device: Device,
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
    #[serde(rename = "avty_t")]
    availability_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_template: Option<String>,
}

#[derive(Serialize)]
struct BinarySensor {
    origin: Origin,
    device: Device,
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
    #[serde(rename = "avty_t")]
    availability_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    payload_on: String,
    payload_off: String,
}

#[derive(Serialize)]
struct Switch {
    origin: Origin,
    device: Device,
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
    #[serde(rename = "avty_t")]
    availability_topic: String,
    #[serde(rename = "cmd_t")]
    command_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    payload_on: String,
    payload_off: String,
    state_on: String,
    state_off: String,
}

#[derive(Serialize)]
struct DeviceTracker {
    origin: Origin,
    device: Device,
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "json_attr_t")]
    json_attributes_topic: String,
    #[serde(rename = "avty_t")]
    availability_topic: String,
    source_type: String,
}

fn make_device(bike: &StromerBike) -> Device {
    Device {
        name: format!("Stromer {}", bike.bike_name()),
        identifiers: vec![format!("stromer_{}", bike.bike_id())],
        model: bike.bike_model().to_string(),
        manufacturer: String::from("Stromer"),
    }
}

struct SensorDef {
    field: &'static str,
    name: &'static str,
    unit: Option<&'static str>,
    device_class: Option<&'static str>,
    state_class: Option<&'static str>,
    icon: Option<&'static str>,
}

#[allow(dead_code)]
struct BinarySensorDef {
    field: &'static str,
    name: &'static str,
    icon_on: &'static str,
    icon_off: &'static str,
}

struct SwitchDef {
    field: &'static str,
    name: &'static str,
    icon: &'static str,
    command_action: &'static str,
}

const SENSORS: &[SensorDef] = &[
    SensorDef {
        field: "assistance_level",
        name: "Assistance Level",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:bike"),
    },
    SensorDef {
        field: "atmospheric_pressure",
        name: "Atmospheric Pressure",
        unit: Some("bar"),
        device_class: Some("pressure"),
        state_class: Some("measurement"),
        icon: None,
    },
    SensorDef {
        field: "average_energy_consumption",
        name: "Average Energy Consumption",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
        icon: None,
    },
    SensorDef {
        field: "average_speed_total",
        name: "Average Speed Total",
        unit: Some("km/h"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:speedometer"),
    },
    SensorDef {
        field: "average_speed_trip",
        name: "Average Speed Trip",
        unit: Some("km/h"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:speedometer"),
    },
    SensorDef {
        field: "battery_soc",
        name: "Battery SOC",
        unit: Some("%"),
        device_class: Some("battery"),
        state_class: Some("measurement"),
        icon: Some("mdi:battery"),
    },
    SensorDef {
        field: "battery_health",
        name: "Battery Health",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:battery-heart-variant"),
    },
    SensorDef {
        field: "battery_temp",
        name: "Battery Temperature",
        unit: Some("°C"),
        device_class: Some("temperature"),
        state_class: Some("measurement"),
        icon: None,
    },
    SensorDef {
        field: "bike_speed",
        name: "Bike Speed",
        unit: Some("km/h"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:speedometer"),
    },
    SensorDef {
        field: "motor_temp",
        name: "Motor Temperature",
        unit: Some("°C"),
        device_class: Some("temperature"),
        state_class: Some("measurement"),
        icon: None,
    },
    SensorDef {
        field: "power_on_cycles",
        name: "Power On Cycles",
        unit: None,
        device_class: None,
        state_class: Some("total_increasing"),
        icon: Some("mdi:counter"),
    },
    SensorDef {
        field: "speed",
        name: "Speed",
        unit: Some("km/h"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:speedometer"),
    },
    SensorDef {
        field: "total_distance",
        name: "Total Distance",
        unit: Some("km"),
        device_class: None,
        state_class: Some("total_increasing"),
        icon: Some("mdi:map-marker-distance"),
    },
    SensorDef {
        field: "total_energy_consumption",
        name: "Total Energy Consumption",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: None,
    },
    SensorDef {
        field: "total_time",
        name: "Total Time",
        unit: Some("s"),
        device_class: None,
        state_class: Some("total_increasing"),
        icon: Some("mdi:timer"),
    },
    SensorDef {
        field: "trip_distance",
        name: "Trip Distance",
        unit: Some("km"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:map-marker-distance"),
    },
    SensorDef {
        field: "trip_time",
        name: "Trip Time",
        unit: None,
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:timer"),
    },
    SensorDef {
        field: "altitude",
        name: "Altitude",
        unit: Some("m"),
        device_class: Some("distance"),
        state_class: Some("measurement"),
        icon: Some("mdi:elevation-rise"),
    },
    SensorDef {
        field: "rcvts",
        name: "Receive Timestamp",
        unit: None,
        device_class: Some("timestamp"),
        state_class: None,
        icon: None,
    },
    SensorDef {
        field: "rcvts_pos",
        name: "Position Timestamp",
        unit: None,
        device_class: Some("timestamp"),
        state_class: None,
        icon: None,
    },
    SensorDef {
        field: "timets",
        name: "Time Timestamp",
        unit: None,
        device_class: Some("timestamp"),
        state_class: None,
        icon: None,
    },
    SensorDef {
        field: "suiversion",
        name: "SUI Version",
        unit: None,
        device_class: None,
        state_class: None,
        icon: Some("mdi:chip"),
    },
    SensorDef {
        field: "tntversion",
        name: "TNT Version",
        unit: None,
        device_class: None,
        state_class: None,
        icon: Some("mdi:chip"),
    },
    SensorDef {
        field: "next_maintenance_date",
        name: "Next Maintenance Date",
        unit: None,
        device_class: None,
        state_class: None,
        icon: Some("mdi:calendar-wrench"),
    },
    SensorDef {
        field: "next_maintenance_km",
        name: "Next Maintenance Distance",
        unit: Some("km"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:map-marker-distance"),
    },
    SensorDef {
        field: "next_maintenance_interval",
        name: "Maintenance Interval",
        unit: Some("km"),
        device_class: None,
        state_class: None,
        icon: Some("mdi:wrench-clock"),
    },
    SensorDef {
        field: "last_maintenance_reset_date",
        name: "Last Maintenance Date",
        unit: None,
        device_class: None,
        state_class: None,
        icon: Some("mdi:calendar-check"),
    },
    SensorDef {
        field: "last_maintenance_reset_km",
        name: "Last Maintenance Distance",
        unit: Some("km"),
        device_class: None,
        state_class: None,
        icon: Some("mdi:map-marker-check"),
    },
];

const BINARY_SENSORS: &[BinarySensorDef] = &[
    BinarySensorDef {
        field: "light_on",
        name: "Light",
        icon_on: "mdi:lightbulb",
        icon_off: "mdi:lightbulb-off",
    },
    BinarySensorDef {
        field: "lock_flag",
        name: "Lock",
        icon_on: "mdi:lock",
        icon_off: "mdi:lock-open",
    },
    BinarySensorDef {
        field: "theft_flag",
        name: "Theft",
        icon_on: "mdi:alarm-light",
        icon_off: "mdi:shield-moon",
    },
];

const SWITCHES: &[SwitchDef] = &[
    SwitchDef {
        field: "lock_flag",
        name: "Lock",
        icon: "mdi:lock",
        command_action: "lock",
    },
    SwitchDef {
        field: "light_on",
        name: "Light",
        icon: "mdi:light-flood-down",
        command_action: "light",
    },
];

fn create_sensor_messages(bike: &StromerBike, base_topic: &str) -> Result<Vec<MqttData>> {
    let mut messages = Vec::new();
    let bike_id = bike.bike_id();
    let device = make_device(bike);

    for def in SENSORS {
        let unique_id = format!("stromer_{}_{}", bike_id, def.field);
        let sensor = Sensor {
            origin: Origin::stromer2mqtt(),
            device: Device {
                name: device.name.clone(),
                identifiers: device.identifiers.clone(),
                model: device.model.clone(),
                manufacturer: device.manufacturer.clone(),
            },
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
            availability_topic: format!("{}state", base_topic),
            state_class: def.state_class.map(String::from),
            unit_of_measurement: def.unit.map(String::from),
            icon: def.icon.map(String::from),
            device_class: def.device_class.map(String::from),
            value_template: None,
        };

        messages.push(MqttData {
            topic: format!("{}/sensor/{}/config", HASS_DISCOVERY_TOPIC, unique_id),
            payload: serde_json::to_string(&sensor)?,
        });
    }

    Ok(messages)
}

fn create_binary_sensor_messages(bike: &StromerBike, base_topic: &str) -> Result<Vec<MqttData>> {
    let mut messages = Vec::new();
    let bike_id = bike.bike_id();
    let device = make_device(bike);

    for def in BINARY_SENSORS {
        let unique_id = format!("stromer_{}_{}", bike_id, def.field);
        let binary_sensor = BinarySensor {
            origin: Origin::stromer2mqtt(),
            device: Device {
                name: device.name.clone(),
                identifiers: device.identifiers.clone(),
                model: device.model.clone(),
                manufacturer: device.manufacturer.clone(),
            },
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
            availability_topic: format!("{}state", base_topic),
            device_class: None,
            icon: Some(def.icon_on.to_string()),
            payload_on: "true".to_string(),
            payload_off: "false".to_string(),
        };

        messages.push(MqttData {
            topic: format!(
                "{}/binary_sensor/{}/config",
                HASS_DISCOVERY_TOPIC, unique_id
            ),
            payload: serde_json::to_string(&binary_sensor)?,
        });
    }

    Ok(messages)
}

fn create_switch_messages(bike: &StromerBike, base_topic: &str) -> Result<Vec<MqttData>> {
    let mut messages = Vec::new();
    let bike_id = bike.bike_id();
    let device = make_device(bike);

    for def in SWITCHES {
        let unique_id = format!("stromer_{}_{}_sw", bike_id, def.field);
        let switch = Switch {
            origin: Origin::stromer2mqtt(),
            device: Device {
                name: device.name.clone(),
                identifiers: device.identifiers.clone(),
                model: device.model.clone(),
                manufacturer: device.manufacturer.clone(),
            },
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
            availability_topic: format!("{}state", base_topic),
            command_topic: format!(
                "{}stromer_{}/cmnd/{}",
                base_topic, bike_id, def.command_action
            ),
            icon: Some(def.icon.to_string()),
            payload_on: "ON".to_string(),
            payload_off: "OFF".to_string(),
            state_on: "true".to_string(),
            state_off: "false".to_string(),
        };

        messages.push(MqttData {
            topic: format!("{}/switch/{}/config", HASS_DISCOVERY_TOPIC, unique_id),
            payload: serde_json::to_string(&switch)?,
        });
    }

    Ok(messages)
}

fn create_device_tracker_message(bike: &StromerBike, base_topic: &str) -> Result<MqttData> {
    let bike_id = bike.bike_id();
    let unique_id = format!("stromer_{}_location", bike_id);
    let device = make_device(bike);

    let tracker = DeviceTracker {
        origin: Origin::stromer2mqtt(),
        device,
        name: "Location".to_string(),
        obj_id: unique_id.clone(),
        unique_id: unique_id.clone(),
        json_attributes_topic: format!("{}stromer_{}/position", base_topic, bike_id),
        availability_topic: format!("{}state", base_topic),
        source_type: "gps".to_string(),
    };

    Ok(MqttData {
        topic: format!(
            "{}/device_tracker/{}/config",
            HASS_DISCOVERY_TOPIC, unique_id
        ),
        payload: serde_json::to_string(&tracker)?,
    })
}

/// Generate all Home Assistant MQTT discovery payloads for the given bike.
///
/// Returns a Vec of MqttData entries, each containing a discovery topic
/// and JSON payload for sensors, binary sensors, switches, and device tracker.
pub fn create_discovery_messages(bike: &StromerBike, base_topic: &str) -> Result<Vec<MqttData>> {
    let mut messages = Vec::new();

    messages.extend(create_sensor_messages(bike, base_topic)?);
    messages.extend(create_binary_sensor_messages(bike, base_topic)?);
    messages.extend(create_switch_messages(bike, base_topic)?);
    messages.push(create_device_tracker_message(bike, base_topic)?);

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_bike() -> StromerBike {
        StromerBike::new("12345".into(), "MyBike".into(), "ST5".into(), None)
    }

    #[test]
    fn test_discovery_message_count() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        // 22 sensors + 3 binary sensors + 2 switches + 1 device tracker = 28
        let sensor_count = messages
            .iter()
            .filter(|m| m.topic.contains("/sensor/") && !m.topic.contains("/binary_sensor/"))
            .count();
        let binary_sensor_count = messages
            .iter()
            .filter(|m| m.topic.contains("/binary_sensor/"))
            .count();
        let switch_count = messages
            .iter()
            .filter(|m| m.topic.contains("/switch/"))
            .count();
        let tracker_count = messages
            .iter()
            .filter(|m| m.topic.contains("/device_tracker/"))
            .count();

        assert_eq!(
            sensor_count,
            SENSORS.len(),
            "Expected {} sensors",
            SENSORS.len()
        );
        assert_eq!(
            binary_sensor_count,
            BINARY_SENSORS.len(),
            "Expected {} binary sensors",
            BINARY_SENSORS.len()
        );
        assert_eq!(
            switch_count,
            SWITCHES.len(),
            "Expected {} switches",
            SWITCHES.len()
        );
        assert_eq!(tracker_count, 1, "Expected 1 device tracker");

        assert_eq!(
            messages.len(),
            SENSORS.len() + BINARY_SENSORS.len() + SWITCHES.len() + 1
        );
    }

    #[test]
    fn test_discovery_topic_format() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        for msg in &messages {
            assert!(
                msg.topic.starts_with("homeassistant/"),
                "Discovery topic should start with homeassistant/: {}",
                msg.topic
            );
            assert!(
                msg.topic.ends_with("/config"),
                "Discovery topic should end with /config: {}",
                msg.topic
            );
        }
    }

    #[test]
    fn test_sensor_payload_contains_required_fields() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        // Check the battery_soc sensor specifically
        let battery_msg = messages
            .iter()
            .find(|m| m.topic.contains("stromer_12345_battery_soc/config"))
            .expect("Should have battery_soc sensor");

        let payload: serde_json::Value =
            serde_json::from_str(&battery_msg.payload).expect("Payload should be valid JSON");

        assert_eq!(payload["name"], "Battery SOC");
        assert_eq!(payload["stat_t"], "stromer/stromer_12345/battery_soc");
        assert_eq!(payload["avty_t"], "stromer/state");
        assert_eq!(payload["unit_of_measurement"], "%");
        assert_eq!(payload["device_class"], "battery");
        assert_eq!(payload["state_class"], "measurement");
        assert!(payload["origin"]["name"].as_str().is_some());
        assert!(payload["origin"]["sw"].as_str().is_some());
    }

    #[test]
    fn test_binary_sensor_payload() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        let light_msg = messages
            .iter()
            .find(|m| m.topic.contains("/binary_sensor/") && m.topic.contains("light_on"))
            .expect("Should have light_on binary sensor");

        let payload: serde_json::Value =
            serde_json::from_str(&light_msg.payload).expect("Payload should be valid JSON");

        assert_eq!(payload["name"], "Light");
        assert_eq!(payload["payload_on"], "true");
        assert_eq!(payload["payload_off"], "false");
        assert_eq!(payload["icon"], "mdi:lightbulb");
    }

    #[test]
    fn test_switch_payload() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        let lock_sw = messages
            .iter()
            .find(|m| m.topic.contains("/switch/") && m.topic.contains("lock_flag"))
            .expect("Should have lock_flag switch");

        let payload: serde_json::Value =
            serde_json::from_str(&lock_sw.payload).expect("Payload should be valid JSON");

        assert_eq!(payload["name"], "Lock");
        assert_eq!(payload["cmd_t"], "stromer/stromer_12345/cmnd/lock");
        assert_eq!(payload["payload_on"], "ON");
        assert_eq!(payload["payload_off"], "OFF");
        assert_eq!(payload["state_on"], "true");
        assert_eq!(payload["state_off"], "false");
        assert_eq!(payload["icon"], "mdi:lock");
    }

    #[test]
    fn test_device_tracker_payload() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        let tracker_msg = messages
            .iter()
            .find(|m| m.topic.contains("/device_tracker/"))
            .expect("Should have device tracker");

        let payload: serde_json::Value =
            serde_json::from_str(&tracker_msg.payload).expect("Payload should be valid JSON");

        assert_eq!(payload["name"], "Location");
        assert_eq!(payload["json_attr_t"], "stromer/stromer_12345/position");
        assert_eq!(payload["source_type"], "gps");
    }

    #[test]
    fn test_device_info_in_payloads() {
        let bike = make_test_bike();
        let messages =
            create_discovery_messages(&bike, "stromer/").expect("Should generate messages");

        for msg in &messages {
            let payload: serde_json::Value =
                serde_json::from_str(&msg.payload).expect("Payload should be valid JSON");

            assert_eq!(payload["device"]["manufacturer"], "Stromer");
            assert_eq!(payload["device"]["model"], "ST5");
            assert!(
                payload["device"]["identifiers"]
                    .as_array()
                    .expect("identifiers should be array")
                    .contains(&serde_json::json!("stromer_12345")),
                "Device identifiers should contain stromer_12345"
            );
        }
    }
}
