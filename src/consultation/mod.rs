// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Product transformations over a complete divination graph candidate.
//!
//! The ordinary window reads the candidate from the resident and records the
//! resulting stable-id edits through its admitted journal transaction. This
//! controller owns divination meaning and validation, never the resident store.

use std::time::{SystemTime, UNIX_EPOCH};

use muniment::Backend;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::moirai::clotho::{EntropySource, OsEntropy};
use crate::{
    AstrologyChart, AstrologyError, AstrologyFacts, CleromancyHost, Concurrence, ContextSnapshot,
    DerivedSelection, Field, HostError, Reading, ReadingEngine, ReadingError, ReadingSession,
    Reflection, SelectionMode, SessionSummary, SkyDayFacts, SpreadTemplate, TarotPack,
    TarotQualification,
};

mod ambient;
mod compare;
mod drafts;
mod journal_policy;

use compare::compare_details;
pub use drafts::{
    AstrologyCalculationDraft, AstrologyChartDraft, ContextDraft, MANUAL_CONTEXT_SCHEMA,
    SpreadTemplateDraft,
};
pub use journal_policy::{
    JOURNAL_PAGE_SIZE, JournalDateRange, JournalPage, filter_journal_summaries,
    format_relative_date, journal_page,
};

/// Stable picker/history values derived from graph truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultationCatalog {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub natal_readings: Vec<crate::natal::NatalReading>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub natal_reflections: Vec<crate::Reflection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ambient_memory: Vec<crate::ambient::MemorySummary>,
    pub contexts: Vec<ContextSnapshot>,
    pub fields: Vec<Field>,
    pub spread_templates: Vec<SpreadTemplate>,
    pub astrology_facts: Vec<AstrologyFacts>,
    /// A replay-verified facts receipt paired with the exact stored chart it
    /// describes. The legacy facts picker remains separate for authoring.
    pub astrology_charts: Vec<StoredAstrologyChart>,
    /// Stored sky timelines. Storage/query stay available without compiling
    /// the optional numerical adapter; only the Sky surface is feature-gated.
    pub sky_day_facts: Vec<SkyDayFacts>,
    /// Saved occasions projected for list surfaces, newest first. Anything
    /// needing replayed truth crosses the explicit [`Consultation::detail`]
    /// boundary instead of keeping every full session in a picker catalog.
    pub session_summaries: Vec<SessionSummary>,
}

/// A chart surface row. `facts` is replay-verified before this projection is
/// made, while `chart` is the original stored source receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredAstrologyChart {
    pub facts: AstrologyFacts,
    pub chart: AstrologyChart,
}

/// One saved occasion with every value required by the first reading and
/// journal surfaces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultationDetail {
    pub session: ReadingSession,
    pub context: ContextSnapshot,
    pub field: Field,
    pub readings: Vec<Reading>,
    pub reflections: Vec<Reflection>,
    pub concurrences: Vec<Concurrence>,
    /// The authored layout bound to this session, when it was read through
    /// one. Its labels and relations are the positions' meaning; a built-in
    /// position name never overrides them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread_template: Option<SpreadTemplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub situation: Option<crate::situation::ReadingSituation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferences: Vec<crate::inference::GeneratedInference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kept_inferences: Vec<String>,
}

/// A replay-derived comparison of two sealed session receipts. It never writes
/// graph truth and is intentionally silent about a card's interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptComparison {
    pub left_session_id: String,
    pub right_session_id: String,
    pub same_context: bool,
    pub same_field: bool,
    pub same_position_names: bool,
    pub entries: Vec<ReceiptComparisonEntry>,
}

/// Comparison values for one explicitly named placement in a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptComparisonEntry {
    pub position: String,
    pub left_candidate: Option<String>,
    pub right_candidate: Option<String>,
    pub same_candidate: Option<bool>,
    pub left_mode: Option<SelectionMode>,
    pub right_mode: Option<SelectionMode>,
    pub same_mode: Option<bool>,
    pub left_algorithm: Option<String>,
    pub right_algorithm: Option<String>,
    pub same_receipt: Option<bool>,
}

#[derive(Debug, Error)]
pub enum ConsultationError {
    #[error(transparent)]
    Situation(#[from] crate::situation::SituationError),
    #[error(transparent)]
    Inference(#[from] crate::inference::InferenceError),
    #[error(transparent)]
    Astrology(#[from] AstrologyError),
    #[error(transparent)]
    Host(#[from] HostError),
    #[error(transparent)]
    Reading(#[from] ReadingError),
    #[error("manual context is invalid: {0}")]
    InvalidContext(&'static str),
    #[error("authored spread is invalid: {0}")]
    InvalidSpread(&'static str),
    #[error("astrology chart import is invalid: {0}")]
    InvalidAstrology(&'static str),
    #[error("ephemeris operation failed: {0}")]
    Ephemeris(String),
    #[error("derived selection requires a disclosed seed and domain")]
    DerivedSelectionRequired,
    #[error("a seed and domain are only valid for derived selection")]
    UnexpectedDerivation,
    #[error("the local consultation authority must be reopened after a storage failure")]
    Faulted,
    #[error("the system clock precedes the Unix epoch")]
    Clock,
}

/// Product transactions over one Cleromancy graph authority.
pub struct Consultation<B> {
    host: CleromancyHost<B>,
    faulted: bool,
}

fn unix_time_ms() -> Result<u64, ConsultationError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConsultationError::Clock)?
        .as_millis();
    u64::try_from(millis).map_err(|_| ConsultationError::Clock)
}

fn unix_time_secs() -> Result<u64, ConsultationError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConsultationError::Clock)?
        .as_secs())
}

mod operations;
