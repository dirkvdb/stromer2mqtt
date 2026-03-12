use crate::Result;
use anyhow::anyhow;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;
use reqwest::redirect::Policy;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

fn bool_from_int_or_bool<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<bool>, D::Error> {
    use serde::de::{Error, Unexpected, Visitor};
    use std::fmt;

    struct BoolOrInt;

    impl<'de> Visitor<'de> for BoolOrInt {
        type Value = Option<bool>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a boolean or 0/1 integer, or null")
        }

        fn visit_bool<E: Error>(self, v: bool) -> std::result::Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
            match v {
                0 => Ok(Some(false)),
                1 => Ok(Some(true)),
                _ => Err(E::invalid_value(Unexpected::Signed(v), &self)),
            }
        }

        fn visit_u64<E: Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
            match v {
                0 => Ok(Some(false)),
                1 => Ok(Some(true)),
                _ => Err(E::invalid_value(Unexpected::Unsigned(v), &self)),
            }
        }

        fn visit_none<E: Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }
    }

    d.deserialize_any(BoolOrInt)
}

fn build_auth_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .build()
}

#[derive(Debug, Clone, PartialEq)]
enum ApiVersion {
    V3,
    V4,
}

pub struct StromerApi {
    base_url: String,
    api_version: ApiVersion,
    username: String,
    password: String,
    client_id: String,
    client_secret: Option<String>,
    token: Option<String>,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceInfo {
    pub last_maintenance_reset_date: Option<String>,
    pub last_maintenance_reset_km: Option<f64>,
    pub next_maintenance_date: Option<String>,
    pub next_maintenance_interval: Option<f64>,
    pub next_maintenance_km: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BikeDetectInfo {
    #[serde(rename = "bikeid")]
    pub bike_id: serde_json::Value, // may be integer or string
    pub nickname: String,
    #[serde(rename = "biketype")]
    pub bike_type: String,
    #[serde(rename = "bikemodel")]
    pub bike_model: Option<String>,
    pub maintenance_feature: Option<MaintenanceInfo>,
}

impl BikeDetectInfo {
    pub fn bike_id_string(&self) -> String {
        match &self.bike_id {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BikeStatus {
    pub assistance_level: Option<f64>,
    pub atmospheric_pressure: Option<f64>,
    pub average_energy_consumption: Option<f64>,
    pub average_speed_total: Option<f64>,
    pub average_speed_trip: Option<f64>,
    #[serde(rename = "battery_SOC")]
    pub battery_soc: Option<f64>,
    pub battery_health: Option<f64>,
    pub battery_temp: Option<f64>,
    pub bike_speed: Option<f64>,
    pub motor_temp: Option<f64>,
    pub power_on_cycles: Option<u64>,
    pub speed: Option<f64>,
    pub total_distance: Option<f64>,
    pub total_energy_consumption: Option<f64>,
    pub total_time: Option<u64>,
    pub trip_distance: Option<f64>,
    pub trip_time: Option<u64>,
    #[serde(default, deserialize_with = "bool_from_int_or_bool")]
    pub light_on: Option<bool>,
    #[serde(default, deserialize_with = "bool_from_int_or_bool")]
    pub lock_flag: Option<bool>,
    #[serde(default, deserialize_with = "bool_from_int_or_bool")]
    pub theft_flag: Option<bool>,
    pub rcvts: Option<i64>,
    pub timets: Option<i64>,
    pub suiversion: Option<String>,
    pub tntversion: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BikePosition {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub rcvts: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: serde_json::Value,
}

impl StromerApi {
    pub fn new(
        username: String,
        password: String,
        client_id: String,
        client_secret: Option<String>,
    ) -> Self {
        let api_version = if client_secret.is_some() {
            ApiVersion::V3
        } else {
            ApiVersion::V4
        };

        log::debug!("Initializing Stromer API with version {:?}", api_version);

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build HTTP client");

        StromerApi {
            base_url: String::from("https://api3.stromer-portal.ch"),
            api_version,
            username,
            password,
            client_id,
            client_secret,
            token: None,
            client,
        }
    }

    fn login_url(&self) -> String {
        match self.api_version {
            ApiVersion::V3 => format!("{}/users/login/", self.base_url),
            ApiVersion::V4 => format!("{}/mobile/v4/login/", self.base_url),
        }
    }

    fn token_url(&self) -> String {
        match self.api_version {
            ApiVersion::V3 => format!("{}/o/token/", self.base_url),
            ApiVersion::V4 => format!("{}/mobile/v4/o/token/", self.base_url),
        }
    }

    fn authorize_path(&self) -> &str {
        match self.api_version {
            ApiVersion::V3 => "/o/authorize/",
            ApiVersion::V4 => "/mobile/v4/o/authorize/",
        }
    }

    fn api_prefix(&self) -> &str {
        match self.api_version {
            ApiVersion::V3 => "/rapi/mobile/v2/",
            ApiVersion::V4 => "/rapi/mobile/v4.1/",
        }
    }

    fn api_url(&self, endpoint: &str) -> String {
        format!("{}{}{}", self.base_url, self.api_prefix(), endpoint)
    }

    fn redirect_uri(&self) -> &str {
        match self.api_version {
            ApiVersion::V3 => "stromerauth://auth",
            ApiVersion::V4 => "stromer://auth",
        }
    }

    pub async fn stromer_connect(&mut self) -> Result<()> {
        log::debug!("Connecting to Stromer API");

        // Use a single no-redirect client with a cookie jar throughout the entire auth flow.
        let auth_client = build_auth_client()?;
        let login_url = self.login_url();

        // GET login page to obtain the CSRF token from Set-Cookie.
        let res = auth_client.get(&login_url).send().await?;
        let cookie_header = res
            .headers()
            .get("set-cookie")
            .ok_or_else(|| anyhow!("No Set-Cookie header in login response"))?
            .to_str()
            .map_err(|e| anyhow!("Invalid Set-Cookie header: {}", e))?
            .to_owned();

        let pattern = Regex::new(r"=(.*?);").map_err(|e| anyhow!("Regex error: {}", e))?;
        let csrftoken = pattern
            .captures(&cookie_header)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| anyhow!("Failed to extract CSRF token from Set-Cookie header"))?;

        log::debug!("Got CSRF token");

        // POST credentials to the login page.
        let scope = "bikeposition bikestatus bikeconfiguration bikelock biketheft bikedata bikepin bikeblink userprofile";
        let authorize_path = self.authorize_path().to_string();
        let qs = format!(
            "client_id={}&response_type=code&redirect_url={}&scope={}",
            utf8_percent_encode(&self.client_id, NON_ALPHANUMERIC),
            utf8_percent_encode(self.redirect_uri(), NON_ALPHANUMERIC),
            utf8_percent_encode(scope, NON_ALPHANUMERIC),
        );
        let next = format!("{}?{}", authorize_path, qs);

        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("username", self.username.clone());
        form.insert("password", self.password.clone());
        form.insert("csrfmiddlewaretoken", csrftoken);
        form.insert("next", next);

        let res = auth_client
            .post(&login_url)
            .header("Referer", &login_url)
            .form(&form)
            .send()
            .await?;

        let location = res
            .headers()
            .get("location")
            .ok_or_else(|| anyhow!("No Location header after login POST"))?
            .to_str()
            .map_err(|e| anyhow!("Invalid Location header: {}", e))?
            .to_owned();

        log::debug!("Got redirect location after login");

        // Follow the authorize redirect (still no auto-redirect).
        if !location.starts_with('/') && !location.starts_with('?') {
            return Err(anyhow!(
                "Invalid next location: '{}'. Expected start with '/' or '?'.",
                location
            ));
        }
        let next_url = format!("{}{}", self.base_url, location);

        let res = auth_client.get(&next_url).send().await?;
        let code_location = res
            .headers()
            .get("location")
            .ok_or_else(|| anyhow!("No Location header after authorize redirect"))?
            .to_str()
            .map_err(|e| anyhow!("Invalid Location header: {}", e))?
            .to_owned();

        // Extract code from location (e.g. "stromer://auth?code=XXXX" or "stromerauth://auth?code=XXXX").
        let code = code_location
            .split("code=")
            .nth(1)
            .ok_or_else(|| {
                anyhow!(
                    "Failed to extract authorization code from: {}",
                    code_location
                )
            })?
            .split('&')
            .next()
            .ok_or_else(|| anyhow!("Failed to parse authorization code"))?
            .to_string();

        log::debug!("Got authorization code");

        // Step 4: Exchange authorization code for an access token.
        let token_url = self.token_url();
        let mut token_form: HashMap<&str, String> = HashMap::new();
        token_form.insert("grant_type", "authorization_code".to_string());
        token_form.insert("client_id", self.client_id.clone());
        token_form.insert("code", code);
        token_form.insert("redirect_uri", self.redirect_uri().to_string());

        if self.api_version == ApiVersion::V3 {
            if let Some(ref secret) = self.client_secret {
                token_form.insert("client_secret", secret.clone());
            }
        }

        let res = auth_client
            .post(&token_url)
            .form(&token_form)
            .send()
            .await?;

        let token_response: serde_json::Value = res.json().await?;
        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("No access_token in token response: {}", token_response))?
            .to_string();

        self.token = Some(access_token);
        // Rebuild the main client with a fresh cookie jar now that auth is done.
        self.client = reqwest::Client::builder().cookie_store(true).build()?;

        log::info!("Stromer API connected successfully");
        Ok(())
    }

    fn auth_header(&self) -> Result<String> {
        match &self.token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err(anyhow!("Not authenticated - call stromer_connect() first")),
        }
    }

    pub async fn stromer_detect(&self) -> Result<Vec<BikeDetectInfo>> {
        let url = self.api_url("bike/");
        let auth = self.auth_header()?;

        let res = self
            .client
            .get(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(anyhow!("Bike detection failed with status: {}", status));
        }

        let response: ApiResponse = res.json().await?;
        log::debug!("stromer_detect response: {}", response.data);
        let data = response
            .data
            .as_array()
            .ok_or_else(|| anyhow!("Expected data array in detect response"))?;

        let bikes: Vec<BikeDetectInfo> = data
            .iter()
            .filter_map(|item| match serde_json::from_value(item.clone()) {
                Ok(info) => Some(info),
                Err(e) => {
                    log::warn!("Failed to parse bike detect entry: {}", e);
                    None
                }
            })
            .collect();

        Ok(bikes)
    }

    async fn stromer_api_call(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = self.api_url(endpoint);
        let auth = self.auth_header()?;

        log::debug!("API call: GET {}", url);

        let res = self
            .client
            .get(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        let status = res.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!("Authentication expired (HTTP 401)"));
        }
        if !status.is_success() {
            return Err(anyhow!("API call failed with status: {}", status));
        }

        let response: ApiResponse = res.json().await?;
        log::debug!("API response for {}: {}", endpoint, response.data);
        Ok(response.data)
    }

    pub async fn stromer_get_status(&self, bike_id: &str) -> Result<BikeStatus> {
        let endpoint = format!("bike/{}/state/", bike_id);
        let data = self.stromer_api_call(&endpoint).await?;

        let first = if let Some(arr) = data.as_array() {
            arr.first()
                .ok_or_else(|| anyhow!("Empty data array in status response"))?
                .clone()
        } else {
            data
        };

        let status: BikeStatus = serde_json::from_value(first)?;
        Ok(status)
    }

    pub async fn stromer_get_position(&self, bike_id: &str) -> Result<BikePosition> {
        let endpoint = format!("bike/{}/position/", bike_id);
        let data = self.stromer_api_call(&endpoint).await?;

        let first = if let Some(arr) = data.as_array() {
            arr.first()
                .ok_or_else(|| anyhow!("Empty data array in position response"))?
                .clone()
        } else {
            data
        };

        let position: BikePosition = serde_json::from_value(first)?;
        Ok(position)
    }

    pub async fn stromer_call_lock(&self, bike_id: &str, state: bool) -> Result<()> {
        let endpoint = format!("bike/{}/settings/", bike_id);
        let url = self.api_url(&endpoint);
        let auth = self.auth_header()?;

        let mut body = HashMap::new();
        body.insert("lock", serde_json::Value::Bool(state));

        let res = self
            .client
            .post(&url)
            .header("Authorization", &auth)
            .json(&body)
            .send()
            .await?;

        log::debug!("Lock API call status: {}", res.status());
        let ret: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);
        log::debug!("Lock API call returns: {}", ret);

        Ok(())
    }

    pub async fn stromer_call_light(&self, bike_id: &str, mode: &str) -> Result<()> {
        let endpoint = format!("bike/{}/light/", bike_id);
        let url = self.api_url(&endpoint);
        let auth = self.auth_header()?;

        let mut body = HashMap::new();
        body.insert("mode", serde_json::Value::String(mode.to_string()));

        let res = self
            .client
            .post(&url)
            .header("Authorization", &auth)
            .json(&body)
            .send()
            .await?;

        log::debug!("Light API call status: {}", res.status());
        let ret: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);
        log::debug!("Light API call returns: {}", ret);

        Ok(())
    }

    pub async fn stromer_update(&mut self, bike_id: &str) -> Result<(BikeStatus, BikePosition)> {
        let mut attempts = 0;
        loop {
            if attempts >= 10 {
                return Err(anyhow!("Stromer API call failed 10 times, failing"));
            }

            if attempts == 5 {
                log::info!("Reconnecting to Stromer API");
                self.stromer_connect().await?;
            }

            attempts += 1;
            log::debug!("Stromer attempt: {}/10", attempts);

            match self.stromer_get_status(bike_id).await {
                Ok(status) => match self.stromer_get_position(bike_id).await {
                    Ok(position) => {
                        return Ok((status, position));
                    }
                    Err(e) => {
                        log::error!("Stromer error: position call failed: {:#}", e);
                        log::debug!("Stromer retry: {}/10", attempts);
                    }
                },
                Err(e) => {
                    log::error!("Stromer error: status call failed: {:#}", e);
                    log::debug!("Stromer retry: {}/10", attempts);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_json() {
        let json = serde_json::json!({
            "assistance_level": 50.0,
            "atmospheric_pressure": 1.013,
            "average_energy_consumption": 12.5,
            "average_speed_total": 22.3,
            "average_speed_trip": 18.7,
            "battery_SOC": 85.0,
            "battery_health": 97.0,
            "battery_temp": 23.5,
            "bike_speed": 0.0,
            "motor_temp": 31.2,
            "power_on_cycles": 142,
            "speed": 0.0,
            "total_distance": 3456.7,
            "total_energy_consumption": 1234.5,
            "total_time": 123456,
            "trip_distance": 15.3,
            "trip_time": 3600,
            "light_on": false,
            "lock_flag": true,
            "theft_flag": false,
            "rcvts": 1700000000_i64,
            "timets": 1700000100_i64,
            "suiversion": "4.9.2",
            "tntversion": "3.7.1"
        });

        let status: BikeStatus = serde_json::from_value(json).expect("Failed to parse status");
        assert_eq!(status.assistance_level, Some(50.0));
        assert_eq!(status.battery_soc, Some(85.0));
        assert_eq!(status.power_on_cycles, Some(142));
        assert_eq!(status.light_on, Some(false));
        assert_eq!(status.lock_flag, Some(true));
        assert_eq!(status.suiversion, Some("4.9.2".to_string()));
    }

    /// Verbatim state response observed from the real Stromer cloud API for bike 137011.
    /// Notable quirks:
    ///   - `light_on` is integer 0 (not boolean false)
    ///   - `lock_flag` is boolean true (not integer 1)
    ///   - `theft_flag` is boolean false (not integer 0)
    ///   - `speed` and `timets` are absent → None
    ///   - `atmospheric_pressure`, `average_speed_trip` etc. are present but some are 0
    #[test]
    fn test_parse_real_status_response() {
        // The response is wrapped in an array by the API; we parse data[0].
        let raw = serde_json::json!([{
            "assistance_level": 0,
            "atmospheric_pressure": 0,
            "average_energy_consumption": 15,
            "average_speed_total": 37.2,
            "average_speed_trip": 37.2,
            "battery_SOC": 100,
            "battery_health": 96,
            "battery_temp": 21.2,
            "bike_speed": 0.0,
            "light_on": 0,
            "lock_flag": true,
            "motor_temp": 13.8,
            "power_on_cycles": 41,
            "rcvts": 1773344955_i64,
            "suiversion": "4.5.1.6",
            "theft_flag": false,
            "tntversion": "4.7",
            "total_distance": 826.8,
            "total_energy_consumption": 13149,
            "total_time": 79942,
            "trip_distance": 826.8,
            "trip_time": 79942
        }]);

        let first = raw.as_array().unwrap()[0].clone();
        let status: BikeStatus =
            serde_json::from_value(first).expect("Failed to parse real status response");

        // Numeric sensors
        assert_eq!(status.assistance_level, Some(0.0));
        assert_eq!(status.atmospheric_pressure, Some(0.0));
        assert_eq!(status.average_energy_consumption, Some(15.0));
        assert_eq!(status.average_speed_total, Some(37.2));
        assert_eq!(status.average_speed_trip, Some(37.2));
        assert_eq!(status.battery_soc, Some(100.0));
        assert_eq!(status.battery_health, Some(96.0));
        assert_eq!(status.battery_temp, Some(21.2));
        assert_eq!(status.bike_speed, Some(0.0));
        assert_eq!(status.motor_temp, Some(13.8));
        assert_eq!(status.power_on_cycles, Some(41));
        assert_eq!(status.total_distance, Some(826.8));
        assert_eq!(status.total_energy_consumption, Some(13149.0));
        assert_eq!(status.total_time, Some(79942));
        assert_eq!(status.trip_distance, Some(826.8));
        assert_eq!(status.trip_time, Some(79942));

        // Booleans: mixed integer (0) and actual bool (true/false) in the same response
        assert_eq!(status.light_on, Some(false)); // came as integer 0
        assert_eq!(status.lock_flag, Some(true)); // came as boolean true
        assert_eq!(status.theft_flag, Some(false)); // came as boolean false

        // Timestamps
        assert_eq!(status.rcvts, Some(1773344955));

        // Fields absent from this response
        assert_eq!(status.speed, None);
        assert_eq!(status.timets, None);

        // Version strings
        assert_eq!(status.suiversion, Some("4.5.1.6".to_string()));
        assert_eq!(status.tntversion, Some("4.7".to_string()));
    }

    /// Verbatim position response observed from the real Stromer cloud API for bike 137011.
    /// Notable: `altitude`, `speed`, and `timets` are extra fields not in BikePosition
    /// and must be silently ignored by serde.
    #[test]
    fn test_parse_real_position_response() {
        // The response is wrapped in an array by the API; we parse data[0].
        let raw = serde_json::json!([{
            "altitude": 10.0,
            "latitude": 52.0,
            "longitude": 4.0,
            "rcvts": 1773336081_i64,
            "speed": 0.219,
            "timets": 1773335724_i64
        }]);

        let first = raw.as_array().unwrap()[0].clone();
        let position: BikePosition =
            serde_json::from_value(first).expect("Failed to parse real position response");

        assert_eq!(position.latitude, Some(52.0));
        assert_eq!(position.longitude, Some(4.0));
        assert_eq!(position.altitude, Some(10.0));
        assert_eq!(position.rcvts, Some(1773336081));
        // speed and timets are unknown fields and must be ignored (not cause an error)
    }

    #[test]
    fn test_parse_status_json_integer_booleans() {
        // The real Stromer API returns 0/1 integers for boolean fields.
        let json = serde_json::json!({
            "assistance_level": 3.0,
            "battery_SOC": 72.0,
            "speed": 0.0,
            "light_on": 0,
            "lock_flag": 1,
            "theft_flag": 0,
            "rcvts": 1700000000_i64,
            "timets": 1700000100_i64,
        });

        let status: BikeStatus =
            serde_json::from_value(json).expect("Failed to parse status with integer booleans");
        assert_eq!(status.light_on, Some(false));
        assert_eq!(status.lock_flag, Some(true));
        assert_eq!(status.theft_flag, Some(false));
        assert_eq!(status.battery_soc, Some(72.0));
    }

    #[test]
    fn test_parse_status_json_with_missing_fields() {
        let json = serde_json::json!({
            "battery_SOC": 42.0,
            "speed": 15.5
        });

        let status: BikeStatus = serde_json::from_value(json).expect("Failed to parse status");
        assert_eq!(status.battery_soc, Some(42.0));
        assert_eq!(status.speed, Some(15.5));
        assert_eq!(status.assistance_level, None);
        assert_eq!(status.light_on, None);
        assert_eq!(status.suiversion, None);
    }

    #[test]
    fn test_parse_status_json_null_booleans() {
        // Booleans missing entirely (absent field) should deserialize as None.
        let json = serde_json::json!({
            "battery_SOC": 80.0,
        });

        let status: BikeStatus =
            serde_json::from_value(json).expect("Failed to parse status with absent booleans");
        assert_eq!(status.light_on, None);
        assert_eq!(status.lock_flag, None);
        assert_eq!(status.theft_flag, None);
    }

    #[test]
    fn test_parse_position_json() {
        let json = serde_json::json!({
            "latitude": 47.3769,
            "longitude": 8.5417,
            "rcvts": 1700000000_i64
        });

        let position: BikePosition =
            serde_json::from_value(json).expect("Failed to parse position");
        assert_eq!(position.latitude, Some(47.3769));
        assert_eq!(position.longitude, Some(8.5417));
        assert_eq!(position.rcvts, Some(1700000000));
    }

    #[test]
    fn test_parse_position_json_with_missing_fields() {
        let json = serde_json::json!({});

        let position: BikePosition =
            serde_json::from_value(json).expect("Failed to parse position");
        assert_eq!(position.latitude, None);
        assert_eq!(position.longitude, None);
        assert_eq!(position.rcvts, None);
    }
}
