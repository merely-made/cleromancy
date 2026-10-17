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

#[test]
fn original_native_natal_receipt_still_replays_byte_identically() {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../receipts/c8_natal_hardened_20261017/exact-readings-05.json"
    ))
    .unwrap();
    let records: Vec<NatalReading> = serde_json::from_value(raw.clone()).unwrap();
    for r in &records {
        assert!(r.houses.is_none());
        r.validate().unwrap();
    }
    assert_eq!(serde_json::to_value(records).unwrap(), raw);
}
#[test]
fn whole_sign_and_equal_are_explicit_replayable_and_tamper_checked() {
    use cleromancy::natal_houses::*;
    let mut f = birth().facts;
    f.place = Some(BirthPlace {
        label: "Public New York coordinate fixture".into(),
        latitude_microdegrees: Some(40_712_800),
        longitude_microdegrees: Some(-74_006_000),
        precision: "Public synthetic".into(),
    });
    let base = read(
        personal_facts::birth_resource(&f).unwrap(),
        FoldChoice::Refuse,
    )
    .unwrap();
    let whole = base.clone().with_houses(HouseSystem::WholeSign).unwrap();
    let equal = base.with_houses(HouseSystem::Equal).unwrap();
    assert_ne!(whole.id, equal.id);
    for r in [&whole, &equal] {
        r.validate().unwrap();
        let saved = serde_json::to_vec(r).unwrap();
        let again: NatalReading = serde_json::from_slice(&saved).unwrap();
        again.validate().unwrap();
        assert_eq!(serde_json::to_vec(&again).unwrap(), saved);
    }
    let get = |r: &NatalReading| match &r.houses.as_ref().unwrap().outcome {
        HouseOutcome::Calculated {
            ascendant_microdegrees,
            cusps_microdegrees,
            placements,
            ..
        } => (
            *ascendant_microdegrees,
            *cusps_microdegrees,
            placements.clone(),
        ),
        _ => panic!(),
    };
    let (wa, wc, wp) = get(&whole);
    let (ea, ec, ep) = get(&equal);
    assert_eq!(wa, ea);
    assert_ne!(wc, ec);
    assert_eq!(wc[0] % 30_000_000, 0);
    assert_eq!(ec[0], ea);
    assert_eq!(wp.len(), 10);
    assert_eq!(ep.len(), 10);
    for change in ["cusp", "system", "rotation", "placement"] {
        let mut bad = serde_json::to_value(&whole).unwrap();
        match change {
            "cusp" => bad["houses"]["outcome"]["cusps_microdegrees"][0] = serde_json::json!(123),
            "system" => bad["houses"]["system"] = serde_json::json!("equal"),
            "rotation" => bad["houses"]["ut1_minus_utc_milliseconds"] = serde_json::json!(500),
            _ => bad["houses"]["outcome"]["placements"][0]["house"] = serde_json::json!(13),
        }
        let mut content = bad.clone();
        content.as_object_mut().unwrap().remove("id");
        bad["id"] = serde_json::json!(
            blake3::hash(&serde_json::to_vec(&content).unwrap())
                .to_hex()
                .to_string()
        );
        let bad: NatalReading = serde_json::from_value(bad).unwrap();
        assert!(bad.validate().is_err(), "{change}");
    }
}
#[test]
fn missing_place_unknown_time_and_degenerate_pole_never_invent_houses() {
    use cleromancy::natal_houses::*;
    let r = read(birth(), FoldChoice::Refuse)
        .unwrap()
        .with_houses(HouseSystem::WholeSign)
        .unwrap();
    assert!(matches!(
        r.houses.unwrap().outcome,
        HouseOutcome::Unavailable { .. }
    ));
    for unknown in [true, false] {
        let mut f = birth().facts;
        f.place = Some(BirthPlace {
            label: "Public pole fixture".into(),
            latitude_microdegrees: Some(90_000_000),
            longitude_microdegrees: Some(0),
            precision: "Synthetic".into(),
        });
        if unknown {
            f.time = BirthTime::Unknown;
        }
        let r = read(
            personal_facts::birth_resource(&f).unwrap(),
            FoldChoice::Refuse,
        )
        .unwrap()
        .with_houses(HouseSystem::Equal)
        .unwrap();
        r.validate().unwrap();
        assert!(matches!(
            r.houses.unwrap().outcome,
            HouseOutcome::Unavailable { .. }
        ));
    }
}
