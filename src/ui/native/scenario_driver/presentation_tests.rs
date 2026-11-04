// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::presentation_timeout;

#[test]
fn resident_scenario_sources_parse_with_the_shared_assertion_grammar() {
    for (name, script) in [
        (
            "C6 global",
            include_str!("../../../../scenarios/c6-global.scn"),
        ),
        (
            "C6 global offline reopen",
            include_str!("../../../../scenarios/c6-global-reopen.scn"),
        ),
        (
            "C6 weather",
            include_str!("../../../../scenarios/c6-weather.scn"),
        ),
        (
            "C6 weather offline reopen",
            include_str!("../../../../scenarios/c6-weather-reopen.scn"),
        ),
        (
            "seed",
            include_str!("../../../../scenarios/c1-resident-seed.scn"),
        ),
        (
            "reopen",
            include_str!("../../../../scenarios/c1-resident-reopen.scn"),
        ),
        (
            "retained",
            include_str!("../../../../scenarios/c1-resident-retained.scn"),
        ),
        (
            "compact controls",
            include_str!("../../../../scenarios/c1-resident-compact-controls.scn"),
        ),
        (
            "reading narrative",
            include_str!("../../../../scenarios/c2-reading-narrative.scn"),
        ),
        (
            "reading reopen",
            include_str!("../../../../scenarios/c2-reading-reopen.scn"),
        ),
    ] {
        taproot::Scenario::parse(script).unwrap_or_else(|error| panic!("{name} scenario: {error}"));
    }
    for script in [
        include_str!("../../../../scenarios/c8-die-os.scn"),
        include_str!("../../../../scenarios/c8-die-seed.scn"),
        include_str!("../../../../scenarios/c8-die-os-reopen.scn"),
        include_str!("../../../../scenarios/c8-die-seed-reopen.scn"),
    ] {
        taproot::Scenario::parse(script).expect("C8 die scenario");
    }
    assert!(taproot::Scenario::parse("assert snap readings = 3").is_err());
}

#[test]
fn scenario_allowance_keeps_the_default_and_refuses_unbounded_waits() {
    for (value, seconds) in [
        (None, 10),
        (Some("invalid"), 10),
        (Some("-1"), 10),
        (Some("0"), 10),
        (Some("30"), 30),
        (Some("60"), 60),
        (Some("18446744073709551615"), 60),
        (Some("18446744073709551616"), 10),
    ] {
        assert_eq!(presentation_timeout(value).as_secs(), seconds);
    }
}
