// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed catalog queries and canonical-address resolvers over graph truth.

use super::*;
use crate::{AstrologyChart, AstrologyFacts, Concurrence, Reflection, Spread, SpreadTemplate};

impl<B: Backend> CleromancyHost<B> {
    /// Every stored context, decoded from its canonical digest address.
    ///
    /// Product surfaces use this typed catalog instead of walking raw graph
    /// facets. A malformed or misplaced context is an error rather than a row
    /// that silently disappears from the picker.
    pub fn contexts(&self) -> Result<Vec<ContextSnapshot>, HostError> {
        let mut contexts =
            self.canonical_facet_values(CONTEXT_FACET, ContextSnapshot::digest, |digest| {
                format!("cleromancy://context/{digest}")
            })?;
        contexts.sort_by_key(|context| (context.label.to_lowercase(), context.digest()));
        Ok(contexts)
    }

    /// Every stored candidate field, ordered for a stable product picker.
    pub fn fields(&self) -> Result<Vec<Field>, HostError> {
        let mut fields = self.canonical_facet_values(FIELD_FACET, Field::digest, |digest| {
            format!("cleromancy://field/{digest}")
        })?;
        fields.sort_by_key(|field| {
            (
                field.system.to_lowercase(),
                field.rules.clone(),
                field.digest(),
            )
        });
        Ok(fields)
    }

    /// Every immutable authored layout, ordered for a stable local picker.
    pub fn spread_templates(&self) -> Result<Vec<SpreadTemplate>, HostError> {
        let mut templates = self.canonical_facet_values(
            SPREAD_TEMPLATE_FACET,
            |template: &SpreadTemplate| template.id.clone(),
            |id| format!("cleromancy://spread-template/{id}"),
        )?;
        for template in &templates {
            template.validate()?;
        }
        templates.sort_by_key(|template| (template.label.to_lowercase(), template.id.clone()));
        Ok(templates)
    }

    /// Every stored and replay-verified astrology facts record, ordered by its
    /// immutable digest for a stable local selector.
    pub fn astrology_facts(&self) -> Result<Vec<AstrologyFacts>, HostError> {
        let mut facts =
            self.canonical_facet_values(ASTROLOGY_FACTS_FACET, AstrologyFacts::digest, |digest| {
                format!("cleromancy://astrology/facts/{digest}")
            })?;
        for value in &facts {
            self.replay_astrology_facts(value)?;
        }
        facts.sort_by_key(AstrologyFacts::digest);
        Ok(facts)
    }

    /// Every saved reading occasion, newest first. Each row must replay from
    /// its graph-resident context, field, and readings before it is returned.
    pub fn sessions(&self) -> Result<Vec<ReadingSession>, HostError> {
        let mut sessions = self.canonical_facet_values(
            SESSION_FACET,
            |session: &ReadingSession| session.id.clone(),
            |id| format!("cleromancy://session/{id}"),
        )?;
        for session in &sessions {
            session.validate()?;
            self.replay_session(session)?;
        }
        sessions.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(sessions)
    }

    /// Resolve one context from its content digest and verify that the stored
    /// value still has that digest.
    pub fn context_for_digest(&self, digest: &str) -> Result<ContextSnapshot, HostError> {
        let context: ContextSnapshot = self.stored_facet(
            &format!("cleromancy://context/{digest}"),
            CONTEXT_FACET,
            "context",
            digest,
        )?;
        if context.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: CONTEXT_FACET,
                reason: "context digest does not match its canonical address".to_string(),
            });
        }
        Ok(context)
    }

    /// Immutable reflections attached to one saved occasion, newest first.
    pub fn reflections_for_session(&self, id: &str) -> Result<Vec<Reflection>, HostError> {
        self.reading_session_for_id(id)?;
        let mut reflections = self.canonical_facet_values(
            REFLECTION_FACET,
            |reflection: &Reflection| reflection.id.clone(),
            |reflection_id| format!("cleromancy://reflection/{reflection_id}"),
        )?;
        for reflection in &reflections {
            reflection.validate()?;
        }
        reflections.retain(|reflection| reflection.session_id == id);
        reflections.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(reflections)
    }

    /// Every saved grouping that explicitly contains one session. The group
    /// remains an association, not evidence that any member caused another.
    pub fn concurrences_for_session(&self, id: &str) -> Result<Vec<Concurrence>, HostError> {
        self.reading_session_for_id(id)?;
        let address = format!("cleromancy://session/{id}");
        let mut concurrences = self.canonical_facet_values(
            CONCURRENCE_FACET,
            |concurrence: &Concurrence| concurrence.id.clone(),
            |concurrence_id| format!("cleromancy://concurrence/{concurrence_id}"),
        )?;
        concurrences.retain(|concurrence| {
            concurrence
                .members
                .iter()
                .any(|member| member.address == address)
        });
        for concurrence in &concurrences {
            self.replay_concurrence(concurrence)?;
        }
        concurrences.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(concurrences)
    }

    pub fn astrology_chart_for_digest(&self, digest: &str) -> Result<AstrologyChart, HostError> {
        let chart: AstrologyChart = self.stored_facet(
            &format!("cleromancy://astrology/chart/{digest}"),
            ASTROLOGY_CHART_FACET,
            "astrology chart",
            digest,
        )?;
        if chart.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: ASTROLOGY_CHART_FACET,
                reason: "chart digest does not match its canonical address".to_string(),
            });
        }
        Ok(chart)
    }

    pub fn astrology_facts_for_digest(&self, digest: &str) -> Result<AstrologyFacts, HostError> {
        let facts: AstrologyFacts = self.stored_facet(
            &format!("cleromancy://astrology/facts/{digest}"),
            ASTROLOGY_FACTS_FACET,
            "astrology facts",
            digest,
        )?;
        if facts.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: ASTROLOGY_FACTS_FACET,
                reason: "facts digest does not match its canonical address".to_string(),
            });
        }
        Ok(facts)
    }

    /// Resolve one stored reading session and verify all of its replay
    /// dependencies before returning its exact saved identity.
    pub fn reading_session_for_id(&self, id: &str) -> Result<ReadingSession, HostError> {
        let session: ReadingSession = self.stored_facet(
            &format!("cleromancy://session/{id}"),
            SESSION_FACET,
            "session",
            id,
        )?;
        self.replay_session(&session)?;
        Ok(session)
    }

    pub fn field_for_digest(&self, digest: &str) -> Result<Field, HostError> {
        let address = format!("cleromancy://field/{digest}");
        let key = self
            .graph
            .get_node_by_url(&address)
            .map(|(key, _)| key)
            .ok_or_else(|| HostError::MissingReadingDependency {
                kind: "field",
                digest: digest.to_string(),
            })?;
        let value = self.facet_value(key, FIELD_FACET).ok_or_else(|| {
            HostError::MissingReadingDependency {
                kind: "field",
                digest: digest.to_string(),
            }
        })?;
        let field = serde_json::from_value::<Field>(value.clone()).map_err(|error| {
            HostError::InvalidStoredFacet {
                facet: FIELD_FACET,
                reason: error.to_string(),
            }
        })?;
        if field.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: FIELD_FACET,
                reason: "field digest does not match its canonical address".to_string(),
            });
        }
        Ok(field)
    }

    pub fn spread_template_for_id(&self, id: &str) -> Result<SpreadTemplate, HostError> {
        let template: SpreadTemplate = self.stored_facet(
            &format!("cleromancy://spread-template/{id}"),
            SPREAD_TEMPLATE_FACET,
            "spread template",
            id,
        )?;
        template.validate()?;
        if template.id != id {
            return Err(HostError::InvalidStoredFacet {
                facet: SPREAD_TEMPLATE_FACET,
                reason: "template id does not match its canonical address".to_string(),
            });
        }
        Ok(template)
    }

    pub fn spread_for_id(&self, id: &str) -> Result<Spread, HostError> {
        let spread: Spread = self.stored_facet(
            &format!("cleromancy://spread/{id}"),
            SPREAD_FACET,
            "spread",
            id,
        )?;
        spread.validate()?;
        self.replay_spread(&spread)?;
        Ok(spread)
    }
}
