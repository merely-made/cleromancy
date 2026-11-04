// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

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
