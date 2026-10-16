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
use mere::kernel::geometry::PortablePoint;
#[cfg(feature = "legacy-receipts")]
use mere::kernel::graph::NodeFacetStore;
use mere::kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use mere::kernel::graph::{Graph, NodeKey, RelationKind, SemanticSubKind};
#[cfg(feature = "legacy-receipts")]
use mere::kernel::persistence::GraphSnapshot;
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
#[cfg(not(feature = "portable-core"))]
mod projection;
mod records;
mod replay;
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
    projection_channels: std::sync::Mutex<mere::canvas::ChannelRegistry>,
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
            projection_channels: std::sync::Mutex::new(mere::canvas::ChannelRegistry::new()),
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

impl<B: Backend> CleromancyHost<B> {
    pub fn empty(_backend: B) -> Self {
        Self {
            _backend: std::marker::PhantomData,
            #[cfg(feature = "legacy-receipts")]
            slots: Some(JsonSlots::new(_backend)),
            projection_session: ProjectionSession(LOCAL_SESSION.to_string()),
            graph: Graph::new(),
            #[cfg(not(feature = "portable-core"))]
            projection_channels: std::sync::Mutex::new(mere::canvas::ChannelRegistry::new()),
            projection_epoch: 1,
            projection_revision: 1,
            resources: BTreeMap::new(),
            active_instances: HashMap::new(),
            last_snapshot: None,
            #[cfg(feature = "legacy-receipts")]
            persisted_document: None,
            dirty: true,
        }
    }

    #[cfg(feature = "legacy-receipts")]
    pub async fn open(backend: B) -> Result<Self, HostError> {
        let slots = JsonSlots::new(backend);
        let Some(saved): Option<PersistedHost> = slots.load(HOST_SLOT).await? else {
            return Ok(Self {
                _backend: std::marker::PhantomData,
                slots: Some(slots),
                projection_session: ProjectionSession(LOCAL_SESSION.to_string()),
                graph: Graph::new(),
                #[cfg(not(feature = "portable-core"))]
                projection_channels: std::sync::Mutex::new(mere::canvas::ChannelRegistry::new()),
                projection_epoch: 1,
                projection_revision: 1,
                resources: BTreeMap::new(),
                active_instances: HashMap::new(),
                last_snapshot: None,
                #[cfg(feature = "legacy-receipts")]
                persisted_document: None,
                dirty: true,
            });
        };
        let persisted_document = saved.clone();
        let mut graph = Graph::from_snapshot(&saved.graph);
        graph.overlay_facets(saved.facets);
        Ok(Self {
            _backend: std::marker::PhantomData,
            slots: Some(slots),
            projection_session: ProjectionSession(LOCAL_SESSION.to_string()),
            graph,
            #[cfg(not(feature = "portable-core"))]
            projection_channels: std::sync::Mutex::new(mere::canvas::ChannelRegistry::new()),
            projection_epoch: saved.projection_epoch,
            projection_revision: saved.projection_revision,
            resources: BTreeMap::new(),
            active_instances: HashMap::new(),
            last_snapshot: None,
            persisted_document: Some(persisted_document),
            dirty: false,
        })
    }

    /// Validate a transient candidate. Historical receipt harnesses may opt
    /// into their retired private slot; the ordinary product never compiles it.
    pub async fn persist(&mut self, _saved_at_secs: u64) -> Result<(), HostError> {
        #[cfg(feature = "legacy-receipts")]
        if let Some(slots) = &self.slots {
            let document = match &self.persisted_document {
                Some(document)
                    if !self.dirty && document.graph.timestamp_secs == _saved_at_secs =>
                {
                    document.clone()
                },
                _ => {
                    let mut graph = self.graph.to_snapshot();
                    graph.timestamp_secs = _saved_at_secs;
                    PersistedHost {
                        graph,
                        facets: self.graph.facets().clone(),
                        projection_epoch: self.projection_epoch,
                        projection_revision: self.projection_revision,
                    }
                },
            };
            slots.save(HOST_SLOT, &document).await?;
            self.persisted_document = Some(document);
            self.dirty = false;
            return Ok(());
        }
        validate_divination_graph(&self.graph)?;
        self.dirty = false;
        Ok(())
    }

    /// Capture product edits for the resident journal, using stable node ids.
    /// Installing a recorder does not persist or grant application authority.
    pub fn record_edits(
        &mut self,
        edits: std::sync::Arc<std::sync::Mutex<Vec<mere::kernel::graph::CapturedDelta>>>,
    ) {
        self.graph
            .set_recorder(Some(std::sync::Arc::new(move |delta| {
                edits
                    .lock()
                    .expect("candidate edit buffer")
                    .push(delta.clone());
            })));
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn was_reopened(&self) -> bool {
        #[cfg(feature = "legacy-receipts")]
        {
            return self.persisted_document.is_some();
        }
        #[cfg(not(feature = "legacy-receipts"))]
        {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.graph.nodes().next().is_none()
    }

    pub fn session(&self) -> ProjectionSession {
        self.projection_session.clone()
    }

    #[cfg(all(
        feature = "legacy-receipts",
        feature = "graphshell-admission",
        not(target_arch = "wasm32")
    ))]
    pub(crate) fn swap_projection_state(&mut self, state: &mut CleromancyProjectionState) {
        std::mem::swap(&mut self.projection_session, &mut state.projection_session);
        std::mem::swap(&mut self.resources, &mut state.resources);
        std::mem::swap(&mut self.active_instances, &mut state.active_instances);
        std::mem::swap(&mut self.last_snapshot, &mut state.last_snapshot);
    }

    #[cfg(all(
        feature = "legacy-receipts",
        feature = "graphshell-admission",
        not(target_arch = "wasm32")
    ))]
    pub(crate) fn current_projection_revision(&self) -> (SceneEpoch, Revision) {
        (
            SceneEpoch(self.projection_epoch),
            Revision(self.projection_revision),
        )
    }

    /// Rebind this in-memory endpoint before it serves an already-admitted
    /// Graphshell session. The durable reading graph stays local; the session
    /// name and all disclosed resources are per connection.
    #[cfg(all(
        feature = "legacy-receipts",
        feature = "graphshell-admission",
        not(target_arch = "wasm32")
    ))]
    pub(crate) fn bind_projection_session(&mut self, session: ProjectionSession) {
        self.projection_session = session;
        self.resources.clear();
        self.active_instances.clear();
        self.last_snapshot = None;
    }

    #[cfg(not(feature = "portable-core"))]
    pub fn local_request(&self) -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: self.session(),
            score: self.score(),
        }
    }

    pub(crate) fn active_revision(&self) -> Option<(SceneEpoch, Revision)> {
        self.last_snapshot.map(|_| {
            (
                SceneEpoch(self.projection_epoch),
                Revision(self.projection_revision),
            )
        })
    }

    /// The concrete revision this projection session last rendered.
    ///
    /// Unlike [`Self::active_revision`], this does not follow later durable
    /// graph changes. Resident endpoints use it to decide whether another
    /// admitted session advanced their view.
    #[cfg(all(
        feature = "legacy-receipts",
        feature = "graphshell-admission",
        not(target_arch = "wasm32")
    ))]
    pub(crate) fn last_snapshot_revision(&self) -> Option<(SceneEpoch, Revision)> {
        self.last_snapshot
    }

    pub(crate) fn context_for_instance(&self, instance: InstanceId) -> Option<ContextSnapshot> {
        let key = *self.active_instances.get(&instance)?;
        serde_json::from_value(self.facet_value(key, CONTEXT_FACET)?.clone()).ok()
    }

    pub fn facet_value(&self, key: NodeKey, facet: &str) -> Option<&Value> {
        let node = self.graph.get_node(key)?;
        self.graph.facets().get(&node.id, &FacetId::new(facet))
    }

    fn canonical_facet_values<T>(
        &self,
        facet: &'static str,
        identity: impl Fn(&T) -> String,
        address: impl Fn(&str) -> String,
    ) -> Result<Vec<T>, HostError>
    where
        T: DeserializeOwned,
    {
        let mut values = BTreeMap::new();
        for (key, node) in self.graph.nodes() {
            let Some(value) = self.facet_value(key, facet) else {
                continue;
            };
            let decoded: T = serde_json::from_value(value.clone()).map_err(|error| {
                HostError::InvalidStoredFacet {
                    facet,
                    reason: error.to_string(),
                }
            })?;
            let id = identity(&decoded);
            let expected = address(&id);
            if node.url() != expected {
                return Err(HostError::InvalidStoredFacet {
                    facet,
                    reason: format!("value identity belongs at {expected}, not {}", node.url()),
                });
            }
            if values.insert(id.clone(), decoded).is_some() {
                return Err(HostError::InvalidStoredFacet {
                    facet,
                    reason: format!("duplicate canonical identity {id}"),
                });
            }
        }
        Ok(values.into_values().collect())
    }

    fn stored_facet<T: DeserializeOwned>(
        &self,
        address: &str,
        facet: &'static str,
        kind: &'static str,
        digest: &str,
    ) -> Result<T, HostError> {
        let (key, _) = self.graph.get_node_by_url(address).ok_or_else(|| {
            HostError::MissingReadingDependency {
                kind,
                digest: digest.to_string(),
            }
        })?;
        let value =
            self.facet_value(key, facet)
                .ok_or_else(|| HostError::MissingReadingDependency {
                    kind,
                    digest: digest.to_string(),
                })?;
        serde_json::from_value(value.clone()).map_err(|error| HostError::InvalidStoredFacet {
            facet,
            reason: error.to_string(),
        })
    }

    fn session_key(&self, session: &ReadingSession) -> Result<NodeKey, HostError> {
        let (key, _) = self
            .graph
            .get_node_by_url(&format!("cleromancy://session/{}", session.id))
            .ok_or_else(|| HostError::MissingSessionDependency {
                kind: "session",
                id: session.id.clone(),
            })?;
        let stored = self.facet_value(key, SESSION_FACET).ok_or_else(|| {
            HostError::MissingSessionDependency {
                kind: "session",
                id: session.id.clone(),
            }
        })?;
        let stored: ReadingSession = serde_json::from_value(stored.clone()).map_err(|error| {
            HostError::InvalidStoredFacet {
                facet: SESSION_FACET,
                reason: error.to_string(),
            }
        })?;
        if stored != *session {
            return Err(SessionError::InvalidSession("stored value".to_string()).into());
        }
        Ok(key)
    }

    fn validate_session_bindings(
        &self,
        context: &ContextSnapshot,
        field: &Field,
        readings: &[Reading],
        session: &ReadingSession,
    ) -> Result<(), HostError> {
        session.validate()?;
        if session.context_digest != context.digest() {
            return Err(SessionError::InvalidSession("bound context digest".to_string()).into());
        }
        if session.field_digest != field.digest() {
            return Err(SessionError::InvalidSession("bound field digest".to_string()).into());
        }
        for placement in &session.placements {
            let reading = readings
                .iter()
                .find(|reading| reading.id == placement.reading_id)
                .ok_or_else(|| HostError::MissingSessionDependency {
                    kind: "reading",
                    id: placement.reading_id.clone(),
                })?;
            if reading.receipt.context_digest != session.context_digest {
                return Err(
                    SessionError::InvalidSession("reading context digest".to_string()).into(),
                );
            }
            if reading.receipt.field_digest != session.field_digest {
                return Err(
                    SessionError::InvalidSession("reading field digest".to_string()).into(),
                );
            }
            let replayed = ReadingEngine::replay(context, field, &reading.receipt)?;
            if replayed != *reading {
                return Err(ReadingError::ReceiptMismatch("sealed reading".to_string()).into());
            }
        }
        Ok(())
    }

    fn upsert_node<'a>(
        &mut self,
        address: &str,
        title: &str,
        tags: impl IntoIterator<Item = &'a str>,
    ) -> NodeKey {
        let key = self
            .graph
            .get_node_by_url(address)
            .map(|(key, _)| key)
            .unwrap_or_else(|| {
                add_node(
                    &mut self.graph,
                    Some(Graph::node_namespace_id(address)),
                    address.to_string(),
                    PortablePoint::new(0.0, 0.0),
                )
            });
        apply_graph_delta(
            &mut self.graph,
            GraphDelta::SetNodeTitle {
                key,
                title: title.to_string(),
            },
        );
        for tag in tags {
            apply_graph_delta(
                &mut self.graph,
                GraphDelta::InsertNodeTag {
                    key,
                    tag: tag.to_string(),
                },
            );
        }
        self.changed();
        key
    }

    fn set_facet(&mut self, key: NodeKey, facet: &str, value: Value) -> Result<(), HostError> {
        apply_graph_delta(
            &mut self.graph,
            GraphDelta::SetNodeFacet {
                key,
                facet: facet.to_string(),
                value,
            },
        );
        self.changed();
        Ok(())
    }

    fn changed(&mut self) {
        self.projection_revision = self.projection_revision.wrapping_add(1);
        self.resources.clear();
        self.dirty = true;
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
