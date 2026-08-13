// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's small native Genet host.
//!
//! This module deliberately owns only the window, retained DOM/layout/render
//! pipeline, and event routing. The graph controller and Redb persistence live
//! in [`crate::ui::worker`], which publishes durable results back to this
//! loop. `render` owns the frame and input dispatch, `handler` the winit
//! lifecycle, and `probe` the genet-probe surface.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use accesskit::NodeId as AccessNodeId;
use cambium::{GenetAppRunner, Modifiers};
use genet_layout::IncrementalLayout;
use genet_scripted_dom::NodeId;
use genet_winit_host::{AccessKitBridge, SurfaceHost};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::ui::scenario::{self, Observation, Phase, ReportedIds};
use crate::ui::worker::{ConsultationWorker, WorkerUpdate, spawn};
use crate::{ConsultationAction, ConsultationUi, ConsultationView};

mod handler;
mod probe;
mod render;

type Runner = GenetAppRunner<
    ConsultationUi,
    fn(&ConsultationUi) -> ConsultationView,
    ConsultationView,
    ConsultationAction,
>;

#[derive(Clone, Debug)]
enum HostEvent {
    Worker(WorkerUpdate),
}

const SCENARIO_TICK: Duration = Duration::from_millis(16);

/// Start the ordinary local consultation window.
pub fn run(data_root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(data_root)
        .map_err(|error| format!("create {}: {error}", data_root.display()))?;
    let scenario = scenario::load();
    let capture_dir = scenario.as_ref().map(|run| run.dir.clone());
    let scenario_phase = scenario.as_ref().map(|run| run.phase);
    let event_loop = EventLoop::<HostEvent>::with_user_event()
        .build()
        .map_err(|error| format!("create native event loop: {error}"))?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let worker = spawn(data_root.join("cleromancy.redb"), move |update| {
        let _ = proxy.send_event(HostEvent::Worker(update));
    });
    let mut app = NativeApp {
        window: None,
        host: None,
        runner: None,
        layout: None,
        layout_size: (0.0, 0.0),
        cursor: (0.0, 0.0),
        modifiers: Modifiers::default(),
        a11y: None,
        a11y_route: HashMap::new(),
        worker,
        queued_updates: Vec::new(),
        catalog_ready: false,
        scenario,
        scenario_phase,
        scenario_stage: 0,
        capture_dir,
        pending_capture: None,
        capture_error: None,
        probe_events: Vec::new(),
    };
    event_loop
        .run_app(&mut app)
        .map_err(|error| format!("run native event loop: {error}"))
}

struct NativeApp {
    window: Option<Arc<Window>>,
    host: Option<SurfaceHost>,
    runner: Option<Runner>,
    /// The retained layout is both the hit-test and paint authority.
    layout: Option<IncrementalLayout<NodeId>>,
    layout_size: (f32, f32),
    cursor: (f32, f32),
    modifiers: Modifiers,
    a11y: Option<AccessKitBridge>,
    a11y_route: HashMap<AccessNodeId, NodeId>,
    worker: ConsultationWorker,
    queued_updates: Vec<WorkerUpdate>,
    /// The worker has published the initial durable picker/history catalog.
    catalog_ready: bool,
    /// An optional genet-probe run, advanced by `new_events` in `handler`.
    scenario: Option<scenario::Run>,
    scenario_phase: Option<Phase>,
    scenario_stage: u8,
    capture_dir: Option<PathBuf>,
    pending_capture: Option<PathBuf>,
    capture_error: Option<String>,
    probe_events: Vec<String>,
}

impl NativeApp {
    fn submit_actions(&mut self, actions: Vec<ConsultationAction>) {
        for action in actions {
            if let Err(message) = self.worker.command(action) {
                if let Some(runner) = self.runner.as_mut() {
                    runner.update(|ui| ui.present_error(message));
                }
            }
        }
    }

    fn apply_worker_update(&mut self, update: WorkerUpdate) {
        let Some(runner) = self.runner.as_mut() else {
            self.queued_updates.push(update);
            return;
        };
        match update {
            WorkerUpdate::Catalog(catalog) => {
                self.catalog_ready = true;
                runner.update(|ui| ui.replace_catalog(catalog));
            }
            #[cfg(feature = "ephemeris")]
            WorkerUpdate::Ephemeris(status) => {
                runner.update(|ui| ui.present_ephemeris_status(status));
            }
            #[cfg(feature = "ephemeris")]
            WorkerUpdate::AstrologyChart {
                catalog,
                facts_digest,
            } => {
                self.probe_events
                    .push(format!("durable astrology chart saved {facts_digest}"));
                runner.update(move |ui| ui.present_calculated_chart(catalog, facts_digest));
            }
            WorkerUpdate::Reading { catalog, detail } => {
                self.probe_events
                    .push(format!("durable reading saved {}", detail.session.id));
                runner.update(move |ui| ui.present_reading(catalog, detail));
            }
            WorkerUpdate::Reflection { catalog, detail } => {
                self.probe_events
                    .push(format!("durable reflection saved {}", detail.session.id));
                runner.update(move |ui| ui.present_reflection(catalog, detail));
            }
            WorkerUpdate::Session { catalog, detail } => {
                self.probe_events
                    .push(format!("durable session recovered {}", detail.session.id));
                runner.update(move |ui| ui.present_session(catalog, detail));
            }
            WorkerUpdate::Comparison {
                catalog,
                detail,
                comparison,
            } => {
                self.probe_events.push(format!(
                    "durable receipts compared {} {}",
                    comparison.left_session_id, comparison.right_session_id
                ));
                runner.update(move |ui| ui.present_comparison(catalog, detail, comparison));
            }
            WorkerUpdate::Error { catalog, message } => {
                runner.update(move |ui| {
                    if let Some(catalog) = catalog {
                        ui.replace_catalog(catalog);
                    }
                    ui.present_error(message);
                });
            }
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn flush_queued_updates(&mut self) {
        for update in std::mem::take(&mut self.queued_updates) {
            self.apply_worker_update(update);
        }
    }

    /// The app-specific scenario verb. It changes only the typed controls then
    /// invokes the same product actions as the visible buttons; the probe does
    /// not synthesize a different persistence route.
    fn scenario_advance(&mut self) -> Result<(), String> {
        let phase = self
            .scenario_phase
            .ok_or_else(|| "advance is available only during a scenario run".to_string())?;
        if matches!(phase, Phase::Reopen) && self.scenario_stage > 0 {
            self.probe_events
                .push("semantic headed scenario settled".to_string());
            self.scenario_stage += 1;
            return Ok(());
        }
        let Some(runner) = self.runner.as_mut() else {
            return Err("the headed consultation surface is not ready".to_string());
        };
        let mut action = None;
        let mut error = None;
        runner.update(|ui| match phase {
            Phase::First => match self.scenario_stage {
                0 => {
                    ui.context_label = cambium::TextInput::new("H3 durable threshold");
                    ui.question = cambium::TextInput::new(
                        "What deserves attention before this threshold changes?",
                    );
                    ui.tags = cambium::TextInput::new("change, reflection, threshold");
                    ui.field_select.selected = 0;
                    ui.mode.selected = 0;
                    action = ui.request_read();
                    error = ui.error().map(str::to_string);
                }
                1 => {
                    ui.reflection = cambium::TextInput::new(
                        "Keep the useful constraint revisable after the threshold moves.",
                    );
                    action = ui.request_reflection();
                    error = ui.error().map(str::to_string);
                }
                _ => error = Some("first scenario has no further semantic action".to_string()),
            },
            Phase::Reopen => {
                let Some(session) = ui.catalog.sessions.first() else {
                    error = Some("the reopened catalog has no saved session".to_string());
                    return;
                };
                action = Some(ui.request_session(session.id.clone()));
            }
        });
        let action = action.ok_or_else(|| {
            error.unwrap_or_else(|| "scenario action did not produce a product command".to_string())
        })?;
        self.scenario_stage += 1;
        self.probe_events.push(match phase {
            Phase::First if self.scenario_stage == 1 => {
                "semantic consultation authored".to_string()
            }
            Phase::First => "semantic reflection authored".to_string(),
            Phase::Reopen => "semantic recovered session selected".to_string(),
        });
        self.submit_actions(vec![action]);
        Ok(())
    }

    fn scenario_observation(&self) -> Observation {
        let Some(runner) = self.runner.as_ref() else {
            return Observation {
                status: "surface unavailable".to_string(),
                catalog_ready: self.catalog_ready,
                sessions: 0,
                readings: 0,
                reflections: 0,
                ids: None,
            };
        };
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
            catalog_ready: self.catalog_ready,
            sessions: ui.catalog().sessions.len(),
            readings: detail.map_or(0, |detail| detail.readings.len()),
            reflections: detail.map_or(0, |detail| detail.reflections.len()),
            ids,
        }
    }
}

const SHEET: &str = r#"
* { box-sizing: border-box; }
.cleromancy-consultation {
  min-height: 100vh;
  padding: 24px;
  color: #f1ede4;
  background: #181714;
  font-family: sans-serif;
}
.app-header { border-bottom: 1px solid #625d50; padding-bottom: 12px; }
.cleromancy-regions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}
.eyebrow { color: #d7b46a; font-size: 13px; text-transform: uppercase; }
h1, h2, h3, p { margin-top: 0; }
section[role='region'] {
  display: block;
  padding: 18px;
  border: 1px solid #625d50;
  border-radius: 10px;
  background: #24221d;
}
.control { display: block; margin: 0 0 12px; }
.control-label { display: block; margin-bottom: 5px; color: #d7c9a9; font-size: 13px; }
input, textarea, select, button {
  width: 100%; padding: 9px 10px; border: 1px solid #7b725f; border-radius: 5px;
  color: #f7f2e7; background: #302d26; font: inherit;
}
textarea { min-height: 90px; resize: vertical; }
button { margin-top: 4px; cursor: pointer; background: #6e522a; }
button:focus, input:focus, textarea:focus, select:focus { outline: 2px solid #d7b46a; outline-offset: 2px; }
[role='alert'] { padding: 10px; color: #ffd8d2; background: #542d29; }
[role='status'] { color: #d7c9a9; }
.selection-explanation, .empty-reading { color: #c4bcad; font-size: 14px; }
"#;
