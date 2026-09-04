// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The journal region: reflections, saved sessions, and receipt comparison.

use cambium::{SelectState, button, el, map_action, map_state, select};

use super::shared::{reflection_editor, saved_reflections, session_row};
use super::{ConsultationView, labelled_control, mode_label, never_select_action, short_digest};
use crate::ui::state::ConsultationUi;

pub(super) fn journal_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ()>("h2", "Journal"))];
    reflection_editor(ui, &mut children);
    saved_reflections(ui, &mut children);

    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h3",
        "Recent sessions",
    )));
    if ui.catalog.sessions.is_empty() {
        children.push(Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "No saved sessions yet.",
        )));
    } else {
        for session in &ui.catalog.sessions {
            children.push(session_row(&session.id));
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
    children.push(Box::new(el::<_, ConsultationUi, ()>(
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
            let action = ui.request_comparison();
            ui.record_action(action);
        })
        .attr("data-key", "compare-receipts")
        .attr("aria-label", "Compare receipts"),
    ));
    if let Some(comparison) = &ui.comparison {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>(
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
                el::<_, ConsultationUi, ()>(
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
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Journal")
            .attr("data-key", "region:journal")
            .attr("id", "cleromancy-region-journal"),
    )
}

fn same_or_different(value: bool) -> &'static str {
    if value { "same" } else { "different" }
}

fn comparison_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.comparison_select
}
