// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Controls rendered identically by more than one surface.
//!
//! The reflection editor and the session row appear on both `Today`'s trail and
//! the `Journal`. They are defined once here so the two surfaces cannot drift
//! into two different controls carrying the same `data-key`.

use cambium::{TextInput, button, button_with, el};

use super::{ConsultationView, labelled_text, short_digest};
use crate::ui::state::ConsultationUi;
use crate::{SessionSummary, format_relative_date};

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
pub(super) fn session_row(summary: &SessionSummary, now_ms: u64) -> ConsultationView {
    let id = summary.session_id.clone();
    let relative_date = format_relative_date(summary.created_at_ms, now_ms);
    let modes = summary
        .modes()
        .into_iter()
        .map(selection_mode_label)
        .collect::<Vec<_>>();
    let mode_text = modes.join(" and ");
    let placement_count = summary.placement_count();
    let placement_text = format!(
        "{placement_count} placement/card{}",
        plural(placement_count)
    );
    let card_text = summary
        .placements
        .iter()
        .map(|placement| format!("{}: {}", placement.position, placement.title))
        .collect::<Vec<_>>()
        .join(", ");
    let visible_label = format!(
        "{} · {} · {} · {}",
        short_digest(&summary.session_id),
        relative_date,
        placement_text,
        mode_text,
    );
    let aria_label = format!(
        "Open session {}. Context: {}. Date: {}. {}. Cards: {}. Modes: {}.",
        summary.session_id,
        summary.context_label,
        relative_date,
        placement_text,
        card_text,
        mode_text,
    );
    let mut row: Vec<ConsultationView> = vec![Box::new(
        el::<_, ConsultationUi, ()>("span", visible_label).attr("class", "session-summary"),
    )];
    row.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "span",
            (0..placement_count)
                .map(|_| Box::new(el::<_, ConsultationUi, ()>("span", "▫")) as ConsultationView)
                .collect::<Vec<_>>(),
        )
        .attr("class", "session-pips")
        .attr("data-key", format!("session-pips:{}", summary.session_id))
        .attr("aria-hidden", "true"),
    ));
    Box::new(
        button_with(row, move |ui: &mut ConsultationUi, _| {
            let action = ui.request_session(id.clone());
            ui.record_action(Some(action));
        })
        .attr("data-key", format!("session:{}", summary.session_id))
        .attr("aria-label", aria_label),
    )
}

fn selection_mode_label(mode: crate::SelectionMode) -> &'static str {
    match mode {
        crate::SelectionMode::Calculated => "Calculated",
        crate::SelectionMode::Cast => "Cast",
        crate::SelectionMode::Derived => "Derived",
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
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
