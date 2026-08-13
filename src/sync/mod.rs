// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Selected personal-sync mapping between the local graph and Graphshell H7.
//!
//! `collect`/`export` project local truth into H7 events; `decode`/`import`
//! validate a materialized projection before local mutation. `shared` holds
//! the node wrapper and identity helpers both directions use.

use std::fmt;
use std::str::FromStr;

use graphshell::personal_sync::{PersonalGraphEvent, SyncSelection as PersonalSyncSelection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::HostError;
use crate::host::{
    ASTROLOGY_CHART_FACET, ASTROLOGY_FACTS_FACET, CONCURRENCE_FACET, CONTEXT_FACET, FIELD_FACET,
    READING_FACET, REFLECTION_FACET, SESSION_FACET, SPREAD_FACET, SPREAD_TEMPLATE_FACET,
    THREE_CARD_SPREAD_FACET,
};

mod collect;
mod decode;
mod export;
mod import;
mod shared;

pub use export::export_sync_batch;
pub use import::import_sync_projection;

pub const SYNC_BATCH_SCHEMA: &str = "cleromancy.sync-batch/v6";

/// The explicit local setting controlling which Cleromancy truth may enter
/// Graphshell's personal graph. Reading sync includes its contexts and exact
/// candidate fields because a receipt without either dependency cannot be
/// independently replayed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleromancySyncSelection {
    #[default]
    Off,
    Contexts,
    /// Context, field, sealed result, saved occasions, astrology facts, and
    /// pattern occasions. It does not export reflective notes.
    ContextsAndReadings,
    /// The full reading history, including separately attached reflections.
    ContextsReadingsAndReflections,
}

/// The supplied local-consent name did not identify a supported selection.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("unknown Cleromancy sync selection {value:?}")]
pub struct CleromancySyncSelectionParseError {
    value: String,
}

impl CleromancySyncSelection {
    /// Stable, human-entered name for the local consent command.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Contexts => "contexts",
            Self::ContextsAndReadings => "contexts-and-readings",
            Self::ContextsReadingsAndReflections => "contexts-readings-and-reflections",
        }
    }

    pub fn includes_contexts(self) -> bool {
        !matches!(self, Self::Off)
    }

    pub fn includes_readings(self) -> bool {
        matches!(
            self,
            Self::ContextsAndReadings | Self::ContextsReadingsAndReflections
        )
    }

    pub fn includes_sessions(self) -> bool {
        self.includes_readings()
    }

    pub fn includes_astrology(self) -> bool {
        self.includes_readings()
    }

    pub fn includes_concurrences(self) -> bool {
        self.includes_readings()
    }

    pub fn includes_reflections(self) -> bool {
        matches!(self, Self::ContextsReadingsAndReflections)
    }

    /// Configure H7 to materialize only the named facets Cleromancy selected.
    pub fn personal_graph_selection(self) -> PersonalSyncSelection {
        let mut facets = Vec::new();
        if self.includes_contexts() {
            facets.push(CONTEXT_FACET);
        }
        if self.includes_readings() {
            facets.push(FIELD_FACET);
            facets.push(READING_FACET);
            facets.push(SESSION_FACET);
            facets.push(THREE_CARD_SPREAD_FACET);
            facets.push(SPREAD_TEMPLATE_FACET);
            facets.push(SPREAD_FACET);
            facets.push(ASTROLOGY_CHART_FACET);
            facets.push(ASTROLOGY_FACTS_FACET);
            facets.push(CONCURRENCE_FACET);
        }
        if self.includes_reflections() {
            facets.push(REFLECTION_FACET);
        }
        PersonalSyncSelection::default().with_facets(facets)
    }
}

impl fmt::Display for CleromancySyncSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CleromancySyncSelection {
    type Err = CleromancySyncSelectionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.replace('_', "-").as_str() {
            "off" => Ok(Self::Off),
            "contexts" => Ok(Self::Contexts),
            "contexts-and-readings" => Ok(Self::ContextsAndReadings),
            "contexts-readings-and-reflections" => Ok(Self::ContextsReadingsAndReflections),
            _ => Err(CleromancySyncSelectionParseError {
                value: value.to_string(),
            }),
        }
    }
}

/// A deterministic, bounded set of H7 events ready for an admitted personal
/// replica or resident `PersonalSyncHost` to author.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CleromancySyncBatch {
    pub schema: &'static str,
    pub selection: CleromancySyncSelection,
    pub events: Vec<PersonalGraphEvent>,
    pub contexts: usize,
    pub fields: usize,
    pub readings: usize,
    pub sessions: usize,
    pub spreads: usize,
    pub spread_templates: usize,
    pub charts: usize,
    pub facts: usize,
    pub concurrences: usize,
    pub reflections: usize,
    pub digest: String,
}

impl CleromancySyncBatch {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn into_events(self) -> Vec<PersonalGraphEvent> {
        self.events
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CleromancySyncImport {
    pub contexts: usize,
    pub fields: usize,
    pub readings: usize,
    pub sessions: usize,
    pub spreads: usize,
    pub spread_templates: usize,
    pub charts: usize,
    pub facts: usize,
    pub concurrences: usize,
    pub reflections: usize,
}

#[derive(Debug, Error)]
pub enum CleromancySyncError {
    #[error("Cleromancy sync node {node} is invalid: {reason}")]
    InvalidNode { node: Uuid, reason: String },
    #[error("synced reading {reading} has no selected context {context_digest}")]
    MissingContext {
        reading: String,
        context_digest: String,
    },
    #[error("synced reading {reading} has no selected field {field_digest}")]
    MissingField {
        reading: String,
        field_digest: String,
    },
    #[error("synced session {session} has no selected context {context_digest}")]
    MissingSessionContext {
        session: String,
        context_digest: String,
    },
    #[error("synced session {session} has no selected field {field_digest}")]
    MissingSessionField {
        session: String,
        field_digest: String,
    },
    #[error("synced session {session} has no selected reading {reading}")]
    MissingSessionReading { session: String, reading: String },
    #[error("synced spread {spread} has no selected session {session}")]
    MissingSpreadSession { spread: String, session: String },
    #[error("synced spread {spread} has no selected reading {reading}")]
    MissingSpreadReading { spread: String, reading: String },
    #[error("synced spread {spread} has no selected template {template}")]
    MissingSpreadTemplate { spread: String, template: String },
    #[error("synced reflection {reflection} has no selected session {session}")]
    MissingReflectionSession { reflection: String, session: String },
    #[error("synced astrology facts {facts} has no selected chart {chart_digest}")]
    MissingAstrologyChart { facts: String, chart_digest: String },
    #[error("synced astrology chart {chart_digest} has no selected facts")]
    MissingAstrologyFacts { chart_digest: String },
    #[error("synced concurrence {concurrence} has no selected member {address}")]
    MissingConcurrenceMember {
        concurrence: String,
        address: String,
    },
    #[error("personal graph projection has {0} operations waiting for causal history")]
    PendingHistory(usize),
    #[error("personal graph projection has an unresolved Cleromancy conflict at {0}")]
    Conflict(String),
    #[error(transparent)]
    Host(#[from] HostError),
}
