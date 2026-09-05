// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The application-specific half of Cleromancy's headed scenario receipt.

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{AppCtx, HostPointer, read_frame};
use image::ImageEncoder as _;

use super::{ConsultationCtx, ConsultationRunner, Logic, NativeState, SHEET, submit_pending};
use crate::ui::scenario::{self, Observation, Phase, ReportedIds};
use crate::{ConsultationUi, ConsultationView};

pub(super) fn arm_capture(ctx: &mut ConsultationCtx<'_>, state: &Rc<RefCell<NativeState>>) {
    let path = state.borrow_mut().pending_capture.take();
    let Some(path) = path else { return };
    let capture_error = state.clone();
    *ctx.capture = Some(Box::new(move |surface, view, width, height| {
        let result = read_frame(surface, view, width, height)
            .ok_or_else(|| "read frame".to_string())
            .and_then(|frame| write_png(&path, &frame.rgba, frame.width, frame.height));
        if let Err(error) = result {
            capture_error.borrow_mut().capture_error = Some(format!("{}: {error}", path.display()));
        }
    }));
}

pub(super) fn after_frame(ctx: &mut ConsultationCtx<'_>, state: &Rc<RefCell<NativeState>>) {
    let run = state.borrow_mut().scenario.take();
    let Some(mut run) = run else { return };
    let progress = {
        let mut state = state.borrow_mut();
        let mut driver = ScenarioDriver {
            ctx,
            state: &mut state,
        };
        run.scenario.tick(&mut driver)
    };
    if progress == genet_probe::Progress::Done && state.borrow().pending_capture.is_none() {
        let state = state.borrow();
        let outcome = run.scenario.finish();
        scenario::write_done(
            &run.dir,
            run.phase,
            &outcome,
            observation(ctx.runner, state.catalog_ready),
            state.capture_error.as_deref(),
        );
        *ctx.close = true;
    } else {
        state.borrow_mut().scenario = Some(run);
        if let Some(window) = ctx.window {
            window.request_redraw();
        }
    }
}

struct ScenarioDriver<'a, 'ctx> {
    ctx: &'a mut AppCtx<'ctx, ConsultationUi, Logic, ConsultationView>,
    state: &'a mut NativeState,
}

impl genet_probe::Automatable for ScenarioDriver<'_, '_> {
    fn with_surfaces<R>(&self, f: impl FnOnce(&[genet_probe::ProbeSurface<'_>]) -> R) -> R {
        let dom = self.ctx.runner.dom();
        let dom = dom.borrow();
        let (width, height) = self.ctx.logical_size;
        f(&[genet_probe::ProbeSurface {
            name: "cleromancy",
            dom: &dom,
            rect: [0.0, 0.0, width, height],
            sheet: SHEET,
        }])
    }

    fn snapshot(&self) -> genet_probe::ProbeSnapshot {
        let observed = observation(self.ctx.runner, self.state.catalog_ready);
        let mut snapshot = genet_probe::ProbeSnapshot::default()
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

    fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.state.probe_events)
    }

    fn act(&mut self, _label: &str) -> bool {
        false
    }

    fn press(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Press(x, y));
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Moved(x, y));
    }

    fn release(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Release(x, y));
    }

    fn busy(&mut self) -> Option<bool> {
        Some(!self.state.catalog_ready || self.ctx.runner.state().status().is_busy())
    }
}

impl genet_probe::Driveable for ScenarioDriver<'_, '_> {
    fn capture(&mut self, name: &str) -> bool {
        if name.is_empty() || name.contains(['/', '\\']) {
            return false;
        }
        let Some(dir) = self.state.capture_dir.as_ref() else {
            return false;
        };
        self.state.pending_capture = Some(dir.join(format!("{name}.png")));
        true
    }

    fn app_step(&mut self, line: &str) -> Result<(), String> {
        match line {
            "advance" => self.advance(),
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
                    ui.context_label = cambium::TextInput::new("H3 durable threshold");
                    ui.question = cambium::TextInput::new(
                        "What deserves attention before this threshold changes?",
                    );
                    ui.tags = cambium::TextInput::new("change, reflection, threshold");
                    ui.field_select.selected = 0;
                    ui.mode.selected = 0;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                }
                1 => {
                    ui.reflection = cambium::TextInput::new(
                        "Keep the useful constraint revisable after the threshold moves.",
                    );
                    let action = ui.request_reflection();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                }
                _ => error = Some("first scenario has no further semantic action".to_string()),
            },
            Phase::Reopen => {
                let Some(session) = ui.catalog.session_summaries.first() else {
                    error = Some("the reopened catalog has no saved session".to_string());
                    return;
                };
                let action = ui.request_session(session.session_id.clone());
                ui.record_action(Some(action));
            }
        });
        if let Some(error) = error {
            return Err(error);
        }
        self.state.scenario_stage += 1;
        self.state.probe_events.push(match phase {
            Phase::First if self.state.scenario_stage == 1 => {
                "semantic consultation authored".to_string()
            }
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
    }
}

fn write_png(path: &std::path::Path, rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    image::codecs::png::PngEncoder::new(file)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|error| error.to_string())
}
