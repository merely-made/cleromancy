// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `Today`'s bounded recent trail.
//!
//! This is the reflection editor plus the last few occasions, and deliberately
//! nothing more: the full archive — every session, its context, and receipt
//! comparison — is the `Journal` surface's job. The bound is a product policy
//! stated in one constant rather than a scroll that silently grows for years.

use cambium::el;

use super::ConsultationView;
use super::shared::{reflection_editor, saved_reflections, session_row};
use crate::ui::state::ConsultationUi;

/// How many occasions `Today` shows before deferring to the `Journal`.
pub(super) const TRAIL_LIMIT: usize = 5;

pub(super) fn trail_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ()>("h2", "Recent"))];
    reflection_editor(ui, &mut children);
    saved_reflections(ui, &mut children);

    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h3",
        "Recent sessions",
    )));
    if ui.catalog.session_summaries.is_empty() {
        children.push(Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "No saved sessions yet.",
        )));
    } else {
        // The catalog is already newest first, so taking a prefix bounds the
        // column without reordering anything.
        for summary in ui.catalog.session_summaries.iter().take(TRAIL_LIMIT) {
            children.push(session_row(summary, ui.journal.now_ms));
        }
    }
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            format!(
                "This trail shows at most {TRAIL_LIMIT} recent sessions. The full archive is on \
                 the Journal surface."
            ),
        )
        .attr("class", "trail-explanation")
        .attr("data-key", "trail-bound"),
    ));

    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Recent")
            .attr("data-key", "region:trail")
            .attr("id", "cleromancy-region-trail"),
    )
}
