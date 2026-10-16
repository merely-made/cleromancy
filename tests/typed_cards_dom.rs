// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "support/cleromancy_dom.rs"]
mod support;

use cleromancy::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
use mere::kernel::graph::Graph;
use support::{click_key, harness, rect_at_key, text_at_key};

#[test]
fn reader_crosswalks_without_drawing_and_discloses_each_tradition() {
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
    let snapshot = graph_state(consultation.host().graph());
    let catalog = consultation.catalog().unwrap();
    let mut h = harness(catalog.clone());
    let expected = detail.clone();
    h.update(move |ui| ui.present_reading(catalog, detail));
    assert!(text_at_key(&h, "typed-card-axes").contains("number: 8"));
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

// to_snapshot stamps a new serialization time; compare the graph's durable
// contents, not the wall clock at which each inspection was performed.
fn graph_state(graph: &Graph) -> serde_json::Value {
    let mut value = serde_json::to_value(graph.to_snapshot()).unwrap();
    value.as_object_mut().unwrap().remove("timestamp_secs");
    value
}
