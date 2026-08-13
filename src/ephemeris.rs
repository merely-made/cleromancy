// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The ephemeris adapter behind the existing `AstrologyAdapter` seam.
//!
//! [`analytic`] adapts the rev-pinned Turquet engine, which reaches neither
//! the filesystem nor the network. Cleromancy owns the chart contract, the
//! integer normalization, and the derived facts; Turquet owns the celestial
//! math and its own verification against JPL.
//!
//! The DE440s kernel lane that used to live here moved to Turquet's opt-in
//! `verify` feature, where it generates golden vectors instead of shipping in
//! a product. No consumer downloads a kernel.

mod analytic;

pub use analytic::{
    ANALYTIC_EPHEMERIS_ALGORITHM, AnalyticEphemerisAdapter, AnalyticEphemerisError,
    TURQUET_REVISION,
};
