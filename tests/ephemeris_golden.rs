// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "ephemeris")]

use std::path::PathBuf;

use cleromancy::{AstrologyMoment, JplEphemerisAdapter, calculate_with_adapter};

const MAX_ERROR_MILLIDEGREES: i32 = 2;

struct GoldenChart {
    moment: AstrologyMoment,
    positions: &'static [(&'static str, i32, i32)],
    chart_digest: &'static str,
}

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

/// Re-run with:
/// `CLEROMANCY_DE440S=/path/to/de440s.bsp cargo test --features ephemeris
/// --test ephemeris_golden -- --ignored --nocapture`
#[test]
#[ignore = "requires the canonical 31 MiB NASA DE440s kernel"]
fn nasa_horizons_observer_longitudes_and_latitudes() {
    let path = std::env::var_os("CLEROMANCY_DE440S")
        .map(PathBuf::from)
        .expect("set CLEROMANCY_DE440S to the canonical NASA DE440s kernel");
    let adapter = JplEphemerisAdapter::open_de440s(path).expect("load canonical DE440s");
    let charts = [
        GoldenChart {
            moment: AstrologyMoment::global("2000-01-01T12:00:00Z"),
            positions: J2000,
            chart_digest: "50c27f953931bb57c4856d498cfc114736e2ba40feacf9e66e62595e3d4ebb96",
        },
        GoldenChart {
            moment: AstrologyMoment::at("2024-04-08T18:00:00Z", 32_776_700, -96_797_000),
            positions: ECLIPSE_2024,
            chart_digest: "e7083f39a82a011813710a2fb4f98eb2bfa8bc257c1892e0db33a2f6f3384741",
        },
        GoldenChart {
            moment: AstrologyMoment::global("2026-08-13T12:00:00Z"),
            positions: CURRENT_2026,
            chart_digest: "693a261b7dca73d16d8dc3cfdfa242ee564d3892acd94a01a4ac2d4103ee30d4",
        },
    ];

    let mut actual_digests = Vec::new();
    let expected_digests = charts
        .iter()
        .map(|chart| chart.chart_digest)
        .collect::<Vec<_>>();
    for golden in charts {
        let chart = calculate_with_adapter(&adapter, &golden.moment).expect("calculate chart");
        actual_digests.push(chart.digest());
        println!("{} {}", chart.moment.instant_utc, chart.digest());
        println!(
            "{}",
            serde_json::to_string_pretty(&chart).expect("serialize chart")
        );
        for &(body, expected_longitude, expected_latitude) in golden.positions {
            let actual = chart.position(body).expect("golden body exists");
            let longitude_error =
                circular_error(actual.longitude_millidegrees as i32, expected_longitude);
            let latitude_error = actual.latitude_millidegrees - expected_latitude;
            assert!(
                longitude_error.abs() <= MAX_ERROR_MILLIDEGREES,
                "{body} longitude at {}: expected {expected_longitude}, got {}, error {longitude_error}",
                golden.moment.instant_utc,
                actual.longitude_millidegrees,
            );
            assert!(
                latitude_error.abs() <= MAX_ERROR_MILLIDEGREES,
                "{body} latitude at {}: expected {expected_latitude}, got {}, error {latitude_error}",
                golden.moment.instant_utc,
                actual.latitude_millidegrees,
            );
        }
    }
    assert_eq!(actual_digests, expected_digests);
}

fn circular_error(actual: i32, expected: i32) -> i32 {
    (actual - expected + 180_000).rem_euclid(360_000) - 180_000
}
