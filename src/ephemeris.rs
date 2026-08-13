// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ephemeris adapters behind the existing `AstrologyAdapter` seam.
//!
//! Two engines satisfy one contract, and every chart records which produced
//! it:
//!
//! - [`jpl`] reads the public NASA/JPL DE440s kernel through the pinned ANISE
//!   fork. It needs a verified 31 MiB kernel on disk.
//! - [`analytic`] evaluates VSOP87D, the fork-pinned partial ELP-2000/82
//!   Moon, and a truncated Pluto series directly, so it reaches neither the
//!   filesystem nor the network. Both engines cover the same ten bodies.
//!
//! Cleromancy still owns the chart contract, the integer normalization, and
//! the derived facts. Neither engine chooses a house system or interprets.

#[cfg(feature = "analytic-ephemeris")]
mod analytic;
#[cfg(feature = "ephemeris")]
mod jpl;
#[cfg(feature = "ephemeris")]
mod provision;

#[cfg(feature = "analytic-ephemeris")]
pub use analytic::{
    ANALYTIC_EPHEMERIS_ALGORITHM, ASTRO_RUST_FORK_REVISION, AnalyticEphemerisAdapter,
    AnalyticEphemerisError,
};
#[cfg(feature = "ephemeris")]
pub use jpl::{
    ANISE_FORK_REVISION, CLEROMANCY_EPHEMERIS_ALGORITHM, JplEphemerisAdapter, JplEphemerisError,
};
#[cfg(feature = "ephemeris")]
pub use provision::{
    DE440S_BYTES, DE440S_DOWNLOAD_URL, EphemerisInstall, EphemerisProvisionError,
    EphemerisProvisioner, EphemerisStatus,
};

/// SHA-256 of the canonical NASA/NAIF DE440s kernel. The provisioner checks a
/// download against it and the adapter rechecks the installed bytes.
#[cfg(feature = "ephemeris")]
pub const DE440S_SHA256: &str =
    "c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2";
