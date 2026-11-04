// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl ConsultationUi {
    pub(crate) fn clear_selection(&mut self) {
        self.ambient = Default::default();
        self.detail = None;
        self.comparison = None;
        self.reading_focus.clear();
        self.position_hover = None;
        self.reflection = TextInput::default();
        self.status = ConsultationStatus::Ready;
    }

    /// Replace picker and history values after the persistence worker commits a
    /// transaction; the worker is the authority for what remains available.
    pub(crate) fn replace_catalog(&mut self, catalog: ConsultationCatalog) {
        let selected_field = self
            .catalog
            .fields
            .get(self.field_select.selected)
            .map(crate::Field::digest);
        if self.detail.as_ref().is_some_and(|detail| {
            !catalog
                .session_summaries
                .iter()
                .any(|summary| summary.session_id == detail.session.id)
        }) {
            self.clear_selection();
        }
        self.context_select.selected = self.context_select.selected.min(catalog.contexts.len());
        self.field_select.selected = selected_field
            .and_then(|digest| {
                catalog
                    .fields
                    .iter()
                    .position(|field| field.digest() == digest)
            })
            .unwrap_or_else(|| {
                self.field_select
                    .selected
                    .min(catalog.fields.len().saturating_sub(1))
            });
        if self
            .card_preview
            .as_ref()
            .is_some_and(|(digest, _)| !catalog.fields.iter().any(|f| f.digest() == *digest))
        {
            self.card_preview = None;
        }
        self.comparison_select.selected = self.comparison_select.selected.min(
            catalog
                .session_summaries
                .len()
                .saturating_sub(if self.detail.is_some() { 1 } else { 0 }),
        );
        self.template_select.selected = self
            .template_select
            .selected
            .min(catalog.spread_templates.len());
        self.astrology_facts_select.selected = self
            .astrology_facts_select
            .selected
            .min(catalog.astrology_facts.len());
        self.chart.selected_chart.selected = self
            .chart
            .selected_chart
            .selected
            .min(catalog.astrology_charts.len().saturating_sub(1));
        let aspect_count = catalog
            .astrology_charts
            .get(self.chart.selected_chart.selected)
            .map_or(0, |stored| stored.facts.aspects.len());
        self.chart.selected_aspect.selected = self
            .chart
            .selected_aspect
            .selected
            .min(aspect_count.saturating_sub(1));
        self.sky.selected_day.selected = self
            .sky
            .selected_day
            .selected
            .min(catalog.sky_day_facts.len().saturating_sub(1));
        self.catalog = catalog;
        self.error = None;
        // A resident catalog refresh can precede the operation's detail.
        // Only its completion update may clear the pending product action.
    }

    /// The focused position name for a detail: the person's selection, or
    /// the first placement.
    pub(in super::super) fn focused_position(&self, detail: &ConsultationDetail) -> String {
        self.reading_focus
            .get(&detail.session.id)
            .filter(|position| {
                detail
                    .session
                    .placements
                    .iter()
                    .any(|placement| &placement.position == *position)
            })
            .cloned()
            .or_else(|| {
                detail
                    .session
                    .placements
                    .first()
                    .map(|placement| placement.position.clone())
            })
            .unwrap_or_default()
    }

    /// Worker results never move the person to another surface: only tab
    /// activation changes `surface_tabs`, so every `present_*` method below
    /// updates content and status alone.
    pub fn present_reading(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let session_id = detail.session.id.clone();
        self.adopt_detail(catalog, detail);
        self.workings.expanded = false;
        self.reflection = TextInput::default();
        self.comparison = None;
        self.status = ConsultationStatus::ReadingSaved(session_id);
    }

    pub fn present_reflection(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let reflection_id = detail
            .reflections
            .first()
            .map(|reflection| reflection.id.clone())
            .unwrap_or_else(|| detail.session.id.clone());
        self.adopt_detail(catalog, detail);
        self.reflection = TextInput::default();
        self.status = ConsultationStatus::ReflectionSaved(reflection_id);
    }

    pub fn present_session(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let session_id = detail.session.id.clone();
        self.adopt_detail(catalog, detail);
        self.comparison = None;
        self.status = ConsultationStatus::ViewingSession(session_id);
    }

    /// An unsolicited owner focus projection is inspection, not completion of
    /// the product transaction already queued on the worker.
    pub(crate) fn present_external_session(
        &mut self,
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
    ) {
        let pending = self.status.is_busy().then(|| self.status.clone());
        self.present_session(catalog, detail);
        if let Some(status) = pending {
            self.status = status;
        }
    }

    pub fn present_comparison(
        &mut self,
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
        comparison: ReceiptComparison,
    ) {
        let compared_session = comparison.right_session_id.clone();
        self.adopt_detail(catalog, detail);
        self.comparison_select.selected = self
            .comparison_candidates()
            .iter()
            .position(|session_id| session_id == &compared_session)
            .map_or(0, |index| index + 1);
        self.comparison = Some(comparison);
        self.status = ConsultationStatus::ViewingComparison(compared_session);
    }

    pub fn present_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.status = ConsultationStatus::Ready;
    }

    #[cfg(feature = "analytic-ephemeris")]
    pub(crate) fn present_calculated_chart(
        &mut self,
        catalog: ConsultationCatalog,
        facts_digest: String,
    ) {
        self.replace_catalog(catalog);
        self.astrology_facts_select.selected = self
            .catalog
            .astrology_facts
            .iter()
            .position(|facts| facts.digest() == facts_digest)
            .map_or(0, |index| index + 1);
        self.select_chart_facts(&facts_digest);
        self.status = ConsultationStatus::SetupSaved(format!(
            "chart {}",
            facts_digest.get(..12).unwrap_or(&facts_digest)
        ));
    }

    pub(super) fn adopt_detail(
        &mut self,
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
    ) {
        if self
            .ambient
            .context
            .as_ref()
            .is_some_and(|c| c.data.session_id != detail.session.id)
        {
            self.ambient.close();
        }
        self.card_preview = None;
        // Refresh the display clock after a draw; the startup clock precedes
        // new readings and would incorrectly label them "in the future".
        self.journal.now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| {
                duration.as_millis().try_into().unwrap_or(u64::MAX)
            });
        self.position_hover = None;
        self.context_select.selected = catalog
            .contexts
            .iter()
            .position(|context| context.digest() == detail.session.context_digest)
            .map_or(0, |index| index + 1);
        self.field_select.selected = catalog
            .fields
            .iter()
            .position(|field| field.digest() == detail.session.field_digest)
            .unwrap_or(0);
        self.mode.selected = match detail.readings.first().map(|reading| reading.receipt.mode) {
            Some(SelectionMode::Cast) => 1,
            Some(SelectionMode::Derived) => 2,
            Some(SelectionMode::Calculated) | None => 0,
        };
        let derivation = detail
            .readings
            .first()
            .and_then(|reading| reading.public_seed());
        self.derived_seed = TextInput::new(
            derivation
                .map(|selection| selection.0.to_string())
                .unwrap_or_default(),
        );
        self.derived_domain = TextInput::new(
            derivation
                .map(|selection| selection.1.to_string())
                .unwrap_or_default(),
        );
        self.template_select.selected = detail
            .spread_template
            .as_ref()
            .and_then(|template| {
                catalog
                    .spread_templates
                    .iter()
                    .position(|saved| saved.id == template.id)
            })
            .map_or(0, |index| index + 1);
        self.layout.selected = if detail.spread_template.is_some() {
            2
        } else if detail
            .session
            .placements
            .iter()
            .map(|placement| placement.position.as_str())
            .eq(["foundation", "tension", "next_step"])
        {
            1
        } else {
            0
        };
        self.context_label = TextInput::new(detail.context.label.clone());
        self.question = TextInput::new(
            detail
                .context
                .facts
                .get("question")
                .cloned()
                .unwrap_or_default(),
        );
        self.tags = TextInput::new(
            detail
                .context
                .tags
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        );
        self.additional_facts = TextInput::new(
            detail
                .context
                .facts
                .iter()
                .filter(|(name, _)| name.as_str() != "question")
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        self.catalog = catalog;
        self.detail = Some(detail);
        self.comparison_select.selected = 0;
        self.error = None;
    }

    pub(super) fn comparison_candidates(&self) -> Vec<String> {
        let selected_session = self.detail.as_ref().map(|detail| &detail.session.id);
        self.catalog
            .session_summaries
            .iter()
            .filter(|summary| Some(&summary.session_id) != selected_session)
            .map(|summary| summary.session_id.clone())
            .collect()
    }
}
