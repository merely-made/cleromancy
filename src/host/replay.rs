// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Replay of sealed records from graph-resident truth alone.

use super::*;
use crate::{AstrologyFacts, Concurrence, Spread, ThreeCardSpread};

impl<B: Backend> CleromancyHost<B> {
    /// Resolve and verify a graph-resident astrology facts node without
    /// requiring the original adapter to remain installed.
    pub fn replay_astrology_facts(
        &self,
        facts: &AstrologyFacts,
    ) -> Result<AstrologyFacts, HostError> {
        let chart = self.astrology_chart_for_digest(&facts.chart_digest)?;
        facts.verify(&chart)?;
        Ok(chart.facts(facts.orb_millidegrees)?)
    }

    pub fn replay_concurrence(&self, concurrence: &Concurrence) -> Result<Concurrence, HostError> {
        concurrence.validate()?;
        let stored: Concurrence = self.stored_facet(
            &format!("cleromancy://concurrence/{}", concurrence.id),
            CONCURRENCE_FACET,
            "concurrence",
            &concurrence.id,
        )?;
        if stored != *concurrence {
            return Err(ConcurrenceError::Invalid("stored value".to_string()).into());
        }
        for member in &stored.members {
            if self.graph.get_node_by_url(&member.address).is_none() {
                return Err(HostError::MissingConcurrenceMember {
                    address: member.address.clone(),
                });
            }
        }
        Ok(stored)
    }

    /// Replay a reading from graph-resident truth alone. The caller supplies
    /// neither its context nor its candidate field.
    pub fn replay_reading(&self, reading: &Reading) -> Result<Reading, HostError> {
        let context = self.stored_facet::<ContextSnapshot>(
            &format!("cleromancy://context/{}", reading.receipt.context_digest),
            CONTEXT_FACET,
            "context",
            &reading.receipt.context_digest,
        )?;
        let field = self.stored_facet::<Field>(
            &format!("cleromancy://field/{}", reading.receipt.field_digest),
            FIELD_FACET,
            "field",
            &reading.receipt.field_digest,
        )?;
        Ok(ReadingEngine::replay(&context, &field, &reading.receipt)?)
    }

    /// Resolve a stored reading occasion solely from graph-resident context,
    /// field, and sealed result nodes. The returned order is the session's
    /// declared placement order.
    pub fn replay_session(&self, session: &ReadingSession) -> Result<Vec<Reading>, HostError> {
        let stored = self.stored_facet::<ReadingSession>(
            &format!("cleromancy://session/{}", session.id),
            SESSION_FACET,
            "session",
            &session.id,
        )?;
        if stored != *session {
            return Err(SessionError::InvalidSession("stored value".to_string()).into());
        }
        let context = self.stored_facet::<ContextSnapshot>(
            &format!("cleromancy://context/{}", session.context_digest),
            CONTEXT_FACET,
            "context",
            &session.context_digest,
        )?;
        let field = self.stored_facet::<Field>(
            &format!("cleromancy://field/{}", session.field_digest),
            FIELD_FACET,
            "field",
            &session.field_digest,
        )?;
        let mut readings = Vec::with_capacity(session.placements.len());
        for placement in &session.placements {
            let reading = self.stored_facet::<Reading>(
                &format!("cleromancy://reading/{}", placement.reading_id),
                READING_FACET,
                "reading",
                &placement.reading_id,
            )?;
            readings.push(reading);
        }
        self.validate_session_bindings(&context, &field, &readings, session)?;
        Ok(readings)
    }

    pub fn replay_three_card_spread(
        &self,
        spread: &ThreeCardSpread,
    ) -> Result<Vec<Reading>, HostError> {
        let stored = self.stored_facet::<ThreeCardSpread>(
            &format!("cleromancy://spread/three-card/{}", spread.id),
            THREE_CARD_SPREAD_FACET,
            "spread",
            &spread.id,
        )?;
        if stored != *spread {
            return Err(SpreadError::InvalidSpread("stored value".to_string()).into());
        }
        let session = self.stored_facet::<ReadingSession>(
            &format!("cleromancy://session/{}", spread.session_id),
            SESSION_FACET,
            "session",
            &spread.session_id,
        )?;
        let readings = self.replay_session(&session)?;
        let mut ordered = Vec::with_capacity(spread.placements.len());
        for placement in &spread.placements {
            ordered.push(
                readings
                    .iter()
                    .find(|reading| reading.id == placement.reading_id)
                    .ok_or_else(|| HostError::MissingReadingDependency {
                        kind: "reading",
                        digest: placement.reading_id.clone(),
                    })?
                    .clone(),
            );
        }
        Ok(ordered)
    }

    /// Resolve a generic spread from its immutable template, session, and
    /// sealed readings. The returned reading order is the template order.
    pub fn replay_spread(&self, spread: &Spread) -> Result<Vec<Reading>, HostError> {
        let stored = self.stored_facet::<Spread>(
            &format!("cleromancy://spread/{}", spread.id),
            SPREAD_FACET,
            "spread",
            &spread.id,
        )?;
        if stored != *spread {
            return Err(SpreadError::InvalidSpread("stored value".to_string()).into());
        }
        let template = self.spread_template_for_id(&spread.template_id)?;
        let session = self.reading_session_for_id(&spread.session_id)?;
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
        let readings = self.replay_session(&session)?;
        if readings.len() != template.positions.len() {
            return Err(SpreadError::InvalidSpread("reading count".to_string()).into());
        }
        Ok(readings)
    }
}
