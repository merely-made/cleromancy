// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Recording and insertion of the fixed three-card layout and authored
//! spread templates with their session-bound instances.

use mere::kernel::graph::apply::assert_relation;
use mere::kernel::graph::{ContainmentSubKind, EdgeAssertion, ProvenanceSubKind};

use super::*;
use crate::{Spread, SpreadTemplate, ThreeCardPosition, ThreeCardRelationKind, ThreeCardSpread};

impl<B: Backend> CleromancyHost<B> {
    /// Cast the fixed authored three-card layout and save its ordered session
    /// plus graph-visible position and relationship metadata.
    pub fn record_three_card_spread_at_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        created_at_ms: u64,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<(ReadingSession, ThreeCardSpread, Vec<Reading>), HostError> {
        let readings = (0..ThreeCardPosition::ALL.len())
            .map(|_| ReadingEngine::cast_with(context, field, entropy))
            .collect::<Result<Vec<_>, _>>()?;
        let placements = ThreeCardPosition::ALL
            .into_iter()
            .zip(&readings)
            .map(|(position, reading)| crate::ReadingPlacement {
                position: position.as_str().to_string(),
                reading_id: reading.id.clone(),
            })
            .collect();
        let session = ReadingSession::with_placements(
            created_at_ms,
            event_nonce(entropy)?,
            context.digest(),
            field.digest(),
            placements,
            client_token,
        )?;
        self.insert_session(context, field, &readings, &session)?;
        let spread = ThreeCardSpread::new(&session.id, &session.placements)?;
        self.insert_three_card_spread(&session, &readings, &spread)?;
        Ok((session, spread, readings))
    }

    pub fn record_three_card_spread_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<(ReadingSession, ThreeCardSpread, Vec<Reading>), HostError> {
        self.record_three_card_spread_at_with_entropy(
            context,
            field,
            unix_time_ms()?,
            client_token,
            entropy,
        )
    }

    /// Cast one independently sealed reading for each position in a stored
    /// authored layout, then persist its session-bound spread record.
    pub fn record_spread_at_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        template: &SpreadTemplate,
        created_at_ms: u64,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<(ReadingSession, Spread, Vec<Reading>), HostError> {
        template.validate()?;
        let readings = template
            .positions
            .iter()
            .map(|_| ReadingEngine::cast_with(context, field, entropy))
            .collect::<Result<Vec<_>, _>>()?;
        let placements = template
            .positions
            .iter()
            .zip(&readings)
            .map(|(position, reading)| crate::ReadingPlacement {
                position: position.name.clone(),
                reading_id: reading.id.clone(),
            })
            .collect();
        let session = ReadingSession::with_placements(
            created_at_ms,
            event_nonce(entropy)?,
            context.digest(),
            field.digest(),
            placements,
            client_token,
        )?;
        self.insert_session(context, field, &readings, &session)?;
        let spread = Spread::new(template, &session)?;
        self.insert_spread_template(template)?;
        self.insert_spread(&session, &readings, template, &spread)?;
        Ok((session, spread, readings))
    }

    pub fn record_spread_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        template: &SpreadTemplate,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<(ReadingSession, Spread, Vec<Reading>), HostError> {
        self.record_spread_at_with_entropy(
            context,
            field,
            template,
            unix_time_ms()?,
            client_token,
            entropy,
        )
    }

    pub fn insert_three_card_spread(
        &mut self,
        session: &ReadingSession,
        readings: &[Reading],
        spread: &ThreeCardSpread,
    ) -> Result<NodeKey, HostError> {
        session.validate()?;
        spread.validate()?;
        if spread.session_id != session.id {
            return Err(SpreadError::InvalidSpread("bound session id".to_string()).into());
        }
        let session_key = self.session_key(session)?;
        let mut reading_keys = BTreeMap::new();
        for placement in &spread.placements {
            let session_placement = session
                .placements
                .iter()
                .find(|candidate| candidate.position == placement.position.as_str())
                .ok_or_else(|| SpreadError::InvalidSpread("session positions".to_string()))?;
            if session_placement.reading_id != placement.reading_id {
                return Err(SpreadError::InvalidSpread("placement reading id".to_string()).into());
            }
            let reading = readings
                .iter()
                .find(|reading| reading.id == placement.reading_id)
                .ok_or_else(|| HostError::MissingReadingDependency {
                    kind: "reading",
                    digest: placement.reading_id.clone(),
                })?;
            let key = self
                .graph
                .get_node_by_url(&format!("cleromancy://reading/{}", reading.id))
                .map(|(key, _)| key)
                .ok_or_else(|| HostError::MissingReadingDependency {
                    kind: "reading",
                    digest: reading.id.clone(),
                })?;
            reading_keys.insert(placement.position, key);
        }
        let key = self.upsert_node(
            &format!("cleromancy://spread/three-card/{}", spread.id),
            "Three-card spread",
            ["spread", "three-card-spread"],
        );
        self.set_facet(
            key,
            THREE_CARD_SPREAD_FACET,
            serde_json::to_value(spread).unwrap(),
        )?;
        assert_relation(
            &mut self.graph,
            key,
            session_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        for reading_key in reading_keys.values() {
            assert_relation(
                &mut self.graph,
                key,
                *reading_key,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::CollectionMember,
                },
            );
        }
        for relation in &spread.relations {
            assert_relation(
                &mut self.graph,
                reading_keys[&relation.from],
                reading_keys[&relation.to],
                EdgeAssertion::Semantic {
                    sub_kind: match relation.kind {
                        ThreeCardRelationKind::Questions => SemanticSubKind::Questions,
                        ThreeCardRelationKind::NextStep => SemanticSubKind::NextStep,
                    },
                    label: Some(relation.label.clone()),
                    decay_progress: None,
                },
            );
        }
        self.changed();
        Ok(key)
    }

    /// Store a reusable authored layout independently of every session that
    /// later uses it.
    pub fn insert_spread_template(
        &mut self,
        template: &SpreadTemplate,
    ) -> Result<NodeKey, HostError> {
        template.validate()?;
        let key = self.upsert_node(
            &format!("cleromancy://spread-template/{}", template.id),
            &template.label,
            ["spread", "spread-template"],
        );
        self.set_facet(
            key,
            SPREAD_TEMPLATE_FACET,
            serde_json::to_value(template).unwrap(),
        )?;
        self.changed();
        Ok(key)
    }

    /// Bind one stored template to one stored session and project the authored
    /// position relationships onto that session's sealed readings.
    pub fn insert_spread(
        &mut self,
        session: &ReadingSession,
        readings: &[Reading],
        template: &SpreadTemplate,
        spread: &Spread,
    ) -> Result<NodeKey, HostError> {
        session.validate()?;
        template.validate()?;
        spread.validate()?;
        if spread.template_id != template.id || spread.session_id != session.id {
            return Err(
                SpreadError::InvalidSpread("bound template or session id".to_string()).into(),
            );
        }
        if !template
            .positions
            .iter()
            .map(|position| position.name.as_str())
            .eq(session
                .placements
                .iter()
                .map(|placement| placement.position.as_str()))
        {
            return Err(SpreadError::InvalidSpread("session positions".to_string()).into());
        }
        self.validate_session_bindings(
            &self.context_for_digest(&session.context_digest)?,
            &self.field_for_digest(&session.field_digest)?,
            readings,
            session,
        )?;
        let session_key = self.session_key(session)?;
        let template_key = self
            .graph
            .get_node_by_url(&format!("cleromancy://spread-template/{}", template.id))
            .map(|(key, _)| key)
            .ok_or_else(|| HostError::MissingReadingDependency {
                kind: "spread template",
                digest: template.id.clone(),
            })?;
        let mut reading_keys = BTreeMap::new();
        for placement in &session.placements {
            let key = self
                .graph
                .get_node_by_url(&format!("cleromancy://reading/{}", placement.reading_id))
                .map(|(key, _)| key)
                .ok_or_else(|| HostError::MissingReadingDependency {
                    kind: "reading",
                    digest: placement.reading_id.clone(),
                })?;
            reading_keys.insert(placement.position.as_str(), key);
        }
        let key = self.upsert_node(
            &format!("cleromancy://spread/{}", spread.id),
            &template.label,
            ["spread", "authored-spread"],
        );
        self.set_facet(key, SPREAD_FACET, serde_json::to_value(spread).unwrap())?;
        assert_relation(
            &mut self.graph,
            key,
            session_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        assert_relation(
            &mut self.graph,
            key,
            template_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        for reading_key in reading_keys.values() {
            assert_relation(
                &mut self.graph,
                key,
                *reading_key,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::CollectionMember,
                },
            );
        }
        for relation in &template.relations {
            assert_relation(
                &mut self.graph,
                reading_keys[relation.from.as_str()],
                reading_keys[relation.to.as_str()],
                EdgeAssertion::Semantic {
                    sub_kind: semantic_kind(relation.kind),
                    label: Some(relation.label.clone()),
                    decay_progress: None,
                },
            );
        }
        self.changed();
        Ok(key)
    }
}
