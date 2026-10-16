// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::{ReadingPlace, SituationError, error, require};
use serde::{Deserialize, Serialize};

pub const WEATHER_BODY_LIMIT: usize = 65_536;
pub const WEATHER_METHOD: &str = "cleromancy.weather/open-meteo-current-metric-unix/v1";
pub fn weather_url(place: &ReadingPlace) -> Result<String, SituationError> {
    place.validate()?;
    Ok(format!(
        "https://api.open-meteo.com/v1/forecast?latitude={:.6}&longitude={:.6}&current=temperature_2m,relative_humidity_2m,precipitation,weather_code,wind_speed_10m&wind_speed_unit=ms&timeformat=unixtime&timezone=GMT&forecast_days=1",
        place.latitude_microdegrees as f64 / 1e6,
        place.longitude_microdegrees as f64 / 1e6
    ))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherValues {
    pub valid_at_ms: u64,
    pub interval_seconds: u32,
    pub temperature_millicelsius: i32,
    pub humidity_permille: u32,
    pub precipitation_micrometers: u32,
    pub wind_millimeters_per_second: u32,
    pub wmo_code: u32,
    pub grid_latitude_microdegrees: i32,
    pub grid_longitude_microdegrees: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherObservation {
    pub method: String,
    pub request_place: ReadingPlace,
    pub request_url: String,
    pub final_url: String,
    pub retrieved_at_ms: u64,
    pub response_body: String,
    pub response_digest: String,
    pub values: WeatherValues,
    pub provider: String,
    pub source: String,
}
impl WeatherObservation {
    pub fn from_response(
        place: &ReadingPlace,
        retrieved_at_ms: u64,
        final_url: String,
        body: String,
    ) -> Result<Self, SituationError> {
        require(
            body.len() <= WEATHER_BODY_LIMIT,
            "weather response exceeds the recorded-body limit",
        )?;
        require(
            final_url == weather_url(place)?,
            "weather response redirected away from the exact requested source",
        )?;
        let values = decode(&body)?;
        require(
            values.valid_at_ms <= retrieved_at_ms.saturating_add(300_000)
                && retrieved_at_ms <= values.valid_at_ms.saturating_add(7_200_000),
            "weather current interval is stale or in the future",
        )?;
        let response_digest = blake3::hash(body.as_bytes()).to_hex().to_string();
        Ok(Self {method:WEATHER_METHOD.into(),request_place:place.clone(),request_url:weather_url(place)?,final_url,
            retrieved_at_ms,response_body:body,response_digest,values,
            provider:"Open-Meteo current weather; Best Match model estimate, not a station measurement".into(),
            source:"https://open-meteo.com/en/docs; CC BY 4.0; Open-Meteo and contributing weather models".into()})
    }
    pub fn verify(&self, place: &ReadingPlace) -> Result<(), SituationError> {
        let expected = Self::from_response(
            place,
            self.retrieved_at_ms,
            self.final_url.clone(),
            self.response_body.clone(),
        )?;
        require(
            self == &expected,
            "weather response, request, normalization or attribution differs from the saved evidence",
        )
    }
}
fn decode(body: &str) -> Result<WeatherValues, SituationError> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(error)?;
    let units = &v["current_units"];
    for (key, unit) in [
        ("time", "unixtime"),
        ("interval", "seconds"),
        ("temperature_2m", "°C"),
        ("relative_humidity_2m", "%"),
        ("precipitation", "mm"),
        ("weather_code", "wmo code"),
        ("wind_speed_10m", "m/s"),
    ] {
        require(
            units[key].as_str() == Some(unit),
            "weather units are missing or not the requested metric/UTC contract",
        )?;
    }
    require(
        v["utc_offset_seconds"].as_i64() == Some(0),
        "weather response is not UTC",
    )?;
    fn number(v: &serde_json::Value, lo: f64, hi: f64) -> Result<f64, SituationError> {
        let n = v
            .as_f64()
            .ok_or_else(|| error("weather value is missing or null"))?;
        require(
            n.is_finite() && n >= lo && n <= hi,
            "weather value is out of its supported range",
        )?;
        Ok(n)
    }
    let c = &v["current"];
    let seconds = c["time"]
        .as_u64()
        .ok_or_else(|| error("weather valid time is missing"))?;
    require(
        seconds <= 4_102_444_800,
        "weather time exceeds the supported sky era",
    )?;
    Ok(WeatherValues {
        valid_at_ms: seconds * 1000,
        interval_seconds: c["interval"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 86400)
            .ok_or_else(|| error("weather interval is invalid"))? as u32,
        temperature_millicelsius: (number(&c["temperature_2m"], -100., 100.)? * 1000.).round()
            as i32,
        humidity_permille: (number(&c["relative_humidity_2m"], 0., 100.)? * 10.).round() as u32,
        precipitation_micrometers: (number(&c["precipitation"], 0., 10000.)? * 1000.).round()
            as u32,
        wind_millimeters_per_second: (number(&c["wind_speed_10m"], 0., 200.)? * 1000.).round()
            as u32,
        wmo_code: c["weather_code"]
            .as_u64()
            .filter(|n| *n <= 99)
            .ok_or_else(|| error("weather code is invalid"))? as u32,
        grid_latitude_microdegrees: (number(&v["latitude"], -90., 90.)? * 1e6).round() as i32,
        grid_longitude_microdegrees: (number(&v["longitude"], -180., 180.)? * 1e6).round() as i32,
    })
}
