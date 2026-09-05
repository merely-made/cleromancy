// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Semantic acceptance of authored layouts and imported chart facts.

#[path = "support/cleromancy_dom.rs"]
mod support;

use cleromancy::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, CleromancyHost, Consultation,
    ConsultationAction, ConsultationLayout, SpreadPosition, SpreadRelation, SpreadRelationKind,
    SpreadTemplate,
};
use muniment::MemoryBackend;
use support::{choose, click_key, harness, one, select, switch_surface, type_into};

#[test]
fn retained_consultation_dispatches_authored_layout_and_chart_input_actions() {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let field =
        cleromancy::TarotPack::rws_major_arcana().field(cleromancy::TarotQualification::Contextual);
    host.insert_field(&field).unwrap();
    let template = SpreadTemplate::new(
        "Four directions",
        [
            SpreadPosition::new("north", "North"),
            SpreadPosition::new("east", "East"),
            SpreadPosition::new("south", "South"),
            SpreadPosition::new("west", "West"),
        ],
        [SpreadRelation::new(
            "east",
            SpreadRelationKind::Questions,
            "north",
            "tests the north",
        )],
    )
    .unwrap();
    host.insert_spread_template(&template).unwrap();
    let chart = AstrologyChart::new(
        "source-import/v1",
        "example calculator",
        "example ephemeris",
        AstrologyMoment::global("2026-08-08T12:00:00Z"),
        [AstrologyPosition::new("Sun", 135_000, 0)],
    )
    .unwrap();
    host.insert_astrology_chart(&chart, 1_000).unwrap();
    let facts_digest = chart.facts(1_000).unwrap().digest();
    let catalog = Consultation::new(host).catalog().unwrap();
    let mut h = harness(catalog);

    for (label, value) in [
        ("Layout label", "Compass"),
        ("Layout positions", "here | Here\nthere | There"),
        (
            "Layout relationships",
            "there | supports | here | supports here",
        ),
    ] {
        type_into(&mut h, label, value);
    }
    match one(click_key(&mut h, "save-layout")) {
        ConsultationAction::SaveSpreadTemplate { draft } => {
            assert_eq!(draft.label, "Compass");
            assert_eq!(draft.positions, "here | Here\nthere | There");
            assert_eq!(draft.relations, "there | supports | here | supports here");
        }
        other => panic!("expected authored layout action, found {other:?}"),
    }

    switch_surface(&mut h, "Chart");
    for (label, value) in [
        ("Calculation algorithm", "source-import/v1"),
        ("Calculation engine", "example calculator"),
        ("Ephemeris source", "example ephemeris"),
        ("UTC instant", "2026-08-08T12:00:00Z"),
        ("Chart positions", "Sun | 135000 | 0 | false"),
    ] {
        type_into(&mut h, label, value);
    }
    match one(click_key(&mut h, "save-chart-facts")) {
        ConsultationAction::SaveAstrologyChart { draft } => {
            assert_eq!(draft.algorithm, "source-import/v1");
            assert_eq!(draft.positions, "Sun | 135000 | 0 | false");
            assert_eq!(draft.orb_millidegrees, "1000");
        }
        other => panic!("expected chart input action, found {other:?}"),
    }

    switch_surface(&mut h, "Today");
    type_into(&mut h, "Context label", "A four-part concern");
    type_into(&mut h, "Question", "Where is this going?");
    select(&mut h, "Stored field", "Rider-Waite-Smith Major Arcana");
    choose(&mut h, "Cast");
    choose(&mut h, "Authored layout");
    select(&mut h, "Authored layout", "Four directions");
    select(&mut h, "Astrology facts", "Chart");

    match one(click_key(&mut h, "read")) {
        ConsultationAction::Read {
            mode,
            layout,
            astrology_facts_digest,
            ..
        } => {
            assert_eq!(mode, cleromancy::SelectionMode::Cast);
            assert_eq!(layout, ConsultationLayout::Authored(template.id));
            assert_eq!(
                astrology_facts_digest.as_deref(),
                Some(facts_digest.as_str())
            );
        }
        other => panic!("expected authored read action, found {other:?}"),
    }
}
