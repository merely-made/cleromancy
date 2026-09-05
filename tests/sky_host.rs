use cleromancy::sky::{
    SKY_DAY_FACTS_SCHEMA, SKY_INTERPRETATION_SCHEMA, SkyDayFacts, SkyEarthOrientationApproximation,
    SkyEarthOrientationPolicy, SkyFact, SkyFactKind, SkyInterpretation, SkyNumericalPolicy,
    SkyProvenance, SkySearchControls, SkyTtInterval, SkyTwilightMeasurement, SkyTwilightPolicy,
    UtcCivilDay, Wgs84Observer,
};
use cleromancy::{CleromancyHost, HostError};
use mere::kernel::graph::{ProvenanceSubKind, RelationKind};
use muniment::MemoryBackend;

fn facts() -> SkyDayFacts {
    SkyDayFacts::new(
        UtcCivilDay::new(2026, 8, 26).unwrap(),
        Wgs84Observer::new(40_712_800, -74_006_000, 0).unwrap(),
        SkyNumericalPolicy {
            phase_search: SkySearchControls::new(300, 100).unwrap(),
            twilight: SkyTwilightPolicy {
                measurement: SkyTwilightMeasurement::AirlessSolarCenter,
                altitude_millidegrees: -6_000,
                search: SkySearchControls::new(300, 100).unwrap(),
            },
            earth_orientation: SkyEarthOrientationPolicy {
                authority: "fixture".to_string(),
                snapshot: "fixture-1".to_string(),
                approximation:
                    SkyEarthOrientationApproximation::ConstantUt1MinusUtcZeroPolarMotion {
                        ut1_minus_utc_milliseconds: 0,
                    },
            },
        },
        [SkyFact {
            kind: SkyFactKind::Dawn,
            interval: SkyTtInterval::new(2_460_000.1, 2_460_000.10001).unwrap(),
            provenance: SkyProvenance {
                model: "fixture".to_string(),
                provider: "fixture".to_string(),
                provider_snapshot: None,
                transform: "fixture".to_string(),
                earth_orientation: "fixture".to_string(),
            },
        }],
    )
    .unwrap()
}

fn interpretation(facts: &SkyDayFacts) -> SkyInterpretation {
    SkyInterpretation {
        schema: SKY_INTERPRETATION_SCHEMA.to_string(),
        sky_facts_digest: facts.digest(),
        fact_kind: SkyFactKind::Dawn,
        fact_interval: facts.facts[0].interval,
        pack_id: "fixture-pack".to_string(),
        pack_author: "fixture".to_string(),
        pack_version: "1".to_string(),
        pack_digest: "0".repeat(64),
        rule_id: "dawn".to_string(),
        prompt: "A clear beginning.".to_string(),
    }
}

#[test]
fn sky_records_persist_as_distinct_digest_nodes_with_provenance() {
    let backend = MemoryBackend::new();
    let mut host = CleromancyHost::empty(backend.clone());
    let facts = facts();
    let interpretation = interpretation(&facts);
    host.insert_sky_day_facts(&facts).unwrap();
    host.insert_sky_interpretation(&interpretation).unwrap();
    assert_eq!(host.sky_day_facts().unwrap(), vec![facts.clone()]);
    assert_eq!(SKY_DAY_FACTS_SCHEMA, "cleromancy.sky-day-facts/v1");
    assert!(host.sky_day_facts_for_digest(&facts.digest()).is_ok());
    assert_eq!(
        host.sky_interpretation_for_digest(&interpretation.digest())
            .unwrap(),
        interpretation
    );
    assert_eq!(
        host.graph()
            .relations()
            .filter(|relation| {
                relation.kind == RelationKind::Provenance(ProvenanceSubKind::GeneratedFrom)
            })
            .count(),
        1
    );
    pollster::block_on(host.persist(1)).unwrap();
    let reopened = pollster::block_on(CleromancyHost::open(backend)).unwrap();
    assert_eq!(
        reopened.sky_day_facts_for_digest(&facts.digest()).unwrap(),
        facts
    );
}

#[test]
fn sky_interpretation_replay_requires_its_facts_dependency() {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let facts = facts();
    let interpretation = interpretation(&facts);
    assert!(matches!(
        host.insert_sky_interpretation(&interpretation),
        Err(HostError::MissingSkyDependency {
            kind: "sky day facts",
            ..
        })
    ));
}
