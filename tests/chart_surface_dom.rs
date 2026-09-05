// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stored-chart projection and concurrence links through the retained DOM.

#[path = "support/cleromancy_dom.rs"]
mod support;

#[cfg(feature = "analytic-ephemeris")]
use cleromancy::ConsultationAction;
use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, CleromancyHost, Consultation, ContextDraft,
    ReadingError, SelectionMode,
};
use muniment::MemoryBackend;
#[cfg(feature = "analytic-ephemeris")]
use support::one;
use support::{
    attr_at_key, click_key, count_attr, harness, has_key, has_text, switch_surface, take_action,
    text_at_key, type_into,
};

#[test]
fn empty_chart_surface_keeps_manual_import_reachable() {
    let catalog = Consultation::new(CleromancyHost::empty(MemoryBackend::new()))
        .catalog()
        .unwrap();
    let mut h = harness(catalog);
    switch_surface(&mut h, "Chart");
    assert!(has_key(&h, "chart-empty"));
    assert!(has_key(&h, "save-chart-facts"));
}

#[test]
fn chart_surface_projects_stored_values_and_keeps_manual_import_available() {
    let (mut catalog, detail) = fixture(false);
    assert_eq!(catalog.astrology_facts.len(), 2);
    assert_eq!(catalog.astrology_charts.len(), 2);
    let eclipse = catalog
        .astrology_charts
        .iter()
        .find(|stored| stored.chart.moment.instant_utc == "2024-04-08T18:18:00Z")
        .unwrap();
    let chart_digest = eclipse.facts.chart_digest.clone();
    let selector_label = format!(
        "2024-04-08T18:18:00Z · {chart_digest} · 3 placements · 32900000, -96900000 microdegrees"
    );
    catalog
        .astrology_charts
        .sort_by_key(|stored| stored.chart.moment.instant_utc != "2024-04-08T18:18:00Z");
    let mut h = harness(catalog);
    let detail_catalog = h.state().catalog().clone();
    h.update(move |ui| ui.present_session(detail_catalog, detail));
    assert!(!has_key(&h, "reading-chart-concurrence"));
    switch_surface(&mut h, "Chart");
    assert!(has_text(&h, &selector_label));

    assert!(has_key(&h, "region:chart"));
    assert!(text_at_key(&h, "chart-moment").contains("2024-04-08T18:18:00Z"));
    assert!(text_at_key(&h, "chart-observer").contains("32900000, -96900000"));
    let positions = text_at_key(&h, "chart-positions");
    for header in [
        "Longitude (millidegrees)",
        "Latitude (millidegrees)",
        "Sign degree (millidegrees)",
    ] {
        assert!(
            positions.contains(header),
            "missing `{header}` in `{positions}`"
        );
    }
    for expected in [
        "Moon",
        "60000",
        "125",
        "Gemini",
        "retrograde",
        "Sun",
        "0",
        "Aries",
        "unknown",
    ] {
        assert!(
            positions.contains(expected),
            "missing `{expected}` in `{positions}`"
        );
    }
    assert_eq!(
        attr_at_key(&h, "chart-positions", "aria-label").as_deref(),
        Some("Stored positions")
    );
    assert!(has_key(&h, "chart-aspects"));
    let aspects = text_at_key(&h, "chart-aspects");
    for expected in [
        "Sextile",
        "Exact (millidegrees)",
        "Separation (millidegrees)",
        "Orb (millidegrees)",
    ] {
        assert!(
            aspects.contains(expected),
            "missing `{expected}` in `{aspects}`"
        );
    }
    assert_eq!(count_attr(&h, "role", "grid"), 2);
    assert_eq!(
        attr_at_key(&h, "chart-aspects", "aria-label").as_deref(),
        Some("Stored aspects")
    );
    let source = text_at_key(&h, "chart-source");
    for expected in [
        "source-import/v1",
        "fixture calculator",
        "fixture ephemeris",
        chart_digest.as_str(),
    ] {
        assert!(
            source.contains(expected),
            "missing `{expected}` in `{source}`"
        );
    }
    assert!(text_at_key(&h, "chart-digest").contains(&chart_digest));
    assert!(has_key(&h, "chart-facts-digest"));

    for (label, value) in [
        ("Calculation algorithm", "manual/v1"),
        ("Calculation engine", "manual engine"),
        ("Ephemeris source", "manual source"),
        ("UTC instant", "2024-04-09T00:00:00Z"),
        ("Chart positions", "Sun | 1 | 0 | false"),
    ] {
        type_into(&mut h, label, value);
    }
    assert!(click_key(&mut h, "save-chart-facts").is_some());

    #[cfg(feature = "analytic-ephemeris")]
    {
        assert!(has_key(&h, "calculate-chart"));
        match one(click_key(&mut h, "calculate-chart")) {
            ConsultationAction::CalculateAstrologyChart { draft } => {
                assert_eq!(draft.instant_utc, "2024-04-09T00:00:00Z");
                assert_eq!(draft.orb_millidegrees, "1000");
            },
            other => panic!("expected calculated chart action, found {other:?}"),
        }
    }
    #[cfg(not(feature = "analytic-ephemeris"))]
    assert!(!has_key(&h, "calculate-chart"));
}

#[test]
fn chart_concurrence_is_resolved_locally_without_a_product_action() {
    let (mut catalog, detail) = fixture(true);
    catalog
        .astrology_charts
        .sort_by_key(|stored| stored.chart.moment.instant_utc == "2024-04-08T18:18:00Z");
    let concurrence_id = detail.concurrences[0].id.clone();
    let mut h = harness(catalog);
    let detail_catalog = h.state().catalog().clone();
    h.update(move |ui| ui.present_session(detail_catalog, detail));
    assert!(has_key(&h, "reading-chart-concurrence"));
    let strip = text_at_key(&h, "reading-chart-concurrence");
    for expected in [
        "does not claim that the chart caused",
        concurrence_id.as_str(),
        "Moon Gemini",
        "Sun Aries",
    ] {
        assert!(
            strip.contains(expected),
            "missing `{expected}` in `{strip}`"
        );
    }
    assert_eq!(
        attr_at_key(&h, "reading-chart-concurrence", "aria-label").as_deref(),
        Some("Chart concurrence")
    );

    assert!(click_key(&mut h, "open-concurrent-chart").is_none());
    assert!(take_action(&mut h).is_none());
    assert_eq!(h.state().screen().key(), "chart");
    assert!(text_at_key(&h, "chart-moment").contains("2024-04-08T18:18:00Z"));
}

fn fixture(
    with_concurrence: bool,
) -> (
    cleromancy::ConsultationCatalog,
    cleromancy::ConsultationDetail,
) {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let chart = AstrologyChart::new(
        "source-import/v1",
        "fixture calculator",
        "fixture ephemeris",
        AstrologyMoment::at("2024-04-08T18:18:00Z", 32_900_000, -96_900_000),
        [
            AstrologyPosition::new("Sun", 0, 0),
            AstrologyPosition::new("Moon", 60_000, 125).with_retrograde(true),
            AstrologyPosition::new("Mercury", 90_000, -50),
        ],
    )
    .unwrap();
    host.insert_astrology_chart(&chart, 2_000).unwrap();
    let facts_digest = chart.facts(2_000).unwrap().digest();
    let earlier = AstrologyChart::new(
        "source-import/v1",
        "fixture calculator",
        "fixture ephemeris",
        AstrologyMoment::global("2024-01-01T00:00:00Z"),
        [AstrologyPosition::new("Venus", 300_000, 0).with_retrograde(false)],
    )
    .unwrap();
    host.insert_astrology_chart(&earlier, 1_000).unwrap();
    let mut consultation = Consultation::new(host);
    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("Chart fixture", "What was stored?", "chart"),
        2,
    ))
    .unwrap();
    let mut entropy = FixedEntropy::new([1, 2, 3, 4]);
    let detail = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        SelectionMode::Calculated,
        3_000,
        3,
        &mut entropy,
    ))
    .unwrap();
    let detail = if with_concurrence {
        pollster::block_on(consultation.associate_astrology_facts_at(
            &facts_digest,
            &detail.session.id,
            4_000,
            4,
        ))
        .unwrap()
    } else {
        detail
    };
    (consultation.catalog().unwrap(), detail)
}

struct FixedEntropy {
    words: std::collections::VecDeque<u64>,
}

impl FixedEntropy {
    fn new(words: impl IntoIterator<Item = u64>) -> Self {
        Self {
            words: words.into_iter().collect(),
        }
    }
}

impl EntropySource for FixedEntropy {
    fn next_u64(&mut self) -> Result<u64, ReadingError> {
        self.words
            .pop_front()
            .ok_or_else(|| ReadingError::Entropy("fixed source exhausted".to_string()))
    }
}
