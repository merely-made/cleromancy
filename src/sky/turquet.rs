// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Optional Turquet 0.17 adapter for one explicitly configured sky day.
//!
//! Turquet values do not cross this boundary: this adapter immediately copies
//! public event fields into Cleromancy's serializable fact types.

use thiserror::Error;
use turquet_timeline::events::{
    AltitudeCrossingSearch, LunarPhase, SearchWindow, SolarTwilightKind,
    airless_solar_twilight_events, ecliptic_longitude_lunar_phases,
};
use turquet_timeline::foundation::{
    Angle, EastLongitude, JulianDate, Latitude, Length, Observer, ScaleAwareEpoch, TerrestrialTime,
    TimeOffset,
};
use turquet_timeline::observer::EarthOrientation;
use turquet_timeline::provider::{ANALYTICAL_EPHEMERIS, ConstantOffsetEarthOrientation};

use super::{
    SkyDayFacts, SkyEarthOrientationApproximation, SkyError, SkyFact, SkyFactKind,
    SkyNumericalPolicy, SkyProvenance, SkyTtInterval, UtcCivilDay, Wgs84Observer,
};

#[derive(Debug, Error)]
pub enum TurquetSkyError {
    #[error(transparent)]
    Sky(#[from] SkyError),
    #[error("Turquet sky timeline failed: {0}")]
    Turquet(String),
}

/// Calculate a factual UTC-day timeline with Turquet's public analytical,
/// phase, and caller-policy airless-twilight APIs.
pub fn analytical_sky_day(
    civil_day: UtcCivilDay,
    observer: Wgs84Observer,
    policy: SkyNumericalPolicy,
) -> Result<SkyDayFacts, TurquetSkyError> {
    civil_day.validate()?;
    observer.validate()?;
    policy.validate()?;

    let start_utc = utc_midnight(civil_day);
    let end_utc = utc_midnight(next_day(civil_day)?);
    let start = JulianDate::<TerrestrialTime>::from_epoch(start_utc);
    let end = JulianDate::<TerrestrialTime>::from_epoch(end_utc);
    let phase_window = search_window(start, end, policy.phase_search)?;
    let twilight_window = search_window(start, end, policy.twilight.search)?;
    let twilight_search = AltitudeCrossingSearch::new(
        twilight_window,
        Angle::from_degrees(policy.twilight.altitude_millidegrees as f64 / 1_000.0)
            .map_err(value_error)?,
    )
    .map_err(event_error)?;
    let normalized_observer = observer;
    let observer = turquet_observer(normalized_observer)?;
    let earth_orientation = earth_orientation(start_utc, start, &policy)?;

    let mut facts = ecliptic_longitude_lunar_phases(&ANALYTICAL_EPHEMERIS, phase_window)
        .map_err(event_error)?
        .into_iter()
        .filter(|event| event.phase() == LunarPhase::NewMoon)
        .map(|event| SkyFact {
            kind: SkyFactKind::NewMoon,
            interval: interval(event.interval()).expect("Turquet event interval is finite"),
            provenance: SkyProvenance {
                model: "Turquet ecliptic-longitude lunar phase@0.17.0".to_string(),
                provider: model(event.provider_model()),
                provider_snapshot: event.provider_snapshot().map(str::to_owned),
                transform: "not applicable: geocentric ecliptic phase".to_string(),
                earth_orientation: "not applicable: geocentric ecliptic phase".to_string(),
            },
        })
        .collect::<Vec<_>>();

    facts.extend(
        airless_solar_twilight_events(
            &ANALYTICAL_EPHEMERIS,
            &earth_orientation,
            observer,
            twilight_search,
        )
        .map_err(event_error)?
        .into_iter()
        .map(|event| {
            let crossing = event.crossing();
            SkyFact {
                kind: match event.kind() {
                    SolarTwilightKind::Dawn => SkyFactKind::Dawn,
                    SolarTwilightKind::Dusk => SkyFactKind::Dusk,
                },
                interval: interval(crossing.interval()).expect("Turquet event interval is finite"),
                provenance: SkyProvenance {
                    model: model(event.naming_model()),
                    provider: model(crossing.provider_model()),
                    provider_snapshot: crossing.provider_snapshot().map(str::to_owned),
                    transform: model(crossing.transform_model()),
                    earth_orientation: format!(
                        "{} / {}",
                        crossing.earth_orientation_authority(),
                        crossing.earth_orientation_snapshot()
                    ),
                },
            }
        }),
    );

    SkyDayFacts::new(civil_day, normalized_observer, policy, facts).map_err(Into::into)
}

fn search_window(
    start: JulianDate<TerrestrialTime>,
    end: JulianDate<TerrestrialTime>,
    controls: super::SkySearchControls,
) -> Result<SearchWindow, TurquetSkyError> {
    SearchWindow::new(
        start,
        end,
        controls.step_seconds as f64 / 86_400.0,
        controls.tolerance_milliseconds as f64 / 86_400_000.0,
    )
    .map_err(event_error)
}

fn earth_orientation(
    utc: ScaleAwareEpoch,
    tt: JulianDate<TerrestrialTime>,
    policy: &SkyNumericalPolicy,
) -> Result<ConstantOffsetEarthOrientation, TurquetSkyError> {
    let SkyEarthOrientationApproximation::ConstantUt1MinusUtcZeroPolarMotion {
        ut1_minus_utc_milliseconds,
    } = policy.earth_orientation.approximation;
    let offset = TimeOffset::from_seconds(ut1_minus_utc_milliseconds as f64 / 1_000.0)
        .map_err(value_error)?;
    let ut1 = turquet_timeline::foundation::JulianDate::from_utc_epoch(utc, offset);
    let value = EarthOrientation::zero_polar_motion(
        ut1,
        &policy.earth_orientation.authority,
        &policy.earth_orientation.snapshot,
    );
    Ok(ConstantOffsetEarthOrientation::new(tt, value))
}

fn turquet_observer(value: Wgs84Observer) -> Result<Observer, TurquetSkyError> {
    Ok(Observer::new(
        EastLongitude::from_degrees(value.longitude_microdegrees as f64 / 1_000_000.0)
            .map_err(value_error)?,
        Latitude::from_degrees(value.latitude_microdegrees as f64 / 1_000_000.0)
            .map_err(value_error)?,
        Length::from_meters(value.height_millimeters as f64 / 1_000.0).map_err(value_error)?,
    ))
}

fn interval(value: turquet_timeline::events::EventInterval) -> Result<SkyTtInterval, SkyError> {
    SkyTtInterval::new(value.start().day(), value.end().day())
}

fn model(value: turquet_timeline::foundation::Model) -> String {
    format!("{}@{}", value.name(), value.revision())
}

fn utc_midnight(day: UtcCivilDay) -> ScaleAwareEpoch {
    ScaleAwareEpoch::from_gregorian_utc(day.year, day.month, day.day, 0, 0, 0, 0)
}

fn next_day(day: UtcCivilDay) -> Result<UtcCivilDay, SkyError> {
    let days = match day.month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if super::is_leap_year(day.year) => 29,
        2 => 28,
        _ => return Err(SkyError::InvalidCivilDay),
    };
    if day.day < days {
        UtcCivilDay::new(day.year, day.month, day.day + 1)
    } else if day.month < 12 {
        UtcCivilDay::new(day.year, day.month + 1, 1)
    } else {
        UtcCivilDay::new(day.year + 1, 1, 1)
    }
}

fn event_error(error: impl std::fmt::Display) -> TurquetSkyError {
    TurquetSkyError::Turquet(error.to_string())
}

fn value_error(error: impl std::fmt::Debug) -> TurquetSkyError {
    TurquetSkyError::Turquet(format!("{error:?}"))
}
