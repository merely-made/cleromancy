// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod dom;
#[path = "support/ambient_fixture.rs"]
mod fixture;
use cleromancy::ConsultationAction;
use dom::{click_key, harness, rect_at_key, text_at_key};
#[test]
fn accessible_neighborhood_separates_focus_examination_keeping_and_curation() {
    let (mut c, detail) = fixture::fixture();
    let context = c.ambient_context(&detail.session.id, "foundation").unwrap();
    let history = context
        .data
        .items
        .iter()
        .find(|i| i.id.starts_with("history:"))
        .unwrap()
        .id
        .clone();
    let mut h = harness(c.catalog().unwrap());
    let catalog = c.catalog().unwrap();
    let saved = detail.clone();
    h.update(move |ui| ui.present_neighborhood(catalog, detail, context));
    assert!(click_key(&mut h, &format!("ambient-item:{history}")).is_none());
    assert!(text_at_key(&h, "ambient-memory-level").contains("Ambient"));
    assert!(click_key(&mut h, "ambient-foreground").is_none());
    assert!(click_key(&mut h, "ambient-role:pinned").is_none());
    assert!(c.catalog().unwrap().ambient_memory.is_empty());
    let action = click_key(&mut h, "ambient-examine").unwrap();
    let ConsultationAction::ExamineNeighbor {
        session_id,
        position,
        item_id,
        keep,
    } = action
    else {
        panic!("wrong action")
    };
    assert!(!keep);
    pollster::block_on(c.remember_ambient(&session_id, &position, &item_id, keep)).unwrap();
    let context = c.ambient_context(&session_id, &position).unwrap();
    let detail = c.detail(&session_id).unwrap();
    let catalog = c.catalog().unwrap();
    h.update(move |ui| ui.present_neighborhood(catalog, detail, context));
    assert!(text_at_key(&h, "ambient-memory-level").contains("Recent"));
    assert!(!text_at_key(&h, "ambient-inspection").is_empty());
    let action = click_key(&mut h, "ambient-keep").unwrap();
    let ConsultationAction::ExamineNeighbor { keep, .. } = action else {
        panic!("wrong action")
    };
    assert!(keep);
    pollster::block_on(c.remember_ambient(&session_id, &position, &item_id, true)).unwrap();
    assert!(c.catalog().unwrap().ambient_memory[0].kept);
    let view = h.state().neighborhood_view().unwrap();
    assert_eq!(view.roles[&item_id], seiche::Role::Pinned);
    assert!(view.foreground.contains(&item_id));
    assert_eq!(c.detail(&saved.session.id).unwrap(), saved);
    h.layout_at(420.0, 16000.0);
    for key in [
        "save-neighborhood",
        "ambient-examine",
        "ambient-keep",
        "ambient-role:pinned",
    ] {
        let (x, _, w, _) = rect_at_key(&h, key);
        assert!(
            x >= 0.0 && x + w <= 420.5,
            "{key} outside compact bounds: {x} {w}"
        );
    }
}
