// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graphshell projection: advertised intents, bounded action choices, and
//! snapshot construction over the phyllotaxis canvas layout.

use chirograph::{
    ActionFormChoiceV1, BoundsRelationship, CachePolicy, PresentationBinding,
    PresentationCapability, PresentationCodec, PresentationKey, PresentationManifest,
    PresentationOffer, PresentationSemantics, ProjectionSnapshot, SemanticRole,
};
use sceno::{
    Arrangement, Footprint, ProjectedItem, Rect, Representation, RoutedRelation, Scene, Score,
    Size2, SourceRef, Transform2, Vec2,
};
use scenotime::SceneSnapshot;

use super::*;
use crate::AstrologyFacts;

impl<B: Backend> CleromancyHost<B> {
    pub(crate) fn intent_was_advertised(&self, instance: InstanceId, intent: &str) -> bool {
        if crate::intents::scope_for(intent).is_none() {
            return false;
        }
        let Some(key) = self.active_instances.get(&instance).copied() else {
            return false;
        };
        if intent == crate::intents::CREATE_CONCURRENCE_INTENT {
            let forms_are_available = self
                .concurrence_form_choices()
                .map(|(facts, sessions)| !facts.is_empty() && !sessions.is_empty())
                .unwrap_or(false);
            return forms_are_available
                && (self.facet_value(key, ASTROLOGY_FACTS_FACET).is_some()
                    || self.facet_value(key, SESSION_FACET).is_some());
        }
        self.facet_value(key, CONTEXT_FACET).is_some()
    }

    /// Offer only exact saved values that replay against graph truth. The
    /// action form does not encode a correspondence or choose a counterpart:
    /// it makes both member identities an explicit, bounded host selection.
    fn concurrence_form_choices(
        &self,
    ) -> Result<(Vec<ActionFormChoiceV1>, Vec<ActionFormChoiceV1>), HostError> {
        let mut facts_choices = Vec::new();
        let mut session_choices = Vec::new();
        for (key, _) in self.graph.nodes() {
            if let Some(value) = self.facet_value(key, ASTROLOGY_FACTS_FACET) {
                let facts =
                    serde_json::from_value::<AstrologyFacts>(value.clone()).map_err(|error| {
                        HostError::InvalidStoredFacet {
                            facet: ASTROLOGY_FACTS_FACET,
                            reason: error.to_string(),
                        }
                    })?;
                let facts = self.replay_astrology_facts(&facts)?;
                let digest = facts.digest();
                facts_choices.push(
                    ActionFormChoiceV1::new(
                        digest.clone(),
                        format!("Astrology facts {}", short_identity(&digest)),
                    )
                    .with_description(format!(
                        "{} · {} placements · {} aspects",
                        facts.algorithm,
                        facts.placements.len(),
                        facts.aspects.len()
                    )),
                );
            }
            if let Some(value) = self.facet_value(key, SESSION_FACET) {
                let session =
                    serde_json::from_value::<ReadingSession>(value.clone()).map_err(|error| {
                        HostError::InvalidStoredFacet {
                            facet: SESSION_FACET,
                            reason: error.to_string(),
                        }
                    })?;
                self.replay_session(&session)?;
                session_choices.push(
                    ActionFormChoiceV1::new(
                        session.id.clone(),
                        format!("Reading session {}", short_identity(&session.id)),
                    )
                    .with_description(format!(
                        "{} placement(s) · recorded {} ms since Unix epoch",
                        session.placements.len(),
                        session.created_at_ms
                    )),
                );
            }
        }
        facts_choices.sort_by(|left, right| left.value.cmp(&right.value));
        session_choices.sort_by(|left, right| left.value.cmp(&right.value));
        Ok((facts_choices, session_choices))
    }

    pub(crate) fn concurrence_target_matches(
        &self,
        instance: InstanceId,
        astrology_facts_digest: &str,
        reading_session_id: &str,
    ) -> bool {
        let Some(key) = self.active_instances.get(&instance).copied() else {
            return false;
        };
        let Some(node) = self.graph.get_node(key) else {
            return false;
        };
        node.url() == format!("cleromancy://astrology/facts/{astrology_facts_digest}")
            || node.url() == format!("cleromancy://session/{reading_session_id}")
    }

    pub(super) fn score(&self) -> Score {
        mere::canvas::project_canvas_strategy_with_score(
            "phyllotaxis.default",
            &self.graph,
            None,
            1280,
            720,
            None,
            None,
            true,
        )
        .score
        .unwrap_or_else(|| Score::new(Arrangement::Spiral(Default::default())))
    }

    pub(crate) fn build_snapshot(&mut self) -> Result<ProjectionSnapshot, HostError> {
        self.build_snapshot_with_actions(false)
    }

    pub(crate) fn build_snapshot_with_actions(
        &mut self,
        advertise_intents: bool,
    ) -> Result<ProjectionSnapshot, HostError> {
        let (concurrence_facts, concurrence_sessions) = if advertise_intents {
            self.concurrence_form_choices()?
        } else {
            (Vec::new(), Vec::new())
        };
        let mut layout = mere::canvas::project_canvas_strategy_with_score(
            "phyllotaxis.default",
            &self.graph,
            None,
            1280,
            720,
            None,
            None,
            true,
        );
        let mut projection_keys = layout
            .positions
            .iter()
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        // Phyllotaxis supplies a stable point set but currently emits it
        // through a hash map. Pair canonical graph identities with a
        // canonical point order before assigning Graphshell instances.
        projection_keys.sort_by_key(|key| {
            self.graph
                .get_node(*key)
                .expect("layout key remains in graph")
                .id
        });
        let mut projection_points = layout
            .positions
            .iter()
            .map(|(_, point)| *point)
            .collect::<Vec<_>>();
        projection_points.sort_by(|left, right| {
            left.x
                .total_cmp(&right.x)
                .then_with(|| left.y.total_cmp(&right.y))
        });
        layout.positions = projection_keys.into_iter().zip(projection_points).collect();
        let mut scene = Scene::new();
        let mut presentation = PresentationManifest::default();
        let mut resources = BTreeMap::new();
        let mut instance_of = HashMap::with_capacity(layout.positions.len());
        let positions = layout.positions.iter().copied().collect::<HashMap<_, _>>();
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for (index, (key, position)) in layout.positions.iter().copied().enumerate() {
            let node = self
                .graph
                .get_node(key)
                .expect("layout key remains in graph");
            let instance = InstanceId(index as u32);
            instance_of.insert(key, instance);
            min_x = min_x.min(position.x);
            min_y = min_y.min(position.y);
            max_x = max_x.max(position.x);
            max_y = max_y.max(position.y);
            let source =
                scene.intern_source(SourceRef::new("cleromancy.graph", node.id.to_string()));
            scene.items.push(ProjectedItem {
                source,
                space: Scene::WORLD,
                transform: Transform2::translation(position.x, position.y),
                footprint: Footprint::Rect {
                    size: Size2::new(260.0, 150.0),
                },
                representation: Representation::Card,
                layer: 0,
                visible: true,
                hit: None,
                channels: Vec::new(),
            });

            let card = self.card_for(key);
            let bytes = serde_json::to_vec(&card).expect("portable card serializes");
            let resource = ContentHash::of(&bytes);
            let key_ref = PresentationKey(format!("cleromancy:{}", node.id));
            presentation.bindings.push(PresentationBinding {
                instance,
                key: key_ref.clone(),
            });
            presentation.offers.insert(
                key_ref,
                vec![PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource,
                    byte_size: bytes.len() as u64,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: node.title.clone(),
                        role: SemanticRole::Article,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: if !advertise_intents {
                            Vec::new()
                        } else if self.facet_value(key, CONTEXT_FACET).is_some() {
                            crate::intents::advertised_actions()
                        } else if self.facet_value(key, ASTROLOGY_FACTS_FACET).is_some()
                            || self.facet_value(key, SESSION_FACET).is_some()
                        {
                            crate::intents::concurrence_actions(
                                &concurrence_facts,
                                &concurrence_sessions,
                            )
                        } else {
                            Vec::new()
                        },
                    },
                }],
            );
            resources.insert(resource, bytes);
        }

        let mut routed_relations = Vec::new();
        for relation in self.graph.relations() {
            let (Some(&from), Some(&to), Some(from_position), Some(to_position)) = (
                instance_of.get(&relation.from),
                instance_of.get(&relation.to),
                positions.get(&relation.from),
                positions.get(&relation.to),
            ) else {
                continue;
            };
            routed_relations.push(RoutedRelation {
                from,
                to,
                space: Scene::WORLD,
                points: vec![
                    Vec2::new(from_position.x, from_position.y),
                    Vec2::new(to_position.x, to_position.y),
                ],
                kind: Some(relation_kind_label(relation.kind).to_string()),
                weight: Some(1.0),
            });
        }
        routed_relations.sort_by(|left, right| {
            left.from
                .0
                .cmp(&right.from.0)
                .then_with(|| left.to.0.cmp(&right.to.0))
                .then_with(|| left.kind.cmp(&right.kind))
        });
        scene.relations = routed_relations;

        scene.bounds = if layout.positions.is_empty() {
            Rect::new(Vec2::new(0.0, 0.0), Size2::new(0.0, 0.0))
        } else {
            Rect::new(
                Vec2::new(min_x - 130.0, min_y - 75.0),
                Size2::new(max_x - min_x + 260.0, max_y - min_y + 150.0),
            )
        };
        scene.generation = self.projection_revision;
        let scene = SceneSnapshot::from_dense(
            SceneEpoch(self.projection_epoch),
            Revision(self.projection_revision),
            scene,
        )
        .map_err(|error| HostError::InvalidSnapshot(format!("{error:?}")))?;
        self.active_instances = instance_of
            .into_iter()
            .map(|(key, instance)| (instance, key))
            .collect();
        self.last_snapshot = Some((scene.epoch, scene.revision));
        self.resources = resources;
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: self.session(),
            scene,
            presentation,
            cache_policy: CachePolicy::default(),
        })
    }
}
