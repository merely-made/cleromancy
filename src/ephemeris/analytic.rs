// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data-free analytic ephemeris, backed by Turquet.
//!
//! Turquet's `apparent` module owns the celestial math: VSOP87D, the partial
//! ELP-2000/82 Moon, the analytical Pluto series, nutation, light-time, and
//! aberration, measured against NASA/JPL Horizons to within 2 millidegrees on
//! all ten bodies. This adapter owns only the chart contract: UTC parsing,
//! integer millidegree rounding, retrograde flags, and the source-qualified
//! engine strings. It reaches neither the filesystem nor the network.

use turquet::apparent::{self, APPARENT_BODIES, ApparentError};

use crate::astrology::{
    AstrologyAdapter, AstrologyChart, AstrologyError, AstrologyMoment, AstrologyPosition,
};

pub const ANALYTIC_EPHEMERIS_ALGORITHM: &str =
    "cleromancy.ephemeris/analytic-vsop87d-elp2000-apparent-iau1980/v1";
pub const TURQUET_REVISION: &str = "d29145181191b3f545cceda0b50bdc523c58a1da";
const ENGINE: &str =
    "cleromancy-analytic-ephemeris/v2; merely-made/turquet@d29145181191b3f545cceda0b50bdc523c58a1da (turquet/0.1.0)";
const EPHEMERIS: &str = "Turquet apparent: VSOP87D + partial ELP-2000/82 + analytical Pluto (no data file); observer:earth-geocenter; bodies:ten";

#[derive(Debug, thiserror::Error)]
pub enum AnalyticEphemerisError {
    #[error("UTC instant must be an ISO-8601 UTC value: {0}")]
    InvalidInstant(String),
    #[error("instant precedes 1972 and has no defined leap-second offset: {0}")]
    BeforeLeapSecondTable(String),
    #[error("instant is outside the {body} series validity (Julian year {julian_year:.1})")]
    OutsideSeriesRange { body: &'static str, julian_year: f64 },
    #[error(transparent)]
    Chart(#[from] AstrologyError),
}

/// A calculator that needs no kernel, no download, and no filesystem.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnalyticEphemerisAdapter;

impl AnalyticEphemerisAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl AstrologyAdapter for AnalyticEphemerisAdapter {
    type Error = AnalyticEphemerisError;

    fn calculate(&self, moment: &AstrologyMoment) -> Result<AstrologyChart, Self::Error> {
        let (year, month, day, hour, minute, second) = parse_utc(&moment.instant_utc)?;
        let jde_tt = apparent::jde_tt_frm_utc(year, month, day, hour, minute, second)
            .map_err(|error| lift(error, &moment.instant_utc))?;
        let mut positions = Vec::with_capacity(APPARENT_BODIES.len());
        for body in APPARENT_BODIES.iter() {
            let (longitude, latitude) = apparent::geocent_apparent_ecl_pos(body, jde_tt)
                .map_err(|error| lift(error, &moment.instant_utc))?;
            let retrograde = apparent::is_retrograde(body, jde_tt)
                .map_err(|error| lift(error, &moment.instant_utc))?;
            positions.push(
                AstrologyPosition::new(
                    body.name(),
                    (longitude.to_degrees() * 1_000.0).round() as u32,
                    (latitude.to_degrees() * 1_000.0).round() as i32,
                )
                .with_retrograde(retrograde),
            );
        }
        AstrologyChart::new(
            ANALYTIC_EPHEMERIS_ALGORITHM,
            ENGINE,
            EPHEMERIS,
            moment.clone(),
            positions,
        )
        .map_err(Into::into)
    }
}

fn lift(error: ApparentError, instant: &str) -> AnalyticEphemerisError {
    match error {
        ApparentError::BeforeLeapSecondEra => {
            AnalyticEphemerisError::BeforeLeapSecondTable(instant.trim().to_string())
        }
        ApparentError::InvalidCivilTime => {
            AnalyticEphemerisError::InvalidInstant(instant.trim().to_string())
        }
        ApparentError::OutsideSeriesRange { body, julian_year } => {
            AnalyticEphemerisError::OutsideSeriesRange { body, julian_year }
        }
    }
}

type UtcParts = (i32, u32, u32, u32, u32, f64);

/// Parse an ISO-8601 UTC instant. The designator is required: a bare local
/// timestamp is refused rather than assumed to be UTC. Field range checks
/// beyond simple digits belong to Turquet's civil-time validation.
fn parse_utc(instant_utc: &str) -> Result<UtcParts, AnalyticEphemerisError> {
    let text = instant_utc.trim();
    let invalid = || AnalyticEphemerisError::InvalidInstant(text.to_string());
    let body = text
        .strip_suffix('Z')
        .or_else(|| text.strip_suffix(" UTC"))
        .ok_or_else(invalid)?;
    let (date, time) = body
        .split_once('T')
        .or_else(|| body.split_once(' '))
        .ok_or_else(invalid)?;

    let mut date_parts = date.split('-');
    let year = next_field::<i32>(&mut date_parts).ok_or_else(invalid)?;
    let month = next_field::<u32>(&mut date_parts).ok_or_else(invalid)?;
    let day = next_field::<u32>(&mut date_parts).ok_or_else(invalid)?;
    if date_parts.next().is_some() {
        return Err(invalid());
    }

    let mut time_parts = time.split(':');
    let hour = next_field::<u32>(&mut time_parts).ok_or_else(invalid)?;
    let minute = next_field::<u32>(&mut time_parts).ok_or_else(invalid)?;
    let second = next_field::<f64>(&mut time_parts).ok_or_else(invalid)?;
    if time_parts.next().is_some() {
        return Err(invalid());
    }
    Ok((year, month, day, hour, minute, second))
}

fn next_field<T: std::str::FromStr>(parts: &mut std::str::Split<'_, char>) -> Option<T> {
    parts.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instant_without_a_utc_designator_is_refused() {
        assert!(parse_utc("2026-08-13T12:00:00").is_err());
        assert!(parse_utc("2026-08-13T12:00:00Z").is_ok());
        assert!(parse_utc("2026-08-13 12:00:00 UTC").is_ok());
    }

    #[test]
    fn invalid_civil_fields_are_refused_before_calculation() {
        let adapter = AnalyticEphemerisAdapter::new();
        let moment = AstrologyMoment::global("2026-13-13T12:00:00Z");
        assert!(matches!(
            adapter.calculate(&moment),
            Err(AnalyticEphemerisError::InvalidInstant(_))
        ));
    }

    #[test]
    fn pre_1972_instants_are_refused() {
        let adapter = AnalyticEphemerisAdapter::new();
        let moment = AstrologyMoment::global("1969-07-20T20:17:00Z");
        assert!(matches!(
            adapter.calculate(&moment),
            Err(AnalyticEphemerisError::BeforeLeapSecondTable(_))
        ));
    }
}
