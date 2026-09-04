// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Controls rendered identically by more than one surface.
//!
//! The reflection editor and the session row appear on both `Today`'s trail and
//! the `Journal`. They are defined once here so the two surfaces cannot drift
//! into two different controls carrying the same `data-key`.

use cambium::{TextInput, button, el};

use super::{ConsultationView, labelled_text, short_digest};
use crate::ui::state::ConsultationUi;

/// The append-only reflection editor: field, control, and its explanation.
pub(super) fn reflection_editor(ui: &ConsultationUi, children: &mut Vec<ConsultationView>) {
    children.push(labelled_text(
        "Reflection",
        "cleromancy-reflection",
        true,
        &ui.reflection,
        reflection_state,
    ));
    children.push(Box::new(
        button("Add reflection", |ui: &mut ConsultationUi, _| {
            let action = ui.request_reflection();
            ui.record_action(action);
        })
        .attr("data-key", "save-reflection")
        .attr("aria-label", "Add reflection"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("p", "Each follow-up is saved as a separate immutable note.")
            .attr("class", "reflection-explanation"),
    ));
}

/// The current detail's saved reflections, newest first as the catalog orders
/// them.
pub(super) fn saved_reflections(ui: &ConsultationUi, children: &mut Vec<ConsultationView>) {
    let Some(detail) = &ui.detail else { return };
    for reflection in &detail.reflections {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("article", reflection.body.clone())
                .attr("data-key", format!("reflection:{}", reflection.id))
                .attr("aria-label", "Saved reflection"),
        ));
    }
}

/// One saved occasion as an openable row.
pub(super) fn session_row(session_id: &str) -> ConsultationView {
    let id = session_id.to_string();
    let label = format!("Session {}", short_digest(session_id));
    Box::new(
        button(label.clone(), move |ui: &mut ConsultationUi, _| {
            let action = ui.request_session(id.clone());
            ui.record_action(Some(action));
        })
        .attr("data-key", format!("session:{session_id}"))
        .attr("aria-label", format!("Open {label}")),
    )
}

/// A surface that is announced but not yet built.
pub(super) fn placeholder_region(
    label: &'static str,
    key: &'static str,
    body: &'static str,
) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ()>(
            "section",
            vec![
                Box::new(el::<_, ConsultationUi, ()>("h2", label)) as ConsultationView,
                Box::new(el::<_, ConsultationUi, ()>("p", body)),
            ],
        )
        .attr("role", "region")
        .attr("aria-label", label)
        .attr("data-key", format!("region:{key}"))
        .attr("id", format!("cleromancy-region-{key}")),
    )
}

pub(super) fn reflection_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.reflection
}
