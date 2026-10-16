// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use crate::situation::{ReadingSituation, SITUATION_FACET};
use mere::kernel::graph::apply::assert_relation;
use mere::kernel::graph::{EdgeAssertion, ProvenanceSubKind};

fn invalid(e: impl std::fmt::Display) -> HostError {
    HostError::InvalidStoredFacet {
        facet: SITUATION_FACET,
        reason: e.to_string(),
    }
}
impl<B: Backend> CleromancyHost<B> {
    pub fn reading_situations(&self) -> Result<Vec<ReadingSituation>, HostError> {
        let records =
            self.canonical_facet_values(SITUATION_FACET, ReadingSituation::digest, |digest| {
                format!("cleromancy://situation/{digest}")
            })?;
        let mut sessions = std::collections::BTreeSet::new();
        for record in &records {
            if !sessions.insert(&record.session_id) {
                return Err(invalid(
                    "more than one situation for the same reading occasion",
                ));
            }
            let session = self.reading_session_for_id(&record.session_id)?;
            record.verify(&session).map_err(invalid)?;
        }
        Ok(records)
    }
    pub fn situation_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<ReadingSituation>, HostError> {
        let records =
            self.canonical_facet_values(SITUATION_FACET, ReadingSituation::digest, |digest| {
                format!("cleromancy://situation/{digest}")
            })?;
        let selected: Vec<_> = records
            .into_iter()
            .filter(|v| v.session_id == session_id)
            .collect();
        if selected.len() > 1 {
            return Err(invalid("more than one situation for this reading occasion"));
        }
        if let Some(record) = selected.first() {
            record
                .verify(&self.reading_session_for_id(session_id)?)
                .map_err(invalid)?;
        }
        Ok(selected.into_iter().next())
    }
    pub fn insert_reading_situation(&mut self, record: &ReadingSituation) -> Result<(), HostError> {
        let session = self.reading_session_for_id(&record.session_id)?;
        record.verify(&session).map_err(invalid)?;
        if let Some(old) = self.situation_for_session(&record.session_id)? {
            if old == *record {
                return Ok(());
            }
            return Err(invalid(
                "the situation for this saved occasion is already sealed",
            ));
        }
        let key = self.upsert_node(
            &format!("cleromancy://situation/{}", record.digest()),
            "Recorded reading situation",
            ["situation"],
        );
        self.set_facet(
            key,
            SITUATION_FACET,
            serde_json::to_value(record).map_err(invalid)?,
        )?;
        let session_key = self
            .graph
            .get_node_by_url(&format!("cleromancy://session/{}", record.session_id))
            .expect("resolved session")
            .0;
        assert_relation(
            &mut self.graph,
            key,
            session_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        self.changed();
        Ok(())
    }
}

impl<B: Backend> crate::Consultation<B> {
    /// The ordinary resident worker stages this beside the draw in one
    /// revision-checked candidate. No second persistence owner is introduced.
    pub fn record_situation(
        &mut self,
        record: &ReadingSituation,
    ) -> Result<crate::ConsultationDetail, crate::ConsultationError> {
        self.ensure_writable()?;
        self.host_mut_candidate().insert_reading_situation(record)?;
        self.detail(&record.session_id)
    }
}
