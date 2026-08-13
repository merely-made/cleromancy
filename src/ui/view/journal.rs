// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The journal region: reflections, saved sessions, and receipt comparison.

use cambium::{SelectState, TextInput, button, el, map_action, map_state, select};

use super::{
    ConsultationView, labelled_control, labelled_text, mode_label, never_select_action,
    short_digest,
};
use crate::ui::state::{ConsultationAction, ConsultationUi};

pub(super) fn journal_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "h2", "Journal",
        ))];
    children.push(labelled_text(
        "Reflection",
        "cleromancy-reflection",
        true,
        &ui.reflection,
        reflection_state,
    ));
    children.push(Box::new(
        button("Add reflection", |ui: &mut ConsultationUi, _| {
            ui.request_reflection()
        })
        .attr("data-key", "save-reflection")
        .attr("aria-label", "Add reflection"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ConsultationAction>(
            "p",
            "Each follow-up is saved as a separate immutable note.",
        )
        .attr("class", "reflection-explanation"),
    ));

    if let Some(detail) = &ui.detail {
        for reflection in &detail.reflections {
            children.push(Box::new(
                el::<_, ConsultationUi, ConsultationAction>("article", reflection.body.clone())
                    .attr("data-key", format!("reflection:{}", reflection.id))
                    .attr("aria-label", "Saved reflection"),
            ));
        }
    }

    children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
        "h3",
        "Recent sessions",
    )));
    if ui.catalog.sessions.is_empty() {
        children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "p",
            "No saved sessions yet.",
        )));
    } else {
        for session in &ui.catalog.sessions {
            let id = session.id.clone();
            let label = format!("Session {}", short_digest(&id));
            children.push(Box::new(
                button(label.clone(), move |ui: &mut ConsultationUi, _| {
                    ui.request_session(id.clone())
                })
                .attr("data-key", format!("session:{}", session.id))
                .attr("aria-label", format!("Open {label}")),
            ));
        }
    }

    let comparison_sessions = ui
        .catalog
        .sessions
        .iter()
        .filter(|session| Some(&session.id) != ui.detail.as_ref().map(|detail| &detail.session.id))
        .collect::<Vec<_>>();
    let comparison_labels = std::iter::once("Choose a saved session".to_string())
        .chain(
            comparison_sessions
                .iter()
                .map(|session| format!("Session {}", short_digest(&session.id))),
        )
        .collect::<Vec<_>>();
    let comparison_refs = comparison_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let comparison_select = map_action(
        select(&ui.comparison_select, &comparison_refs),
        never_select_action,
    );
    let comparison_select = map_state(comparison_select, comparison_select_state);
    children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
        "h3",
        "Receipt comparison",
    )));
    children.push(labelled_control(
        "Compare with",
        "cleromancy-compare-with",
        Box::new(comparison_select),
    ));
    children.push(Box::new(
        button("Compare receipts", |ui: &mut ConsultationUi, _| {
            ui.request_comparison()
        })
        .attr("data-key", "compare-receipts")
        .attr("aria-label", "Compare receipts"),
    ));
    if let Some(comparison) = &ui.comparison {
        children.push(Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                format!(
                    "Context: {}; field: {}; position names: {}.",
                    same_or_different(comparison.same_context),
                    same_or_different(comparison.same_field),
                    same_or_different(comparison.same_position_names),
                ),
            )
            .attr("data-key", "receipt-comparison-summary"),
        ));
        for entry in &comparison.entries {
            children.push(Box::new(
                el::<_, ConsultationUi, ConsultationAction>(
                    "article",
                    (
                        el("h4", entry.position.clone()),
                        el(
                            "p",
                            format!(
                                "Candidate: {} / {}. Mode: {} / {}. Receipt: {}.",
                                entry.left_candidate.as_deref().unwrap_or("not present"),
                                entry.right_candidate.as_deref().unwrap_or("not present"),
                                entry.left_mode.map(mode_label).unwrap_or("not present"),
                                entry.right_mode.map(mode_label).unwrap_or("not present"),
                                entry
                                    .same_receipt
                                    .map(same_or_different)
                                    .unwrap_or("not comparable"),
                            ),
                        ),
                    ),
                )
                .attr("data-key", format!("receipt-comparison:{}", entry.position))
                .attr("aria-label", "Receipt comparison entry"),
            ));
        }
    }

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Journal")
            .attr("data-key", "region:journal"),
    )
}

fn same_or_different(value: bool) -> &'static str {
    if value { "same" } else { "different" }
}

fn comparison_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.comparison_select
}

fn reflection_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.reflection
}
