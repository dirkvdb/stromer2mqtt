use crate::Result;
use crate::bike::StromerBike;
use crate::mqtt::MqttData;
use serde::Serialize;
use std::collections::BTreeMap;

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
struct DeviceDiscovery {
    device: Device,
    origin: Origin,
    #[serde(rename = "avty_t")]
    availability_topic: String,
    #[serde(rename = "cmps")]
    components: BTreeMap<String, Component>,
}

#[derive(Serialize)]
#[serde(tag = "p", rename_all = "snake_case")]
enum Component {
    Sensor(Sensor),
    BinarySensor(BinarySensor),
    Switch(Switch),
    DeviceTracker(DeviceTracker),
}

#[derive(Serialize)]
struct Sensor {
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
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
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    payload_on: String,
    payload_off: String,
}

#[derive(Serialize)]
struct Switch {
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "stat_t")]
    state_topic: String,
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
    name: String,
    obj_id: String,
    unique_id: String,
    #[serde(rename = "json_attr_t")]
    json_attributes_topic: String,
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

fn add_sensors(components: &mut BTreeMap<String, Component>, bike_id: &str, base_topic: &str) {
    for def in SENSORS {
        let unique_id = format!("stromer_{}_{}", bike_id, def.field);
        let sensor = Sensor {
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
            state_class: def.state_class.map(String::from),
            unit_of_measurement: def.unit.map(String::from),
            icon: def.icon.map(String::from),
            device_class: def.device_class.map(String::from),
            value_template: None,
        };

        components.insert(unique_id, Component::Sensor(sensor));
    }
}

fn add_binary_sensors(
    components: &mut BTreeMap<String, Component>,
    bike_id: &str,
    base_topic: &str,
) {
    for def in BINARY_SENSORS {
        let unique_id = format!("stromer_{}_{}", bike_id, def.field);
        let binary_sensor = BinarySensor {
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
            device_class: None,
            icon: Some(def.icon_on.to_string()),
            payload_on: "true".to_string(),
            payload_off: "false".to_string(),
        };

        components.insert(unique_id, Component::BinarySensor(binary_sensor));
    }
}

fn add_switches(components: &mut BTreeMap<String, Component>, bike_id: &str, base_topic: &str) {
    for def in SWITCHES {
        let unique_id = format!("stromer_{}_{}_sw", bike_id, def.field);
        let switch = Switch {
            name: def.name.to_string(),
            obj_id: unique_id.clone(),
            unique_id: unique_id.clone(),
            state_topic: format!("{}stromer_{}/{}", base_topic, bike_id, def.field),
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

        components.insert(unique_id, Component::Switch(switch));
    }
}

fn add_device_tracker(
    components: &mut BTreeMap<String, Component>,
    bike_id: &str,
    base_topic: &str,
) {
    let unique_id = format!("stromer_{}_location", bike_id);
    let tracker = DeviceTracker {
        name: "Location".to_string(),
        obj_id: unique_id.clone(),
        unique_id: unique_id.clone(),
        json_attributes_topic: format!("{}stromer_{}/position", base_topic, bike_id),
        source_type: "gps".to_string(),
    };

    components.insert(unique_id, Component::DeviceTracker(tracker));
}

/// Generate device discovery and ordered migration/cleanup messages for legacy topics.
pub fn create_discovery_messages(bike: &StromerBike, base_topic: &str) -> Result<Vec<MqttData>> {
    let mut components = BTreeMap::new();
    let bike_id = bike.bike_id();
    add_sensors(&mut components, bike_id, base_topic);
    add_binary_sensors(&mut components, bike_id, base_topic);
    add_switches(&mut components, bike_id, base_topic);
    add_device_tracker(&mut components, bike_id, base_topic);

    let legacy_topics: Vec<String> = components
        .iter()
        .map(|(unique_id, component)| {
            let platform = match component {
                Component::Sensor(_) => "sensor",
                Component::BinarySensor(_) => "binary_sensor",
                Component::Switch(_) => "switch",
                Component::DeviceTracker(_) => "device_tracker",
            };
            format!("{}/{}/{}/config", HASS_DISCOVERY_TOPIC, platform, unique_id)
        })
        .collect();

    let discovery = DeviceDiscovery {
        device: make_device(bike),
        origin: Origin::stromer2mqtt(),
        availability_topic: format!("{}state", base_topic),
        components,
    };

    // Unload legacy discovery without deleting entity registry settings, then
    // publish the device configuration before clearing the old retained topics.
    let mut messages: Vec<MqttData> = legacy_topics
        .iter()
        .map(|topic| MqttData::new(topic.as_str(), r#"{"migrate_discovery":true}"#))
        .collect();
    messages.push(MqttData {
        topic: format!("{}/device/stromer_{}/config", HASS_DISCOVERY_TOPIC, bike_id),
        payload: serde_json::to_string(&discovery)?,
    });
    messages.extend(legacy_topics.into_iter().map(|topic| MqttData {
        topic,
        payload: String::new(),
    }));

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_bike() -> StromerBike {
        StromerBike::new("12345".into(), "MyBike".into(), "ST5".into(), None)
    }

    fn device_payload(base_topic: &str) -> serde_json::Value {
        let messages = create_discovery_messages(&make_test_bike(), base_topic)
            .expect("Should generate messages");
        let device = messages
            .iter()
            .find(|m| m.topic == "homeassistant/device/stromer_12345/config")
            .expect("Should have a device discovery message");
        serde_json::from_str(&device.payload).expect("Payload should be valid JSON")
    }

    #[test]
    fn test_device_discovery_components() {
        let payload = device_payload("stromer/");
        let components = payload["cmps"]
            .as_object()
            .expect("Components should be a map");
        assert_eq!(
            components.len(),
            SENSORS.len() + BINARY_SENSORS.len() + SWITCHES.len() + 1
        );
        for (platform, count) in [
            ("sensor", SENSORS.len()),
            ("binary_sensor", BINARY_SENSORS.len()),
            ("switch", SWITCHES.len()),
            ("device_tracker", 1),
        ] {
            assert_eq!(
                components.values().filter(|c| c["p"] == platform).count(),
                count
            );
        }
        for (id, component) in components {
            assert_eq!(component["unique_id"], id.as_str());
            assert_eq!(component["obj_id"], id.as_str());
            assert!(component.get("device").is_none());
            assert!(component.get("origin").is_none());
            assert!(component.get("avty_t").is_none());
        }
    }

    #[test]
    fn test_legacy_discovery_migration_order() {
        let messages = create_discovery_messages(&make_test_bike(), "stromer/")
            .expect("Should generate messages");
        let count = SENSORS.len() + BINARY_SENSORS.len() + SWITCHES.len() + 1;
        assert_eq!(messages.len(), count * 2 + 1);
        assert_eq!(
            messages[count].topic,
            "homeassistant/device/stromer_12345/config"
        );
        let payload = device_payload("stromer/");
        let components = payload["cmps"].as_object().unwrap();
        for ((id, component), (migration, cleanup)) in components
            .iter()
            .zip(messages[..count].iter().zip(&messages[count + 1..]))
        {
            let topic = format!(
                "homeassistant/{}/{}/config",
                component["p"].as_str().unwrap(),
                id
            );
            assert_eq!(migration.topic, topic);
            assert_eq!(migration.payload, r#"{"migrate_discovery":true}"#);
            assert_eq!(cleanup.topic, topic);
            assert!(cleanup.payload.is_empty());
        }
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
        let discovery = device_payload("stromer/");
        let payload = &discovery["cmps"]["stromer_12345_battery_soc"];

        assert_eq!(payload["name"], "Battery SOC");
        assert_eq!(payload["stat_t"], "stromer/stromer_12345/battery_soc");
        assert_eq!(discovery["avty_t"], "stromer/state");
        assert_eq!(payload["unit_of_measurement"], "%");
        assert_eq!(payload["device_class"], "battery");
        assert_eq!(payload["state_class"], "measurement");
        assert_eq!(discovery["origin"]["name"], env!("CARGO_PKG_NAME"));
        assert_eq!(discovery["origin"]["sw"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_binary_sensor_payload() {
        let discovery = device_payload("stromer/");
        let payload = &discovery["cmps"]["stromer_12345_light_on"];
        assert_eq!(payload["stat_t"], "stromer/stromer_12345/light_on");

        assert_eq!(payload["name"], "Light");
        assert_eq!(payload["payload_on"], "true");
        assert_eq!(payload["payload_off"], "false");
        assert_eq!(payload["icon"], "mdi:lightbulb");
    }

    #[test]
    fn test_switch_payload() {
        let discovery = device_payload("stromer/");
        let payload = &discovery["cmps"]["stromer_12345_lock_flag_sw"];
        assert_eq!(payload["stat_t"], "stromer/stromer_12345/lock_flag");

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
        let discovery = device_payload("stromer/");
        let payload = &discovery["cmps"]["stromer_12345_location"];

        assert_eq!(payload["name"], "Location");
        assert_eq!(payload["json_attr_t"], "stromer/stromer_12345/position");
        assert_eq!(payload["source_type"], "gps");
    }

    #[test]
    fn test_shared_device_info() {
        let payload = device_payload("stromer/");
        assert_eq!(payload["device"]["name"], "Stromer MyBike");
        assert_eq!(payload["device"]["manufacturer"], "Stromer");
        assert_eq!(payload["device"]["model"], "ST5");
        assert_eq!(
            payload["device"]["identifiers"],
            serde_json::json!(["stromer_12345"])
        );
    }

    #[test]
    fn test_custom_base_topic() {
        let payload = device_payload("home/bikes/");
        assert_eq!(payload["avty_t"], "home/bikes/state");
        assert_eq!(
            payload["cmps"]["stromer_12345_battery_soc"]["stat_t"],
            "home/bikes/stromer_12345/battery_soc"
        );
        assert_eq!(
            payload["cmps"]["stromer_12345_lock_flag_sw"]["cmd_t"],
            "home/bikes/stromer_12345/cmnd/lock"
        );
        assert_eq!(
            payload["cmps"]["stromer_12345_location"]["json_attr_t"],
            "home/bikes/stromer_12345/position"
        );
    }

    #[test]
    fn test_bikes_have_distinct_devices_and_entities() {
        let other_bike = StromerBike::new("67890".into(), "OtherBike".into(), "ST3".into(), None);
        let messages = create_discovery_messages(&other_bike, "stromer/").unwrap();
        let device = messages
            .iter()
            .find(|m| m.topic == "homeassistant/device/stromer_67890/config")
            .unwrap();
        let payload: serde_json::Value = serde_json::from_str(&device.payload).unwrap();
        assert_eq!(
            payload["device"]["identifiers"],
            serde_json::json!(["stromer_67890"])
        );
        for (id, component) in payload["cmps"].as_object().unwrap() {
            assert!(id.starts_with("stromer_67890_"));
            assert_eq!(component["unique_id"], id.as_str());
        }
    }
}
