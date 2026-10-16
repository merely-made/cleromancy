// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit network opt-in, using the shared bounded fetch handle.
use crate::situation::{
    SituationError, SituationPreferences, WEATHER_BODY_LIMIT, WeatherObservation, weather_url,
};

pub(super) fn collect(
    preferences: &SituationPreferences,
) -> Result<Option<WeatherObservation>, SituationError> {
    preferences.validate()?;
    if !preferences.weather {
        return Ok(None);
    }
    let place = preferences.place.clone().expect("validated weather place");
    // The resident command is executing inside its runtime. NetFetch's
    // blocking handle belongs on a plain worker thread, never that runtime.
    std::thread::spawn(move || {
        use fetch::Fetch;
        let stores=fetch::Stores::in_memory();
        let fetch=fetch::NetFetch::new(&stores).map_err(|e|SituationError(e.to_string()))?
            .with_patience(std::time::Duration::from_secs(10));
        let url=weather_url(&place)?;
        let body=fetch.read_all(&url,Some("application/json"),WEATHER_BODY_LIMIT as u64)
            .map_err(|e|SituationError(format!("Weather was requested but could not be captured: {e}. No reading was saved; turn weather off to read offline.")))?;
        let raw=String::from_utf8(body.bytes).map_err(|e|SituationError(e.to_string()))?;
        let retrieved=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|SituationError(e.to_string()))?.as_millis() as u64;
        WeatherObservation::from_response(&place,retrieved,body.facts.final_url,raw).map(Some)
    }).join().map_err(|_|SituationError("weather worker stopped before returning its observation".into()))?
}
