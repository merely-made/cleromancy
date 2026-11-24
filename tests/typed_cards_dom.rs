// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "support/cleromancy_dom.rs"]
mod support;

use cleromancy::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
use kernel::graph::Graph;
use support::{click_key, harness, rect_at_key, text_at_key};

#[test]
fn reader_crosswalks_without_drawing_and_discloses_each_tradition() {
    let (consultation, detail) = strength_reading();
    let snapshot = graph_state(consultation.host().graph());
    let catalog = consultation.catalog().unwrap();
    let mut h = harness(catalog.clone());
    let expected = detail.clone();
    h.update(move |ui| ui.present_reading(catalog, detail));
    assert!(text_at_key(&h, "typed-card-axes").contains("number: 8"));
    let ontology = text_at_key(&h, "reading-ontology");
    let gd = text_at_key(&h, "correspondence:book-t");
    assert!(gd.contains("Golden Dawn / Book T"));
    assert!(gd.contains("Leo"));
    assert!(gd.contains("undated transcription"));
    assert!(gd.contains("consulted 2026-10-15"));
    let th = text_at_key(&h, "correspondence:thierens-1930");
    assert!(th.contains("Thierens"));
    assert!(th.contains("Aquarius"));
    assert!(th.contains("1930"));
    let (gx, gy, gw, _) = rect_at_key(&h, "correspondence:book-t");
    let (tx, ty, _, _) = rect_at_key(&h, "correspondence:thierens-1930");
    assert!(
        (gy - ty).abs() < 1.0 && tx >= gx + gw,
        "traditions should occupy separate columns at wide width"
    );
    assert!(
        click_key(
            &mut h,
            "crosswalk:cleromancy.tarot/marseille-complete-reflective-v1"
        )
        .is_none()
    );
    assert!(text_at_key(&h, "typed-card-heading").contains("Marseille structure"));
    assert!(text_at_key(&h, "typed-card-axes").contains("number: 11"));
    assert_eq!(text_at_key(&h, "reading-ontology"), ontology);
    assert_eq!(h.state().detail().unwrap(), &expected);
    assert!(click_key(&mut h, "return-drawn-card").is_none());
    assert!(text_at_key(&h, "typed-card-axes").contains("number: 8"));
    assert_eq!(graph_state(consultation.host().graph()), snapshot);
    // Compact layout keeps the same two independently disclosed traditions.
    h.layout_at(420.0, 8000.0);
    let (x, y, w, _) = rect_at_key(&h, "correspondence:book-t");
    let (x2, y2, w2, _) = rect_at_key(&h, "correspondence:thierens-1930");
    assert!(x >= 0.0 && x + w <= 420.0 && x2 >= 0.0 && x2 + w2 <= 420.0);
    assert!(y2 > y);
}

#[test]
fn cited_sign_opens_matching_cards_without_changing_the_draw() {
    let (consultation, detail) = strength_reading();
    let snapshot = graph_state(consultation.host().graph());
    let catalog = consultation.catalog().unwrap();
    let mut h = harness(catalog.clone());
    let saved = detail.clone();
    h.update(move |ui| ui.present_reading(catalog, detail));
    let graph = text_at_key(&h, "reading-ontology");
    let source = "correspondence-term:book-t:Sign:Leo";
    assert!(click_key(&mut h, source).is_none());
    let matches = text_at_key(&h, "correspondence-results");
    assert!(matches.contains("Golden Dawn / Book T"));
    assert!(matches.contains("Strength") && matches.contains("Five of Wands"));
    // Thierens assigns Leo to The Hierophant. It belongs to a different citation.
    assert!(!matches.contains("The Hierophant"));
    assert!(click_key(&mut h, "correspondence-card:wands-five").is_none());
    assert!(text_at_key(&h, "typed-card-heading").contains("Five of Wands"));
    assert!(text_at_key(&h, "correspondence:thierens-1930").contains("Sign: not transcribed"));
    assert!(!support::has_key(&h, "correspondence-results"));
    assert_eq!(text_at_key(&h, "reading-ontology"), graph);
    assert!(click_key(&mut h, "return-drawn-card").is_none());
    assert!(text_at_key(&h, "typed-card-heading").contains("Strength"));
    assert_eq!(h.state().detail().unwrap(), &saved);
    assert_eq!(graph_state(consultation.host().graph()), snapshot);
}

fn strength_reading() -> (
    Consultation<muniment::MemoryBackend>,
    cleromancy::ConsultationDetail,
) {
    let mut consultation = Consultation::new(CleromancyHost::candidate(Graph::new()).unwrap());
    let fields = pollster::block_on(consultation.install_typed_tarot_at(1)).unwrap();
    let context = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("Courage", "What can I meet patiently?", "courage"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &fields[0], SelectionMode::Calculated))
            .unwrap();
    assert_eq!(detail.readings[0].title, "Strength");
    (consultation, detail)
}

#[test]
fn selected_ontology_projects_the_saved_draw_through_the_reading_view() {
    let (consultation, detail) = strength_reading();
    let catalog = consultation.catalog().unwrap();
    let mut h = harness(catalog.clone());
    h.update({
        let catalog = catalog.clone();
        let detail = detail.clone();
        move |ui| ui.present_reading(catalog, detail)
    });
    let ontology = text_at_key(&h, "reading-ontology");
    assert_drawn_card(&ontology, &detail.readings[0].id);
    drop(h);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("lens.toml"),
        include_str!("../assets/ontology/lens.toml"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("custom.toml"),
        include_str!("../assets/ontology/tarot.toml")
            .replace("reading:drawnCard", "reading:selectedCard"),
    )
    .unwrap();
    let state =
        cleromancy::ConsultationUi::with_ontology_directory(catalog.clone(), Some(dir.path()))
            .unwrap();
    let mut alternative = support::harness_with_state(state);
    let expected = detail.clone();
    alternative.update(move |ui| ui.present_reading(catalog, detail));
    assert_eq!(
        text_at_key(&alternative, "reading-ontology"),
        ontology.replace("cleromancy#drawnCard", "cleromancy#selectedCard")
    );
    assert_eq!(alternative.state().detail().unwrap(), &expected);
}

fn assert_drawn_card(ontology: &str, id: &str) {
    const LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
    let graph: serde_json::Value = serde_json::from_str(ontology).unwrap();
    let nodes = graph["@graph"].as_array().unwrap();
    let reading_id = format!("cleromancy://reading/{id}");
    let result = nodes.iter().find(|node| node["@id"] == reading_id).unwrap();
    assert_eq!(result[LABEL], "Strength");
    let card_id = &result["https://mere.computer/ns/cleromancy#drawnCard"]["@id"];
    let card = nodes.iter().find(|node| node["@id"] == *card_id).unwrap();
    assert_eq!(card[LABEL], "Strength");
    assert!(
        card["@type"]
            .as_array()
            .unwrap()
            .iter()
            .any(|iri| iri == "https://w3id.org/taro/ontology#MajorArcanaCard")
    );
    let deck_id = &card["https://w3id.org/taro/ontology#inDeck"]["@id"];
    let deck = nodes.iter().find(|node| node["@id"] == *deck_id).unwrap();
    assert_eq!(deck[LABEL], "Rider–Waite–Smith · reflective edition 1");
    assert_eq!(
        deck["@type"],
        serde_json::json!(["https://w3id.org/taro/ontology#Deck"])
    );
}

// to_snapshot stamps a new serialization time; compare the graph's durable
// contents, not the wall clock at which each inspection was performed.
fn graph_state(graph: &Graph) -> serde_json::Value {
    let mut value = serde_json::to_value(graph.to_snapshot()).unwrap();
    value.as_object_mut().unwrap().remove("timestamp_secs");
    value
}
