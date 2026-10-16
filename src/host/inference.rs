// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use crate::inference::*;
use mere::kernel::graph::apply::assert_relation;
use mere::kernel::graph::{EdgeAssertion, ProvenanceSubKind};

fn invalid(e: impl std::fmt::Display) -> HostError {
    HostError::InvalidStoredFacet {
        facet: INFERENCE_FACET,
        reason: e.to_string(),
    }
}
fn excerpt(s: &str, limit: usize) -> String {
    s.chars().take(limit).collect()
}
impl<B: Backend> CleromancyHost<B> {
    /// Select the most recent eight earlier occasions. The explicit selection
    /// is sealed in the inference, so later history never rewrites its inputs.
    pub fn inference_inputs(&self, session_id: &str) -> Result<InferenceInputs, HostError> {
        let session = self.reading_session_for_id(session_id)?;
        let mut history = self.sessions()?;
        history.retain(|s| (s.created_at_ms, &s.id) < (session.created_at_ms, &session.id));
        history.sort_by(|a, b| (b.created_at_ms, &b.id).cmp(&(a.created_at_ms, &a.id)));
        history.truncate(8);
        let ids: Vec<_> = history.into_iter().map(|s| s.id).collect();
        let mut refs = Vec::new();
        for id in std::iter::once(session_id).chain(ids.iter().map(String::as_str)) {
            refs.extend(self.reflections_for_session(id)?);
        }
        refs.sort_by(|a, b| (b.created_at_ms, &b.id).cmp(&(a.created_at_ms, &a.id)));
        refs.truncate(16);
        self.resolve_inference_inputs(
            session_id,
            &ids,
            &refs.into_iter().map(|r| r.id).collect::<Vec<_>>(),
        )
    }
    fn resolve_inference_inputs(
        &self,
        session_id: &str,
        history_ids: &[String],
        reflection_ids: &[String],
    ) -> Result<InferenceInputs, HostError> {
        if history_ids.len() > 8
            || reflection_ids.len() > 16
            || history_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != history_ids.len()
            || reflection_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != reflection_ids.len()
        {
            return Err(invalid(
                "input selection exceeds bounds or contains duplicates",
            ));
        }
        let current = self.reading_session_for_id(session_id)?;
        let current_cards = self.replay_session(&current)?;
        let mut text = String::new();
        let mut repeated = std::collections::BTreeSet::new();
        let mut reflections = std::collections::BTreeMap::new();
        for (index, id) in std::iter::once(session_id)
            .chain(history_ids.iter().map(String::as_str))
            .enumerate()
        {
            let session = self.reading_session_for_id(id)?;
            if index > 0
                && (session.created_at_ms, &session.id) >= (current.created_at_ms, &current.id)
            {
                return Err(invalid("history source is not an earlier occasion"));
            }
            let context = self.context_for_digest(&session.context_digest)?;
            let readings = self.replay_session(&session)?;
            let template = self.spread_template_for_session(id)?;
            text.push_str(&format!(
                "{} occasion {} at {}: {}\n",
                if index == 0 { "Current" } else { "Earlier" },
                id,
                session.created_at_ms,
                excerpt(context.facts.get("question").unwrap_or(&context.label), 160)
            ));
            for placement in &session.placements {
                let reading = readings
                    .iter()
                    .find(|r| r.id == placement.reading_id)
                    .ok_or_else(|| invalid("missing placement reading"))?;
                // Resolve the saved authored position label, never the built-in id label.
                let meaning = template
                    .as_ref()
                    .and_then(|t| t.positions.iter().find(|p| p.name == placement.position))
                    .map(|p| p.label.clone())
                    .unwrap_or_default();
                text.push_str(&format!(
                    "{}: {}. {}\n",
                    excerpt(&placement.position, 80),
                    excerpt(&reading.display_title(), 120),
                    excerpt(&reading.interpretation, 160)
                ));
                if index == 0 && !meaning.is_empty() {
                    text.push_str(&format!("Saved template: {}\n", excerpt(&meaning, 320)));
                }
                if index > 0
                    && current_cards.iter().any(|c| {
                        c.candidate_id == reading.candidate_id && c.system == reading.system
                    })
                {
                    repeated.insert(reading.title.clone());
                }
            }
            for reflection in self.reflections_for_session(id)? {
                reflections.insert(reflection.id.clone(), reflection);
            }
        }
        for id in reflection_ids {
            let reflection = reflections
                .get(id)
                .ok_or_else(|| invalid("reflection is not part of the selected occasions"))?;
            text.push_str(&format!(
                "Saved reflection {}: {}\n",
                id,
                excerpt(&reflection.body, 240)
            ));
        }
        if text.len() > 16384 {
            return Err(invalid(
                "selected context exceeds 16384 bytes; choose fewer history sources",
            ));
        }
        Ok(InferenceInputs {
            session_id: session_id.into(),
            history_session_ids: history_ids.into(),
            reflection_ids: reflection_ids.into(),
            context_text: text,
            repeated_cards: repeated.into_iter().collect(),
        })
    }
    fn verify_inference(&self, record: &GeneratedInference) -> Result<(), HostError> {
        record.validate().map_err(invalid)?;
        let inputs = &record.inputs;
        let resolved = self.resolve_inference_inputs(
            &inputs.session_id,
            &inputs.history_session_ids,
            &inputs.reflection_ids,
        )?;
        if inputs != &resolved {
            return Err(invalid("stored inputs differ from their immutable sources"));
        }
        Ok(())
    }
    pub fn generated_inferences(&self) -> Result<Vec<GeneratedInference>, HostError> {
        let records =
            self.canonical_facet_values(INFERENCE_FACET, GeneratedInference::digest, |id| {
                format!("cleromancy://inference/{id}")
            })?;
        for record in &records {
            self.verify_inference(record)?;
        }
        Ok(records)
    }
    pub fn inferences_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<GeneratedInference>, HostError> {
        let records =
            self.canonical_facet_values(INFERENCE_FACET, GeneratedInference::digest, |id| {
                format!("cleromancy://inference/{id}")
            })?;
        let records: Vec<_> = records
            .into_iter()
            .filter(|r| r.inputs.session_id == session_id)
            .collect();
        for record in &records {
            self.verify_inference(record)?;
        }
        Ok(records)
    }
    pub fn insert_generated_inference(
        &mut self,
        record: &GeneratedInference,
    ) -> Result<(), HostError> {
        self.verify_inference(record)?; // Refusal before any candidate mutation.
        let url = format!("cleromancy://inference/{}", record.digest());
        if self.graph.get_node_by_url(&url).is_some() {
            return Ok(());
        }
        let key = self.upsert_node(&url, &record.label, []);
        self.set_facet(
            key,
            INFERENCE_FACET,
            serde_json::to_value(record).map_err(invalid)?,
        )?;
        apply_graph_delta(&mut self.graph, GraphDelta::TouchNodeLastVisited { key });
        for id in
            std::iter::once(&record.inputs.session_id).chain(&record.inputs.history_session_ids)
        {
            let source = self
                .graph
                .get_node_by_url(&format!("cleromancy://session/{id}"))
                .expect("verified session")
                .0;
            assert_relation(
                &mut self.graph,
                key,
                source,
                EdgeAssertion::Provenance {
                    sub_kind: ProvenanceSubKind::GeneratedFrom,
                },
            );
        }
        self.changed();
        Ok(())
    }
    pub fn keep_generated_inference(&mut self, digest: &str) -> Result<(), HostError> {
        let record = self
            .generated_inferences()?
            .into_iter()
            .find(|r| r.digest() == digest)
            .ok_or_else(|| invalid("inference is not stored"))?;
        let key = self
            .graph
            .get_node_by_url(&format!("cleromancy://inference/{}", record.digest()))
            .expect("canonical record")
            .0;
        apply_graph_delta(
            &mut self.graph,
            GraphDelta::InsertNodeTag {
                key,
                tag: "kept".into(),
            },
        );
        self.changed();
        Ok(())
    }
}
impl<B: Backend> crate::Consultation<B> {
    pub fn record_inference(
        &mut self,
        record: &GeneratedInference,
    ) -> Result<crate::ConsultationDetail, crate::ConsultationError> {
        self.ensure_writable()?;
        self.host_mut_candidate()
            .insert_generated_inference(record)?;
        self.detail(&record.inputs.session_id)
    }
    pub fn keep_inference(
        &mut self,
        session_id: &str,
        digest: &str,
    ) -> Result<crate::ConsultationDetail, crate::ConsultationError> {
        self.ensure_writable()?;
        if !self
            .host()
            .inferences_for_session(session_id)?
            .iter()
            .any(|r| r.digest() == digest)
        {
            return Err(invalid("inference belongs to another reading occasion").into());
        }
        self.host_mut_candidate().keep_generated_inference(digest)?;
        self.detail(session_id)
    }
}
