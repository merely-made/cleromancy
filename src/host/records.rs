// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Insertion and recording of contexts, fields, readings, sessions,
//! reflections, astrology records, and pattern occasions.

use mere::kernel::graph::apply::assert_relation;
use mere::kernel::graph::{ContainmentSubKind, EdgeAssertion, ProvenanceSubKind};

use super::*;
use crate::{AstrologyChart, Concurrence, Reflection};

impl<B: Backend> CleromancyHost<B> {
    pub fn insert_context(&mut self, context: &ContextSnapshot) -> Result<NodeKey, HostError> {
        let address = format!("cleromancy://context/{}", context.digest());
        let key = self.upsert_node(
            &address,
            &context.label,
            context.tags.iter().map(String::as_str).chain(["context"]),
        );
        self.set_facet(key, CONTEXT_FACET, serde_json::to_value(context).unwrap())?;
        Ok(key)
    }

    /// Retain the complete candidate field as first-class graph truth. The
    /// digest-addressed node is shared by every reading made from this exact
    /// field, so replay does not depend on a catalog or caller remaining
    /// installed.
    pub fn insert_field(&mut self, field: &Field) -> Result<NodeKey, HostError> {
        let address = format!("cleromancy://field/{}", field.digest());
        let key = self.upsert_node(&address, &field.system, ["field", field.system.as_str()]);
        self.set_facet(key, FIELD_FACET, serde_json::to_value(field).unwrap())?;
        Ok(key)
    }

    /// Store an adapter-produced chart and its structured facts as separate
    /// graph truth. The facts node is explicitly generated from the chart and
    /// remains bound to its chart digest for replay.
    pub fn insert_astrology_chart(
        &mut self,
        chart: &AstrologyChart,
        orb_millidegrees: u32,
    ) -> Result<(NodeKey, NodeKey), HostError> {
        chart.validate()?;
        let facts = chart.facts(orb_millidegrees)?;
        let chart_digest = chart.digest();
        let chart_key = self.upsert_node(
            &format!("cleromancy://astrology/chart/{chart_digest}"),
            &format!("Astrology chart ({})", chart.engine),
            ["astrology", "chart", chart.engine.as_str()],
        );
        self.set_facet(
            chart_key,
            ASTROLOGY_CHART_FACET,
            serde_json::to_value(chart).unwrap(),
        )?;

        let facts_key = self.upsert_node(
            &format!("cleromancy://astrology/facts/{}", facts.digest()),
            &format!("Astrology facts ({})", chart.moment.instant_utc),
            ["astrology", "facts"],
        );
        self.set_facet(
            facts_key,
            ASTROLOGY_FACTS_FACET,
            serde_json::to_value(&facts).unwrap(),
        )?;
        assert_relation(
            &mut self.graph,
            facts_key,
            chart_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        self.changed();
        Ok((chart_key, facts_key))
    }

    /// Retain a grouping of independently produced graph values. Membership
    /// records that the values were consulted together; it does not make one
    /// member provenance for, or an explanation of, another.
    pub fn insert_concurrence(&mut self, concurrence: &Concurrence) -> Result<NodeKey, HostError> {
        concurrence.validate()?;
        let member_keys = concurrence
            .members
            .iter()
            .map(|member| {
                self.graph
                    .get_node_by_url(&member.address)
                    .map(|(key, _)| key)
                    .ok_or_else(|| HostError::MissingConcurrenceMember {
                        address: member.address.clone(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let key = self.upsert_node(
            &format!("cleromancy://concurrence/{}", concurrence.id),
            "Pattern occasion",
            ["concurrence", "pattern-occasion"],
        );
        self.set_facet(
            key,
            CONCURRENCE_FACET,
            serde_json::to_value(concurrence).unwrap(),
        )?;
        for member_key in member_keys {
            assert_relation(
                &mut self.graph,
                key,
                member_key,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::CollectionMember,
                },
            );
        }
        self.changed();
        Ok(key)
    }

    /// Resolve two saved values into one new pattern occasion. Every input is
    /// replayed from graph truth before the concurrence node is created.
    pub fn create_astrology_reading_concurrence(
        &mut self,
        astrology_facts_digest: &str,
        reading_session_id: &str,
    ) -> Result<Concurrence, HostError> {
        self.create_astrology_reading_concurrence_at(
            astrology_facts_digest,
            reading_session_id,
            unix_time_ms()?,
        )
    }

    /// Record a source-qualified chart and a reading in one explicit pattern
    /// occasion at a caller-supplied timestamp. This makes local controller
    /// tests deterministic without weakening the production convenience API.
    pub fn create_astrology_reading_concurrence_at(
        &mut self,
        astrology_facts_digest: &str,
        reading_session_id: &str,
        created_at_ms: u64,
    ) -> Result<Concurrence, HostError> {
        self.validate_astrology_reading_concurrence_members(
            astrology_facts_digest,
            reading_session_id,
        )?;
        let concurrence = Concurrence::astrology_reading(
            created_at_ms,
            astrology_facts_digest,
            reading_session_id,
        )?;
        self.insert_concurrence(&concurrence)?;
        Ok(concurrence)
    }

    pub(crate) fn validate_astrology_reading_concurrence_members(
        &self,
        astrology_facts_digest: &str,
        reading_session_id: &str,
    ) -> Result<(), HostError> {
        let facts = self.astrology_facts_for_digest(astrology_facts_digest)?;
        self.replay_astrology_facts(&facts)?;
        self.reading_session_for_id(reading_session_id)?;
        Ok(())
    }

    pub fn insert_reading(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        reading: &Reading,
    ) -> Result<NodeKey, HostError> {
        let replayed = ReadingEngine::replay(context, field, &reading.receipt)?;
        if replayed != *reading {
            return Err(ReadingError::ReceiptMismatch("sealed reading".to_string()).into());
        }
        let context_key = self.insert_context(context)?;
        let field_key = self.insert_field(field)?;
        let address = format!("cleromancy://reading/{}", reading.id);
        let mode = format!("{:?}", reading.receipt.mode).to_lowercase();
        let mut tags = vec!["reading", mode.as_str(), reading.system.as_str()];
        if reading.receipt.enrichment.is_some() {
            tags.push("externally-qualified");
        }
        let key = self.upsert_node(&address, &reading.title, tags);
        self.set_facet(key, READING_FACET, serde_json::to_value(reading).unwrap())?;
        assert_relation(
            &mut self.graph,
            key,
            context_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        assert_relation(
            &mut self.graph,
            key,
            field_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        self.changed();
        Ok(key)
    }

    /// Record a new occasion for a sealed result. This leaves the result
    /// receipt untouched, so repeated calculated reads can remain separately
    /// saved even when they resolve to the same content-addressed reading.
    pub fn record_reading_session_at_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        reading: &Reading,
        created_at_ms: u64,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<ReadingSession, HostError> {
        let event_nonce = event_nonce(entropy)?;
        let session = ReadingSession::single(
            created_at_ms,
            event_nonce,
            context.digest(),
            field.digest(),
            &reading.id,
            client_token,
        )?;
        self.insert_session(context, field, std::slice::from_ref(reading), &session)?;
        Ok(session)
    }

    /// Production convenience for a session timestamped at the local host.
    /// Tests and import use the explicit `*_at_with_entropy` form above.
    pub fn record_reading_session_with_entropy(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        reading: &Reading,
        client_token: Option<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<ReadingSession, HostError> {
        self.record_reading_session_at_with_entropy(
            context,
            field,
            reading,
            unix_time_ms()?,
            client_token,
            entropy,
        )
    }

    /// Record an immutable reflection without mutating the sealed result or
    /// the session to which it belongs.
    pub fn record_reflection_at_with_entropy(
        &mut self,
        session: &ReadingSession,
        created_at_ms: u64,
        body: impl Into<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<Reflection, HostError> {
        let reflection = Reflection::new(&session.id, created_at_ms, event_nonce(entropy)?, body)?;
        self.insert_reflection(session, &reflection)?;
        Ok(reflection)
    }

    /// Production convenience for a reflection timestamped at the local host.
    pub fn record_reflection_with_entropy(
        &mut self,
        session: &ReadingSession,
        body: impl Into<String>,
        entropy: &mut impl EntropySource,
    ) -> Result<Reflection, HostError> {
        self.record_reflection_at_with_entropy(session, unix_time_ms()?, body, entropy)
    }

    /// Store a session whose references have already been validated by the
    /// caller or a sync projection. This is public so import remains a thin
    /// mapping layer instead of a second model.
    pub fn insert_session(
        &mut self,
        context: &ContextSnapshot,
        field: &Field,
        readings: &[Reading],
        session: &ReadingSession,
    ) -> Result<NodeKey, HostError> {
        self.validate_session_bindings(context, field, readings, session)?;
        let context_key = self.insert_context(context)?;
        let field_key = self.insert_field(field)?;
        let mut reading_keys = Vec::with_capacity(session.placements.len());
        for placement in &session.placements {
            let reading = readings
                .iter()
                .find(|reading| reading.id == placement.reading_id)
                .expect("session bindings were validated before insertion");
            reading_keys.push(self.insert_reading(context, field, reading)?);
        }
        let key = self.upsert_node(
            &format!("cleromancy://session/{}", session.id),
            "Reading session",
            ["reading-session"],
        );
        self.set_facet(key, SESSION_FACET, serde_json::to_value(session).unwrap())?;
        assert_relation(
            &mut self.graph,
            key,
            context_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        assert_relation(
            &mut self.graph,
            key,
            field_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        for reading_key in reading_keys {
            assert_relation(
                &mut self.graph,
                key,
                reading_key,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::CollectionMember,
                },
            );
        }
        self.changed();
        Ok(key)
    }

    /// Attach one immutable reflection to a stored session. Later edits are a
    /// new reflection node, preserving the earlier record for inspection.
    pub fn insert_reflection(
        &mut self,
        session: &ReadingSession,
        reflection: &Reflection,
    ) -> Result<NodeKey, HostError> {
        session.validate()?;
        reflection.validate()?;
        if reflection.session_id != session.id {
            return Err(SessionError::InvalidReflection("bound session id".to_string()).into());
        }
        let session_key = self.session_key(session)?;
        let key = self.upsert_node(
            &format!("cleromancy://reflection/{}", reflection.id),
            "Reading reflection",
            ["reflection"],
        );
        self.set_facet(
            key,
            REFLECTION_FACET,
            serde_json::to_value(reflection).unwrap(),
        )?;
        assert_relation(
            &mut self.graph,
            key,
            session_key,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Elaborates,
                label: Some("reflects on".to_string()),
                decay_progress: None,
            },
        );
        self.changed();
        Ok(key)
    }
}
