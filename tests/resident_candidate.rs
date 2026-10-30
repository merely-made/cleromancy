// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The product transforms a resident candidate; its recorder emits journal
//! edits without opening a private store or drawing during validation.
use std::sync::{Arc, Mutex};

use cleromancy::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
use mere::kernel::graph::Graph;

#[test]
fn candidate_actions_capture_stable_edits_and_leave_input_unchanged() {
    let input = Graph::new();
    let before = graph_state(&input);
    let mut host = CleromancyHost::candidate(input.clone()).unwrap();
    let edits = Arc::new(Mutex::new(Vec::new()));
    host.record_edits(edits.clone());
    let mut consultation = Consultation::new(host);
    let field = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("A boundary", "What should remain explicit?", "ownership"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &field, SelectionMode::Calculated)).unwrap();
    assert_eq!(detail.readings.len(), 1);
    let recorded = edits.lock().unwrap().clone();
    assert!(!recorded.is_empty());
    let replayed = mere::kernel::graph::replay_captured_deltas(recorded);
    cleromancy::host::validate_divination_graph(&replayed).unwrap();
    let replayed = Consultation::new(CleromancyHost::candidate(replayed).unwrap());
    assert_eq!(replayed.detail(&detail.session.id).unwrap(), detail);
    assert_eq!(graph_state(&input), before);
    cleromancy::host::validate_divination_graph(consultation.host().graph()).unwrap();
    let reopened =
        Consultation::new(CleromancyHost::candidate(consultation.host().graph().clone()).unwrap());
    assert_eq!(reopened.detail(&detail.session.id).unwrap(), detail);
}

#[test]
fn candidate_constructor_refuses_tampered_resident_graph() {
    let mut consultation = Consultation::new(CleromancyHost::candidate(Graph::new()).unwrap());
    let field = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("A boundary", "What should remain explicit?", "ownership"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &field, SelectionMode::Calculated)).unwrap();
    let mut graph = consultation.host().graph().clone();
    let id = graph
        .get_node_by_url(&format!("cleromancy://reading/{}", detail.readings[0].id))
        .unwrap()
        .1
        .id;
    let facet = chartulary::FacetId::new(cleromancy::host::READING_FACET);
    let mut value = graph.facets().get(&id, &facet).unwrap().clone();
    value["candidate_id"] = serde_json::json!("invented");
    let mut facets = graph.facets().clone();
    facets.set(id, facet, value, &chartulary::AcceptAll).unwrap();
    graph.install_facets(facets).unwrap();
    assert!(CleromancyHost::candidate(graph).is_err());
}

#[cfg(not(feature = "legacy-receipts"))]
#[test]
fn default_transformer_neither_loads_nor_rewrites_the_retired_slot() {
    use muniment::{Backend, MemoryBackend};
    let backend = MemoryBackend::new();
    let retired = "cleromancy/mere-host/v1";
    let preserved = b"unread retired store fixture";
    pollster::block_on(backend.put(retired, preserved)).unwrap();
    let mut consultation = Consultation::new(CleromancyHost::empty(backend.clone()));
    assert!(
        consultation.host().is_empty(),
        "the new domain starts fresh"
    );
    pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    assert_eq!(
        backend.len(),
        1,
        "product transformations create no private slots"
    );
    assert_eq!(
        pollster::block_on(backend.get(retired)).unwrap().unwrap(),
        preserved
    );
    assert_eq!(consultation.catalog().unwrap().fields.len(), 1);
}

#[test]
fn typed_decks_survive_journal_replay_and_forged_metadata_is_refused() {
    let mut host = CleromancyHost::candidate(Graph::new()).unwrap();
    let edits = Arc::new(Mutex::new(Vec::new()));
    host.record_edits(edits.clone());
    let mut consultation = Consultation::new(host);
    let initial = pollster::block_on(
        consultation.install_tarot_deck("cleromancy.tarot/rws-complete-reflective-v1"),
    )
    .unwrap();
    assert_eq!(consultation.catalog().unwrap().fields.len(), 1);
    let fields = pollster::block_on(consultation.install_typed_tarot_at(1)).unwrap();
    assert_eq!(fields[0], initial);
    assert_eq!(fields.len(), 2);
    let context = pollster::block_on(
        consultation.save_context_at(ContextDraft::new("C3", "What is available?", ""), 2),
    )
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &fields[0], SelectionMode::Calculated))
            .unwrap();
    let saved = consultation.host().graph().to_snapshot();
    let recorded = edits.lock().unwrap().clone();
    let replayed = mere::kernel::graph::replay_captured_deltas(recorded);
    let reopened = Consultation::new(CleromancyHost::candidate(replayed).unwrap());
    assert_eq!(reopened.detail(&detail.session.id).unwrap(), detail);
    assert_eq!(reopened.catalog().unwrap().fields.len(), 2);
    let meaning = cleromancy::narrative::position_meaning(&detail, "focus");
    assert_eq!(
        meaning.meaning,
        detail
            .field
            .position_meanings
            .as_ref()
            .unwrap()
            .positions
            .iter()
            .find(|p| p.name == "focus")
            .unwrap()
            .meaning
    );
    let mut graph = consultation.host().graph().clone();
    let id = graph
        .get_node_by_url(&format!("cleromancy://field/{}", fields[0]))
        .unwrap()
        .1
        .id;
    let facet = chartulary::FacetId::new(cleromancy::host::FIELD_FACET);
    let mut value = graph.facets().get(&id, &facet).unwrap().clone();
    value["tarot"]["cards"][0]["interpretation"]["text"] = serde_json::json!("unbound replacement");
    let mut facets = graph.facets().clone();
    facets.set(id, facet, value, &chartulary::AcceptAll).unwrap();
    graph.install_facets(facets).unwrap();
    assert!(cleromancy::host::validate_divination_graph(&graph).is_err());
    assert_eq!(graph_state(consultation.host().graph()), {
        let mut value = serde_json::to_value(saved).unwrap();
        value.as_object_mut().unwrap().remove("timestamp_secs");
        value
    });
}

// to_snapshot stamps a new serialization time; compare the graph's durable
// contents, not the wall clock at which each inspection was performed.
fn graph_state(graph: &Graph) -> serde_json::Value {
    let mut value = serde_json::to_value(graph.to_snapshot()).unwrap();
    value.as_object_mut().unwrap().remove("timestamp_secs");
    value
}
