// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/ambient_fixture.rs"]
mod support;
use cleromancy::{CleromancyHost, Consultation, ambient::*};
use mere::kernel::graph::Graph;
use std::collections::BTreeSet;

#[test]
fn source_engines_keep_occurrences_traditions_and_draw_time_evidence_distinct() {
    let (c, detail) = support::fixture();
    let data = c
        .ambient_context(&detail.session.id, "foundation")
        .unwrap()
        .data;
    let families: BTreeSet<_> = data.relations.iter().map(|r| r.family).collect();
    assert_eq!(families, RelationFamily::ALL.into_iter().collect());
    assert!(data.items.iter().any(|i| i.title.contains("Marseille")));
    assert!(
        data.items
            .iter()
            .any(|i| i.body.contains("Earlier reflection"))
    );
    assert!(data.items.iter().any(|i| i.title.contains("Sun")));
    assert!(
        data.items
            .iter()
            .any(|i| i.title.contains("What needs attention"))
    );
    assert_eq!(
        detail.readings[0].candidate_id, detail.readings[1].candidate_id,
        "fixture includes repeated card sources"
    );
    let a = data
        .items
        .iter()
        .find(|i| i.id.ends_with(":attention"))
        .unwrap();
    let b = data.items.iter().find(|i| i.id.ends_with(":next")).unwrap();
    assert_ne!(a.id, b.id);
    assert_eq!(
        a.source, b.source,
        "one source, separate position occurrences"
    );
    assert!(data.relations.iter().all(|r| !r.reason.is_empty()
        && !r.method.is_empty()
        && !r.evidence.is_empty()
        && r.version == 1
        && r.engine == r.family.engine()));
    assert_eq!(c.detail(&detail.session.id).unwrap(), detail);
}

#[test]
fn keeping_uses_shared_memory_levels_and_pose_curation_replays_without_sealed_edits() {
    let (c, detail) = support::fixture();
    let before_sealed = sealed(c.host().graph());
    let mut host = CleromancyHost::candidate(c.host().graph().clone()).unwrap();
    let edits = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    host.record_edits(edits.clone());
    let context = host
        .ambient_context(&detail.session.id, "foundation")
        .unwrap();
    let item = context
        .data
        .items
        .iter()
        .find(|i| i.id.starts_with("history:"))
        .unwrap()
        .id
        .clone();
    assert!(host.ambient_memories().unwrap().is_empty());
    let mut view = context.view;
    view.snapshot.selected_occurrence = Some(item.clone());
    view.foreground.insert(item.clone());
    view.roles.insert(item.clone(), seiche::Role::Pinned);
    view.positions.insert(item.clone(), [123000, 456000]);
    view.paused = true;
    host.insert_ambient_view(&view).unwrap();
    assert!(
        host.ambient_memories().unwrap().is_empty(),
        "focus, foreground and position pins do not keep"
    );
    host.examine_ambient(&detail.session.id, "foundation", &item, false, 4000)
        .unwrap();
    assert!(!host.ambient_memories().unwrap()[0].kept);
    host.examine_ambient(&detail.session.id, "foundation", &item, true, 5000)
        .unwrap();
    assert!(host.ambient_memories().unwrap()[0].kept);
    // A second position has a different subject and disclosed spread items.
    // Candidate validation must not reuse the first position's neighborhood.
    let other = host
        .ambient_context(&detail.session.id, "attention")
        .unwrap();
    let original_subject = other
        .data
        .items
        .iter()
        .find(|i| i.id == format!("reading:{}:foundation", detail.session.id))
        .unwrap()
        .id
        .clone();
    let mut other_view = other.view;
    other_view.snapshot.recipe.definition.arrangement.kind = "grid.ordered".into();
    host.insert_ambient_view(&other_view).unwrap();
    host.examine_ambient(
        &detail.session.id,
        "attention",
        &original_subject,
        true,
        6000,
    )
    .unwrap();
    assert_eq!(sealed(host.graph()), before_sealed);
    let mut replay = c.host().graph().clone();
    mere::kernel::graph::replay_captured_deltas_onto(&mut replay, edits.lock().unwrap().clone());
    let reopened = CleromancyHost::candidate(replay).unwrap();
    let views = reopened.ambient_views().unwrap();
    assert_eq!(views.len(), 2);
    assert!(views.contains(&view) && views.contains(&other_view));
    assert_eq!(reopened.ambient_memories().unwrap().len(), 2);
    assert!(reopened.ambient_memories().unwrap()[0].kept);
    assert_eq!(
        Consultation::new(reopened)
            .detail(&detail.session.id)
            .unwrap(),
        detail
    );
}

#[test]
fn malformed_or_unattributed_ambient_writes_are_refused_before_mutation() {
    let (c, detail) = support::fixture();
    let mut host = CleromancyHost::candidate(c.host().graph().clone()).unwrap();
    let before = serde_json::to_value(host.graph().to_snapshot_at(0)).unwrap();
    let mut view = host
        .ambient_context(&detail.session.id, "foundation")
        .unwrap()
        .view;
    view.roles
        .insert("undisclosed".into(), seiche::Role::Pinned);
    assert!(host.insert_ambient_view(&view).is_err());
    assert_eq!(
        serde_json::to_value(host.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    assert!(
        host.examine_ambient(&detail.session.id, "foundation", "invented", true, 1)
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(host.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    let mut view = host
        .ambient_context(&detail.session.id, "foundation")
        .unwrap()
        .view;
    view.snapshot.selected_relationship = Some("undisclosed".into());
    assert!(host.insert_ambient_view(&view).is_err());
    view.snapshot.selected_relationship = None;
    view.snapshot.recipe.definition.encoding.x = scenograph::Channel::Field("invented".into());
    assert!(host.insert_ambient_view(&view).is_err());
    assert_eq!(
        serde_json::to_value(host.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    let slot = scenograph::DynamicsSlot::from_json(
        1,
        "{\"version\":999,\"root\":{\"kind\":\"preset\",\"id\":\"unknown\"}}",
    )
    .unwrap();
    assert!(validate_motion(&slot).is_err());
}
fn sealed(graph: &Graph) -> serde_json::Value {
    let mut records = std::collections::BTreeMap::new();
    for (id, facets) in graph.facets().iter() {
        for (name, value) in facets.iter() {
            if name.as_str().starts_with("cleromancy.")
                && !name.as_str().starts_with("cleromancy.ambient-")
            {
                records.insert(format!("{id}/{}", name.as_str()), value.clone());
            }
        }
    }
    serde_json::to_value(records).unwrap()
}
