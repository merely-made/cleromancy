// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's divination graph transformations, replay and projection.
//!
//! The submodules split one `CleromancyHost` impl by concern: typed catalog
//! queries, graph-truth replay, record insertion, spread insertion, snapshot
//! projection, and portable-card rendering.

use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

use chartulary::{FacetError, FacetId};
use chirograph::{ContentHash, ProjectionSession};
#[cfg(not(feature = "portable-core"))]
use chirograph::{ProjectionRequest, ProtocolVersion};
use kernel::geometry::PortablePoint;
#[cfg(feature = "legacy-receipts")]
use kernel::graph::NodeFacetStore;
use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use kernel::graph::{Graph, NodeKey, RelationKind, SemanticSubKind};
#[cfg(feature = "legacy-receipts")]
use kernel::persistence::GraphSnapshot;
#[cfg(feature = "legacy-receipts")]
use muniment::JsonSlots;
use muniment::{Backend, StoreError};
use sceno::InstanceId;
use scenotime::{Revision, SceneEpoch};
use serde::de::DeserializeOwned;
#[cfg(feature = "legacy-receipts")]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::moirai::clotho::EntropySource;
use crate::{
    AstrologyError, ConcurrenceError, ContextSnapshot, Field, Reading, ReadingEngine, ReadingError,
    ReadingSession, SessionError, SpreadError, SpreadRelationKind,
};

mod ambient;
mod cards;
mod catalog;
mod domain;
mod inference;
mod natal;
#[cfg(not(feature = "portable-core"))]
mod projection;
mod records;
mod replay;
mod situation;
mod sky;
mod spreads;
mod summary;

pub use ambient::{AMBIENT_MEMORY_FACET, AMBIENT_VIEW_FACET};
pub use domain::validate_divination_graph;

pub use summary::{SessionSummary, SummaryPlacement, summarize};

#[cfg(feature = "legacy-receipts")]
pub const HOST_SLOT: &str = "cleromancy/mere-host/v1";
pub const LOCAL_SESSION: &str = "local:cleromancy";
pub const CONTEXT_FACET: &str = "cleromancy.context/v1";
pub const FIELD_FACET: &str = "cleromancy.field/v1";
pub const READING_FACET: &str = "cleromancy.reading/v1";
pub const SESSION_FACET: &str = "cleromancy.reading-session/v1";
pub const REFLECTION_FACET: &str = "cleromancy.reflection/v1";
pub const THREE_CARD_SPREAD_FACET: &str = "cleromancy.three-card-spread/v1";
pub const SPREAD_TEMPLATE_FACET: &str = "cleromancy.spread-template/v1";
pub const SPREAD_FACET: &str = "cleromancy.spread/v1";
pub const ASTROLOGY_CHART_FACET: &str = "cleromancy.astrology-chart/v1";
pub const ASTROLOGY_FACTS_FACET: &str = "cleromancy.astrology-facts/v1";
pub const CONCURRENCE_FACET: &str = "cleromancy.concurrence/v1";
pub const SKY_DAY_FACTS_FACET: &str = "cleromancy.sky-day-facts/v1";
pub const SKY_INTERPRETATION_FACET: &str = "cleromancy.sky-interpretation/v1";

/// State that belongs to one projection connection rather than Cleromancy's
/// saved reading graph.
///
/// A resident authority keeps one durable [`CleromancyHost`] and gives every
/// admitted endpoint one of these states. Resources and scene instances must
/// not leak from one peer into another peer's session.
#[cfg(all(
    feature = "legacy-receipts",
    feature = "graphshell-admission",
    not(target_arch = "wasm32")
))]
pub(crate) struct CleromancyProjectionState {
    projection_session: ProjectionSession,
    resources: BTreeMap<ContentHash, Vec<u8>>,
    active_instances: HashMap<InstanceId, NodeKey>,
    last_snapshot: Option<(SceneEpoch, Revision)>,
}

#[cfg(all(
    feature = "legacy-receipts",
    feature = "graphshell-admission",
    not(target_arch = "wasm32")
))]
impl CleromancyProjectionState {
    pub(crate) fn for_session(projection_session: ProjectionSession) -> Self {
        Self {
            projection_session,
            resources: BTreeMap::new(),
            active_instances: HashMap::new(),
            last_snapshot: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum HostError {
    #[error("Cleromancy storage: {0}")]
    Store(#[from] StoreError),
    #[error("Cleromancy facet: {0}")]
    Facet(#[from] FacetError),
    #[error("Cleromancy projection: {0}")]
    InvalidSnapshot(String),
    #[error("request names another projection session or protocol major")]
    WrongSession,
    #[error("resource was not disclosed by this session")]
    MissingResource,
    #[error("reading requires stored {kind} {digest}")]
    MissingReadingDependency { kind: &'static str, digest: String },
    #[error("sky record requires stored {kind} {digest}")]
    MissingSkyDependency { kind: &'static str, digest: String },
    #[error("stored {facet} facet does not decode: {reason}")]
    InvalidStoredFacet { facet: &'static str, reason: String },
    #[error("reading session requires stored {kind} {id}")]
    MissingSessionDependency { kind: &'static str, id: String },
    #[error("concurrence requires stored member {address}")]
    MissingConcurrenceMember { address: String },
    #[error("the system clock precedes the Unix epoch")]
    Clock,
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Spread(#[from] SpreadError),
    #[error(transparent)]
    Astrology(#[from] AstrologyError),
    #[error(transparent)]
    Concurrence(#[from] ConcurrenceError),
    #[error(transparent)]
    Reading(#[from] ReadingError),
}

#[cfg(feature = "legacy-receipts")]
#[derive(Clone, Serialize, Deserialize)]
struct PersistedHost {
    graph: GraphSnapshot,
    facets: NodeFacetStore,
    projection_epoch: u64,
    projection_revision: u64,
}

/// A transient divination graph reader and transformer. The ordinary app uses
/// resident-disclosed candidates; retired private slots exist only under the
/// explicit `legacy-receipts` harness feature.
pub struct CleromancyHost<B> {
    _backend: std::marker::PhantomData<B>,
    #[cfg(feature = "legacy-receipts")]
    slots: Option<JsonSlots<B>>,
    projection_session: ProjectionSession,
    pub(crate) graph: Graph,
    // Transient layout facts belong to this graph owner. This is neither a
    // persisted record nor projection-session/custody authority.
    #[cfg(not(feature = "portable-core"))]
    projection_channels: std::sync::Mutex<pictograph::canvas::ChannelRegistry>,
    pub(crate) projection_epoch: u64,
    pub(crate) projection_revision: u64,
    pub(crate) resources: BTreeMap<ContentHash, Vec<u8>>,
    active_instances: HashMap<InstanceId, NodeKey>,
    last_snapshot: Option<(SceneEpoch, Revision)>,
    #[cfg(feature = "legacy-receipts")]
    persisted_document: Option<PersistedHost>,
    dirty: bool,
}

impl CleromancyHost<muniment::MemoryBackend> {
    /// A transient product reader/transformer over a resident's complete graph.
    /// This constructor opens no backend and owns no durable snapshot slot.
    pub fn candidate(graph: Graph) -> Result<Self, HostError> {
        validate_divination_graph(&graph)?;
        Ok(Self {
            _backend: std::marker::PhantomData,
            #[cfg(feature = "legacy-receipts")]
            slots: None,
            projection_session: ProjectionSession(LOCAL_SESSION.to_string()),
            graph,
            #[cfg(not(feature = "portable-core"))]
            projection_channels: std::sync::Mutex::new(pictograph::canvas::ChannelRegistry::new()),
            projection_epoch: 1,
            projection_revision: 1,
            resources: BTreeMap::new(),
            active_instances: HashMap::new(),
            last_snapshot: None,
            #[cfg(feature = "legacy-receipts")]
            persisted_document: None,
            dirty: false,
        })
    }
}

fn event_nonce(entropy: &mut impl EntropySource) -> Result<String, HostError> {
    let high = entropy.next_u64()?;
    let low = entropy.next_u64()?;
    Ok(format!("{high:016x}{low:016x}"))
}

fn short_identity(value: &str) -> String {
    value.chars().take(12).collect()
}

fn unix_time_ms() -> Result<u64, HostError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| HostError::Clock)?;
    u64::try_from(duration.as_millis()).map_err(|_| HostError::Clock)
}

fn semantic_kind(kind: SpreadRelationKind) -> SemanticSubKind {
    match kind {
        SpreadRelationKind::Supports => SemanticSubKind::Supports,
        SpreadRelationKind::Contradicts => SemanticSubKind::Contradicts,
        SpreadRelationKind::Questions => SemanticSubKind::Questions,
        SpreadRelationKind::NextStep => SemanticSubKind::NextStep,
        SpreadRelationKind::Elaborates => SemanticSubKind::Elaborates,
    }
}

fn relation_kind_label(kind: RelationKind) -> &'static str {
    match kind {
        // `OpenPredicate` joined the taxonomy in mere `51329461` and projects
        // to `EdgeFamily::Semantic`, so it carries the semantic label.
        RelationKind::Semantic(_) | RelationKind::OpenPredicate => "semantic",
        RelationKind::Traversal => "traversal",
        RelationKind::Containment(_) => "containment",
        RelationKind::Arrangement(_) => "arrangement",
        RelationKind::Imported(_) => "imported",
        RelationKind::Provenance(_) => "provenance",
    }
}

mod operations;
