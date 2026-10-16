// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use scenograph::relationship::{RelationshipSnapshot, relationship_recipe};
use scenograph::{DynamicsSlot, ProjectionInputBinding, SourceBinding};
use seiche::{
    Role,
    spec::{DynamicsSpec, Node},
};

pub const AMBIENT_VIEW_SCHEMA: &str = "cleromancy.ambient-view/v1";
pub const AMBIENT_MEMORY_SCHEMA: &str = "cleromancy.ambient-memory/v1";

/// The host exposes these two shared recipes in this bounded consumer. A
/// complete portable DynamicsSpec is stored, not a private force definition.
pub fn supported_motion(id: &str) -> Result<DynamicsSlot, String> {
    if !["spring.rapier", "flock.boids"].contains(&id) {
        return Err(format!("unsupported dynamics preset {id}"));
    }
    DynamicsSlot::from_json(
        1,
        &serde_json::to_string(&DynamicsSpec::new(Node::preset(id))).map_err(|e| e.to_string())?,
    )
}
pub fn validate_motion(slot: &DynamicsSlot) -> Result<(), String> {
    slot.validate()?;
    let _: DynamicsSpec =
        serde_json::from_str(&slot.spec).map_err(|e| format!("dynamics.spec: {e}"))?;
    if ["spring.rapier", "flock.boids"]
        .into_iter()
        .any(|id| supported_motion(id).as_ref() == Ok(slot))
    {
        Ok(())
    } else {
        Err("dynamics.spec: this consumer supports the complete bare Springs and Flock specs only; terms, channels and overrides outside that contract are refused".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientView {
    pub schema: String,
    pub session_id: String,
    pub position: String,
    pub snapshot: RelationshipSnapshot,
    pub families: BTreeSet<RelationFamily>,
    /// World coordinates in thousandths of a scene unit, bounded at admission.
    pub positions: BTreeMap<String, [i32; 2]>,
    pub roles: BTreeMap<String, Role>,
    /// Foreground emphasis is separate from roles and durable memory.
    pub foreground: BTreeSet<String>,
    pub pan_milli: [i32; 2],
    pub zoom_milli: u32,
    pub paused: bool,
    pub reduced_motion: bool,
}
impl AmbientView {
    pub fn new(data: &CardNeighborhood) -> Self {
        let source = source_binding(data);
        let mut recipe = relationship_recipe(
            "cleromancy.card-neighborhood/v1",
            "Card neighborhood",
            "Cleromancy",
            "1",
            BTreeMap::from([(
                "divination".into(),
                ProjectionInputBinding {
                    source,
                    expects_generation: Some(data.revision.clone().into()),
                    revision_evidence: Default::default(),
                },
            )]),
        );
        recipe.definition.arrangement.kind = "spiral".into();
        recipe.definition.dynamics =
            Some(supported_motion("spring.rapier").expect("supported shared preset"));
        Self {
            schema: AMBIENT_VIEW_SCHEMA.into(),
            session_id: data.session_id.clone(),
            position: data.position.clone(),
            snapshot: RelationshipSnapshot {
                recipe,
                source_name: "divination".into(),
                selected_occurrence: Some(data.subject.id.clone()),
                selected_relationship: None,
            },
            families: RelationFamily::ALL.into_iter().collect(),
            positions: BTreeMap::new(),
            roles: BTreeMap::new(),
            foreground: BTreeSet::new(),
            pan_milli: [0, 0],
            zoom_milli: 1000,
            paused: false,
            reduced_motion: false,
        }
    }
    pub fn id(&self) -> String {
        digest(&(self.session_id.as_str(), self.position.as_str()))
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != AMBIENT_VIEW_SCHEMA
            || self.session_id.len() != 64
            || self.position.trim().is_empty()
        {
            return Err("invalid ambient view identity/schema".into());
        }
        self.snapshot
            .validate()
            .map_err(|e| format!("ambient recipe: {e:?}"))?;
        if !["spiral", "grid.default", "grid.ordered"]
            .contains(&self.snapshot.recipe.definition.arrangement.kind.as_str())
        {
            return Err("unsupported neighborhood arrangement".into());
        }
        let slot = self
            .snapshot
            .recipe
            .definition
            .dynamics
            .as_ref()
            .ok_or("saved neighborhood requires an explicit dynamics slot")?;
        validate_motion(slot)?;
        if self.pan_milli.iter().any(|v| v.unsigned_abs() > 100000)
            || !(100..=8000).contains(&self.zoom_milli)
        {
            return Err("ambient camera outside supported bounds".into());
        }
        if self.positions.len() > MAX_NEIGHBORS + 1
            || self.roles.len() > MAX_NEIGHBORS + 1
            || self.foreground.len() > MAX_NEIGHBORS + 1
        {
            return Err("ambient view exceeds occurrence bounds".into());
        }
        if self
            .positions
            .values()
            .flatten()
            .any(|v| v.unsigned_abs() > 100_000_000)
        {
            return Err("ambient pose outside supported world bounds".into());
        }
        let binding = self
            .snapshot
            .recipe
            .definition
            .sources
            .get(&self.snapshot.source_name)
            .ok_or("missing divination binding")?;
        if binding.source
            != (SourceBinding {
                authority: "resident.divination".into(),
                domain: "divination".into(),
                resource: format!("reading:{}:{}", self.session_id, self.position),
            })
        {
            return Err("ambient view names another authority/resource".into());
        }
        // This consumer admits the common recipe with its supported scene and
        // dynamics choices. Extra sources or changed semantic channels require
        // another product contract, rather than silently bypassing compilation.
        let mut expected = relationship_recipe(
            "cleromancy.card-neighborhood/v1",
            "Card neighborhood",
            "Cleromancy",
            "1",
            self.snapshot.recipe.definition.sources.clone(),
        );
        expected.definition.arrangement.kind =
            self.snapshot.recipe.definition.arrangement.kind.clone();
        expected.definition.dynamics = self.snapshot.recipe.definition.dynamics.clone();
        if self.snapshot.source_name != "divination"
            || expected.definition.sources.len() != 1
            || self.snapshot.recipe != expected
        {
            return Err("unsupported neighborhood recipe/channel binding".into());
        }
        Ok(())
    }
    /// Rebinding is explicit: a saved recipe addresses this exact occasion;
    /// its current disclosure revision comes from the current admitted source.
    pub fn rebind(&mut self, data: &CardNeighborhood) {
        if let Some(binding) = self
            .snapshot
            .recipe
            .definition
            .sources
            .get_mut(&self.snapshot.source_name)
        {
            binding.expects_generation = Some(data.revision.clone().into());
        }
        let known: BTreeSet<_> = std::iter::once(&data.subject)
            .chain(data.items.iter())
            .map(|i| i.id.clone())
            .collect();
        self.positions.retain(|id, _| known.contains(id));
        self.roles.retain(|id, _| known.contains(id));
        self.foreground.retain(|id| known.contains(id));
        if self
            .snapshot
            .selected_occurrence
            .as_ref()
            .is_none_or(|id| !known.contains(id))
        {
            self.snapshot.selected_occurrence = Some(data.subject.id.clone());
        }
        if self
            .snapshot
            .selected_relationship
            .as_ref()
            .is_some_and(|id| !data.relations.iter().any(|r| &r.id == id))
        {
            self.snapshot.selected_relationship = None;
        }
    }
}
pub fn source_binding(data: &CardNeighborhood) -> SourceBinding {
    SourceBinding {
        authority: "resident.divination".into(),
        domain: "divination".into(),
        resource: format!("reading:{}:{}", data.session_id, data.position),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientMemory {
    pub schema: String,
    pub id: String,
    pub session_id: String,
    pub position: String,
    pub item: AmbientItem,
    pub reasons: Vec<AmbientRelation>,
    pub examined_at_ms: u64,
}
impl AmbientMemory {
    pub fn new(
        data: &CardNeighborhood,
        item_id: &str,
        examined_at_ms: u64,
    ) -> Result<Self, String> {
        let item = data
            .items
            .iter()
            .find(|i| i.id == item_id)
            .ok_or("only a disclosed ambient item can be examined")?
            .clone();
        let reasons = data.relations_for(item_id).cloned().collect::<Vec<_>>();
        Ok(Self {
            schema: AMBIENT_MEMORY_SCHEMA.into(),
            id: digest(&(data.session_id.as_str(), data.position.as_str(), item_id)),
            session_id: data.session_id.clone(),
            position: data.position.clone(),
            item,
            reasons,
            examined_at_ms,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != AMBIENT_MEMORY_SCHEMA
            || self.id
                != digest(&(
                    self.session_id.as_str(),
                    self.position.as_str(),
                    self.item.id.as_str(),
                ))
            || self.reasons.is_empty()
            || self.reasons.iter().any(|r| {
                r.item_id != self.item.id
                    || r.version != ENGINE_VERSION
                    || r.engine != r.family.engine()
            })
        {
            return Err("invalid ambient memory identity or engine attribution".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemorySummary {
    pub memory: AmbientMemory,
    pub kept: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbientContext {
    pub data: CardNeighborhood,
    pub view: AmbientView,
    pub memory: Vec<MemorySummary>,
}
