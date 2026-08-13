// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "analytic-ephemeris")]

//! Measures the data-free analytic engine against the same NASA/JPL Horizons
//! vectors the DE440s golden test uses. Unlike that test this one needs no
//! kernel, so it runs in the ordinary suite.

use cleromancy::{AnalyticEphemerisAdapter, AstrologyMoment, calculate_with_adapter};

/// The worst residual the analytic engine is allowed against Horizons. It is
/// a measured ceiling, not an accuracy claim: through Turquet, every body
/// lands on the Horizons value exactly or within 2 millidegrees. Turquet
/// carries the same vectors in its own suite; this copy proves the adapter
/// preserves them through the chart contract.
const MAX_ERROR_MILLIDEGREES: i32 = 5;

const J2000: &[(&str, i32, i32)] = &[
    ("sun", 280_369, 0),
    ("moon", 223_324, 5_171),
    ("mercury", 271_889, -995),
    ("venus", 241_566, 2_066),
    ("mars", 327_963, -1_068),
    ("jupiter", 25_253, -1_262),
    ("saturn", 40_396, -2_445),
    ("uranus", 314_809, -658),
    ("neptune", 303_193, 235),
    ("pluto", 251_455, 10_855),
];

const ECLIPSE_2024: &[(&str, i32, i32)] = &[
    ("sun", 19_386, 0),
    ("moon", 19_183, 329),
    ("mercury", 24_807, 2_836),
    ("venus", 4_427, -1_497),
    ("mars", 343_040, -1_245),
    ("jupiter", 49_043, -802),
    ("saturn", 344_454, -1_684),
    ("uranus", 51_170, -271),
    ("neptune", 358_190, -1_222),
    ("pluto", 301_967, -2_964),
];

const CURRENT_2026: &[(&str, i32, i32)] = &[
    ("sun", 140_769, 0),
    ("moon", 151_005, -112),
    ("mercury", 126_483, 741),
    ("venus", 186_638, -1_216),
    ("mars", 91_419, 265),
    ("jupiter", 129_719, 496),
    ("saturn", 14_486, -2_581),
    ("uranus", 65_361, -155),
    ("neptune", 4_065, -1_402),
    ("pluto", 303_889, -4_291),
];

#[test]
fn analytic_residuals_against_nasa_horizons() {
    let adapter = AnalyticEphemerisAdapter::new();
    let charts = [
        (AstrologyMoment::global("2000-01-01T12:00:00Z"), J2000),
        (
            AstrologyMoment::at("2024-04-08T18:00:00Z", 32_776_700, -96_797_000),
            ECLIPSE_2024,
        ),
        (AstrologyMoment::global("2026-08-13T12:00:00Z"), CURRENT_2026),
    ];

    let mut worst_longitude = 0;
    let mut worst_latitude = 0;
    let mut worst_body = "none";
    let mut failures = Vec::new();

    for (moment, golden) in &charts {
        let chart = calculate_with_adapter(&adapter, moment).expect("calculate analytic chart");
        println!("\n{}", moment.instant_utc);
        println!(
            "{:<9} {:>12} {:>12} {:>8} {:>8}",
            "body", "horizons", "analytic", "d-lon", "d-lat"
        );
        for &(body, expected_longitude, expected_latitude) in *golden {
            let actual = chart
                .position(body)
                .unwrap_or_else(|| panic!("{body} is missing from the analytic chart"));
            let longitude_error =
                circular_error(actual.longitude_millidegrees as i32, expected_longitude);
            let latitude_error = actual.latitude_millidegrees - expected_latitude;
            println!(
                "{body:<9} {expected_longitude:>12} {:>12} {longitude_error:>8} {latitude_error:>8}",
                actual.longitude_millidegrees,
            );
            if longitude_error.abs() > worst_longitude {
                worst_longitude = longitude_error.abs();
                worst_body = body;
            }
            worst_latitude = worst_latitude.max(latitude_error.abs());
            if longitude_error.abs() > MAX_ERROR_MILLIDEGREES
                || latitude_error.abs() > MAX_ERROR_MILLIDEGREES
            {
                failures.push(format!(
                    "{body} at {}: d-lon {longitude_error}, d-lat {latitude_error}",
                    moment.instant_utc
                ));
            }
        }
    }

    println!(
        "\nworst longitude residual {worst_longitude} millidegrees ({worst_body}); \
         worst latitude residual {worst_latitude} millidegrees"
    );
    assert!(
        failures.is_empty(),
        "residuals exceeded {MAX_ERROR_MILLIDEGREES} millidegrees:\n{}",
        failures.join("\n")
    );
}

/// Both engines cover the same ten bodies, so a chart from either reads the
/// same way downstream.
#[test]
fn the_analytic_chart_carries_the_same_ten_bodies_as_the_kernel_engine() {
    let adapter = AnalyticEphemerisAdapter::new();
    let chart = calculate_with_adapter(&adapter, &AstrologyMoment::global("2026-08-13T12:00:00Z"))
        .expect("calculate analytic chart");
    for body in [
        "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
        "pluto",
    ] {
        assert!(
            chart.position(body).is_some(),
            "{body} must be present in an analytic chart"
        );
    }
}

fn circular_error(actual: i32, expected: i32) -> i32 {
    (actual - expected + 180_000).rem_euclid(360_000) - 180_000
}
