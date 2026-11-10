// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy commands and durable receipts over the shared Mesquite lane.

use super::appearance_view::{ApplicationUi, Child};
use super::{
    ApplicationLogic as Logic, ConsultationCtx, ConsultationRunner, NativeState, SHEET,
    submit_mere_pending, submit_pending,
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

pub(super) fn drive(lane: &mut mandrake::Lane<Product>, ctx: &mut ConsultationCtx<'_>) {
    lane.product_mut().sheet = ctx.runner.state().appearance.stylesheet();
    lane.after_frame(ctx);
}

pub(super) fn lane(state: Rc<RefCell<NativeState>>, run: scenario::Run) -> mandrake::Lane<Product> {
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
    mandrake::Lane::from_config(
        mandrake::LaneConfig {
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
        std::env::var("CLEROMANCY_PRESENTATION_TIMEOUT_SECS")
            .ok()
            .as_deref(),
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
mod presentation_tests;

struct ScenarioDriver<'a, 'ctx> {
    ctx: &'a mut AppCtx<'ctx, ApplicationUi, Logic, Child>,
    state: &'a mut NativeState,
}

impl ScenarioDriver<'_, '_> {
    fn control_step(&mut self, line: &str, command: &str) -> Result<(), String> {
        let (action, key) = command.split_once(' ').ok_or("C4 expects action and key")?;
        let key = if key == "natal-latest-reflect" || key == "natal-latest-example" {
            let r = self
                .ctx
                .runner
                .state()
                .consultation
                .catalog
                .natal_readings
                .first()
                .ok_or("no natal occasion")?;
            format!(
                "natal-{}-{}",
                if key == "natal-latest-reflect" {
                    "reflect"
                } else {
                    "example"
                },
                r.id
            )
        } else if key == "c8-saved-os" || key == "c8-saved-seed" {
            let label = if key == "c8-saved-os" {
                "C8 operating-system roll"
            } else {
                "C8 public-seed roll"
            };
            let summary = self
                .ctx
                .runner
                .state()
                .consultation
                .catalog
                .session_summaries
                .iter()
                .find(|s| s.context_label == label)
                .ok_or("no saved C8 occasion")?;
            format!("session:{}", summary.session_id)
        } else if key == "inference-keep-model" {
            let detail = self
                .ctx
                .runner
                .state()
                .consultation
                .detail()
                .ok_or("no current reading")?;
            let record = detail
                .inferences
                .iter()
                .find(|r| matches!(r.generation, crate::inference::Generation::Model { .. }))
                .ok_or("no stored model prompt")?;
            format!("inference-keep:{}", record.digest())
        } else if key == "history" {
            let data = self
                .ctx
                .runner
                .state()
                .consultation
                .neighborhood_data()
                .ok_or("no C4 neighborhood")?;
            format!(
                "ambient-item:{}",
                data.items
                    .iter()
                    .find(|i| i.id.starts_with("history:"))
                    .ok_or("no earlier occurrence")?
                    .id
            )
        } else {
            key.into()
        };
        if line.starts_with("c8-control-") && key == "read" {
            let ui = &self.ctx.runner.state().consultation;
            if !ui
                .catalog
                .fields
                .get(ui.field_select.selected)
                .is_some_and(|f| f.rules == crate::UNIFORM_DIE_RULE)
            {
                return Err("C8 fixture refuses to submit a non-die reading".into());
            }
        }
        if line.starts_with("c8-control-") && key == "save-reflection" {
            let ui = &self.ctx.runner.state().consultation;
            let detail = ui
                .detail()
                .ok_or("C8 reflection requires a selected die occasion")?;
            if detail.field.rules != crate::UNIFORM_DIE_RULE
                || !detail.reflections.is_empty()
                || !matches!(
                    detail.context.label.as_str(),
                    "C8 operating-system roll" | "C8 public-seed roll"
                )
            {
                return Err("C8 reflection refuses to write to an older non-die occasion".into());
            }
        }
        let node = {
            let projection = self
                .ctx
                .a11y_projection()
                .ok_or("host accessibility projection unavailable for scenario control")?;
            let dom = self.ctx.runner.dom();
            let dom = dom.borrow();
            taproot::matching_with_projection(
                &dom,
                &taproot::Selector::role("button").with_attr("data-key", &key),
                &projection,
            )
            .into_iter()
            .next()
            .ok_or_else(|| format!("missing C4 mounted control {key}"))?
        };
        if action == "reveal" {
            self.ctx
                .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Start);
            return Ok(());
        }
        let painted = self.ctx.painted_rect(node).ok_or("C4 control unpainted")?;
        let visible = self.ctx.visible_rect(node).ok_or("C4 control invisible")?;
        if painted.2 <= 0.0
            || painted.3 <= 0.0
            || [
                painted.0 - visible.0,
                painted.1 - visible.1,
                painted.2 - visible.2,
                painted.3 - visible.3,
            ]
            .iter()
            .any(|v| v.abs() > 1.0)
        {
            return Err(format!(
                "C4 control clipped: {key}: {painted:?} {visible:?}"
            ));
        }
        eprintln!("cleromancy: C4 {action} {key}: painted={painted:?}, visible={visible:?}");
        match action {
            "click" => {
                self.ctx.runner.dispatch_click(
                    node,
                    cambium::PointerClick::at((painted.2 / 2.0, painted.3 / 2.0)),
                );
            },
            "key" => {
                self.ctx.runner.set_focus(Some(node));
                self.ctx
                    .runner
                    .dispatch_key(cambium::KeyEvent::new(cambium::Key::Named(
                        cambium::NamedKey::Enter,
                    )));
            },
            _ => return Err("unknown C4 action".into()),
        };
        submit_pending(self.ctx.runner, self.state.worker.as_ref());
        Ok(())
    }
}

impl ScenarioDriver<'_, '_> {
    // Read-only fixture selection over the real shared controls. Reveal uses
    // the same host-owned scroll path as pointer/accessibility target reveal.
    fn mere_control(&self, step: &str) -> Result<genet_scripted_dom::NodeId, String> {
        use layout_dom_api::LayoutDom;
        let model = &self.ctx.runner.state().mere.model;
        let container = match step {
            "fork" | "trash" | "save-archive" => {
                let id = model
                    .sessions
                    .iter()
                    .find(|session| session.attached)
                    .ok_or("no attached session for control reveal")?
                    .id;
                taproot::Selector::class("mere-view-session")
                    .with_attr("data-session", id.as_uuid().to_string())
            },
            "open-archive" => {
                let entry = model
                    .archive
                    .as_ref()
                    .and_then(|archive| archive.entries.first())
                    .ok_or("no archive for control reveal")?;
                taproot::Selector::class("mere-view-archive")
                    .with_attr("data-archive", entry.key.clone())
            },
            _ => return Err(format!("unknown read-only mere-control {step}")),
        };
        let dom = self.ctx.runner.dom();
        let dom = dom.borrow();
        let containers = taproot::matching_classes(&dom, &container);
        let projection = self
            .ctx
            .a11y_projection()
            .ok_or("host accessibility projection unavailable for mere control")?;
        taproot::matching_with_projection(
            &dom,
            &taproot::Selector::role("button").with_attr("data-step", step),
            &projection,
        )
        .into_iter()
        .find(|node| {
            let mut parent = dom.parent(*node);
            while let Some(node) = parent {
                if containers.contains(&node) {
                    return true;
                }
                parent = dom.parent(node);
            }
            false
        })
        .ok_or_else(|| format!("shared control is missing: {step}"))
    }

    fn mere_step(&mut self, line: &str) -> Result<(), String> {
        use layout_dom_api::LayoutDom;
        let model = &self.ctx.runner.state().mere.model;
        let attached = model
            .sessions
            .iter()
            .find(|session| session.attached)
            .map(|session| session.id);
        let mut container = None;
        let selector = match line {
            "show-mere" => {
                taproot::Selector::role("button").with_attr("data-action", "toggle-mere")
            },
            "mere-mint" => taproot::Selector::role("button").with_attr("data-request", "mint"),
            "mere-fork" | "mere-trash" | "mere-save" | "mere-switch-first"
            | "mere-restore-first" => {
                let id = match line {
                    "mere-switch-first" => model.sessions.iter().find(|s| !s.trashed).map(|s| s.id),
                    "mere-restore-first" => model.sessions.iter().find(|s| s.trashed).map(|s| s.id),
                    _ => attached,
                }
                .ok_or("no requested mere session")?;
                // The shared owner correctly omits Switch for the already
                // attached row. Trash can return to the original session before
                // Restore, so a fixture's final switch may already be satisfied.
                if line == "mere-switch-first" && attached == Some(id) {
                    self.state.probe_events.push(format!(
                        "shared switch target {} already attached; no action needed",
                        id.as_uuid()
                    ));
                    return Ok(());
                }
                container = Some(
                    taproot::Selector::class("mere-view-session")
                        .with_attr("data-session", id.as_uuid().to_string()),
                );
                let step = match line {
                    "mere-fork" => "fork",
                    "mere-trash" => "trash",
                    "mere-save" => "save-archive",
                    "mere-switch-first" => "switch",
                    _ => "restore",
                };
                taproot::Selector::role("button").with_attr("data-step", step)
            },
            "mere-open-archive" => {
                let entry = model
                    .archive
                    .as_ref()
                    .and_then(|a| a.entries.first())
                    .ok_or("no saved archive")?;
                taproot::Selector::role("button")
                    .with_attr("aria-label", format!("Open {} as session", entry.name))
            },
            "mere-select-occasion" => {
                let key = model
                    .graph
                    .nodes
                    .iter()
                    .find(|node| node.label == "Reading session")
                    .ok_or("no saved occasion node")?
                    .key
                    .clone();
                taproot::Selector::class("graph-canvas-swatch-node").with_attr("data-key", key)
            },
            "mere-select-reading" => {
                let reading = self
                    .ctx
                    .runner
                    .state()
                    .consultation
                    .detail()
                    .and_then(|d| d.readings.first())
                    .ok_or("no selected reading")?;
                let key = kernel::graph::Graph::node_namespace_id(&format!(
                    "cleromancy://reading/{}",
                    reading.id
                ))
                .to_string();
                taproot::Selector::class("graph-canvas-swatch-node").with_attr("data-key", key)
            },
            _ => return Err(format!("unknown mere fixture action {line}")),
        };
        let node = {
            let dom = self.ctx.runner.dom();
            let dom = dom.borrow();
            let containers = container
                .as_ref()
                .map(|selector| taproot::matching_classes(&dom, selector));
            let matches = match &selector.matcher {
                taproot::Match::Class(_) => taproot::matching_classes(&dom, &selector),
                taproot::Match::Role(_) => {
                    let projection = self
                        .ctx
                        .a11y_projection()
                        .ok_or("host accessibility projection unavailable for mere action")?;
                    taproot::matching_with_projection(&dom, &selector, &projection)
                },
            };
            matches
                .into_iter()
                .find(|node| {
                    let Some(containers) = &containers else {
                        return true;
                    };
                    let mut parent = dom.parent(*node);
                    while let Some(at) = parent {
                        if containers.contains(&at) {
                            return true;
                        }
                        parent = dom.parent(at);
                    }
                    false
                })
                .ok_or_else(|| format!("shared mounted control is missing: {selector:?}"))?
        };
        let (_, _, width, height) = self
            .ctx
            .painted_rect(node)
            .ok_or("shared control has no painted bounds")?;
        if width <= 0.0 || height <= 0.0 {
            return Err("shared control has empty painted bounds".into());
        }
        // Dispatch to the actual mounted shared component callback. The helper
        // chooses an id, never constructs the product/resident action itself.
        self.ctx
            .runner
            .dispatch_click(node, cambium::PointerClick::at((width / 2.0, height / 2.0)));
        submit_mere_pending(self.ctx.runner, self.state.worker.as_ref());
        Ok(())
    }

    /// `class data-key`: one keyed element of a class, as the receipts name it.
    fn keyed(&self, target: &str) -> Result<genet_scripted_dom::NodeId, String> {
        let (class, key) = target
            .split_once(' ')
            .ok_or_else(|| format!("expected 'class data-key', got {target:?}"))?;
        let dom = self.ctx.runner.dom();
        let dom = dom.borrow();
        taproot::matching_classes(
            &dom,
            &taproot::Selector::class(class).with_attr("data-key", key),
        )
        .into_iter()
        .next()
        .ok_or_else(|| format!("no .{class} with data-key {key}"))
    }

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
                Phase::NatalFirst | Phase::NatalReopen => {
                    error = Some("Use the explicit natal scenario actions".into());
                },
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
            Phase::NatalFirst | Phase::NatalReopen => "natal scenario".into(),
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

mod app_steps;

mod product;
