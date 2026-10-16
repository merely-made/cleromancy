// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleromancy's domain binding to the shared relationship compiler, occurrence
//! dynamics runner and Cambium graph canvas. It owns no graph or force law.
use crate::ambient::*;
use cambium::{GraphCanvasNode, GraphCanvasRelation, GraphCanvasSubgraph, GraphCanvasSwatch};
use mere::canvas::projection_dynamics::ProjectionDynamics;
use sceno::{Scene, Size2, SourceRef};
use scenograph::dataset::{
    ProjectionDataset, ProjectionFieldType, ProjectionOccurrence, ProjectionValue,
};
use scenomise::projection::{
    CompiledRelationshipProjection, DisclosedRelationship, ItemSizes, ProjectionCompiler,
    RelationshipDataset, RelationshipProvenance,
};
use std::collections::{BTreeMap, BTreeSet};

pub const AMBIENT_LEAF: u64 = 0x434c_414d_4249_454e;

pub fn disclosed(data: &CardNeighborhood, view: &AmbientView) -> RelationshipDataset {
    let relationships: Vec<_> = data
        .relations
        .iter()
        .filter(|r| view.families.contains(&r.family))
        .map(|r| DisclosedRelationship {
            id: r.id.clone(),
            from_occurrence: data.subject.id.clone(),
            to_occurrence: r.item_id.clone(),
            kind: format!("{:?}", r.family),
            label: r.family.label().into(),
            explanation: r.reason.clone(),
            provenance: RelationshipProvenance {
                source: source_binding(data),
                source_revision: data.revision.clone().into(),
                method: r.method.clone(),
                method_version: r.version,
                provider: r.engine.clone(),
                evidence: r
                    .evidence
                    .iter()
                    .map(|id| SourceRef::new("cleromancy.evidence", id))
                    .collect(),
            },
        })
        .collect();
    let visible: BTreeSet<_> = relationships
        .iter()
        .map(|r| r.to_occurrence.as_str())
        .collect();
    let occurrences = std::iter::once(&data.subject)
        .chain(
            data.items
                .iter()
                .filter(|i| visible.contains(i.id.as_str())),
        )
        .enumerate()
        .map(|(index, item)| ProjectionOccurrence {
            occurrence_id: item.id.clone(),
            source: SourceRef::new("cleromancy.ambient", &item.source),
            values: BTreeMap::from([
                (
                    "occurrence_id".into(),
                    ProjectionValue::Text(item.id.clone()),
                ),
                ("label".into(), ProjectionValue::Text(item.title.clone())),
                ("order".into(), ProjectionValue::Number(index as f64)),
            ]),
        })
        .collect();
    RelationshipDataset {
        dataset: ProjectionDataset {
            source: source_binding(data),
            revision: data.revision.clone().into(),
            fields: BTreeMap::from([
                ("occurrence_id".into(), ProjectionFieldType::Text),
                ("label".into(), ProjectionFieldType::Text),
                ("order".into(), ProjectionFieldType::Number),
            ]),
            occurrences,
        },
        facets: BTreeSet::from([
            scenograph::relationship::EXPLAINED_RELATIONSHIPS_FACET.into(),
            scenograph::relationship::OCCURRENCE_LABELS_FACET.into(),
            scenograph::relationship::AUTHORED_ORDER_FACET.into(),
        ]),
        relationships,
    }
}

pub struct NeighborhoodRuntime {
    pub compiled: CompiledRelationshipProjection,
    pub dynamics: ProjectionDynamics,
}
impl NeighborhoodRuntime {
    pub fn new(data: &CardNeighborhood, view: &AmbientView) -> Result<Self, String> {
        view.validate()?;
        let mut snapshot = view.snapshot.clone();
        let disclosed = disclosed(data, view);
        if !disclosed
            .dataset
            .occurrences
            .iter()
            .any(|i| Some(&i.occurrence_id) == snapshot.selected_occurrence.as_ref())
        {
            snapshot.selected_occurrence = Some(data.subject.id.clone());
        }
        if !disclosed
            .relationships
            .iter()
            .any(|r| Some(&r.id) == snapshot.selected_relationship.as_ref())
        {
            snapshot.selected_relationship = None;
        }
        let mut compiled = ProjectionCompiler::new(ItemSizes {
            card: Size2::new(64.0, 40.0),
        })
        .compile_relationship_snapshot(&snapshot, &disclosed)
        .map_err(|issues| format!("neighborhood projection: {issues:?}"))?;
        for (id, instance) in &compiled.projection.instance_by_occurrence {
            if let Some([x, y]) = view.positions.get(id) {
                compiled.projection.scene.items[instance.0 as usize]
                    .transform
                    .translate = sceno::Vec2::new(*x as f32 / 1000.0, *y as f32 / 1000.0);
            }
        }
        let definition = snapshot
            .recipe
            .definition
            .bind(&snapshot.source_name, None)
            .map_err(|e| format!("neighborhood binding: {e:?}"))?;
        let slot = definition
            .dynamics
            .as_ref()
            .ok_or("missing dynamics slot")?;
        // A hidden family can temporarily remove an occurrence with a saved
        // role. Keep that role in the authored state, bind only visible bodies.
        let roles = view
            .roles
            .iter()
            .filter(|(id, _)| compiled.projection.instance_by_occurrence.contains_key(*id))
            .map(|(id, r)| (id.clone(), *r))
            .collect();
        let dynamics = ProjectionDynamics::new_relationship(&definition, &compiled, slot, &roles)
            .map_err(|e| e.to_string())?;
        Ok(Self { compiled, dynamics })
    }
    pub fn tick(&mut self) -> Result<bool, String> {
        self.dynamics.tick().map_err(|e| e.to_string())
    }
    pub fn scene(&self) -> &Scene {
        self.dynamics.scene()
    }
    pub fn retain_pose(&self, view: &mut AmbientView) {
        for (id, instance) in &self.compiled.projection.instance_by_occurrence {
            let at = self.scene().items[instance.0 as usize].transform.translate;
            view.positions.insert(
                id.clone(),
                [
                    (at.x * 1000.0).round() as i32,
                    (at.y * 1000.0).round() as i32,
                ],
            );
        }
    }
    pub fn swatch(
        &self,
        data: &CardNeighborhood,
        view: &AmbientView,
        width: u32,
    ) -> GraphCanvasSwatch<String, RelationFamily> {
        let scene = self.scene();
        let xs = scene
            .items
            .iter()
            .map(|i| i.transform.translate.x)
            .collect::<Vec<_>>();
        let ys = scene
            .items
            .iter()
            .map(|i| i.transform.translate.y)
            .collect::<Vec<_>>();
        let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let max_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let min_y = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let max_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let normalize = |x: f32, y: f32| {
            (
                0.10 + 0.80 * (x - min_x) / (max_x - min_x).max(100.0),
                0.10 + 0.80 * (y - min_y) / (max_y - min_y).max(100.0),
            )
        };
        let nodes = self
            .compiled
            .projection
            .instance_by_occurrence
            .iter()
            .filter_map(|(id, instance)| {
                let item = data.item(id)?;
                let at = scene.items[instance.0 as usize].transform.translate;
                let family = data
                    .relations_for(id)
                    .next()
                    .map_or(RelationFamily::Spread, |r| r.family);
                Some(GraphCanvasNode {
                    id: id.clone(),
                    kind: family,
                    position: normalize(at.x, at.y),
                    label: item.title.clone(),
                    key: Some(format!("ambient-node:{id}")),
                })
            })
            .collect();
        let relations = self
            .compiled
            .relationships
            .iter()
            .map(|r| GraphCanvasRelation {
                id: r.disclosure.id.clone(),
                from: r.disclosure.from_occurrence.clone(),
                to: r.disclosure.to_occurrence.clone(),
                kind: r.disclosure.kind.clone(),
                label: r.disclosure.explanation.clone(),
                route: Vec::new(),
                visible: true,
                emphasized: view.snapshot.selected_relationship.as_ref() == Some(&r.disclosure.id),
            })
            .collect();
        let mut swatch = GraphCanvasSwatch::new(
            AMBIENT_LEAF,
            GraphCanvasSubgraph {
                nodes,
                edges: Vec::new(),
            },
        )
        .with_size(width, 320)
        .with_label("Card neighborhood; use the item list for full explanations")
        .with_expand(false)
        .with_node_labels(true)
        .with_label_culling(true)
        .with_relations(relations);
        swatch.selected = view.snapshot.selected_occurrence.clone();
        for item in std::iter::once(&data.subject).chain(data.items.iter()) {
            swatch = swatch
                .with_visible_label(
                    item.id.clone(),
                    item.title.chars().take(22).collect::<String>(),
                )
                .with_node_footprint(item.id.clone(), 32.0, 32.0)
                .with_node_attr(
                    item.id.clone(),
                    "data-attention",
                    if view.foreground.contains(&item.id) {
                        "foreground"
                    } else {
                        "background"
                    },
                );
        }
        swatch
    }
    /// Invert the exact displayed node-normalization for an intentional drag.
    pub fn world_at(&self, normalized: (f32, f32)) -> [i32; 2] {
        let mut lo = [f32::INFINITY; 2];
        let mut hi = [f32::NEG_INFINITY; 2];
        for item in &self.scene().items {
            let at = item.transform.translate;
            lo[0] = lo[0].min(at.x);
            lo[1] = lo[1].min(at.y);
            hi[0] = hi[0].max(at.x);
            hi[1] = hi[1].max(at.y);
        }
        [
            ((lo[0] + (normalized.0 - 0.10) / 0.80 * (hi[0] - lo[0]).max(100.0)) * 1000.0).round()
                as i32,
            ((lo[1] + (normalized.1 - 0.10) / 0.80 * (hi[1] - lo[1]).max(100.0)) * 1000.0).round()
                as i32,
        ]
    }
}
