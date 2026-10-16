// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Recorded draw-time facts. Replaying this module never performs network I/O.
mod sky;
mod weather;
pub use sky::{LocalSky, PlanetaryHour, RecordedSky, SolarEvent, SolarEventKind, utc_label};
pub use weather::{WEATHER_BODY_LIMIT, WeatherObservation, WeatherValues, weather_url};

use crate::{ReadingSession, context::canonical_digest};
use serde::{Deserialize, Serialize};

pub const SITUATION_SCHEMA: &str = "cleromancy.situation/v1";
pub const SITUATION_FACET: &str = SITUATION_SCHEMA;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("reading situation: {0}")]
pub struct SituationError(pub String);
pub(super) fn error(e: impl std::fmt::Debug) -> SituationError {
    SituationError(format!("{e:?}"))
}
pub(super) fn require(ok: bool, why: &str) -> Result<(), SituationError> {
    if ok {
        Ok(())
    } else {
        Err(SituationError(why.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingPlace {
    pub latitude_microdegrees: i32,
    pub longitude_microdegrees: i32,
}
impl ReadingPlace {
    pub fn new(
        latitude_microdegrees: i32,
        longitude_microdegrees: i32,
    ) -> Result<Self, SituationError> {
        let value = Self {
            latitude_microdegrees,
            longitude_microdegrees,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), SituationError> {
        require(
            (-90_000_000..=90_000_000).contains(&self.latitude_microdegrees)
                && (-180_000_000..=180_000_000).contains(&self.longitude_microdegrees),
            "place coordinates are outside their degree ranges",
        )
    }
    pub fn from_degrees(latitude: &str, longitude: &str) -> Result<Self, SituationError> {
        fn parse(value: &str) -> Result<i32, SituationError> {
            let v = value.trim().parse::<f64>().map_err(error)?;
            require(
                v.is_finite() && v.abs() <= 180.0,
                "enter finite decimal degrees",
            )?;
            Ok((v * 1_000_000.0).round() as i32)
        }
        Self::new(parse(latitude)?, parse(longitude)?)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationPreferences {
    pub place: Option<ReadingPlace>,
    pub weather: bool,
}
impl SituationPreferences {
    pub fn validate(&self) -> Result<(), SituationError> {
        if let Some(place) = &self.place {
            place.validate()?;
        }
        require(
            !self.weather || self.place.is_some(),
            "weather needs an explicitly supplied place; place has not been enabled",
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnavailableFact {
    pub fact: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingSituation {
    pub schema: String,
    pub session_id: String,
    pub instant_ms: u64,
    pub place: Option<ReadingPlace>,
    pub sky: RecordedSky,
    pub local: Option<LocalSky>,
    pub weather: Option<WeatherObservation>,
    pub unavailable: Vec<UnavailableFact>,
}
impl ReadingSituation {
    pub fn capture(
        session: &ReadingSession,
        preferences: &SituationPreferences,
        weather: Option<WeatherObservation>,
    ) -> Result<Self, SituationError> {
        session.validate().map_err(error)?;
        preferences.validate()?;
        require(
            preferences.weather == weather.is_some(),
            "requested weather observation is missing or was not opted in",
        )?;
        if let Some(observation) = &weather {
            observation.verify(
                preferences
                    .place
                    .as_ref()
                    .ok_or_else(|| error("weather has no opted-in place"))?,
            )?;
            require(
                observation.retrieved_at_ms <= session.created_at_ms.saturating_add(300_000)
                    && session.created_at_ms <= observation.retrieved_at_ms.saturating_add(300_000),
                "weather retrieval is not from this draw",
            )?;
        }
        let (sky, local, mut unavailable) =
            sky::calculate(session.created_at_ms, preferences.place.as_ref())?;
        if !preferences.weather {
            unavailable.push(UnavailableFact {
                fact: "Weather".into(),
                reason: "Weather is off; no network source was contacted for this draw.".into(),
            });
        }
        Ok(Self {
            schema: SITUATION_SCHEMA.into(),
            session_id: session.id.clone(),
            instant_ms: session.created_at_ms,
            place: preferences.place.clone(),
            sky,
            local,
            weather,
            unavailable,
        })
    }
    pub fn digest(&self) -> String {
        canonical_digest(self)
    }
    pub fn verify(&self, session: &ReadingSession) -> Result<(), SituationError> {
        require(self.schema == SITUATION_SCHEMA, "unsupported schema")?;
        let expected = Self::capture(
            session,
            &SituationPreferences {
                place: self.place.clone(),
                weather: self.weather.is_some(),
            },
            self.weather.clone(),
        )?;
        require(
            self == &expected,
            "stored situation differs from its exact draw-time replay",
        )
    }
}
