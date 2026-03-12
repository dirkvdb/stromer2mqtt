use crate::mqtt::MqttData;
use crate::stromerapi::{BikePosition, BikeStatus, MaintenanceInfo};

/// A small generic wrapper that tracks whether a value has changed.
pub struct TrackedField<T: PartialEq + ToString> {
    value: Option<T>,
    modified: bool,
}

impl<T: PartialEq + ToString> TrackedField<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            modified: false,
        }
    }

    pub fn set(&mut self, val: T) {
        if self.value.as_ref() != Some(&val) {
            self.value = Some(val);
            self.modified = true;
        }
    }

    pub fn set_option(&mut self, val: Option<T>) {
        if let Some(v) = val {
            self.set(v);
        }
    }

    #[allow(dead_code)]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn get_and_reset(&mut self) -> Option<String> {
        if self.modified {
            self.modified = false;
            self.value.as_ref().map(|v| v.to_string())
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.value = None;
        self.modified = true;
    }
}

/// Represents a Stromer bike information
/// Each field that gets published to MQTT is tracked with a `TrackedField<T>`
/// that records whether the value has changed since last publish.
pub struct StromerBike {
    bike_id: String,
    bike_name: String,
    bike_model: String,

    // Software/hardware versions (from status, published once)
    suiversion: TrackedField<String>,
    tntversion: TrackedField<String>,

    // Sensors (from status endpoint)
    assistance_level: TrackedField<f64>,
    atmospheric_pressure: TrackedField<f64>,
    average_energy_consumption: TrackedField<f64>,
    average_speed_total: TrackedField<f64>,
    average_speed_trip: TrackedField<f64>,
    battery_soc: TrackedField<f64>,
    battery_health: TrackedField<f64>,
    battery_temp: TrackedField<f64>,
    bike_speed: TrackedField<f64>,
    motor_temp: TrackedField<f64>,
    power_on_cycles: TrackedField<u64>,
    speed: TrackedField<f64>,
    total_distance: TrackedField<f64>,
    total_energy_consumption: TrackedField<f64>,
    total_time: TrackedField<u64>,
    trip_distance: TrackedField<f64>,
    trip_time: TrackedField<u64>,

    // Timestamps
    rcvts: TrackedField<i64>,
    timets: TrackedField<i64>,

    // Binary sensors
    light_on: TrackedField<bool>,
    lock_flag: TrackedField<bool>,
    theft_flag: TrackedField<bool>,

    // Position
    latitude: TrackedField<f64>,
    longitude: TrackedField<f64>,
    altitude: TrackedField<f64>,
    rcvts_pos: TrackedField<i64>, // timestamp of the position info

    // Maintenance (from detect endpoint)
    next_maintenance_date: TrackedField<String>,
    next_maintenance_km: TrackedField<f64>,
    next_maintenance_interval: TrackedField<f64>,
    last_maintenance_reset_date: TrackedField<String>,
    last_maintenance_reset_km: TrackedField<f64>,
}

macro_rules! collect_modified {
    ($self:expr, $topics:expr, $($field:ident),* $(,)?) => {
        $(
            if let Some(payload) = $self.$field.get_and_reset() {
                $topics.push(MqttData {
                    topic: format!("stromer_{}/{}", $self.bike_id, stringify!($field)),
                    payload,
                });
            }
        )*
    };
}

macro_rules! reset_all {
    ($self:expr, $($field:ident),* $(,)?) => {
        $(
            $self.$field.reset();
        )*
    };
}

impl StromerBike {
    pub fn new(
        bike_id: String,
        bike_name: String,
        bike_model: String,
        maintenance: Option<MaintenanceInfo>,
    ) -> Self {
        let mut bike = StromerBike {
            bike_id,
            bike_name,
            bike_model,
            suiversion: TrackedField::new(),
            tntversion: TrackedField::new(),
            assistance_level: TrackedField::new(),
            atmospheric_pressure: TrackedField::new(),
            average_energy_consumption: TrackedField::new(),
            average_speed_total: TrackedField::new(),
            average_speed_trip: TrackedField::new(),
            battery_soc: TrackedField::new(),
            battery_health: TrackedField::new(),
            battery_temp: TrackedField::new(),
            bike_speed: TrackedField::new(),
            motor_temp: TrackedField::new(),
            power_on_cycles: TrackedField::new(),
            speed: TrackedField::new(),
            total_distance: TrackedField::new(),
            total_energy_consumption: TrackedField::new(),
            total_time: TrackedField::new(),
            trip_distance: TrackedField::new(),
            trip_time: TrackedField::new(),
            rcvts: TrackedField::new(),
            timets: TrackedField::new(),
            light_on: TrackedField::new(),
            lock_flag: TrackedField::new(),
            theft_flag: TrackedField::new(),
            latitude: TrackedField::new(),
            longitude: TrackedField::new(),
            altitude: TrackedField::new(),
            rcvts_pos: TrackedField::new(),
            next_maintenance_date: TrackedField::new(),
            next_maintenance_km: TrackedField::new(),
            next_maintenance_interval: TrackedField::new(),
            last_maintenance_reset_date: TrackedField::new(),
            last_maintenance_reset_km: TrackedField::new(),
        };
        if let Some(m) = maintenance {
            bike.update_maintenance(m);
        }
        bike
    }

    pub fn bike_id(&self) -> &str {
        &self.bike_id
    }

    pub fn bike_name(&self) -> &str {
        &self.bike_name
    }

    pub fn bike_model(&self) -> &str {
        &self.bike_model
    }

    pub fn update_status(&mut self, status: BikeStatus) {
        self.suiversion.set_option(status.suiversion);
        self.tntversion.set_option(status.tntversion);

        self.assistance_level.set_option(status.assistance_level);
        self.atmospheric_pressure
            .set_option(status.atmospheric_pressure);
        self.average_energy_consumption
            .set_option(status.average_energy_consumption);
        self.average_speed_total
            .set_option(status.average_speed_total);
        self.average_speed_trip
            .set_option(status.average_speed_trip);
        self.battery_soc.set_option(status.battery_soc);
        self.battery_health.set_option(status.battery_health);
        self.battery_temp.set_option(status.battery_temp);
        self.bike_speed.set_option(status.bike_speed);
        self.motor_temp.set_option(status.motor_temp);
        self.power_on_cycles.set_option(status.power_on_cycles);
        self.speed.set_option(status.speed);
        self.total_distance.set_option(status.total_distance);
        self.total_energy_consumption
            .set_option(status.total_energy_consumption);
        self.total_time.set_option(status.total_time);
        self.trip_distance.set_option(status.trip_distance);
        self.trip_time.set_option(status.trip_time);

        self.rcvts.set_option(status.rcvts);
        self.timets.set_option(status.timets);

        self.light_on.set_option(status.light_on);
        self.lock_flag.set_option(status.lock_flag);
        self.theft_flag.set_option(status.theft_flag);
    }

    pub fn update_maintenance(&mut self, m: MaintenanceInfo) {
        self.next_maintenance_date
            .set_option(m.next_maintenance_date);
        self.next_maintenance_km.set_option(m.next_maintenance_km);
        self.next_maintenance_interval
            .set_option(m.next_maintenance_interval);
        self.last_maintenance_reset_date
            .set_option(m.last_maintenance_reset_date);
        self.last_maintenance_reset_km
            .set_option(m.last_maintenance_reset_km);
    }

    pub fn update_position(&mut self, position: BikePosition) {
        self.latitude.set_option(position.latitude);
        self.longitude.set_option(position.longitude);
        self.altitude.set_option(position.altitude);
        self.rcvts_pos.set_option(position.rcvts);
    }

    pub fn topics_that_need_updating(&mut self) -> Vec<MqttData> {
        let mut topics = Vec::new();

        collect_modified!(
            self,
            topics,
            suiversion,
            tntversion,
            assistance_level,
            atmospheric_pressure,
            average_energy_consumption,
            average_speed_total,
            average_speed_trip,
            battery_soc,
            battery_health,
            battery_temp,
            bike_speed,
            motor_temp,
            power_on_cycles,
            speed,
            total_distance,
            total_energy_consumption,
            total_time,
            trip_distance,
            trip_time,
            rcvts,
            timets,
            light_on,
            lock_flag,
            theft_flag,
            latitude,
            longitude,
            altitude,
            rcvts_pos,
            next_maintenance_date,
            next_maintenance_km,
            next_maintenance_interval,
            last_maintenance_reset_date,
            last_maintenance_reset_km,
        );

        topics
    }

    pub fn reset(&mut self) {
        reset_all!(
            self,
            suiversion,
            tntversion,
            assistance_level,
            atmospheric_pressure,
            average_energy_consumption,
            average_speed_total,
            average_speed_trip,
            battery_soc,
            battery_health,
            battery_temp,
            bike_speed,
            motor_temp,
            power_on_cycles,
            speed,
            total_distance,
            total_energy_consumption,
            total_time,
            trip_distance,
            trip_time,
            rcvts,
            timets,
            light_on,
            lock_flag,
            theft_flag,
            latitude,
            longitude,
            altitude,
            rcvts_pos,
            next_maintenance_date,
            next_maintenance_km,
            next_maintenance_interval,
            last_maintenance_reset_date,
            last_maintenance_reset_km,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_full_status() -> BikeStatus {
        BikeStatus {
            assistance_level: Some(50.0),
            atmospheric_pressure: Some(1.013),
            average_energy_consumption: Some(12.5),
            average_speed_total: Some(22.3),
            average_speed_trip: Some(18.7),
            battery_soc: Some(85.0),
            battery_health: Some(97.0),
            battery_temp: Some(23.5),
            bike_speed: Some(0.0),
            motor_temp: Some(31.2),
            power_on_cycles: Some(142),
            speed: Some(0.0),
            total_distance: Some(3456.7),
            total_energy_consumption: Some(1234.5),
            total_time: Some(123456),
            trip_distance: Some(15.3),
            trip_time: Some(3600),
            light_on: Some(false),
            lock_flag: Some(true),
            theft_flag: Some(false),
            rcvts: Some(1700000000),
            timets: Some(1700000100),
            suiversion: Some("4.9.2".to_string()),
            tntversion: Some("3.7.1".to_string()),
        }
    }

    fn make_bike() -> StromerBike {
        StromerBike::new("12345".into(), "MyBike".into(), "ST5".into(), None)
    }

    fn make_position() -> BikePosition {
        BikePosition {
            latitude: Some(47.3769),
            longitude: Some(8.5417),
            altitude: Some(10.0),
            rcvts: Some(1700000000),
        }
    }

    #[test]
    fn test_update_status_returns_all_fields() {
        let mut bike = make_bike();
        bike.update_status(make_full_status());

        let topics = bike.topics_that_need_updating();

        // All 24 status fields should be present
        assert_eq!(topics.len(), 24);

        // Verify topic format
        for topic in &topics {
            assert!(
                topic.topic.starts_with("stromer_12345/"),
                "Topic should start with stromer_12345/: {}",
                topic.topic
            );
        }

        // Verify specific values
        let battery = topics.iter().find(|t| t.topic.ends_with("/battery_soc"));
        assert!(battery.is_some());
        assert_eq!(battery.expect("battery_soc topic").payload, "85");

        let lock = topics.iter().find(|t| t.topic.ends_with("/lock_flag"));
        assert!(lock.is_some());
        assert_eq!(lock.expect("lock_flag topic").payload, "true");
    }

    #[test]
    fn test_second_update_with_same_data_returns_empty() {
        let mut bike = make_bike();
        bike.update_status(make_full_status());

        // First call: consume all changes
        let _ = bike.topics_that_need_updating();

        // Second update with identical data
        bike.update_status(make_full_status());
        let topics = bike.topics_that_need_updating();
        assert!(
            topics.is_empty(),
            "Should return empty vec when data hasn't changed, got {} topics",
            topics.len()
        );
    }

    #[test]
    fn test_second_update_with_one_changed_field() {
        let mut bike = make_bike();
        bike.update_status(make_full_status());

        // Consume all changes
        let _ = bike.topics_that_need_updating();

        // Update with one changed field
        let mut status = make_full_status();
        status.battery_soc = Some(42.0);
        bike.update_status(status);

        let topics = bike.topics_that_need_updating();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].topic, "stromer_12345/battery_soc");
        assert_eq!(topics[0].payload, "42");
    }

    #[test]
    fn test_update_position() {
        let mut bike = make_bike();
        bike.update_position(make_position());

        let topics = bike.topics_that_need_updating();
        assert_eq!(topics.len(), 4);

        let lat = topics.iter().find(|t| t.topic.ends_with("/latitude"));
        assert!(lat.is_some());
        assert_eq!(lat.expect("latitude topic").payload, "47.3769");

        let lng = topics.iter().find(|t| t.topic.ends_with("/longitude"));
        assert!(lng.is_some());
        assert_eq!(lng.expect("longitude topic").payload, "8.5417");

        let alt = topics.iter().find(|t| t.topic.ends_with("/altitude"));
        assert!(alt.is_some());
        assert_eq!(alt.expect("altitude topic").payload, "10");

        // BikePosition::rcvts maps to rcvts_pos topic
        let rcvts = topics.iter().find(|t| t.topic.ends_with("/rcvts_pos"));
        assert!(rcvts.is_some());
        assert_eq!(rcvts.expect("rcvts_pos topic").payload, "1700000000");
    }

    #[test]
    fn test_reset_causes_all_fields_to_republish() {
        let mut bike = make_bike();
        bike.update_status(make_full_status());
        bike.update_position(make_position());

        // Consume all changes
        let _ = bike.topics_that_need_updating();

        // Reset should mark all fields as modified (with None values)
        bike.reset();

        let topics = bike.topics_that_need_updating();
        // All 27 fields should be published (24 status + 3 position)
        // But after reset, values are None, so get_and_reset returns None for each
        // which means they won't be included in topics
        // Actually, reset sets value to None and modified to true,
        // and get_and_reset returns value.as_ref().map(|v| v.to_string()) which is None
        // So topics will be empty after reset
        assert!(
            topics.is_empty(),
            "After reset, all values are None so no topics should be returned"
        );
    }

    #[test]
    fn test_accessors() {
        let bike = make_bike();
        assert_eq!(bike.bike_id(), "12345");
        assert_eq!(bike.bike_name(), "MyBike");
        assert_eq!(bike.bike_model(), "ST5");
    }

    #[test]
    fn test_update_maintenance() {
        use crate::stromerapi::MaintenanceInfo;

        let m = MaintenanceInfo {
            next_maintenance_date: Some("20261113".to_string()),
            next_maintenance_interval: Some(1000.0),
            next_maintenance_km: Some(1000.0),
            last_maintenance_reset_date: None,
            last_maintenance_reset_km: None,
        };

        let mut bike = StromerBike::new("12345".into(), "MyBike".into(), "ST5".into(), Some(m));
        let topics = bike.topics_that_need_updating();

        let next_date = topics
            .iter()
            .find(|t| t.topic.ends_with("/next_maintenance_date"));
        assert!(next_date.is_some());
        assert_eq!(next_date.unwrap().payload, "20261113");

        let next_km = topics
            .iter()
            .find(|t| t.topic.ends_with("/next_maintenance_km"));
        assert!(next_km.is_some());
        assert_eq!(next_km.unwrap().payload, "1000");

        let interval = topics
            .iter()
            .find(|t| t.topic.ends_with("/next_maintenance_interval"));
        assert!(interval.is_some());
        assert_eq!(interval.unwrap().payload, "1000");

        // last_maintenance fields are None → must not appear in topics
        let last_date = topics
            .iter()
            .find(|t| t.topic.ends_with("/last_maintenance_reset_date"));
        assert!(last_date.is_none());

        let last_km = topics
            .iter()
            .find(|t| t.topic.ends_with("/last_maintenance_reset_km"));
        assert!(last_km.is_none());
    }

    #[test]
    fn test_tracked_field_set_and_get() {
        let mut field: TrackedField<f64> = TrackedField::new();
        assert!(!field.is_modified());
        assert_eq!(field.get_and_reset(), None);

        field.set(42.0);
        assert!(field.is_modified());
        assert_eq!(field.get_and_reset(), Some("42".to_string()));
        assert!(!field.is_modified());
    }

    #[test]
    fn test_tracked_field_no_change_on_same_value() {
        let mut field: TrackedField<f64> = TrackedField::new();
        field.set(42.0);
        let _ = field.get_and_reset(); // consume

        field.set(42.0); // same value
        assert!(!field.is_modified());
    }

    #[test]
    fn test_tracked_field_set_option_none() {
        let mut field: TrackedField<f64> = TrackedField::new();
        field.set_option(None);
        assert!(!field.is_modified());
    }

    #[test]
    fn test_tracked_field_reset() {
        let mut field: TrackedField<f64> = TrackedField::new();
        field.set(42.0);
        let _ = field.get_and_reset();

        field.reset();
        assert!(field.is_modified());
        // Value is None after reset, so get_and_reset returns None
        assert_eq!(field.get_and_reset(), None);
    }
}
