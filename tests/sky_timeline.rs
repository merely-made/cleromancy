// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "sky-timeline")]

use cleromancy::CleromancyHost;
use cleromancy::sky::turquet::analytical_sky_day;
use cleromancy::sky::{
    SkyDayFacts, SkyEarthOrientationApproximation, SkyEarthOrientationPolicy, SkyFactKind,
    SkyNumericalPolicy, SkyRule, SkyRulePack, SkySearchControls, SkyTwilightMeasurement,
    SkyTwilightPolicy, UtcCivilDay, Wgs84Observer, interpret_sky_day,
};
use mere::kernel::graph::{ProvenanceSubKind, RelationKind};
use muniment::MemoryBackend;

fn dallas() -> Wgs84Observer {
    Wgs84Observer::new(32_776_700, -96_797_000, 130_000).unwrap()
}

fn policy() -> SkyNumericalPolicy {
    SkyNumericalPolicy {
        phase_search: SkySearchControls::new(3_600, 100).unwrap(),
        twilight: SkyTwilightPolicy {
            measurement: SkyTwilightMeasurement::AirlessSolarCenter,
            altitude_millidegrees: -6_000,
            search: SkySearchControls::new(900, 100).unwrap(),
        },
        earth_orientation: SkyEarthOrientationPolicy {
            authority: "constant UT1-minus-UTC approximation".to_string(),
            snapshot: "Dallas 2024-04-08: -12 ms, zero polar motion".to_string(),
            approximation: SkyEarthOrientationApproximation::ConstantUt1MinusUtcZeroPolarMotion {
                ut1_minus_utc_milliseconds: -12,
            },
        },
    }
}

fn new_moon_pack(version: &str, prompt: &str) -> SkyRulePack {
    SkyRulePack::new(
        "example.sky-day",
        "Mark AB",
        version,
        [SkyRule {
            id: "new-moon".to_string(),
            fact_kind: SkyFactKind::NewMoon,
            prompt: prompt.to_string(),
        }],
    )
    .unwrap()
}

fn fact(facts: &SkyDayFacts, kind: SkyFactKind) -> &cleromancy::sky::SkyFact {
    facts
        .facts
        .iter()
        .find(|fact| fact.kind == kind)
        .unwrap_or_else(|| panic!("expected {kind:?} in the Dallas timeline"))
}

#[test]
fn dallas_eclipse_day_is_a_durable_turquet_timeline_with_authored_interpretations() {
    let day = UtcCivilDay::new(2024, 4, 8).unwrap();
    let policy = policy();
    let facts = analytical_sky_day(day, dallas(), policy.clone()).unwrap();
    let repeated = analytical_sky_day(day, dallas(), policy.clone()).unwrap();

    assert_eq!(facts, repeated);
    assert_eq!(facts.digest(), repeated.digest());
    assert_eq!(facts.policy, policy);
    for kind in [SkyFactKind::NewMoon, SkyFactKind::Dawn, SkyFactKind::Dusk] {
        let fact = fact(&facts, kind);
        assert!(fact.interval.start_julian_day < fact.interval.end_julian_day);
        assert!(!fact.provenance.model.is_empty());
        assert!(!fact.provenance.provider.is_empty());
        assert!(!fact.provenance.transform.is_empty());
        assert!(!fact.provenance.earth_orientation.is_empty());
    }

    let first_pack = new_moon_pack("1.0.0", "Name what is newly available.");
    let revised_pack = new_moon_pack("1.1.0", "Treat the threshold as a question.");
    let first = interpret_sky_day(&facts, &first_pack).unwrap();
    let revised = interpret_sky_day(&facts, &revised_pack).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(revised.len(), 1);
    assert_eq!(first[0].sky_facts_digest, facts.digest());
    assert_eq!(revised[0].sky_facts_digest, facts.digest());
    assert_ne!(first[0], revised[0]);
    first[0].verify_with_pack(&facts, &first_pack).unwrap();
    revised[0].verify_with_pack(&facts, &revised_pack).unwrap();

    let backend = MemoryBackend::new();
    let mut host = CleromancyHost::empty(backend.clone());
    pollster::block_on(host.persist(0)).unwrap();
    host.insert_sky_day_facts(&facts).unwrap();
    pollster::block_on(host.persist(1)).unwrap();

    let mut reopened = pollster::block_on(CleromancyHost::open(backend.clone())).unwrap();
    assert_eq!(
        reopened.sky_day_facts_for_digest(&facts.digest()).unwrap(),
        facts
    );
    reopened.insert_sky_interpretation(&first[0]).unwrap();
    assert_eq!(
        reopened
            .graph()
            .relations()
            .filter(|relation| {
                relation.kind == RelationKind::Provenance(ProvenanceSubKind::GeneratedFrom)
            })
            .count(),
        1
    );
    pollster::block_on(reopened.persist(2)).unwrap();

    let reopened_again = pollster::block_on(CleromancyHost::open(backend)).unwrap();
    assert_eq!(
        reopened_again
            .graph()
            .relations()
            .filter(|relation| {
                relation.kind == RelationKind::Provenance(ProvenanceSubKind::GeneratedFrom)
            })
            .count(),
        1
    );
    assert_eq!(
        reopened_again
            .sky_interpretation_for_digest(&first[0].digest())
            .unwrap(),
        first[0]
    );
}
