// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Node wrapper and identity helpers shared by export and import.

use std::collections::BTreeSet;

use graphshell::personal_sync::PersonalGraphEvent;
use mere::kernel::graph::Graph;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::{CleromancySyncError, CleromancySyncSelection, SYNC_BATCH_SCHEMA};
use crate::SpreadRelationKind;

#[derive(Clone)]
pub(super) struct SelectedNode {
    pub(super) id: Uuid,
    pub(super) address: String,
    pub(super) title: String,
    pub(super) tags: Vec<String>,
    pub(super) facet: &'static str,
    pub(super) value: Value,
}

pub(super) fn selected_node(
    node: &mere::kernel::graph::Node,
    facet: &'static str,
    value: Value,
) -> SelectedNode {
    SelectedNode {
        id: node.id,
        address: node.url().to_string(),
        title: node.title.clone(),
        tags: node.tags.iter().cloned().collect(),
        facet,
        value,
    }
}

pub(super) fn append_node_events(events: &mut Vec<PersonalGraphEvent>, node: &SelectedNode) {
    events.push(PersonalGraphEvent::AddNode {
        id: node.id,
        address: node.address.clone(),
        title: node.title.clone(),
    });
    let tags = node.tags.iter().cloned().collect::<BTreeSet<_>>();
    events.extend(
        tags.into_iter()
            .map(|tag| PersonalGraphEvent::AddTag { node: node.id, tag }),
    );
    events.push(PersonalGraphEvent::SetFacet {
        node: node.id,
        facet: node.facet.to_string(),
        value: node.value.clone(),
    });
}

pub(super) fn validate_identity(
    id: Uuid,
    actual: &str,
    expected: &str,
) -> Result<(), CleromancySyncError> {
    if actual != expected {
        return Err(invalid(
            id,
            format!("address {actual:?} does not match {expected:?}"),
        ));
    }
    if Graph::node_namespace_id(expected) != id {
        return Err(invalid(id, "stable node id does not match its address"));
    }
    Ok(())
}

pub(super) fn batch_digest(
    selection: CleromancySyncSelection,
    events: &[PersonalGraphEvent],
) -> String {
    #[derive(Serialize)]
    struct DigestInput<'a> {
        schema: &'static str,
        selection: CleromancySyncSelection,
        events: &'a [PersonalGraphEvent],
    }
    let bytes = serde_json::to_vec(&DigestInput {
        schema: SYNC_BATCH_SCHEMA,
        selection,
        events,
    })
    .expect("Cleromancy sync events always serialize");
    blake3::hash(&bytes).to_hex().to_string()
}

pub(super) fn generic_semantic_kind(kind: SpreadRelationKind) -> mere::kernel::graph::SemanticSubKind {
    match kind {
        SpreadRelationKind::Supports => mere::kernel::graph::SemanticSubKind::Supports,
        SpreadRelationKind::Contradicts => mere::kernel::graph::SemanticSubKind::Contradicts,
        SpreadRelationKind::Questions => mere::kernel::graph::SemanticSubKind::Questions,
        SpreadRelationKind::NextStep => mere::kernel::graph::SemanticSubKind::NextStep,
        SpreadRelationKind::Elaborates => mere::kernel::graph::SemanticSubKind::Elaborates,
    }
}

pub(super) fn invalid(node: Uuid, reason: impl Into<String>) -> CleromancySyncError {
    CleromancySyncError::InvalidNode {
        node,
        reason: reason.into(),
    }
}
