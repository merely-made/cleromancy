// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy commands and durable receipts over the shared Mesquite lane.

use super::appearance_view::{ApplicationUi, Child};
use super::{
    ApplicationLogic as Logic, ConsultationCtx, ConsultationRunner, NativeState, SHEET,
    submit_pending,
};
use crate::ui::scenario::{self, Observation, Phase, ReportedIds};
use cambium_genet_winit_host::AppCtx;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub(super) struct Product {
    state: Rc<RefCell<NativeState>>,
    dir: PathBuf,
    phase: Phase,
    keep_open: bool,
    sheet: String,
}

pub(super) fn drive(lane: &mut mesquite::Lane<Product>, ctx: &mut ConsultationCtx<'_>) {
    lane.product_mut().sheet = ctx.runner.state().appearance.stylesheet();
    lane.after_frame(ctx);
}

pub(super) fn lane(state: Rc<RefCell<NativeState>>, run: scenario::Run) -> mesquite::Lane<Product> {
    let product = Product {
        sheet: format!(
            "{}\n{}\n{}",
            SHEET,
            cambium::TITLE_BAR_CSS,
            super::appearance_view::CHROME_CSS
        ),
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
    .with_presentation_timeout(presentation_timeout(
        std::env::var("CLEROMANCY_PRESENTATION_TIMEOUT_SECS").ok().as_deref(),
    ))
}

fn presentation_timeout(value: Option<&str>) -> std::time::Duration {
    std::time::Duration::from_secs(
        value
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(10, 60),
    )
}

#[cfg(test)]
mod presentation_tests {
    use super::presentation_timeout;

    #[test]
    fn scenario_allowance_keeps_the_default_and_refuses_unbounded_waits() {
        for (value, seconds) in [
            (None, 10),
            (Some("invalid"), 10),
            (Some("-1"), 10),
            (Some("0"), 10),
            (Some("30"), 30),
            (Some("60"), 60),
            (Some("18446744073709551615"), 60),
            (Some("18446744073709551616"), 10),
        ] {
            assert_eq!(presentation_timeout(value).as_secs(), seconds);
        }
    }
}

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
        let appearance = &ctx.runner.state().appearance;
        snapshot = snapshot
            .with_field("appearance-id", appearance.active_id())
            .with_field("font-reading", appearance.font_preferences().0.get(&tabard::typography::FontRole::Reading).and_then(|stack| stack.first()).map_or("default", |family| family.name()))
            .with_field("font-heading", appearance.font_preferences().0.get(&tabard::typography::FontRole::Heading).and_then(|stack| stack.first()).map_or("default", |family| family.name()))
            .with_field("font-label", appearance.font_preferences().0.get(&tabard::typography::FontRole::Label).and_then(|stack| stack.first()).map_or("default", |family| family.name()))
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
        snapshot
    }
    fn viewport(&self, ctx: &ConsultationCtx<'_>) -> Option<mesquite::Viewport> {
        if ctx.runner.state().appearance.editor_open {
            return None;
        }
        let node = taproot::matching(
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
                || ctx.runner.state().consultation.status().is_busy(),
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
    ctx: &'a mut AppCtx<'ctx, ApplicationUi, Logic, Child>,
    state: &'a mut NativeState,
}

impl ScenarioDriver<'_, '_> {
    fn app_step(&mut self, line: &str) -> Result<(), String> {
        match line {
            "advance" => self.advance(),
            "show-chart" => {
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
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
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
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
        self.ctx.runner.update(|state| {
            let ui = &mut state.consultation;
            match phase {
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
            }
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
    let ui = &runner.state().consultation;
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
