// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data-free analytic ephemeris.
//!
//! VSOP87D supplies heliocentric coordinates already referred to the mean
//! ecliptic and equinox of date, so this adapter needs no precession step and
//! no kernel file. It applies light-time, annual aberration from a numerically
//! differentiated Earth velocity, and the same SOFARS IAU 1980 nutation the
//! JPL adapter uses, so the two engines differ only in their source of
//! planetary positions.
//!
//! It covers the Sun, the eight planets, and Pluto, which carries its own
//! truncated series because it is outside VSOP87. The Moon needs a separate
//! lunar theory and is deliberately absent rather than approximated; see the
//! analytic-parity design doc.

use vsop87::{SphericalCoordinates, vsop87d};

use crate::astrology::{
    AstrologyAdapter, AstrologyChart, AstrologyError, AstrologyMoment, AstrologyPosition,
};

pub const ANALYTIC_EPHEMERIS_ALGORITHM: &str =
    "cleromancy.ephemeris/analytic-vsop87d-apparent-iau1980/v1";
const ENGINE: &str = "cleromancy-analytic-ephemeris/v1; vsop87/3.0.0; sofars/0.6.1";
const EPHEMERIS: &str =
    "VSOP87D (no data file); observer:earth-geocenter; bodies:sun,mercury..neptune";

/// Astronomical units travelled by light in one day.
const LIGHT_SPEED_AU_PER_DAY: f64 = 173.144_632_674_24;
/// Central-difference step for Earth's velocity, in days.
const VELOCITY_STEP_DAYS: f64 = 0.01;
const HALF_DAY: f64 = 0.5;
const J2000_JD: f64 = 2_451_545.0;
/// TT - TAI, the fixed offset in seconds.
const TT_MINUS_TAI_SECONDS: f64 = 32.184;

/// TAI - UTC in seconds, keyed by the Gregorian date the value took effect.
/// Charts before 1972 fall back to the earliest entry; the adapter never
/// silently guesses a future leap second.
const LEAP_SECONDS: [(i32, u32, u32, f64); 28] = [
    (1972, 1, 1, 10.0),
    (1972, 7, 1, 11.0),
    (1973, 1, 1, 12.0),
    (1974, 1, 1, 13.0),
    (1975, 1, 1, 14.0),
    (1976, 1, 1, 15.0),
    (1977, 1, 1, 16.0),
    (1978, 1, 1, 17.0),
    (1979, 1, 1, 18.0),
    (1980, 1, 1, 19.0),
    (1981, 7, 1, 20.0),
    (1982, 7, 1, 21.0),
    (1983, 7, 1, 22.0),
    (1985, 7, 1, 23.0),
    (1988, 1, 1, 24.0),
    (1990, 1, 1, 25.0),
    (1991, 1, 1, 26.0),
    (1992, 7, 1, 27.0),
    (1993, 7, 1, 28.0),
    (1994, 7, 1, 29.0),
    (1996, 1, 1, 30.0),
    (1997, 7, 1, 31.0),
    (1999, 1, 1, 32.0),
    (2006, 1, 1, 33.0),
    (2009, 1, 1, 34.0),
    (2012, 7, 1, 35.0),
    (2015, 7, 1, 36.0),
    (2017, 1, 1, 37.0),
];

/// The bodies VSOP87D can place. Ordering matches the JPL adapter so the two
/// charts read the same way where they overlap.
const BODIES: [Body; 9] = [
    Body::Sun,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Sun,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
}

impl Body {
    fn name(self) -> &'static str {
        match self {
            Self::Sun => "sun",
            Self::Mercury => "mercury",
            Self::Venus => "venus",
            Self::Mars => "mars",
            Self::Jupiter => "jupiter",
            Self::Saturn => "saturn",
            Self::Uranus => "uranus",
            Self::Neptune => "neptune",
            Self::Pluto => "pluto",
        }
    }

    /// Heliocentric rectangular coordinates on the mean ecliptic of date.
    /// The Sun sits at the origin of that frame by construction.
    fn heliocentric(self, jde: f64) -> [f64; 3] {
        match self {
            Self::Sun => [0.0, 0.0, 0.0],
            Self::Mercury => rectangular(&vsop87d::mercury(jde)),
            Self::Venus => rectangular(&vsop87d::venus(jde)),
            Self::Mars => rectangular(&vsop87d::mars(jde)),
            Self::Jupiter => rectangular(&vsop87d::jupiter(jde)),
            Self::Saturn => rectangular(&vsop87d::saturn(jde)),
            Self::Uranus => rectangular(&vsop87d::uranus(jde)),
            Self::Neptune => rectangular(&vsop87d::neptune(jde)),
            Self::Pluto => pluto_heliocentric(jde),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyticEphemerisError {
    #[error("UTC instant must be an ISO-8601 UTC value: {0}")]
    InvalidInstant(String),
    #[error("instant precedes 1972 and has no defined leap-second offset: {0}")]
    BeforeLeapSecondTable(String),
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

    /// Apparent geocentric ecliptic longitude and latitude, in degrees.
    fn apparent(body: Body, jde_tt: f64) -> (f64, f64) {
        let earth = vsop87d::earth(jde_tt);
        let earth = rectangular(&earth);

        // Light-time: the body is seen where it was when the light left it.
        let mut offset = [0.0_f64; 3];
        let mut light_time_days = 0.0;
        for _ in 0..3 {
            let target = body.heliocentric(jde_tt - light_time_days);
            offset = [
                target[0] - earth[0],
                target[1] - earth[1],
                target[2] - earth[2],
            ];
            light_time_days = norm(offset) / LIGHT_SPEED_AU_PER_DAY;
        }

        // Annual aberration displaces the apparent direction toward the
        // observer's motion. Earth's velocity comes from a central difference
        // rather than a memorised orbit constant.
        let velocity = earth_velocity(jde_tt);
        let direction = normalize(offset);
        let aberrated = normalize([
            direction[0] + velocity[0] / LIGHT_SPEED_AU_PER_DAY,
            direction[1] + velocity[1] / LIGHT_SPEED_AU_PER_DAY,
            direction[2] + velocity[2] / LIGHT_SPEED_AU_PER_DAY,
        ]);

        // Mean equinox of date to true equinox of date.
        let (nutation_in_longitude, _) = sofars::pnp::nut80(J2000_JD, jde_tt - J2000_JD);
        let longitude = aberrated[1].atan2(aberrated[0]) + nutation_in_longitude;
        let latitude = aberrated[2].asin();
        (
            longitude.to_degrees().rem_euclid(360.0),
            latitude.to_degrees(),
        )
    }

    fn is_retrograde(body: Body, jde_tt: f64) -> bool {
        let before = Self::apparent(body, jde_tt - HALF_DAY).0;
        let after = Self::apparent(body, jde_tt + HALF_DAY).0;
        let delta = (after - before + 180.0).rem_euclid(360.0) - 180.0;
        delta < 0.0
    }
}

impl AstrologyAdapter for AnalyticEphemerisAdapter {
    type Error = AnalyticEphemerisError;

    fn calculate(&self, moment: &AstrologyMoment) -> Result<AstrologyChart, Self::Error> {
        let jde_tt = terrestrial_julian_day(&moment.instant_utc)?;
        let mut positions = Vec::with_capacity(BODIES.len());
        for body in BODIES {
            let (longitude, latitude) = Self::apparent(body, jde_tt);
            positions.push(
                AstrologyPosition::new(
                    body.name(),
                    (longitude * 1_000.0).round() as u32,
                    (latitude * 1_000.0).round() as i32,
                )
                .with_retrograde(Self::is_retrograde(body, jde_tt)),
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

fn rectangular(coordinates: &SphericalCoordinates) -> [f64; 3] {
    let (longitude, latitude, radius) = (
        coordinates.longitude(),
        coordinates.latitude(),
        coordinates.distance(),
    );
    [
        radius * latitude.cos() * longitude.cos(),
        radius * latitude.cos() * longitude.sin(),
        radius * latitude.sin(),
    ]
}

fn earth_velocity(jde: f64) -> [f64; 3] {
    let before = rectangular(&vsop87d::earth(jde - VELOCITY_STEP_DAYS));
    let after = rectangular(&vsop87d::earth(jde + VELOCITY_STEP_DAYS));
    let scale = 2.0 * VELOCITY_STEP_DAYS;
    [
        (after[0] - before[0]) / scale,
        (after[1] - before[1]) / scale,
        (after[2] - before[2]) / scale,
    ]
}

fn norm(vector: [f64; 3]) -> f64 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let length = norm(vector);
    if length == 0.0 {
        return vector;
    }
    [vector[0] / length, vector[1] / length, vector[2] / length]
}

/// Julian day in Terrestrial Time for an ISO-8601 UTC instant.
fn terrestrial_julian_day(instant_utc: &str) -> Result<f64, AnalyticEphemerisError> {
    let (year, month, day, hour, minute, second) = parse_utc(instant_utc)?;
    let day_fraction = f64::from(day) + (f64::from(hour) + (f64::from(minute) + second / 60.0) / 60.0) / 24.0;
    let julian_day_utc = gregorian_julian_day(year, month, day_fraction);
    let leap = leap_seconds(year, month, day).ok_or_else(|| {
        AnalyticEphemerisError::BeforeLeapSecondTable(instant_utc.trim().to_string())
    })?;
    Ok(julian_day_utc + (TT_MINUS_TAI_SECONDS + leap) / 86_400.0)
}

type UtcParts = (i32, u32, u32, u32, u32, f64);

/// Parse an ISO-8601 UTC instant. The designator is required: a bare local
/// timestamp is refused rather than assumed to be UTC.
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

    let in_range = (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour <= 23
        && minute <= 59
        && (0.0..61.0).contains(&second);
    if !in_range {
        return Err(invalid());
    }
    Ok((year, month, day, hour, minute, second))
}

fn next_field<T: std::str::FromStr>(parts: &mut std::str::Split<'_, char>) -> Option<T> {
    parts.next()?.parse().ok()
}

fn gregorian_julian_day(year: i32, month: u32, day_fraction: f64) -> f64 {
    let (year, month) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let century = (f64::from(year) / 100.0).floor();
    let gregorian = 2.0 - century + (century / 4.0).floor();
    (365.25 * (f64::from(year) + 4_716.0)).floor()
        + (30.6001 * (f64::from(month) + 1.0)).floor()
        + day_fraction
        + gregorian
        - 1_524.5
}

fn leap_seconds(year: i32, month: u32, day: u32) -> Option<f64> {
    let mut current = None;
    for (leap_year, leap_month, leap_day, seconds) in LEAP_SECONDS {
        if (year, month, day) >= (leap_year, leap_month, leap_day) {
            current = Some(seconds);
        }
    }
    current
}

/// Meeus's truncated Pluto series, valid 1885 to 2099. Pluto is outside
/// VSOP87, so it carries its own heliocentric solution.
fn pluto_heliocentric(jde: f64) -> [f64; 3] {
    let centuries = (jde - J2000_JD) / 36_525.0;
    let jupiter_mean = (34.35 + 3_034.9057 * centuries).to_radians();
    let saturn_mean = (50.08 + 1_222.1138 * centuries).to_radians();
    let pluto_mean = (238.96 + 144.9600 * centuries).to_radians();

    let mut longitude = 238.956_785 + 144.96 * centuries;
    let mut latitude = -3.908_202;
    let mut radius = 40.7247248;

    for term in PLUTO_TERMS {
        let angle = f64::from(term.jupiter) * jupiter_mean
            + f64::from(term.saturn) * saturn_mean
            + f64::from(term.pluto) * pluto_mean;
        let (sine, cosine) = angle.sin_cos();
        longitude += (term.longitude_sine * sine + term.longitude_cosine * cosine) * 1e-6;
        latitude += (term.latitude_sine * sine + term.latitude_cosine * cosine) * 1e-6;
        radius += (term.radius_sine * sine + term.radius_cosine * cosine) * 1e-7;
    }

    let longitude = longitude.to_radians();
    let latitude = latitude.to_radians();
    let j2000 = [
        radius * latitude.cos() * longitude.cos(),
        radius * latitude.cos() * longitude.sin(),
        radius * latitude.sin(),
    ];
    precess_ecliptic_from_j2000(j2000, jde)
}

/// Rotate a J2000 ecliptic vector onto the mean ecliptic of date.
///
/// The Pluto series is referred to the J2000 equinox while VSOP87D is referred
/// to the equinox of date; mixing them unrotated leaves a pure precession
/// error that reaches a third of a degree within this century. The rotation
/// routes through the equatorial frame so it uses SOFARS matrices rather than
/// a second set of transcribed ecliptic-precession terms.
fn precess_ecliptic_from_j2000(vector: [f64; 3], jde: f64) -> [f64; 3] {
    let (sine_j2000, cosine_j2000) = sofars::pnp::obl80(J2000_JD, 0.0).sin_cos();
    let equatorial_j2000 = [
        vector[0],
        vector[1] * cosine_j2000 - vector[2] * sine_j2000,
        vector[1] * sine_j2000 + vector[2] * cosine_j2000,
    ];
    let equatorial_date = matrix_vector(
        sofars::pnp::pmat76(J2000_JD, jde - J2000_JD),
        equatorial_j2000,
    );
    let (sine_date, cosine_date) = sofars::pnp::obl80(J2000_JD, jde - J2000_JD).sin_cos();
    [
        equatorial_date[0],
        equatorial_date[1] * cosine_date + equatorial_date[2] * sine_date,
        -equatorial_date[1] * sine_date + equatorial_date[2] * cosine_date,
    ]
}

fn matrix_vector(matrix: [[f64; 3]; 3], vector: [f64; 3]) -> [f64; 3] {
    [
        matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2],
        matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2],
        matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2],
    ]
}

struct PlutoTerm {
    jupiter: i8,
    saturn: i8,
    pluto: i8,
    longitude_sine: f64,
    longitude_cosine: f64,
    latitude_sine: f64,
    latitude_cosine: f64,
    radius_sine: f64,
    radius_cosine: f64,
}

const fn pluto_term(
    jupiter: i8,
    saturn: i8,
    pluto: i8,
    longitude_sine: f64,
    longitude_cosine: f64,
    latitude_sine: f64,
    latitude_cosine: f64,
    radius_sine: f64,
    radius_cosine: f64,
) -> PlutoTerm {
    PlutoTerm {
        jupiter,
        saturn,
        pluto,
        longitude_sine,
        longitude_cosine,
        latitude_sine,
        latitude_cosine,
        radius_sine,
        radius_cosine,
    }
}

const PLUTO_TERMS: [PlutoTerm; 43] = [
    pluto_term(0, 0, 1, -19_798_886.0, 19_848_454.0, -5_453_098.0, -14_974_876.0, 66_867_334.0, 68_955_876.0),
    pluto_term(0, 0, 2, 897_499.0, -4_955_707.0, 3_527_363.0, 1_672_673.0, -11_826_086.0, -333_765.0),
    pluto_term(0, 0, 3, 610_820.0, 1_210_521.0, -1_050_939.0, 327_763.0, 1_593_657.0, -1_439_953.0),
    pluto_term(0, 0, 4, -341_639.0, -189_719.0, 178_691.0, -291_925.0, -18_948.0, 482_443.0),
    pluto_term(0, 0, 5, 129_027.0, -34_863.0, 18_763.0, 100_448.0, -66_634.0, -85_576.0),
    pluto_term(0, 0, 6, -38_215.0, 31_061.0, -30_594.0, -25_838.0, 30_841.0, -5_765.0),
    pluto_term(0, 1, -1, 20_349.0, -9_886.0, 4_965.0, 11_263.0, -6_140.0, 22_254.0),
    pluto_term(0, 1, 0, -4_045.0, -4_904.0, 310.0, -132.0, 4_434.0, 4_247.0),
    pluto_term(0, 1, 1, -5_885.0, -3_238.0, 2_036.0, -947.0, -1_518.0, 1_432.0),
    pluto_term(0, 1, 2, -3_812.0, 3_011.0, -2.0, -674.0, -5.0, 1_524.0),
    pluto_term(0, 1, 3, -601.0, 3_468.0, -329.0, -563.0, 1_916.0, -3_064.0),
    pluto_term(0, 2, -2, 710.0, -4_036.0, 2_644.0, -253.0, -2_117.0, 2_501.0),
    pluto_term(0, 2, -1, 6_770.0, -5_775.0, 2_881.0, 128.0, -1_779.0, 1_816.0),
    pluto_term(0, 2, 0, 6_138.0, 1_299.0, -1_670.0, -1_089.0, -1_048.0, -2_178.0),
    pluto_term(1, -1, 0, -3_646.0, -3_610.0, 1_365.0, -1_912.0, -692.0, -1_397.0),
    pluto_term(1, -1, 1, -5_119.0, -4_997.0, 2_695.0, -2_557.0, -1_351.0, -1_589.0),
    pluto_term(1, 0, -3, 2_141.0, 3_872.0, -2_338.0, 1_474.0, -411.0, 1_636.0),
    pluto_term(1, 0, -2, 1_575.0, 3_034.0, -1_734.0, 1_299.0, -382.0, 1_277.0),
    pluto_term(1, 0, -1, 786.0, 2_154.0, -1_212.0, 852.0, -285.0, 819.0),
    pluto_term(1, 0, 0, 796.0, 1_386.0, -817.0, 610.0, -180.0, 545.0),
    pluto_term(1, 0, 1, 337.0, 796.0, -449.0, 316.0, -113.0, 313.0),
    pluto_term(1, 0, 2, -25.0, 397.0, -207.0, 132.0, -63.0, 148.0),
    pluto_term(1, 0, 3, 249.0, 137.0, -75.0, 118.0, -19.0, 46.0),
    pluto_term(1, 0, 4, 262.0, -23.0, 6.0, 108.0, 12.0, -10.0),
    pluto_term(1, 1, -3, 47.0, 260.0, -137.0, 87.0, -40.0, 100.0),
    pluto_term(1, 1, -2, 27.0, 172.0, -90.0, 55.0, -26.0, 66.0),
    pluto_term(1, 1, -1, 13.0, 113.0, -58.0, 36.0, -17.0, 43.0),
    pluto_term(1, 1, 0, 8.0, 74.0, -38.0, 23.0, -11.0, 28.0),
    pluto_term(1, 1, 1, 6.0, 50.0, -26.0, 15.0, -7.0, 19.0),
    pluto_term(1, 1, 2, 4.0, 34.0, -17.0, 10.0, -5.0, 13.0),
    pluto_term(1, 1, 3, 3.0, 24.0, -12.0, 7.0, -3.0, 9.0),
    pluto_term(2, 0, -6, -18.0, -2.0, 0.0, -8.0, 0.0, 1.0),
    pluto_term(2, 0, -5, -14.0, 2.0, 0.0, -6.0, 1.0, 0.0),
    pluto_term(2, 0, -4, -11.0, 5.0, -1.0, -5.0, 1.0, 0.0),
    pluto_term(2, 0, -3, -8.0, 9.0, -2.0, -4.0, 1.0, -1.0),
    pluto_term(2, 0, -2, -5.0, 13.0, -4.0, -3.0, 2.0, -2.0),
    pluto_term(2, 0, -1, -1.0, 16.0, -5.0, -2.0, 2.0, -3.0),
    pluto_term(2, 0, 0, 2.0, 17.0, -6.0, -1.0, 2.0, -3.0),
    pluto_term(2, 0, 1, 5.0, 16.0, -6.0, 0.0, 2.0, -3.0),
    pluto_term(2, 0, 2, 7.0, 13.0, -6.0, 1.0, 1.0, -3.0),
    pluto_term(2, 0, 3, 9.0, 9.0, -5.0, 1.0, 1.0, -2.0),
    pluto_term(2, 0, 4, 10.0, 5.0, -4.0, 2.0, 0.0, -1.0),
    pluto_term(2, 0, 5, 11.0, 1.0, -3.0, 2.0, 0.0, -1.0),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_noon_utc_is_the_standard_epoch_in_terrestrial_time() {
        let jde = terrestrial_julian_day("2000-01-01T12:00:00Z").expect("parse J2000");
        // TT ran 64.184 s ahead of UTC in 2000.
        let expected = J2000_JD + 64.184 / 86_400.0;
        assert!((jde - expected).abs() < 1e-9, "got {jde}, expected {expected}");
    }

    #[test]
    fn leap_seconds_follow_the_published_table() {
        assert_eq!(leap_seconds(2000, 1, 1), Some(32.0));
        assert_eq!(leap_seconds(2024, 4, 8), Some(37.0));
        assert_eq!(leap_seconds(1971, 12, 31), None);
    }

    #[test]
    fn an_instant_without_a_utc_designator_is_refused() {
        assert!(terrestrial_julian_day("2026-08-13T12:00:00").is_err());
        assert!(terrestrial_julian_day("2026-08-13T12:00:00Z").is_ok());
    }

    #[test]
    fn the_sun_sits_opposite_the_earth() {
        let jde = terrestrial_julian_day("2026-08-13T12:00:00Z").expect("parse instant");
        let (longitude, latitude) = AnalyticEphemerisAdapter::apparent(Body::Sun, jde);
        let earth = vsop87d::earth(jde);
        let opposite = (earth.longitude().to_degrees() + 180.0).rem_euclid(360.0);
        let error = (longitude - opposite + 180.0).rem_euclid(360.0) - 180.0;
        // Light-time, aberration, and nutation account for the small residual.
        assert!(error.abs() < 0.01, "sun {longitude} vs earth-opposite {opposite}");
        assert!(latitude.abs() < 0.01);
    }
}
