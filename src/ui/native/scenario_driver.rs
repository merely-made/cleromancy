// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy commands and durable receipts over the shared Mesquite lane.

use super::{ConsultationCtx, ConsultationRunner, Logic, NativeState, SHEET, submit_pending};
use crate::ui::scenario::{self, Observation, Phase, ReportedIds};
use crate::{ConsultationUi, ConsultationView};
use cambium_genet_winit_host::AppCtx;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub(super) struct Product {
    state: Rc<RefCell<NativeState>>,
    dir: PathBuf,
    phase: Phase,
    keep_open: bool,
}

pub(super) fn lane(state: Rc<RefCell<NativeState>>, run: scenario::Run) -> mesquite::Lane<Product> {
    let product = Product {
        state,
        dir: run.dir.clone(),
        phase: run.phase,
        keep_open: std::env::var("CLEROMANCY_SCENARIO_KEEP_OPEN").as_deref() == Ok("1"),
    };
    mesquite::Lane::from_config(
        mesquite::LaneConfig {
            scenario: run.path,
            capture_dir: Some(run.dir.clone()),
            // The shared receipt includes capture failures and timings. The
            // product also preserves its durable close/reopen receipt below.
            receipt: Some(run.dir.join("lane.done")),
        },
        product,
        cambium_genet_winit_host::read_file,
    )
    .expect("load Cleromancy scenario")
}

impl mesquite::Product for Product {
    type State = ConsultationUi;
    type Logic = Logic;
    type View = ConsultationView;
    const KIND: &'static str = "cleromancy";
    const SURFACE: &'static str = "cleromancy";
    const LOG_PREFIX: &'static str = "cleromancy";

    fn sheet(&self) -> &str {
        SHEET
    }

    fn snapshot(&self, ctx: &ConsultationCtx<'_>, _: usize, _: f32) -> taproot::ProbeSnapshot {
        let observed = observation(ctx.runner, self.state.borrow().catalog_ready);
        let mut snapshot = taproot::ProbeSnapshot::default()
            .with_field("status", observed.status)
            .with_field("catalog-ready", observed.catalog_ready.to_string())
            .with_field("sessions", observed.sessions.to_string())
            .with_field("readings", observed.readings.to_string())
            .with_field("reflections", observed.reflections.to_string());
        if let Some(ids) = observed.ids {
            snapshot = snapshot
                .with_field("session-id", ids.session_id)
                .with_field("reading-id", ids.reading_id)
                .with_field("reflection-id", ids.reflection_id);
        }
        snapshot
    }
    fn drain_events(&mut self, _: &mut ConsultationCtx<'_>) -> Vec<String> {
        std::mem::take(&mut self.state.borrow_mut().probe_events)
    }
    fn busy(&self, ctx: &ConsultationCtx<'_>, capture_pending: bool) -> Option<bool> {
        Some(
            capture_pending
                || !self.state.borrow().catalog_ready
                || ctx.runner.state().status().is_busy(),
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

struct ScenarioDriver<'a, 'ctx> {
    ctx: &'a mut AppCtx<'ctx, ConsultationUi, Logic, ConsultationView>,
    state: &'a mut NativeState,
}

impl ScenarioDriver<'_, '_> {
    fn app_step(&mut self, line: &str) -> Result<(), String> {
        match line {
            "advance" => self.advance(),
            "show-chart" => {
                self.ctx.runner.update(|ui| {
                    crate::ui::screen::select_surface(
                        &mut ui.surface_tabs,
                        crate::ui::screen::ConsultationScreen::Chart,
                    );
                });
                Ok(())
            },
            #[cfg(feature = "analytic-ephemeris")]
            _ if line.starts_with("calculate-chart ") => {
                let instant = line.trim_start_matches("calculate-chart ");
                let mut error = None;
                self.ctx.runner.update(|ui| {
                    ui.astrology_instant_utc = cambium::TextInput::new(instant);
                    let action = ui.request_calculated_astrology_chart();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            _ => Err(format!("unknown Cleromancy scenario verb: {line}")),
        }
    }
}

impl ScenarioDriver<'_, '_> {
    fn advance(&mut self) -> Result<(), String> {
        let phase = self
            .state
            .scenario_phase
            .ok_or_else(|| "advance is available only during a scenario run".to_string())?;
        if matches!(phase, Phase::Reopen) && self.state.scenario_stage > 0 {
            self.state
                .probe_events
                .push("semantic headed scenario settled".to_string());
            self.state.scenario_stage += 1;
            return Ok(());
        }
        let mut error = None;
        self.ctx.runner.update(|ui| match phase {
            Phase::First => match self.state.scenario_stage {
                0 => {
                    ui.context_label = cambium::TextInput::new("Cleromancy and Turquet");
                    ui.question = cambium::TextInput::new(
                        "What deserves attention as these projects become something I can use?",
                    );
                    ui.tags = cambium::TextInput::new("work, creativity, reflection");
                    ui.field_select.selected = 0;
                    ui.mode.selected = 1;
                    ui.layout.selected = 1;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                },
                1 => {
                    ui.reflection = cambium::TextInput::new(
                        "Trial note: return after using the app and record what resonated.",
                    );
                    let action = ui.request_reflection();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                },
                _ => error = Some("first scenario has no further semantic action".to_string()),
            },
            Phase::Reopen => {
                let Some(session) = ui.catalog.session_summaries.first() else {
                    error = Some("the reopened catalog has no saved session".to_string());
                    return;
                };
                let action = ui.request_session(session.session_id.clone());
                ui.record_action(Some(action));
            },
        });
        if let Some(error) = error {
            return Err(error);
        }
        self.state.scenario_stage += 1;
        self.state.probe_events.push(match phase {
            Phase::First if self.state.scenario_stage == 1 => {
                "semantic consultation authored".to_string()
            },
            Phase::First => "semantic reflection authored".to_string(),
            Phase::Reopen => "semantic recovered session selected".to_string(),
        });
        submit_pending(self.ctx.runner, self.state.worker.as_ref());
        Ok(())
    }
}

fn observation(runner: &ConsultationRunner, catalog_ready: bool) -> Observation {
    let ui = runner.state();
    let detail = ui.detail();
    let ids = detail.and_then(|detail| {
        Some(ReportedIds {
            session_id: detail.session.id.clone(),
            reading_id: detail.readings.first()?.id.clone(),
            reflection_id: detail.reflections.first()?.id.clone(),
        })
    });
    Observation {
        status: ui.status().label(),
        catalog_ready,
        sessions: ui.catalog().session_summaries.len(),
        readings: detail.map_or(0, |detail| detail.readings.len()),
        reflections: detail.map_or(0, |detail| detail.reflections.len()),
        ids,
        cards: detail.map_or_else(Vec::new, |detail| {
            detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| scenario::ReadingPreview {
                    position: placement.position.clone(),
                    title: reading.title.clone(),
                    prompt: reading.interpretation.clone(),
                    mode: reading.receipt.mode,
                })
                .collect()
        }),
    }
}
