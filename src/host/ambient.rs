// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ambient records use the existing candidate and resident journal. Keeping is
//! Alembic's tag-based promotion; view roles never supply a tag.
use super::*;
use crate::Consultation;
use crate::ambient::*;

pub const AMBIENT_VIEW_FACET: &str = "cleromancy.ambient-view/v1";
pub const AMBIENT_MEMORY_FACET: &str = "cleromancy.ambient-memory/v1";

impl<B: Backend> CleromancyHost<B> {
    pub fn card_neighborhood(
        &self,
        session_id: &str,
        position: &str,
    ) -> Result<CardNeighborhood, HostError> {
        // Construct a read-only domain decoder over the admitted graph. This
        // opens no persistence backend and performs no recursive admission.
        let mut reader = CleromancyHost::empty(muniment::MemoryBackend::new());
        reader.graph = self.graph.clone();
        let consultation = Consultation::new(reader);
        let detail = consultation.detail(session_id).map_err(ambient_error)?;
        let catalog = consultation.catalog().map_err(ambient_error)?;
        let history = catalog
            .session_summaries
            .iter()
            .map(|s| consultation.detail(&s.session_id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(ambient_error)?;
        AmbientEngines::default()
            .generate(&detail, position, &catalog, &history)
            .map_err(ambient_error)
    }
    pub fn ambient_views(&self) -> Result<Vec<AmbientView>, HostError> {
        let views = self.canonical_facet_values(
            AMBIENT_VIEW_FACET,
            |v: &AmbientView| v.id(),
            |id| format!("cleromancy://ambient-view/{id}"),
        )?;
        for view in &views {
            view.validate().map_err(ambient_error)?;
        }
        Ok(views)
    }
    pub fn ambient_memories(&self) -> Result<Vec<MemorySummary>, HostError> {
        let records = self.canonical_facet_values(
            AMBIENT_MEMORY_FACET,
            |v: &AmbientMemory| v.id.clone(),
            |id| format!("cleromancy://ambient-memory/{id}"),
        )?;
        records
            .into_iter()
            .map(|memory| {
                memory.validate().map_err(ambient_error)?;
                let (key, _) = self
                    .graph
                    .get_node_by_url(&format!("cleromancy://ambient-memory/{}", memory.id))
                    .expect("canonical enumeration");
                let kept = matches!(
                    alembic::memory_levels::level_of(&self.graph, key),
                    alembic::memory_levels::MemoryLevel::LongTerm
                );
                Ok(MemorySummary { memory, kept })
            })
            .collect()
    }
    pub fn ambient_context(
        &self,
        session_id: &str,
        position: &str,
    ) -> Result<AmbientContext, HostError> {
        let data = self.card_neighborhood(session_id, position)?.bounded();
        let mut view = self
            .ambient_views()?
            .into_iter()
            .find(|v| v.session_id == session_id && v.position == position)
            .unwrap_or_else(|| AmbientView::new(&data));
        view.rebind(&data);
        Ok(AmbientContext {
            data,
            view,
            memory: self.ambient_memories()?,
        })
    }
    pub fn insert_ambient_view(&mut self, view: &AmbientView) -> Result<(), HostError> {
        self.validate_ambient_view(view)?;
        let key = self.upsert_node(
            &format!("cleromancy://ambient-view/{}", view.id()),
            "Card neighborhood view",
            [],
        );
        self.set_facet(
            key,
            AMBIENT_VIEW_FACET,
            serde_json::to_value(view).map_err(ambient_error)?,
        )
    }
    pub fn examine_ambient(
        &mut self,
        session_id: &str,
        position: &str,
        item_id: &str,
        keep: bool,
        at_ms: u64,
    ) -> Result<AmbientMemory, HostError> {
        let data = self.card_neighborhood(session_id, position)?;
        let mut record = AmbientMemory::new(&data, item_id, at_ms).map_err(ambient_error)?;
        if let Some(previous) = self
            .ambient_memories()?
            .into_iter()
            .find(|m| m.memory.id == record.id)
        {
            record = previous.memory;
        }
        let key = self.upsert_node(
            &format!("cleromancy://ambient-memory/{}", record.id),
            &record.item.title,
            [],
        );
        self.set_facet(
            key,
            AMBIENT_MEMORY_FACET,
            serde_json::to_value(&record).map_err(ambient_error)?,
        )?;
        apply_graph_delta(&mut self.graph, GraphDelta::TouchNodeLastVisited { key });
        if keep {
            apply_graph_delta(
                &mut self.graph,
                GraphDelta::InsertNodeTag {
                    key,
                    tag: "kept".into(),
                },
            );
        }
        self.changed();
        Ok(record)
    }
    pub(super) fn validate_ambient_view(&self, view: &AmbientView) -> Result<(), HostError> {
        view.validate().map_err(ambient_error)?;
        let data = self.card_neighborhood(&view.session_id, &view.position)?;
        let known = |id: &str| data.item(id).is_some();
        if view
            .snapshot
            .selected_relationship
            .as_ref()
            .is_some_and(|id| !data.relations.iter().any(|r| &r.id == id))
        {
            return Err(ambient_error(
                "ambient view names an undisclosed relationship",
            ));
        }
        if view
            .positions
            .keys()
            .chain(view.roles.keys())
            .chain(view.foreground.iter())
            .any(|id| !known(id))
            || view
                .snapshot
                .selected_occurrence
                .as_ref()
                .is_some_and(|id| !known(id))
        {
            return Err(ambient_error(
                "ambient view names an undisclosed occurrence",
            ));
        }
        Ok(())
    }
    pub(super) fn validate_ambient_records(&self) -> Result<(), HostError> {
        for view in self.ambient_views()? {
            self.validate_ambient_view(&view)?;
        }
        for summary in self.ambient_memories()? {
            let record = summary.memory;
            let data = self.card_neighborhood(&record.session_id, &record.position)?;
            let rebuilt = AmbientMemory::new(&data, &record.item.id, record.examined_at_ms)
                .map_err(ambient_error)?;
            if rebuilt != record {
                return Err(ambient_error(
                    "ambient memory differs from its exact source-backed engine result",
                ));
            }
        }
        Ok(())
    }
}
fn ambient_error(error: impl ToString) -> HostError {
    HostError::Reading(ReadingError::ReceiptMismatch(format!(
        "ambient: {}",
        error.to_string()
    )))
}
