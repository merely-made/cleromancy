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

#[test]
fn earlier_qualified_house_receipts_replay_without_supplements() {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../receipts/natal_houses_20261017/exact-readings.json"
    ))
    .unwrap();
    let records: Vec<NatalReading> = serde_json::from_value(raw.clone()).unwrap();
    for r in &records {
        assert!(r.supplement.is_none());
        r.validate().unwrap();
    }
    assert_eq!(serde_json::to_value(records).unwrap(), raw);
}
fn placed_cap_reading() -> NatalReading {
    let mut f = birth().facts;
    f.place = Some(BirthPlace {
        label: "Public NYC fixture".into(),
        latitude_microdegrees: Some(40_712_800),
        longitude_microdegrees: Some(-74_006_000),
        precision: "Public synthetic".into(),
    });
    read(
        personal_facts::birth_resource(&f).unwrap(),
        FoldChoice::Refuse,
    )
    .unwrap()
}
#[test]
fn motion_and_placidus_seal_replay_and_refuse_recomputed_identity_tampering() {
    use cleromancy::natal_cap::*;
    let r = placed_cap_reading()
        .with_supplement(Some(HouseChoice::Placidus), None)
        .unwrap();
    let s = r.supplement.as_ref().unwrap();
    assert!(matches!(
        s.houses,
        Some(cleromancy::natal_houses::HouseOutcome::Calculated { .. })
    ));
    let MotionOutcome::Calculated { planets, .. } = &s.motion else {
        panic!()
    };
    assert_eq!(planets.len(), 10);
    assert_eq!(planets[0].classification, MotionClass::Direct);
    r.validate().unwrap();
    let bytes = serde_json::to_vec(&r).unwrap();
    let again: NatalReading = serde_json::from_slice(&bytes).unwrap();
    again.validate().unwrap();
    assert_eq!(serde_json::to_vec(&again).unwrap(), bytes);
    for field in ["motion", "cusp", "supplier", "system"] {
        let mut bad = serde_json::to_value(&r).unwrap();
        match field {
            "motion" => {
                bad["supplement"]["motion"]["planets"][0]["microdegrees_per_tt_day"] =
                    serde_json::json!(-50)
            },
            "cusp" => bad["supplement"]["houses"]["cusps_microdegrees"][2] = serde_json::json!(1),
            "supplier" => bad["supplement"]["supplier"] = serde_json::json!("other"),
            _ => bad["supplement"]["house_system"] = serde_json::json!("equal"),
        }
        let mut content = bad.clone();
        content.as_object_mut().unwrap().remove("id");
        bad["id"] = serde_json::json!(
            blake3::hash(&serde_json::to_vec(&content).unwrap())
                .to_hex()
                .to_string()
        );
        assert!(
            serde_json::from_value::<NatalReading>(bad)
                .unwrap()
                .validate()
                .is_err(),
            "{field}"
        );
    }
}
#[test]
fn polar_placidus_is_unavailable_and_missing_place_still_has_motion() {
    use cleromancy::natal_cap::*;
    let mut f = placed_cap_reading().birth.facts;
    f.place.as_mut().unwrap().latitude_microdegrees = Some(70_000_000);
    let r = read(
        personal_facts::birth_resource(&f).unwrap(),
        FoldChoice::Refuse,
    )
    .unwrap()
    .with_supplement(Some(HouseChoice::Placidus), None)
    .unwrap();
    assert!(matches!(
        r.supplement.unwrap().houses,
        Some(cleromancy::natal_houses::HouseOutcome::Unavailable { .. })
    ));
    let r = read(birth(), FoldChoice::Refuse)
        .unwrap()
        .with_supplement(Some(HouseChoice::Placidus), None)
        .unwrap();
    assert!(matches!(
        r.supplement.as_ref().unwrap().motion,
        MotionOutcome::Calculated { .. }
    ));
    assert!(matches!(
        r.supplement.unwrap().houses,
        Some(cleromancy::natal_houses::HouseOutcome::Unavailable { .. })
    ));
}
#[test]
fn retained_final_observations_replay_and_outside_coverage_refuses() {
    use cleromancy::natal_cap::*;
    let text = include_str!("fixtures/natal-observed-finals2000a.txt");
    let r = placed_cap_reading()
        .with_supplement(
            Some(HouseChoice::Placidus),
            Some(OrientationSnapshot {
                finals2000a: text.into(),
            }),
        )
        .unwrap();
    r.validate().unwrap();
    let s = r.supplement.as_ref().unwrap();
    let o = s.observed_values.as_ref().unwrap();
    assert!((o.dut1_microseconds - 355066).abs() <= 1);
    assert!((o.xp_microarcseconds - 43335).abs() <= 1);
    assert!((o.yp_microarcseconds - 377605).abs() <= 1);
    let a = serde_json::to_vec(&r).unwrap();
    let again: NatalReading = serde_json::from_slice(&a).unwrap();
    again.validate().unwrap();
    assert_eq!(serde_json::to_vec(&again).unwrap(), a);
    let mut f = r.birth.facts.clone();
    f.date.year = 2020;
    let outside = read(
        personal_facts::birth_resource(&f).unwrap(),
        FoldChoice::Refuse,
    )
    .unwrap();
    assert!(
        outside
            .with_supplement(Some(HouseChoice::Placidus), s.orientation.clone())
            .is_err()
    );
    assert!(
        placed_cap_reading()
            .with_supplement(None, s.orientation.clone())
            .is_err()
    );
    let mut bad = serde_json::to_value(&r).unwrap();
    let mut rows = text.lines().map(str::to_string).collect::<Vec<_>>();
    rows[1].replace_range(154..165, "  0.3555000");
    let changed = rows.join("\n") + "\n";
    assert_ne!(changed, text);
    bad["supplement"]["orientation"]["finals2000a"] = serde_json::json!(changed);
    let mut content = bad.clone();
    content.as_object_mut().unwrap().remove("id");
    bad["id"] = serde_json::json!(
        blake3::hash(&serde_json::to_vec(&content).unwrap())
            .to_hex()
            .to_string()
    );
    assert!(
        serde_json::from_value::<NatalReading>(bad)
            .unwrap()
            .validate()
            .is_err()
    );
}
#[test]
fn exact_civil_midnight_uses_its_observation_including_after_leap_insertion() {
    use cleromancy::natal_cap::*;
    for (year, text, month, day) in [
        (
            2000,
            include_str!("fixtures/natal-observed-finals2000a.txt"),
            1,
            1,
        ),
        (
            2017,
            include_str!("fixtures/natal-observed-leap-finals2000a.txt"),
            1,
            1,
        ),
    ] {
        let row = text
            .lines()
            .find(|r| {
                r[0..2].trim().parse::<i32>().unwrap() == year % 100
                    && r[2..4].trim() == month.to_string()
                    && r[4..6].trim() == day.to_string()
            })
            .unwrap();
        let mut f = placed_cap_reading().birth.facts;
        f.date = CivilDate { year, month, day };
        f.time = BirthTime::Known {
            hour: 0,
            minute: 0,
            second: 0,
        };
        let base = read(
            personal_facts::birth_resource(&f).unwrap(),
            FoldChoice::Refuse,
        )
        .unwrap();
        for source in [row.to_string(), text.to_string()] {
            let r = base
                .clone()
                .with_supplement(
                    Some(HouseChoice::Placidus),
                    Some(OrientationSnapshot {
                        finals2000a: source,
                    }),
                )
                .unwrap();
            r.validate().unwrap();
            assert!(r.supplement.unwrap().observed_values.is_some());
        }
    }
}

fn aspect_pair_chart(first: u32, second: u32) -> cleromancy::AstrologyChart {
    cleromancy::AstrologyChart::new(
        "synthetic-aspect-boundaries/v1",
        "integer fixture",
        "synthetic retained longitudes",
        cleromancy::AstrologyMoment::global("2000-01-01T12:00:00+00:00"),
        [
            cleromancy::AstrologyPosition::new("A", first, 0),
            cleromancy::AstrologyPosition::new("B", second, 0),
        ],
    )
    .unwrap()
}

#[test]
fn natal_aspects_preset_all_five_inclusive_boundaries_and_wrap() {
    use cleromancy::{AspectKind::*, natal_aspects::*};
    let rules = OrbPolicy::default();
    assert_eq!(
        serde_json::to_value(rules).unwrap(),
        serde_json::json!({
            "conjunction_millidegrees":8000, "opposition_millidegrees":8000,
            "trine_millidegrees":6000, "square_millidegrees":6000,
            "sextile_millidegrees":4000
        })
    );
    for (kind, exact) in [
        (Conjunction, 0),
        (Sextile, 60000),
        (Square, 90000),
        (Trine, 120000),
        (Opposition, 180000),
    ] {
        let orb = rules.orb_millidegrees(kind);
        for sign in [-1_i64, 1] {
            for deviation in [0, orb - 1, orb, orb + 1] {
                // Shift origin close to 360° to exercise both wrap directions.
                let a = 359999;
                let b =
                    (a as i64 + exact as i64 + sign * deviation as i64).rem_euclid(360000) as u32;
                let hits = derive(&aspect_pair_chart(a, b), rules).unwrap();
                let hit = hits.iter().find(|h| h.kind == kind);
                assert_eq!(
                    hit.is_some(),
                    deviation <= orb,
                    "{kind:?} {sign} {deviation}"
                );
                if let Some(hit) = hit {
                    assert_eq!(hit.orb_millidegrees, deviation);
                    assert_eq!(hit.exact_millidegrees, exact);
                    assert_eq!((&*hit.first, &*hit.second), ("A", "B"));
                }
            }
        }
    }
    let hits = derive(&aspect_pair_chart(359999, 0), rules).unwrap();
    assert_eq!(hits[0].separation_millidegrees, 1);
    assert_eq!(hits[0].kind, Conjunction);
}

#[test]
fn natal_aspects_exact_custom_rules_stable_order_and_invalid_inputs() {
    use cleromancy::natal_aspects::*;
    let exact = AspectRules {
        conjunction_millidegrees: 0,
        opposition_millidegrees: 0,
        trine_millidegrees: 0,
        square_millidegrees: 0,
        sextile_millidegrees: 0,
    };
    assert_eq!(
        derive(&aspect_pair_chart(0, 180000), exact).unwrap().len(),
        1
    );
    assert!(
        derive(&aspect_pair_chart(0, 180001), exact)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        derive(&aspect_pair_chart(0, 0), exact).unwrap()[0].orb_millidegrees,
        0
    );
    let mut rules = exact;
    rules.sextile_millidegrees = 4000;
    assert_eq!(
        derive(&aspect_pair_chart(0, 64000), rules).unwrap().len(),
        1
    );
    assert!(
        derive(&aspect_pair_chart(0, 94000), rules)
            .unwrap()
            .is_empty()
    );
    let wide = AspectRules {
        conjunction_millidegrees: 180000,
        opposition_millidegrees: 180000,
        trine_millidegrees: 180000,
        square_millidegrees: 180000,
        sextile_millidegrees: 180000,
    };
    let mut chart = aspect_pair_chart(0, 75000);
    chart
        .positions
        .push(cleromancy::AstrologyPosition::new("C", 270000, 80000));
    let hits = derive(&chart, wide).unwrap();
    assert_eq!(hits.len(), 15);
    assert_eq!(
        hits.iter()
            .map(|h| (h.first.as_str(), h.second.as_str(), h.exact_millidegrees))
            .collect::<Vec<_>>(),
        [("A", "B"), ("A", "C"), ("B", "C")]
            .into_iter()
            .flat_map(|(a, b)| [0, 60000, 90000, 120000, 180000].map(|v| (a, b, v)))
            .collect::<Vec<_>>()
    );
    chart.positions.reverse();
    assert_eq!(derive(&chart, wide).unwrap(), hits);
    chart
        .positions
        .iter_mut()
        .for_each(|p| p.latitude_millidegrees = -80000);
    assert_eq!(derive(&chart, wide).unwrap(), hits);
    for kind in [
        cleromancy::AspectKind::Conjunction,
        cleromancy::AspectKind::Opposition,
        cleromancy::AspectKind::Trine,
        cleromancy::AspectKind::Square,
        cleromancy::AspectKind::Sextile,
    ] {
        let mut value = serde_json::to_value(exact).unwrap();
        let key = format!(
            "{}_millidegrees",
            serde_json::to_value(kind).unwrap().as_str().unwrap()
        );
        value[&key] = serde_json::json!(180001);
        assert!(derive(&chart, serde_json::from_value(value).unwrap()).is_err());
    }
    chart.positions[0].longitude_millidegrees = 360000;
    assert!(derive(&chart, exact).is_err());
}

fn resealed_natal(mut value: serde_json::Value) -> NatalReading {
    value.as_object_mut().unwrap().remove("id");
    let id = blake3::hash(&serde_json::to_vec(&value).unwrap())
        .to_hex()
        .to_string();
    value["id"] = serde_json::json!(id);
    serde_json::from_value(value).unwrap()
}

#[test]
fn natal_aspects_explicit_extension_replays_and_rejects_resealed_tampering() {
    use cleromancy::natal_aspects::*;
    let original = placed_cap_reading()
        .with_houses(cleromancy::natal_houses::HouseSystem::Equal)
        .unwrap()
        .with_supplement(None, None)
        .unwrap();
    let old_bytes = serde_json::to_vec(&original).unwrap();
    assert!(original.aspects.is_none());
    assert!(
        !serde_json::to_value(&original)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("aspects")
    );
    let reading = original
        .clone()
        .with_aspects(AspectRules::default())
        .unwrap();
    assert_ne!(reading.id, original.id);
    assert_eq!(reading.outcome, original.outcome);
    assert_eq!(reading.houses, original.houses);
    assert_eq!(reading.supplement, original.supplement);
    assert_eq!(serde_json::to_vec(&original).unwrap(), old_bytes);
    let AspectOutcome::Calculated { hits, .. } = &reading.aspects.as_ref().unwrap().outcome else {
        panic!()
    };
    assert!(hits.len() > 1);
    let bytes = serde_json::to_vec(&reading).unwrap();
    let reopened: NatalReading = serde_json::from_slice(&bytes).unwrap();
    reopened.validate().unwrap();
    assert_eq!(serde_json::to_vec(&reopened).unwrap(), bytes);
    for change in [
        "schema",
        "method",
        "rules",
        "chart",
        "kind",
        "pair",
        "exact",
        "separation",
        "orb",
        "missing",
        "extra",
        "order",
        "unavailable",
    ] {
        let mut value = serde_json::to_value(&reading).unwrap();
        let aspects = &mut value["aspects"];
        match change {
            "schema" => aspects["schema"] = serde_json::json!("cleromancy.natal-aspects/v2"),
            "method" => aspects["method"] = serde_json::json!("other-edition"),
            "rules" => {
                aspects["rules"] = serde_json::json!({"conjunction_millidegrees":0,"opposition_millidegrees":0,"trine_millidegrees":0,"square_millidegrees":0,"sextile_millidegrees":0})
            },
            "chart" => aspects["outcome"]["chart_digest"] = serde_json::json!("different-chart"),
            "kind" => aspects["outcome"]["hits"][0]["kind"] = serde_json::json!("opposition"),
            "pair" => aspects["outcome"]["hits"][0]["first"] = serde_json::json!("invented"),
            "exact" => aspects["outcome"]["hits"][0]["exact_millidegrees"] = serde_json::json!(1),
            "separation" => {
                aspects["outcome"]["hits"][0]["separation_millidegrees"] = serde_json::json!(1)
            },
            "orb" => aspects["outcome"]["hits"][0]["orb_millidegrees"] = serde_json::json!(99999),
            "missing" => {
                aspects["outcome"]["hits"].as_array_mut().unwrap().pop();
            },
            "extra" => {
                let hit = aspects["outcome"]["hits"][0].clone();
                aspects["outcome"]["hits"].as_array_mut().unwrap().push(hit);
            },
            "order" => aspects["outcome"]["hits"].as_array_mut().unwrap().reverse(),
            _ => aspects["outcome"] = serde_json::json!({"kind":"unavailable","reason":"invented"}),
        }
        let bad = resealed_natal(value);
        assert!(bad.validate().is_err(), "{change}");
        let mut host = cleromancy::CleromancyHost::empty(muniment::MemoryBackend::new());
        assert!(
            host.insert_natal_reading(&bad).is_err(),
            "admission {change}"
        );
    }
    let mut host = cleromancy::CleromancyHost::empty(muniment::MemoryBackend::new());
    host.insert_natal_reading(&reading).unwrap();
    let reopened = cleromancy::CleromancyHost::candidate(host.graph().clone()).unwrap();
    assert_eq!(reopened.natal_readings().unwrap(), vec![reading]);
}

#[test]
fn natal_aspects_unavailable_is_retained_and_invalid_rules_refuse() {
    use cleromancy::natal_aspects::*;
    let mut facts = birth().facts;
    facts.time = BirthTime::Unknown;
    let reading = read(
        personal_facts::birth_resource(&facts).unwrap(),
        FoldChoice::Refuse,
    )
    .unwrap()
    .with_aspects(AspectRules::default())
    .unwrap();
    assert!(matches!(
        reading.aspects.as_ref().unwrap().outcome,
        AspectOutcome::Unavailable { .. }
    ));
    reading.validate().unwrap();
    let mut value = serde_json::to_value(&reading).unwrap();
    value["aspects"]["outcome"]["reason"] = serde_json::json!("forged");
    assert!(resealed_natal(value).validate().is_err());
    let rules = AspectRules {
        conjunction_millidegrees: u32::MAX,
        ..AspectRules::default()
    };
    assert!(reading.with_aspects(rules).is_err());
}

#[test]
fn consolidation_replays_frozen_native_editions_without_changing_records() {
    let receipt: serde_json::Value = serde_json::from_str(include_str!(
        "../receipts/developer_id_native_20261017/reopen-01/receipt.json"
    ))
    .unwrap();
    let readings = receipt["readings"].as_array().unwrap();
    assert_eq!(readings.len(), 6);
    for stored in readings {
        let reading: NatalReading = serde_json::from_value(stored.clone()).unwrap();
        reading.validate().unwrap();
        assert_eq!(serde_json::to_value(&reading).unwrap(), *stored);
    }
}
