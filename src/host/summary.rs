// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Legible session summaries: a bounded projection of graph truth for list
//! surfaces.
//!
//! A summary answers "which saved occasion is this?" — the question a journal
//! row must answer before a person will open it. It is deliberately *not* a
//! second store: nothing here is persisted, and every value is recomputed from
//! the same facets [`crate::ConsultationDetail`] reads.
//!
//! The distinction from [`CleromancyHost::sessions`] is cost. That method
//! replays every row's sealed receipt through `validate_session_bindings`,
//! which is correct for anything that *reads* a session and proportional to
//! the whole archive for anything that merely *lists* one. [`summarize`]
//! receives neither a [`crate::Field`] nor the reading engine, so a receipt
//! replay is not reachable from this path — the guarantee is structural rather
//! than a promise in a comment.

use serde::{Deserialize, Serialize};

use super::*;
use crate::{Reading, SelectionMode};

/// One placement, reduced to what a list row shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryPlacement {
    /// The session's declared position name, in its declared order.
    pub position: String,
    /// The sealed result's title, for example a card name.
    pub title: String,
    /// The stored candidate identity behind that title.
    pub candidate_id: String,
    /// How this placement was selected.
    pub mode: SelectionMode,
}

/// A saved occasion reduced to the values a journal row needs in order to be
/// identifiable without opening it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSummary {
    pub session_id: String,
    pub created_at_ms: u64,
    /// The bound context's visible label.
    pub context_label: String,
    /// The bound context's question fact, when it carries one.
    pub question: Option<String>,
    /// The bound context's ordered tag set.
    pub tags: Vec<String>,
    pub context_digest: String,
    pub field_digest: String,
    /// Placements in the session's declared order.
    pub placements: Vec<SummaryPlacement>,
}

impl SessionSummary {
    /// How many results this occasion sealed. The journal draws one pip each.
    pub fn placement_count(&self) -> usize {
        self.placements.len()
    }

    /// The distinct selection modes used, in `Calculated`, `Cast`, `Derived`
    /// order. A single-mode occasion yields exactly one, which is what a row
    /// label shows; a mixed occasion yields several rather than picking one to
    /// present as though it governed the whole reading.
    pub fn modes(&self) -> Vec<SelectionMode> {
        let mut modes = Vec::new();
        for mode in [
            SelectionMode::Calculated,
            SelectionMode::Cast,
            SelectionMode::Derived,
        ] {
            if self
                .placements
                .iter()
                .any(|placement| placement.mode == mode)
            {
                modes.push(mode);
            }
        }
        modes
    }
}

/// Project one saved occasion into its list-row summary.
///
/// This function receives no [`crate::Field`] and no reading engine, so it
/// cannot replay a sealed receipt. That is the point: it is the reason a
/// summary pass costs a fraction of [`CleromancyHost::sessions`]. Callers that
/// need replayed truth use [`CleromancyHost::replay_session`] or
/// [`crate::Consultation::detail`] instead.
///
/// `readings` is matched to placements by reading id, so the caller may supply
/// them in any order. A placement whose reading is absent is skipped rather
/// than fabricated; [`CleromancyHost::session_summaries`] resolves every
/// placement before calling this, so an absent reading surfaces there as a
/// missing-dependency error.
pub fn summarize(
    session: &ReadingSession,
    context: &ContextSnapshot,
    readings: &[Reading],
) -> SessionSummary {
    let placements = session
        .placements
        .iter()
        .filter_map(|placement| {
            let reading = readings
                .iter()
                .find(|reading| reading.id == placement.reading_id)?;
            Some(SummaryPlacement {
                position: placement.position.clone(),
                title: reading.title.clone(),
                candidate_id: reading.candidate_id.clone(),
                mode: reading.receipt.mode,
            })
        })
        .collect();
    SessionSummary {
        session_id: session.id.clone(),
        created_at_ms: session.created_at_ms,
        context_label: context.label.clone(),
        question: context.facts.get("question").cloned(),
        tags: context.tags.iter().cloned().collect(),
        context_digest: session.context_digest.clone(),
        field_digest: session.field_digest.clone(),
        placements,
    }
}

impl<B: Backend> CleromancyHost<B> {
    /// Every saved occasion as a legible summary, newest first with the
    /// session id as the final tie-break — the same ordering as
    /// [`CleromancyHost::sessions`].
    ///
    /// Each row is structurally validated and its dependencies are resolved,
    /// so a corrupt or dangling record is an error rather than a row that
    /// silently disappears from the journal. What this path does *not* do is
    /// replay sealed receipts, which is what makes it usable for a list that
    /// grows without bound. Use [`crate::Consultation::detail`] when replayed
    /// truth is required.
    pub fn session_summaries(&self) -> Result<Vec<SessionSummary>, HostError> {
        let sessions = self.canonical_facet_values(
            SESSION_FACET,
            |session: &ReadingSession| session.id.clone(),
            |id| format!("cleromancy://session/{id}"),
        )?;
        let mut summaries = Vec::with_capacity(sessions.len());
        for session in &sessions {
            session.validate()?;
            let context = self.context_for_digest(&session.context_digest)?;
            let mut readings = Vec::with_capacity(session.placements.len());
            for placement in &session.placements {
                readings.push(self.stored_facet::<Reading>(
                    &format!("cleromancy://reading/{}", placement.reading_id),
                    READING_FACET,
                    "reading",
                    &placement.reading_id,
                )?);
            }
            summaries.push(summarize(session, &context, &readings));
        }
        summaries.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        Ok(summaries)
    }
}
