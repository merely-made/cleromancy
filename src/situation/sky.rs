// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::{ReadingPlace, SituationError, UnavailableFact, error, require};
use crate::{AstrologyPlacement, AstrologyPosition};
use serde::{Deserialize, Serialize};
use turquet_timeline::{
    apparent::{APPARENT_BODIES, ApparentBody, ApparentSky},
    events::{
        ConventionalRiseSetKind, ConventionalRiseSetPolicy, ConventionalRiseSetSearch,
        HorizonDipModel, LimbModel, RefractionModel, SearchWindow, conventional_rise_set_events,
        lunar_illumination_at,
    },
    foundation::{
        EastLongitude, JulianDate, Latitude, Length, Observer, ScaleAwareEpoch, TerrestrialTime,
        TimeOffset,
    },
    observer::EarthOrientation,
    provider::{ANALYTICAL_EPHEMERIS, ConstantOffsetEarthOrientation},
};

pub const SKY_METHOD: &str = "cleromancy.situation/turquet-0.17-30875f3-normalized/v1";
pub const LOCAL_METHOD: &str =
    "cleromancy.situation/usno-solar-unequal-hours-mean-solar-weekday/v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedSky {
    pub method: String,
    pub provider: String,
    pub phase_band: String,
    pub lunar_elongation_millidegrees: u32,
    pub illuminated_parts_per_million: u32,
    pub positions: Vec<AstrologyPlacement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarEventKind {
    Rise,
    Set,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolarEvent {
    pub kind: SolarEventKind,
    pub instant_ms: u64,
    pub earliest_ms: u64,
    pub latest_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetaryHour {
    pub ruler: String,
    pub ordinal: u8,
    pub daylight: bool,
    pub starts_ms: u64,
    pub ends_ms: u64,
    pub convention: String,
    pub tradition_source: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalSky {
    pub method: String,
    pub astronomical_season: Option<String>,
    pub solar_events: Vec<SolarEvent>,
    pub planetary_hour: Option<PlanetaryHour>,
    pub circumstances: String,
}

pub fn utc_label(ms: u64) -> String {
    let (y, m, d, h, min, s, _) =
        ScaleAwareEpoch::from_unix_milliseconds(ms as f64).to_gregorian_utc();
    format!("{y:04}-{m:02}-{d:02} {h:02}:{min:02}:{s:02} UTC")
}
type Calculation = (RecordedSky, Option<LocalSky>, Vec<UnavailableFact>);
pub(super) fn calculate(
    ms: u64,
    place: Option<&ReadingPlace>,
) -> Result<Calculation, SituationError> {
    // Resident validation and detail decoding can inspect the same immutable
    // draw repeatedly. Cache only results computed here, never supplied
    // record data or a successful validation decision. Every caller still
    // compares all recorded fields against the computed result. This cache
    // is bounded, process-local, and absent on a fresh process reopen.
    type Entry = (u64, Option<ReadingPlace>, Calculation);
    static CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::VecDeque<Entry>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Ok(held) = cache.lock() {
        if let Some((_, _, value)) = held
            .iter()
            .find(|(at, p, _)| *at == ms && p.as_ref() == place)
        {
            return Ok(value.clone());
        }
    }
    let result = calculate_uncached(ms, place)?;
    if let Ok(mut held) = cache.lock() {
        if held.len() == 64 {
            held.pop_front();
        }
        held.push_back((ms, place.cloned(), result.clone()));
    }
    Ok(result)
}
fn calculate_uncached(
    ms: u64,
    place: Option<&ReadingPlace>,
) -> Result<Calculation, SituationError> {
    let utc = ScaleAwareEpoch::from_unix_milliseconds(ms as f64);
    let year = utc.to_gregorian_utc().0;
    require(
        (1972..=2098).contains(&year),
        "draw time is outside this pinned sky provider's supported 1972–2098 range",
    )?;
    let tt = JulianDate::<TerrestrialTime>::from_epoch(utc);
    let provider = ApparentSky::at(tt);
    let mut positions = Vec::new();
    let mut sun = 0;
    let mut moon = 0;
    for body in APPARENT_BODIES {
        let state = provider.position(body).map_err(error)?.into_value();
        let longitude = ((state.direction().longitude().degrees().rem_euclid(360.0) * 1000.0)
            .round() as u32)
            % 360_000;
        let latitude = (state.direction().latitude().degrees() * 1000.0).round() as i32;
        if body == ApparentBody::Sun {
            sun = longitude;
        }
        if body == ApparentBody::Moon {
            moon = longitude;
        }
        let position = AstrologyPosition::new(body.name(), longitude, latitude);
        positions.push(AstrologyPlacement::from_position(&position));
    }
    let elongation = (moon + 360_000 - sun) % 360_000;
    let bands = [
        "new-moon sector",
        "waxing crescent",
        "first-quarter sector",
        "waxing gibbous",
        "full-moon sector",
        "waning gibbous",
        "last-quarter sector",
        "waning crescent",
    ];
    let illumination = lunar_illumination_at(&ANALYTICAL_EPHEMERIS, tt).map_err(error)?;
    let sky=RecordedSky{method:SKY_METHOD.into(),provider:"Turquet 0.17.0@30875f30; analytical apparent ecliptic model3; geocentric tropical signs; eight equal phase sectors centered on 0/45/90…315 degrees".into(),
        phase_band:bands[(((elongation+22_500)%360_000)/45_000) as usize].into(),lunar_elongation_millidegrees:elongation,
        illuminated_parts_per_million:(illumination.illuminated_fraction()*1_000_000.0).round() as u32,positions};
    let mut missing = Vec::new();
    let Some(place) = place else {
        for name in [
            "Place",
            "Astronomical season",
            "Sunrise and sunset",
            "Planetary hours",
        ] {
            missing.push(UnavailableFact{fact:name.into(),reason:"Place was not supplied for this draw. Geocentric phase and sign positions do not need it.".into()});
        }
        return Ok((sky, None, missing));
    };
    place.validate()?;
    let season = if place.latitude_microdegrees == 0 {
        None
    } else {
        let mut quarter = (sun / 90_000) as usize;
        if place.latitude_microdegrees < 0 {
            quarter = (quarter + 2) % 4;
        }
        Some(["Spring", "Summer", "Autumn", "Winter"][quarter].into())
    };
    if season.is_none() {
        missing.push(UnavailableFact {
            fact: "Astronomical season".into(),
            reason: "No northern/southern four-season convention is assigned at the equator."
                .into(),
        });
    }
    let observer = Observer::new(
        EastLongitude::from_degrees(place.longitude_microdegrees as f64 / 1e6).map_err(error)?,
        Latitude::from_degrees(place.latitude_microdegrees as f64 / 1e6).map_err(error)?,
        Length::from_meters(0.0).map_err(error)?,
    );
    let orientation = EarthOrientation::zero_polar_motion(
        JulianDate::from_utc_epoch(utc, TimeOffset::from_seconds(0.0).map_err(error)?),
        "Cleromancy C6 approximation",
        "UT1-minus-UTC=0; zero polar motion; constant UT1-minus-TT over search",
    );
    let earth = ConstantOffsetEarthOrientation::new(tt, orientation);
    let window = SearchWindow::new(
        tt.offset_days(-2.0).map_err(error)?,
        tt.offset_days(2.0).map_err(error)?,
        1800.0 / 86400.0,
        1.0 / 86400.0,
    )
    .map_err(error)?;
    let policy = ConventionalRiseSetPolicy::new(
        RefractionModel::usno_standard(),
        LimbModel::usno_standard_solar(),
        HorizonDipModel::level(),
    );
    let events = conventional_rise_set_events(
        &ANALYTICAL_EPHEMERIS,
        &earth,
        observer,
        ApparentBody::Sun,
        ConventionalRiseSetSearch::new(window).map_err(error)?,
        policy,
    )
    .map_err(error)?;
    fn timestamp(tt: JulianDate<TerrestrialTime>) -> u64 {
        (tt.to_epoch().to_unix_seconds() * 1000.0).round() as u64
    }
    let solar_events: Vec<_> = events
        .into_iter()
        .map(|e| SolarEvent {
            kind: match e.kind() {
                ConventionalRiseSetKind::Rise => SolarEventKind::Rise,
                ConventionalRiseSetKind::Set => SolarEventKind::Set,
            },
            instant_ms: timestamp(e.interval().midpoint()),
            earliest_ms: timestamp(e.interval().start()),
            latest_ms: timestamp(e.interval().end()),
        })
        .collect();
    let hour = planetary_hour(ms, place, &solar_events);
    if solar_events.is_empty() {
        missing.push(UnavailableFact{fact:"Sunrise and sunset".into(),reason:"No sampled crossing in the bounded four-day window; this can occur in polar day or night.".into()});
    }
    if hour.is_none() {
        missing.push(UnavailableFact {
            fact: "Planetary hours".into(),
            reason:
                "No bracketing sunrise/sunset pair in the bounded window; no hour was invented."
                    .into(),
        });
    }
    let local=LocalSky{method:LOCAL_METHOD.into(),astronomical_season:season,solar_events,planetary_hour:hour,
        circumstances:"Turquet conventional solar rise/set: USNO 34 arcminute constant refraction, 16 arcminute upper solar limb; sea level and level unobstructed horizon; UT1-minus-UTC=0 and zero polar motion approximation. Search ±2 days, 30 minute step, 1 second brackets. Seasons use the apparent Sun's tropical quarter and hemisphere; no meteorological-season claim.".into()};
    Ok((sky, Some(local), missing))
}

fn planetary_hour(ms: u64, place: &ReadingPlace, events: &[SolarEvent]) -> Option<PlanetaryHour> {
    let index = events
        .windows(2)
        .position(|pair| pair[0].instant_ms <= ms && ms < pair[1].instant_ms)?;
    let first = &events[index];
    let last = &events[index + 1];
    if first.kind == last.kind {
        return None;
    }
    let daylight = first.kind == SolarEventKind::Rise;
    let sunrise = if daylight {
        first
    } else {
        events[..index]
            .iter()
            .rev()
            .find(|e| e.kind == SolarEventKind::Rise)?
    };
    let duration = last.instant_ms.checked_sub(first.instant_ms)?;
    let offset = ((ms - first.instant_ms) as u128 * 12 / duration as u128) as u8;
    let ordinal = offset + 1 + if daylight { 0 } else { 12 };
    let mean_solar_ms =
        sunrise.instant_ms as i128 + place.longitude_microdegrees as i128 * 240_000 / 1_000_000;
    let weekday = (mean_solar_ms.div_euclid(86_400_000) + 4).rem_euclid(7) as usize;
    let chaldean = [
        "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
    ];
    let day_ruler = [3, 6, 2, 5, 1, 4, 0][weekday];
    Some(PlanetaryHour{ruler:chaldean[(day_ruler+ordinal as usize-1)%7].into(),ordinal,daylight,
        starts_ms:first.instant_ms+(duration as u128*offset as u128).div_ceil(12) as u64,
        ends_ms:first.instant_ms+(duration as u128*(offset as u128+1)).div_ceil(12) as u64,
        convention:"Chaldean cycle; divide sunrise–sunset and sunset–next sunrise into twelve unequal hours each; weekday from sunrise's local mean solar date".into(),
        tradition_source:"Astrologers' temporal-hour division reported in Agrippa, De occulta philosophia II.34 (1533), John French translation (1651), p.289; https://www.esotericarchives.com/agrippa/agripp2c.htm#chap34 ; local-mean-solar weekday and solar event policy are this implementation's boundary conventions".into()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unequal_hours_cover_each_boundary_and_keep_the_previous_day_before_sunrise() {
        // 2026-10-16 is Friday. Give the daylight span an indivisible
        // millisecond remainder to exercise all rounded boundary instants.
        let rise = ScaleAwareEpoch::from_gregorian_utc(2026, 10, 16, 6, 0, 0, 0).to_unix_seconds()
            as u64
            * 1000;
        let event = |kind, instant_ms| SolarEvent {
            kind,
            instant_ms,
            earliest_ms: instant_ms,
            latest_ms: instant_ms,
        };
        let events = vec![
            event(SolarEventKind::Rise, rise - 86_400_000),
            event(SolarEventKind::Set, rise - 43_200_000),
            event(SolarEventKind::Rise, rise),
            event(SolarEventKind::Set, rise + 46_800_007),
            event(SolarEventKind::Rise, rise + 86_400_000),
        ];
        let place = ReadingPlace::new(51_476_900, 0).unwrap();
        assert_eq!(
            planetary_hour(rise, &place, &events).unwrap().ruler,
            "Venus"
        );
        let previous = planetary_hour(rise - 1, &place, &events).unwrap();
        assert_eq!(previous.ordinal, 24);
        assert!(!previous.daylight);
        let mut cursor = rise;
        for ordinal in 1..=24 {
            let hour = planetary_hour(cursor, &place, &events).unwrap();
            assert_eq!(hour.ordinal, ordinal);
            assert_eq!(hour.starts_ms, cursor);
            assert_eq!(
                planetary_hour(hour.ends_ms - 1, &place, &events)
                    .unwrap()
                    .ordinal,
                ordinal
            );
            cursor = hour.ends_ms;
        }
        assert_eq!(cursor, rise + 86_400_000);
    }
}
