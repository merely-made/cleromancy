// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! R3 receipt: the Sky surface only projects already-stored day facts.

#[path = "support/cleromancy_dom.rs"]
mod support;

#[cfg(feature = "sky-timeline")]
use cleromancy::sky::turquet::analytical_sky_day;
#[cfg(not(feature = "sky-timeline"))]
use cleromancy::sky::{SkyDayFacts, SkyFact, SkyProvenance, SkyTtInterval};
use cleromancy::sky::{
    SkyEarthOrientationApproximation, SkyEarthOrientationPolicy, SkyFactKind, SkyNumericalPolicy,
    SkySearchControls, SkyTwilightMeasurement, SkyTwilightPolicy, UtcCivilDay, Wgs84Observer,
};
use cleromancy::{CleromancyHost, Consultation};
#[cfg(feature = "sky-timeline")]
use cleromancy::{HostError, SKY_DAY_FACTS_FACET};
#[cfg(feature = "sky-timeline")]
use genet_probe::Selector;
#[cfg(feature = "sky-timeline")]
use muniment::Backend;
use muniment::MemoryBackend;
#[cfg(not(feature = "sky-timeline"))]
use support::attr_at_id;
use support::harness;
#[cfg(feature = "sky-timeline")]
use support::{has_key, switch_surface, take_action, text_at_key};

#[cfg(feature = "sky-timeline")]
#[test]
fn stored_dallas_events_are_sorted_and_render_without_a_view_time_calculation() {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let policy = dallas_policy();
    let dallas_eclipse_day = analytical_sky_day(
        UtcCivilDay::new(2024, 4, 8).unwrap(),
        dallas(),
        policy.clone(),
    )
    .unwrap();
    // This second stored record gives the enumerator a civil-day sort receipt.
    // Both calculations are fixture setup; the view receives only the catalog.
    let following_day =
        analytical_sky_day(UtcCivilDay::new(2024, 4, 9).unwrap(), dallas(), policy).unwrap();
    host.insert_sky_day_facts(&following_day).unwrap();
    host.insert_sky_day_facts(&dallas_eclipse_day).unwrap();

    let stored = host.sky_day_facts().unwrap();
    assert_eq!(
        stored,
        vec![dallas_eclipse_day.clone(), following_day.clone()]
    );
    let catalog = Consultation::new(host).catalog().unwrap();
    assert_eq!(catalog.sky_day_facts, stored);

    let mut h = harness(catalog);
    switch_surface(&mut h, "Sky");
    assert!(has_key(&h, "region:sky"));
    assert!(text_at_key(&h, "sky-record-day").contains("2024-04-08"));
    assert!(text_at_key(&h, "sky-observer").contains("32776700"));
    assert!(text_at_key(&h, "sky-record-digest").contains(&dallas_eclipse_day.digest()));
    let ledger = text_at_key(&h, "sky-event-ledger");
    assert!(ledger.contains("New Moon"));
    assert!(ledger.contains("Dawn"));
    assert!(ledger.contains("Dusk"));
    assert!(ledger.contains("TT Julian day"));
    let mut cursor = 0;
    for fact in &dallas_eclipse_day.facts {
        let row = format!(
            "{}: TT Julian day {:?} to {:?}.",
            fact_name(fact.kind),
            fact.interval.start_julian_day,
            fact.interval.end_julian_day,
        );
        let offset = ledger[cursor..]
            .find(&row)
            .unwrap_or_else(|| panic!("missing or out-of-order stored event row: {row}"));
        cursor += offset + row.len();
    }
    assert!(text_at_key(&h, "sky-time-scale").contains("Terrestrial Time (TT)"));

    assert!(h.click_on(
        &Selector::role("button").with_attr("id", "cleromancy-sky-policy-and-provenance-trigger",),
    ));
    assert!(take_action(&mut h).is_none());
    assert!(text_at_key(&h, "sky-policy-phase-search").contains("step 3600 seconds"));
    assert!(text_at_key(&h, "sky-policy-twilight").contains("AirlessSolarCenter"));
    for fact in &dallas_eclipse_day.facts {
        let provenance = text_at_key(&h, &format!("sky-provenance:{}", fact_key(fact.kind)));
        for expected in [
            format!("model {}", fact.provenance.model),
            format!("provider {}", fact.provenance.provider),
            format!(
                "provider snapshot {}",
                fact.provenance
                    .provider_snapshot
                    .as_deref()
                    .unwrap_or("not recorded")
            ),
            format!("transform {}", fact.provenance.transform),
            format!("earth orientation {}", fact.provenance.earth_orientation),
        ] {
            assert!(
                provenance.contains(&expected),
                "missing `{expected}` in `{provenance}`"
            );
        }
    }

    let next_label = format!("2024-04-09 ({})", &following_day.digest()[..12]);
    support::select(&mut h, "Stored civil day", &next_label);
    assert!(take_action(&mut h).is_none());
    assert!(text_at_key(&h, "sky-record-day").contains("2024-04-09"));
}

#[cfg(feature = "sky-timeline")]
#[test]
fn catalog_rejects_a_raw_persisted_sky_record_with_an_invalid_schema() {
    let backend = MemoryBackend::new();
    let mut host = CleromancyHost::empty(backend.clone());
    let facts = analytical_sky_day(
        UtcCivilDay::new(2024, 4, 8).unwrap(),
        dallas(),
        dallas_policy(),
    )
    .unwrap();
    host.insert_sky_day_facts(&facts).unwrap();
    pollster::block_on(host.persist(1)).unwrap();

    let bytes = pollster::block_on(backend.get(cleromancy::host::HOST_SLOT))
        .unwrap()
        .unwrap();
    let mut document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(replace_sky_schema(&mut document));
    pollster::block_on(backend.put(
        cleromancy::host::HOST_SLOT,
        &serde_json::to_vec(&document).unwrap(),
    ))
    .unwrap();

    let reopened = pollster::block_on(CleromancyHost::open(backend)).unwrap();
    assert!(matches!(
        reopened.sky_day_facts(),
        Err(HostError::InvalidStoredFacet {
            facet: SKY_DAY_FACTS_FACET,
            ..
        })
    ));
}

#[cfg(feature = "sky-timeline")]
fn replace_sky_schema(value: &mut serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("schema").and_then(serde_json::Value::as_str)
                == Some("cleromancy.sky-day-facts/v1")
            {
                object.insert(
                    "schema".to_string(),
                    serde_json::Value::String("cleromancy.sky-day-facts/invalid".to_string()),
                );
                return true;
            }
            object.values_mut().any(replace_sky_schema)
        },
        serde_json::Value::Array(values) => values.iter_mut().any(replace_sky_schema),
        _ => false,
    }
}

#[cfg(not(feature = "sky-timeline"))]
#[test]
fn stored_sky_facts_remain_catalogued_while_the_sky_tab_is_disabled() {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let facts = stored_fixture();
    host.insert_sky_day_facts(&facts).unwrap();
    let catalog = Consultation::new(host).catalog().unwrap();
    assert_eq!(catalog.sky_day_facts, vec![facts]);

    let h = harness(catalog);
    assert_eq!(
        attr_at_id(&h, "cleromancy-surfaces-item-sky", "aria-disabled").as_deref(),
        Some("true")
    );
}

fn dallas() -> Wgs84Observer {
    Wgs84Observer::new(32_776_700, -96_797_000, 130_000).unwrap()
}

#[cfg(feature = "sky-timeline")]
fn fact_name(kind: SkyFactKind) -> &'static str {
    match kind {
        SkyFactKind::NewMoon => "New Moon",
        SkyFactKind::Dawn => "Dawn",
        SkyFactKind::Dusk => "Dusk",
    }
}

#[cfg(feature = "sky-timeline")]
fn fact_key(kind: SkyFactKind) -> &'static str {
    match kind {
        SkyFactKind::NewMoon => "new-moon",
        SkyFactKind::Dawn => "dawn",
        SkyFactKind::Dusk => "dusk",
    }
}

#[cfg(not(feature = "sky-timeline"))]
fn stored_fixture() -> SkyDayFacts {
    SkyDayFacts::new(
        UtcCivilDay::new(2024, 4, 8).unwrap(),
        dallas(),
        dallas_policy(),
        [SkyFact {
            kind: SkyFactKind::Dawn,
            interval: SkyTtInterval::new(2_460_408.9, 2_460_408.91).unwrap(),
            provenance: SkyProvenance {
                model: "stored fixture".to_string(),
                provider: "stored fixture".to_string(),
                provider_snapshot: None,
                transform: "stored fixture".to_string(),
                earth_orientation: "stored fixture".to_string(),
            },
        }],
    )
    .unwrap()
}

fn dallas_policy() -> SkyNumericalPolicy {
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
