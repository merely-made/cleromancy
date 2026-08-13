// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Source-qualified JPL ephemeris adapter.
//!
//! Cleromancy owns the chart contract and this conversion layer. ANISE reads
//! the public JPL SPK kernel; SOFARS supplies the IAU 1976/1980 matrices.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anise::constants::frames::{
    EARTH_J2000, JUPITER_BARYCENTER_J2000, MARS_BARYCENTER_J2000, MERCURY_J2000, MOON_J2000,
    NEPTUNE_BARYCENTER_J2000, PLUTO_BARYCENTER_J2000, SATURN_BARYCENTER_J2000, SUN_J2000,
    URANUS_BARYCENTER_J2000, VENUS_J2000,
};
use anise::prelude::{Aberration, Almanac, Epoch, Frame, SPK, TimeUnits};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::DE440S_SHA256;
use crate::astrology::{
    AstrologyAdapter, AstrologyChart, AstrologyError, AstrologyMoment, AstrologyPosition,
};

pub const CLEROMANCY_EPHEMERIS_ALGORITHM: &str =
    "cleromancy.ephemeris/geocentric-apparent-iau1976-1980/v1";
pub const ANISE_FORK_REVISION: &str = "71e973a245e6701e14a5d4c88a3c4e7dedbf7702";
const ENGINE: &str = "cleromancy-ephemeris/v1; merely-made/anise@71e973a245e6701e14a5d4c88a3c4e7dedbf7702 (anise/0.10.6); sofars/0.6.1";
const HALF_DAY_SECONDS: f64 = 43_200.0;

const BODIES: [(&str, Frame); 10] = [
    ("sun", SUN_J2000),
    ("moon", MOON_J2000),
    ("mercury", MERCURY_J2000),
    ("venus", VENUS_J2000),
    ("mars", MARS_BARYCENTER_J2000),
    ("jupiter", JUPITER_BARYCENTER_J2000),
    ("saturn", SATURN_BARYCENTER_J2000),
    ("uranus", URANUS_BARYCENTER_J2000),
    ("neptune", NEPTUNE_BARYCENTER_J2000),
    ("pluto", PLUTO_BARYCENTER_J2000),
];

#[derive(Debug, Error)]
pub enum JplEphemerisError {
    #[error("could not read ephemeris kernel {path}: {source}")]
    KernelIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("DE440s kernel digest mismatch: expected {expected}, got {actual}")]
    KernelDigest {
        expected: &'static str,
        actual: String,
    },
    #[error("could not load DE440s kernel: {0}")]
    KernelFormat(String),
    #[error("UTC instant must be an ISO-8601 UTC value: {0}")]
    InvalidInstant(String),
    #[error("could not calculate {body}: {detail}")]
    Calculation { body: &'static str, detail: String },
    #[error(transparent)]
    Chart(#[from] AstrologyError),
}

/// A local DE440s calculator. The kernel remains caller-owned and offline;
/// its digest is retained in every chart receipt.
pub struct JplEphemerisAdapter {
    almanac: Almanac,
    kernel_sha256: String,
}

impl JplEphemerisAdapter {
    /// Load the canonical NASA/NAIF DE440s kernel after verifying its bytes.
    pub fn open_de440s(path: impl AsRef<Path>) -> Result<Self, JplEphemerisError> {
        let path = path.as_ref();
        let kernel_sha256 = sha256_file(path)?;
        if kernel_sha256 != DE440S_SHA256 {
            return Err(JplEphemerisError::KernelDigest {
                expected: DE440S_SHA256,
                actual: kernel_sha256,
            });
        }
        let path_text = path
            .to_str()
            .ok_or_else(|| JplEphemerisError::KernelFormat("kernel path is not UTF-8".into()))?;
        let spk = SPK::load(path_text)
            .map_err(|error| JplEphemerisError::KernelFormat(error.to_string()))?;
        Ok(Self {
            almanac: Almanac::from_spk(spk),
            kernel_sha256,
        })
    }

    pub fn kernel_sha256(&self) -> &str {
        &self.kernel_sha256
    }

    fn epoch(moment: &AstrologyMoment) -> Result<Epoch, JplEphemerisError> {
        let instant = moment.instant_utc.trim();
        let hifitime_value = if let Some(without_z) = instant.strip_suffix('Z') {
            format!("{without_z} UTC")
        } else if instant.ends_with(" UTC") {
            instant.to_string()
        } else {
            return Err(JplEphemerisError::InvalidInstant(instant.to_string()));
        };
        hifitime_value
            .parse::<Epoch>()
            .map_err(|_| JplEphemerisError::InvalidInstant(instant.to_string()))
    }

    fn position(
        &self,
        body: &'static str,
        frame: Frame,
        epoch: Epoch,
    ) -> Result<(f64, f64), JplEphemerisError> {
        let state = self
            .almanac
            .translate(frame, EARTH_J2000, epoch, Aberration::CN_S)
            .map_err(|error| JplEphemerisError::Calculation {
                body,
                detail: error.to_string(),
            })?;
        let jd_tt = epoch.to_jde_tt_days();
        let date1 = 2_451_545.0;
        let date2 = jd_tt - date1;
        let precession = sofars::pnp::pmat76(date1, date2);
        let nutation = sofars::pnp::nutm80(date1, date2);
        let mean_of_date = matrix_vector(precession, state.radius_km.into());
        let radius = matrix_vector(nutation, mean_of_date);
        let obliquity = true_obliquity_iau1980(date1, date2);
        let cos_obliquity = obliquity.cos();
        let sin_obliquity = obliquity.sin();
        let x = radius[0];
        let y = radius[1] * cos_obliquity + radius[2] * sin_obliquity;
        let z = -radius[1] * sin_obliquity + radius[2] * cos_obliquity;
        let longitude = y.atan2(x).to_degrees().rem_euclid(360.0);
        let latitude = z.atan2(x.hypot(y)).to_degrees();
        Ok((longitude, latitude))
    }

    fn is_retrograde(
        &self,
        body: &'static str,
        frame: Frame,
        epoch: Epoch,
    ) -> Result<bool, JplEphemerisError> {
        let before = self
            .position(body, frame, epoch - HALF_DAY_SECONDS.seconds())?
            .0;
        let after = self
            .position(body, frame, epoch + HALF_DAY_SECONDS.seconds())?
            .0;
        let delta = (after - before + 180.0).rem_euclid(360.0) - 180.0;
        Ok(delta < 0.0)
    }
}

impl AstrologyAdapter for JplEphemerisAdapter {
    type Error = JplEphemerisError;

    fn calculate(&self, moment: &AstrologyMoment) -> Result<AstrologyChart, Self::Error> {
        let epoch = Self::epoch(moment)?;
        let mut positions = Vec::with_capacity(BODIES.len());
        for (body, frame) in BODIES {
            let (longitude, latitude) = self.position(body, frame, epoch)?;
            positions.push(
                AstrologyPosition::new(
                    body,
                    (longitude * 1_000.0).round() as u32,
                    (latitude * 1_000.0).round() as i32,
                )
                .with_retrograde(self.is_retrograde(body, frame, epoch)?),
            );
        }
        AstrologyChart::new(
            CLEROMANCY_EPHEMERIS_ALGORITHM,
            ENGINE,
            format!(
                "NASA/JPL DE440s; sha256:{}; observer:earth-geocenter; bodies:planetary-barycenters",
                self.kernel_sha256
            ),
            moment.clone(),
            positions,
        )
        .map_err(Into::into)
    }
}

fn sha256_file(path: &Path) -> Result<String, JplEphemerisError> {
    let file = File::open(path).map_err(|source| JplEphemerisError::KernelIo {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1_024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|source| JplEphemerisError::KernelIo {
                path: path.to_path_buf(),
                source,
            })?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn true_obliquity_iau1980(date1: f64, date2: f64) -> f64 {
    let (_, nutation_in_obliquity) = sofars::pnp::nut80(date1, date2);
    sofars::pnp::obl80(date1, date2) + nutation_in_obliquity
}

fn matrix_vector(matrix: [[f64; 3]; 3], vector: [f64; 3]) -> [f64; 3] {
    [
        matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2],
        matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2],
        matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_requires_an_explicit_utc_designator() {
        assert!(
            JplEphemerisAdapter::epoch(&AstrologyMoment::global("2026-08-13T12:00:00Z")).is_ok()
        );
        assert!(
            JplEphemerisAdapter::epoch(&AstrologyMoment::global("2026-08-13T12:00:00")).is_err()
        );
    }

    #[test]
    fn j2000_true_obliquity_is_bounded() {
        let degrees = true_obliquity_iau1980(2_451_545.0, 0.0).to_degrees();
        assert!((23.43..23.45).contains(&degrees));
    }
}
