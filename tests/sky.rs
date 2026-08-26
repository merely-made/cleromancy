use cleromancy::sky::{
    SkyDayFacts, SkyEarthOrientationApproximation, SkyEarthOrientationPolicy, SkyError, SkyFact,
    SkyFactKind, SkyNumericalPolicy, SkyProvenance, SkyRule, SkyRulePack, SkySearchControls,
    SkyTtInterval, SkyTwilightMeasurement, SkyTwilightPolicy, UtcCivilDay, Wgs84Observer,
    interpret_sky_day,
};

fn policy() -> SkyNumericalPolicy {
    SkyNumericalPolicy {
        phase_search: SkySearchControls::new(3_600, 100).unwrap(),
        twilight: SkyTwilightPolicy {
            measurement: SkyTwilightMeasurement::AirlessSolarCenter,
            altitude_millidegrees: -6_000,
            search: SkySearchControls::new(900, 100).unwrap(),
        },
        earth_orientation: SkyEarthOrientationPolicy {
            authority: "IERS approximation".to_string(),
            snapshot: "fixture-1".to_string(),
            approximation: SkyEarthOrientationApproximation::ConstantUt1MinusUtcZeroPolarMotion {
                ut1_minus_utc_milliseconds: -12,
            },
        },
    }
}

fn fact(kind: SkyFactKind, start: f64) -> SkyFact {
    SkyFact {
        kind,
        interval: SkyTtInterval::new(start, start + 0.000_01).unwrap(),
        provenance: SkyProvenance {
            model: "event-model@1".to_string(),
            provider: "provider@1".to_string(),
            provider_snapshot: Some("snapshot-1".to_string()),
            transform: "transform@1".to_string(),
            earth_orientation: "IERS / fixture-1".to_string(),
        },
    }
}

fn facts() -> SkyDayFacts {
    SkyDayFacts::new(
        UtcCivilDay::new(2024, 4, 8).unwrap(),
        Wgs84Observer::new(32_776_700, -96_797_000, 130_000).unwrap(),
        policy(),
        [
            fact(SkyFactKind::Dusk, 2_460_409.2),
            fact(SkyFactKind::NewMoon, 2_460_409.1),
            fact(SkyFactKind::Dawn, 2_460_408.9),
        ],
    )
    .unwrap()
}

#[test]
fn sky_facts_are_canonical_and_validate_after_json_round_trip() {
    let facts = facts();
    assert_eq!(
        facts.facts.iter().map(|fact| fact.kind).collect::<Vec<_>>(),
        vec![SkyFactKind::Dawn, SkyFactKind::NewMoon, SkyFactKind::Dusk]
    );
    let decoded: SkyDayFacts =
        serde_json::from_slice(&serde_json::to_vec(&facts).unwrap()).unwrap();
    assert_eq!(decoded.digest(), facts.digest());
    decoded.validate().unwrap();
}

#[test]
fn interpretations_bind_facts_and_exact_authored_pack() {
    let facts = facts();
    let facts_digest = facts.digest();
    let first = SkyRulePack::new(
        "sky-day",
        "Mark",
        "1.0.0",
        [SkyRule {
            id: "new-moon".to_string(),
            fact_kind: SkyFactKind::NewMoon,
            prompt: "Notice the new moon.".to_string(),
        }],
    )
    .unwrap();
    let second = SkyRulePack::new(
        "sky-day",
        "Mark",
        "1.0.1",
        [SkyRule {
            id: "new-moon".to_string(),
            fact_kind: SkyFactKind::NewMoon,
            prompt: "Record the new moon.".to_string(),
        }],
    )
    .unwrap();

    let interpretation = interpret_sky_day(&facts, &first).unwrap().pop().unwrap();
    let revised = interpret_sky_day(&facts, &second).unwrap().pop().unwrap();
    assert_eq!(interpretation.sky_facts_digest, facts.digest());
    interpretation.verify_with_pack(&facts, &first).unwrap();
    assert_ne!(first.digest(), second.digest());
    assert_eq!(facts.digest(), facts_digest);
    assert_ne!(interpretation, revised);
    assert_eq!(
        interpretation.verify_with_pack(&facts, &second),
        Err(SkyError::PackMismatch("identity"))
    );
}

#[test]
fn validation_rejects_noncanonical_or_out_of_bounds_policy_controls() {
    let mut unordered = facts();
    unordered.facts.reverse();
    assert_eq!(unordered.validate(), Err(SkyError::NonCanonicalFacts));

    let invalid = SkyTwilightPolicy {
        measurement: SkyTwilightMeasurement::AirlessSolarCenter,
        altitude_millidegrees: -6_000,
        search: SkySearchControls::new(3_601, 100).unwrap(),
    };
    assert_eq!(
        invalid.validate(),
        Err(SkyError::InvalidSearch("twilight step_seconds"))
    );
}
