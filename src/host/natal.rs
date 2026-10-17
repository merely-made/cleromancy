// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use crate::natal::{NATAL_FACET, NatalReading};
fn invalid(reason: impl std::fmt::Display) -> HostError {
    HostError::InvalidStoredFacet {
        facet: NATAL_FACET,
        reason: reason.to_string(),
    }
}
impl<B: Backend> CleromancyHost<B> {
    pub fn natal_reflections(&self, id: &str) -> Result<Vec<crate::Reflection>, HostError> {
        if !self.natal_readings()?.iter().any(|r| r.id == id) {
            return Err(invalid("Unknown natal occasion"));
        }
        let mut records = self.canonical_facet_values(
            REFLECTION_FACET,
            |r: &crate::Reflection| r.id.clone(),
            |id| format!("cleromancy://reflection/{id}"),
        )?;
        records.retain(|r| r.session_id == id);
        for r in &records {
            r.validate()?;
        }
        Ok(records)
    }
    pub fn insert_natal_reflection(&mut self, r: &crate::Reflection) -> Result<(), HostError> {
        r.validate()?;
        self.natal_reflections(&r.session_id)?;
        let key = self.upsert_node(
            &format!("cleromancy://reflection/{}", r.id),
            "Natal reflection",
            ["reflection", "natal"],
        );
        self.set_facet(
            key,
            REFLECTION_FACET,
            serde_json::to_value(r).map_err(invalid)?,
        )?;
        Ok(())
    }
    pub fn insert_natal_reading(&mut self, reading: &NatalReading) -> Result<NodeKey, HostError> {
        reading.validate().map_err(invalid)?;
        let key = self.upsert_node(
            &format!("cleromancy://natal/{}", reading.id),
            &format!("Natal reading for {}", reading.subject_label),
            ["natal", "reading"],
        );
        self.set_facet(
            key,
            NATAL_FACET,
            serde_json::to_value(reading).map_err(invalid)?,
        )?;
        Ok(key)
    }
    pub fn natal_readings(&self) -> Result<Vec<NatalReading>, HostError> {
        let mut records = self.canonical_facet_values(
            NATAL_FACET,
            |r: &NatalReading| r.id.clone(),
            |id| format!("cleromancy://natal/{id}"),
        )?;
        for record in &records {
            record.validate().map_err(invalid)?;
        }
        records.sort_by(|a, b| (b.created_at_ms, &b.id).cmp(&(a.created_at_ms, &a.id)));
        Ok(records)
    }
}
