// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The journal region: bounded archive reader, selected facts, and comparison.

use cambium::{SelectState, TextInput, button, el, map_action, map_state, select};

use super::shared::{reflection_editor, saved_reflections, session_row};
use super::{
    ConsultationView, labelled_control, labelled_text, mode_label, never_select_action,
    short_digest,
};
use crate::ui::state::ConsultationUi;
use crate::{JournalDateRange, journal_page};

pub(super) fn journal_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ()>("h2", "Journal"))];
    journal_filters(ui, &mut children);

    let page = journal_page(
        &ui.catalog.session_summaries,
        ui.journal.tag_filter.text(),
        ui.journal.selected_date_range(),
        ui.journal.now_ms,
        ui.journal.page_index,
    );
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            if page.total == 0 {
                format!(
                    "Showing 0 matching sessions. Page 1 of 1. Each page shows at most {} sessions.",
                    crate::JOURNAL_PAGE_SIZE,
                )
            } else {
                format!(
                    "Showing {}–{} of {} matching sessions. Page {} of {}. Each page shows at most {} sessions.",
                    page.start(), page.end(), page.total, page.page_index + 1, page.page_count,
                    crate::JOURNAL_PAGE_SIZE,
                )
            },
        )
        .attr("data-key", "journal-bound")
        .attr("class", "journal-bound"),
    ));
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h3",
        "Saved sessions",
    )));
    if page.total == 0 {
        let message = if ui.catalog.session_summaries.is_empty() {
            "No saved sessions yet."
        } else {
            "No saved sessions match these filters."
        };
        children.push(Box::new(el::<_, ConsultationUi, ()>("p", message)));
    } else {
        for summary in &page.summaries {
            children.push(session_row(summary, ui.journal.now_ms));
        }
    }
    paging_controls(&page, &mut children);
    selected_detail(ui, &mut children);
    comparison(ui, &mut children);

    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Journal")
            .attr("data-key", "region:journal")
            .attr("id", "cleromancy-region-journal"),
    )
}

fn journal_filters(ui: &ConsultationUi, children: &mut Vec<ConsultationView>) {
    children.push(Box::new(el::<_, ConsultationUi, ()>("h3", "Find sessions")));
    children.push(labelled_text(
        "Tag filter",
        "cleromancy-journal-tag-filter",
        false,
        &ui.journal.tag_filter,
        journal_tag_filter_state,
    ));
    let labels = [
        JournalDateRange::AnyTime.label(),
        JournalDateRange::Past7Days.label(),
        JournalDateRange::Past30Days.label(),
    ];
    let date_range = map_action(select(&ui.journal.date_range, &labels), never_select_action);
    children.push(labelled_control(
        "Date range",
        "cleromancy-journal-date-range",
        Box::new(map_state(date_range, journal_date_range_state)),
    ));
    children.push(Box::new(
        button("Clear filters", |ui: &mut ConsultationUi, _| {
            ui.journal.clear()
        })
        .attr("data-key", "clear-journal-filters")
        .attr("aria-label", "Clear journal filters"),
    ));
}

fn paging_controls(page: &crate::JournalPage<'_>, children: &mut Vec<ConsultationView>) {
    let previous = page.page_index;
    let next = page.page_index;
    let page_count = page.page_count;
    children.push(Box::new(
        button("Newer sessions", move |ui: &mut ConsultationUi, _| {
            ui.journal.newer(previous);
        })
        .attr("data-key", "journal-newer")
        .attr("aria-label", "Newer sessions")
        .attr(
            "aria-disabled",
            if page.page_index == 0 {
                "true"
            } else {
                "false"
            },
        ),
    ));
    children.push(Box::new(
        button("Older sessions", move |ui: &mut ConsultationUi, _| {
            ui.journal.older(next, page_count);
        })
        .attr("data-key", "journal-older")
        .attr("aria-label", "Older sessions")
        .attr(
            "aria-disabled",
            if page.page_index + 1 >= page.page_count {
                "true"
            } else {
                "false"
            },
        ),
    ));
}

fn selected_detail(ui: &ConsultationUi, children: &mut Vec<ConsultationView>) {
    let Some(detail) = &ui.detail else { return };
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h3",
        "Selected session",
    )));
    let mut detail_children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h4", "Context")),
        Box::new(
            el::<_, ConsultationUi, ()>("p", format!("Label: {}", detail.context.label))
                .attr("data-key", "journal-context-label"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Question: {}",
                    detail
                        .context
                        .facts
                        .get("question")
                        .map(String::as_str)
                        .unwrap_or("not recorded"),
                ),
            )
            .attr("data-key", "journal-context-question"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Tags: {}",
                    if detail.context.tags.is_empty() {
                        "none".to_string()
                    } else {
                        detail
                            .context
                            .tags
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                ),
            )
            .attr("data-key", "journal-context-tags"),
        ),
    ];
    for (name, value) in detail
        .context
        .facts
        .iter()
        .filter(|(name, _)| name.as_str() != "question")
    {
        detail_children.push(Box::new(
            el::<_, ConsultationUi, ()>("p", format!("{name}: {value}"))
                .attr("data-key", format!("journal-context-fact:{name}")),
        ));
    }
    detail_children.push(Box::new(el::<_, ConsultationUi, ()>("h4", "Readings")));
    for (placement, reading) in detail.session.placements.iter().zip(&detail.readings) {
        detail_children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "article",
                (
                    el("h5", format!("{}: {}", placement.position, reading.title)),
                    el("p", reading.interpretation.clone()),
                    el("p", format!("Mode: {}", mode_label(reading.receipt.mode))),
                ),
            )
            .attr(
                "data-key",
                format!("journal-reading:{}", placement.position),
            ),
        ));
    }
    detail_children.push(Box::new(el::<_, ConsultationUi, ()>("h4", "Reflections")));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("article", detail_children)
            .attr("data-key", "journal-selected-detail")
            .attr("aria-label", "Selected session detail"),
    ));
    reflection_editor(ui, children);
    saved_reflections(ui, children);
}

fn comparison(ui: &ConsultationUi, children: &mut Vec<ConsultationView>) {
    let comparison_sessions = ui
        .catalog
        .session_summaries
        .iter()
        .filter(|summary| {
            Some(&summary.session_id) != ui.detail.as_ref().map(|detail| &detail.session.id)
        })
        .collect::<Vec<_>>();
    let labels = std::iter::once("Choose a saved session".to_string())
        .chain(comparison_sessions.iter().map(|summary| {
            format!(
                "{} ({})",
                summary.context_label,
                short_digest(&summary.session_id)
            )
        }))
        .collect::<Vec<_>>();
    let refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let comparison_select = map_action(select(&ui.comparison_select, &refs), never_select_action);
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h3",
        "Receipt comparison",
    )));
    children.push(labelled_control(
        "Compare with",
        "cleromancy-compare-with",
        Box::new(map_state(comparison_select, comparison_select_state)),
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
}

fn journal_tag_filter_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.journal.tag_filter
}

fn journal_date_range_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.journal.date_range
}

fn comparison_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.comparison_select
}

fn same_or_different(value: bool) -> &'static str {
    if value { "same" } else { "different" }
}
