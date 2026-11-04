// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl mesquite::Product for Product {
    type State = ApplicationUi;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "cleromancy";
    const SURFACE: &'static str = "cleromancy";
    const LOG_PREFIX: &'static str = "cleromancy";

    fn sheet(&self) -> &str {
        &self.sheet
    }

    fn snapshot(&self, ctx: &ConsultationCtx<'_>, _: usize, _: f32) -> taproot::ProbeSnapshot {
        let observed = observation(ctx.runner, self.state.borrow().catalog_ready);
        let mut snapshot = taproot::ProbeSnapshot::default()
            .with_field("status", observed.status)
            .with_field("catalog-ready", observed.catalog_ready.to_string())
            .with_field("sessions", observed.sessions.to_string())
            .with_field("readings", observed.readings.to_string())
            .with_field("reflections", observed.reflections.to_string());
        let mere = &ctx.runner.state().mere;
        snapshot = snapshot
            .with_field("mere-sessions", mere.model.sessions.len().to_string())
            .with_field(
                "mere-live-sessions",
                mere.model
                    .sessions
                    .iter()
                    .filter(|s| !s.trashed)
                    .count()
                    .to_string(),
            )
            .with_field(
                "mere-trashed-sessions",
                mere.model
                    .sessions
                    .iter()
                    .filter(|s| s.trashed)
                    .count()
                    .to_string(),
            )
            .with_field(
                "mere-attached",
                mere.model
                    .sessions
                    .iter()
                    .find(|s| s.attached)
                    .map(|s| s.id.as_uuid().to_string())
                    .unwrap_or_default(),
            )
            .with_field("mere-nodes", mere.model.graph.nodes.len().to_string())
            .with_field(
                "fields",
                ctx.runner
                    .state()
                    .consultation
                    .catalog()
                    .fields
                    .len()
                    .to_string(),
            )
            .with_field(
                "mere-archives",
                mere.model
                    .archive
                    .as_ref()
                    .map_or(0, |a| a.entries.len())
                    .to_string(),
            )
            .with_field("mere-open", mere.open.to_string())
            .with_field(
                "mere-focused",
                mere.view.focused.clone().unwrap_or_default(),
            )
            .with_field("mere-layout", mere.model.layout.clone())
            .with_field("mere-notice", mere.model.notice.clone().unwrap_or_default());
        let appearance = &ctx.runner.state().appearance;
        snapshot = snapshot
            .with_field("appearance-id", appearance.active_id())
            .with_field(
                "font-reading",
                appearance
                    .font_preferences()
                    .0
                    .get(&tabard::typography::FontRole::Reading)
                    .and_then(|stack| stack.first())
                    .map_or("default", |family| family.name()),
            )
            .with_field(
                "font-heading",
                appearance
                    .font_preferences()
                    .0
                    .get(&tabard::typography::FontRole::Heading)
                    .and_then(|stack| stack.first())
                    .map_or("default", |family| family.name()),
            )
            .with_field(
                "font-label",
                appearance
                    .font_preferences()
                    .0
                    .get(&tabard::typography::FontRole::Label)
                    .and_then(|stack| stack.first())
                    .map_or("default", |family| family.name()),
            )
            .with_field(
                "appearance-mode",
                appearance
                    .applied()
                    .and_then(|v| v.resolved.theme_mode.as_ref())
                    .map_or_else(|| "legacy".into(), |mode| mode.as_key()),
            )
            .with_field("appearance-editor", appearance.editor_open.to_string())
            .with_field(
                "appearance-authoring",
                appearance.authoring_available.to_string(),
            )
            .with_field(
                "appearance-dirty",
                appearance.workshop.has_changes().to_string(),
            )
            .with_field(
                "appearance-name",
                appearance.workshop.draft_theme().name.clone(),
            );
        if let Some(ids) = observed.ids {
            snapshot = snapshot
                .with_field("session-id", ids.session_id)
                .with_field("reading-id", ids.reading_id)
                .with_field("reflection-id", ids.reflection_id);
        }
        let ui = &ctx.runner.state().consultation;
        snapshot = snapshot
            .with_field(
                "mere-unavailable",
                matches!(
                    ctx.runner.state().mere.model.status,
                    mere_view::ViewStatus::Unavailable(_)
                )
                .to_string(),
            )
            .with_field("natal-count", ui.catalog.natal_readings.len().to_string())
            .with_field(
                "natal-aspects",
                ui.catalog
                    .natal_readings
                    .iter()
                    .filter(|r| r.aspects.is_some())
                    .count()
                    .to_string(),
            )
            .with_field(
                "natal-houses-count",
                ui.catalog
                    .natal_readings
                    .iter()
                    .filter(|r| {
                        r.houses.as_ref().is_some_and(|h| {
                            matches!(
                                h.outcome,
                                crate::natal_houses::HouseOutcome::Calculated { .. }
                            )
                        })
                    })
                    .count()
                    .to_string(),
            )
            .with_field(
                "natal-supplements",
                ui.catalog
                    .natal_readings
                    .iter()
                    .filter(|r| r.supplement.is_some())
                    .count()
                    .to_string(),
            )
            .with_field(
                "natal-placidus",
                ui.catalog
                    .natal_readings
                    .iter()
                    .filter(|r| {
                        r.supplement.as_ref().is_some_and(|s| {
                            s.house_system == Some(crate::natal_cap::HouseChoice::Placidus)
                                && matches!(
                                    s.houses,
                                    Some(crate::natal_houses::HouseOutcome::Calculated { .. })
                                )
                        })
                    })
                    .count()
                    .to_string(),
            )
            .with_field(
                "natal-observed",
                ui.catalog
                    .natal_readings
                    .iter()
                    .filter(|r| {
                        r.supplement
                            .as_ref()
                            .is_some_and(|s| s.observed_values.is_some())
                    })
                    .count()
                    .to_string(),
            )
            .with_field(
                "natal-reflections",
                ui.catalog.natal_reflections.len().to_string(),
            )
            .with_field(
                "birth-count",
                ui.birth_catalog
                    .as_ref()
                    .map_or(0, |c| c.births.len())
                    .to_string(),
            )
            .with_field(
                "subject-count",
                ui.birth_catalog
                    .as_ref()
                    .map_or(0, |c| c.subjects.len())
                    .to_string(),
            )
            .with_field("product-busy", ui.status().is_busy().to_string())
            .with_field(
                "ambient-memory",
                ui.catalog.ambient_memory.len().to_string(),
            )
            .with_field(
                "ambient-kept",
                ui.catalog
                    .ambient_memory
                    .iter()
                    .filter(|m| m.kept)
                    .count()
                    .to_string(),
            )
            .with_field("ambient-ticks", ui.ambient.ticks.to_string())
            .with_field("ambient-error", ui.ambient.error.is_some().to_string());
        if let Some(detail) = ui.detail() {
            snapshot = snapshot
                .with_field(
                    "die",
                    (detail.field.rules == crate::UNIFORM_DIE_RULE).to_string(),
                )
                .with_field(
                    "die-seeded",
                    detail
                        .readings
                        .first()
                        .is_some_and(|r| r.receipt.mode == crate::SelectionMode::Derived)
                        .to_string(),
                )
                .with_field("inference-count", detail.inferences.len().to_string())
                .with_field(
                    "inference-kept-count",
                    detail.kept_inferences.len().to_string(),
                )
                .with_field(
                    "inference-formula-count",
                    detail
                        .inferences
                        .iter()
                        .filter(|r| {
                            matches!(r.generation, crate::inference::Generation::Formula { .. })
                        })
                        .count()
                        .to_string(),
                )
                .with_field(
                    "inference-model-count",
                    detail
                        .inferences
                        .iter()
                        .filter(|r| {
                            matches!(r.generation, crate::inference::Generation::Model { .. })
                        })
                        .count()
                        .to_string(),
                )
                .with_field("situation", detail.situation.is_some().to_string())
                .with_field(
                    "situation-place",
                    detail
                        .situation
                        .as_ref()
                        .is_some_and(|s| s.place.is_some())
                        .to_string(),
                )
                .with_field(
                    "situation-weather",
                    detail
                        .situation
                        .as_ref()
                        .is_some_and(|s| s.weather.is_some())
                        .to_string(),
                )
                .with_field(
                    "situation-positions",
                    detail
                        .situation
                        .as_ref()
                        .map_or(0, |s| s.sky.positions.len())
                        .to_string(),
                )
                .with_field(
                    "future-place-consent",
                    (ui.place_consent.selected == 1).to_string(),
                )
                .with_field(
                    "future-weather-consent",
                    (ui.weather_consent.selected == 1).to_string(),
                )
                .with_field("product-error", ui.error().unwrap_or(""));
            snapshot = snapshot
                .with_field(
                    "shuffled",
                    detail
                        .readings
                        .iter()
                        .all(|r| r.receipt.shuffle.is_some())
                        .to_string(),
                )
                .with_field(
                    "unique-cards",
                    detail
                        .readings
                        .iter()
                        .map(|r| &r.candidate_id)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        .to_string(),
                )
                .with_field(
                    "reversed-cards",
                    detail
                        .readings
                        .iter()
                        .filter(|r| r.orientation() == crate::Orientation::Reversed)
                        .count()
                        .to_string(),
                );
        }
        if let Some(c) = &ui.ambient.context {
            snapshot = snapshot
                .with_field(
                    "ambient-scene",
                    c.view.snapshot.recipe.definition.arrangement.kind.clone(),
                )
                .with_field("ambient-still", c.view.reduced_motion.to_string())
                .with_field("ambient-paused", c.view.paused.to_string())
                .with_field("ambient-families", c.view.families.len().to_string())
                .with_field(
                    "ambient-focus",
                    c.view
                        .snapshot
                        .selected_occurrence
                        .clone()
                        .unwrap_or_default(),
                );
        }
        snapshot
    }
    fn viewport(&self, ctx: &ConsultationCtx<'_>) -> Option<mesquite::Viewport> {
        // Shared workspace captures need both the mere and its focused reading.
        // The ordinary consultation crop would discard the owner controls.
        if ctx.runner.state().appearance.editor_open || ctx.runner.state().mere.open {
            return None;
        }
        let node = taproot::matching_classes(
            &ctx.runner.dom().borrow(),
            &taproot::Selector::class("cleromancy-consultation"),
        )
        .into_iter()
        .next()?;
        let (x, y, width, height) = ctx.painted_rect(node)?;
        let scale = ctx.window?.scale_factor() as f32 * ctx.ui_zoom;
        Some(mesquite::Viewport::new([x, y, width, height], scale, 1.0))
    }
    fn drain_events(&mut self, _: &mut ConsultationCtx<'_>) -> Vec<String> {
        std::mem::take(&mut self.state.borrow_mut().probe_events)
    }
    fn busy(&self, ctx: &ConsultationCtx<'_>, capture_pending: bool) -> Option<bool> {
        Some(
            capture_pending
                || !self.state.borrow().catalog_ready
                || ctx.runner.state().consultation.status().is_busy()
                || ctx.runner.state().mere.busy,
        )
    }
    fn app_step(
        &mut self,
        ctx: &mut ConsultationCtx<'_>,
        _: mesquite::Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        ScenarioDriver {
            ctx,
            state: &mut self.state.borrow_mut(),
        }
        .app_step(line)
    }
    fn complete(
        &mut self,
        ctx: &mut ConsultationCtx<'_>,
        outcome: &taproot::Outcome,
    ) -> Result<(), String> {
        if matches!(self.phase, Phase::NatalFirst | Phase::NatalReopen) {
            let ui = &ctx.runner.state().consultation;
            let ok = outcome.ok
                && self.state.borrow().catalog_ready
                && !ui.catalog.natal_readings.is_empty()
                && !ui.catalog.natal_reflections.is_empty();
            let receipt = serde_json::json!({"schema":"cleromancy.natal-native/v1","phase":self.phase,"ok":ok,"log":outcome.log,"readings":ui.catalog.natal_readings,"reflections":ui.catalog.natal_reflections,"shared":ui.birth_catalog,"error":ui.error(),"probe_events":self.state.borrow().probe_events});
            std::fs::write(
                self.dir.join("receipt.json"),
                serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            return if ok {
                Ok(())
            } else {
                Err("Natal scenario requires saved readings and reflections".into())
            };
        }
        if let Some(context) = &ctx.runner.state().consultation.ambient.context {
            std::fs::write(self.dir.join("ambient.json"),serde_json::to_vec_pretty(&serde_json::json!({"view":context.view,"data":context.data,"memory":ctx.runner.state().consultation.catalog.ambient_memory,"ticks":ctx.runner.state().consultation.ambient.ticks,"error":ctx.runner.state().consultation.ambient.error})) .map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        }
        if let Some(detail) = ctx.runner.state().consultation.detail() {
            if detail.field.rules == crate::UNIFORM_DIE_RULE {
                std::fs::write(
                    self.dir.join("die-reading.json"),
                    serde_json::to_vec_pretty(detail).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            }
            if !detail.inferences.is_empty() {
                std::fs::write(
                    self.dir.join("generated-reading.json"),
                    serde_json::to_vec_pretty(detail).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            }
            if detail.situation.is_some() {
                std::fs::write(
                    self.dir.join("situated-reading.json"),
                    serde_json::to_vec_pretty(detail).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            }
            if detail.readings.iter().any(|r| r.receipt.shuffle.is_some()) {
                let proof = serde_json::json!({"context":detail.context,"field":detail.field,"session":detail.session,"readings":detail.readings,"template":detail.spread_template,"reflections":detail.reflections});
                std::fs::write(
                    self.dir.join("shuffled-reading.json"),
                    serde_json::to_vec_pretty(&proof).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            }
        }
        scenario::write_done(
            &self.dir,
            self.phase,
            outcome,
            observation(ctx.runner, self.state.borrow().catalog_ready),
        )
    }
    fn close_on_completion(&self) -> bool {
        !self.keep_open
    }
}
