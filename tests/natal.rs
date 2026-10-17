// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use cleromancy::natal::*;
use dramatis::{birth::*, subject::SubjectRef};
fn birth() -> personal_facts::BirthResource {
    let f:BirthFacts=serde_json::from_value(serde_json::json!({"schema":BIRTH_FACTS_SCHEMA,"subject":SubjectRef::Profile{id:personae::ProfileId("synthetic-subject".into())},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":null,"timezone":"UTC","source":{"label":"Public synthetic birth fixture","reference":null}})).unwrap();
    personal_facts::birth_resource(&f).unwrap()
}
fn read(birth: personal_facts::BirthResource, fold: FoldChoice) -> Result<NatalReading, String> {
    NatalReading::new(
        "acting-persona".into(),
        "Other synthetic person".into(),
        birth,
        "How might I reflect on this occasion?".into(),
        fold,
        1792149302000,
        "00000000000000000000000000000008".into(),
    )
}
#[test]
fn exact_chart_seals_subject_civil_resolution_method_and_authored_prompts() {
    let r = read(birth(), FoldChoice::Refuse).unwrap();
    r.validate().unwrap();
    let NatalOutcome::Calculated {
        resolution,
        chart,
        prompts,
        ..
    } = &r.outcome
    else {
        panic!()
    };
    assert_eq!(resolution.utc, "2000-01-01T12:00:00+00:00");
    assert_eq!(resolution.offset_seconds, 0);
    assert_eq!(chart.positions.len(), 10);
    assert!(!prompts.is_empty());
    assert_eq!(r.birth.facts.subject, birth().facts.subject);
    let serialized = serde_json::to_vec(&r).unwrap();
    let reopened: NatalReading = serde_json::from_slice(&serialized).unwrap();
    assert_eq!(r, reopened);
    reopened.validate().unwrap();
}
#[test]
fn unknown_approximate_and_unsupported_era_never_invent_an_exact_chart() {
    for time in [
        BirthTime::Unknown,
        BirthTime::Approximate {
            hour: 12,
            minute: 0,
            second: 0,
            uncertainty_seconds: 3600,
        },
    ] {
        let mut f = birth().facts;
        f.time = time;
        let r = read(
            personal_facts::birth_resource(&f).unwrap(),
            FoldChoice::Refuse,
        )
        .unwrap();
        assert!(matches!(r.outcome, NatalOutcome::Unavailable { .. }));
        r.validate().unwrap();
    }
    let mut f = birth().facts;
    f.date.year = 1965;
    assert!(
        read(
            personal_facts::birth_resource(&f).unwrap(),
            FoldChoice::Refuse
        )
        .unwrap_err()
        .contains("1972")
    );
}
#[test]
fn gaps_refuse_and_folds_require_explicit_choice() {
    let mut f = birth().facts;
    f.timezone = Some("America/New_York".into());
    f.date = CivilDate {
        year: 2024,
        month: 3,
        day: 10,
    };
    f.time = BirthTime::Known {
        hour: 2,
        minute: 30,
        second: 0,
    };
    let b = personal_facts::birth_resource(&f).unwrap();
    assert!(read(b, FoldChoice::Earlier).unwrap_err().contains("gap"));
    f.date = CivilDate {
        year: 2024,
        month: 11,
        day: 3,
    };
    f.time = BirthTime::Known {
        hour: 1,
        minute: 30,
        second: 0,
    };
    let b = personal_facts::birth_resource(&f).unwrap();
    assert!(
        read(b.clone(), FoldChoice::Refuse)
            .unwrap_err()
            .contains("twice")
    );
    let a = read(b.clone(), FoldChoice::Earlier).unwrap();
    let z = read(b, FoldChoice::Later).unwrap();
    assert_ne!(a.outcome, z.outcome);
    a.validate().unwrap();
    z.validate().unwrap();
}
#[test]
fn corrections_preserve_old_natal_and_forged_birth_or_output_refuses() {
    let b = birth();
    let r = read(b.clone(), FoldChoice::Refuse).unwrap();
    let frozen = serde_json::to_vec(&r).unwrap();
    let mut f = b.facts.clone();
    f.time = BirthTime::Known {
        hour: 13,
        minute: 0,
        second: 0,
    };
    assert_ne!(b, personal_facts::birth_resource(&f).unwrap());
    assert_eq!(serde_json::to_vec(&r).unwrap(), frozen);
    let mut forged = b;
    forged.facts = f;
    assert!(read(forged, FoldChoice::Refuse).is_err());
    let mut changed = r;
    changed.subject_label = "someone else".into();
    assert!(changed.validate().is_err());
}
#[test]
fn candidate_validates_natal_and_reflection_after_offline_graph_hydration() {
    let mut host = cleromancy::CleromancyHost::empty(muniment::MemoryBackend::new());
    let r = read(birth(), FoldChoice::Refuse).unwrap();
    host.insert_natal_reading(&r).unwrap();
    let reflection = cleromancy::Reflection::new(
        r.id.clone(),
        1792149303000,
        "00000000000000000000000000000009",
        "A synthetic reflection",
    )
    .unwrap();
    host.insert_natal_reflection(&reflection).unwrap();
    cleromancy::host::validate_divination_graph(host.graph()).unwrap();
    let reopened = cleromancy::CleromancyHost::candidate(host.graph().clone()).unwrap();
    assert_eq!(reopened.natal_readings().unwrap(), vec![r.clone()]);
    assert_eq!(reopened.natal_reflections(&r.id).unwrap(), vec![reflection]);
}
