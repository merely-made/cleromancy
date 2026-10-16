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
mod presentation_tests {
    use super::presentation_timeout;

    #[test]
    fn resident_scenario_sources_parse_with_the_shared_assertion_grammar() {
        for (name, script) in [
            ("C6 global", include_str!("../../../receipts/c6-global.scn")),
            (
                "C6 global offline reopen",
                include_str!("../../../receipts/c6-global-reopen.scn"),
            ),
            (
                "C6 weather",
                include_str!("../../../receipts/c6-weather.scn"),
            ),
            (
                "C6 weather offline reopen",
                include_str!("../../../receipts/c6-weather-reopen.scn"),
            ),
            (
                "seed",
                include_str!("../../../receipts/c1-resident-seed.scn"),
            ),
            (
                "reopen",
                include_str!("../../../receipts/c1-resident-reopen.scn"),
            ),
            (
                "retained",
                include_str!("../../../receipts/c1-resident-retained.scn"),
            ),
            (
                "compact controls",
                include_str!("../../../receipts/c1-resident-compact-controls.scn"),
            ),
            (
                "reading narrative",
                include_str!("../../../receipts/c2-reading-narrative.scn"),
            ),
            (
                "reading reopen",
                include_str!("../../../receipts/c2-reading-reopen.scn"),
            ),
        ] {
            taproot::Scenario::parse(script)
                .unwrap_or_else(|error| panic!("{name} scenario: {error}"));
        }
        assert!(taproot::Scenario::parse("assert snap readings = 3").is_err());
    }

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
        if let Some(context) = &ctx.runner.state().consultation.ambient.context {
            std::fs::write(self.dir.join("ambient.json"),serde_json::to_vec_pretty(&serde_json::json!({"view":context.view,"data":context.data,"memory":ctx.runner.state().consultation.catalog.ambient_memory,"ticks":ctx.runner.state().consultation.ambient.ticks,"error":ctx.runner.state().consultation.ambient.error})) .map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        }
        if let Some(detail) = ctx.runner.state().consultation.detail() {
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

struct ScenarioDriver<'a, 'ctx> {
    ctx: &'a mut AppCtx<'ctx, ApplicationUi, Logic, Child>,
    state: &'a mut NativeState,
}

impl ScenarioDriver<'_, '_> {
    fn app_step(&mut self, line: &str) -> Result<(), String> {
        if let Some(command) = line
            .strip_prefix("c4-")
            .or_else(|| line.strip_prefix("c5-control-"))
            .or_else(|| line.strip_prefix("c7-control-"))
        {
            let (action, key) = command.split_once(' ').ok_or("C4 expects action and key")?;
            let key = if key == "inference-keep-model" {
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
            let node = {
                let dom = self.ctx.runner.dom();
                let dom = dom.borrow();
                taproot::matching(
                    &dom,
                    &taproot::Selector::role("button").with_attr("data-key", &key),
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
            return Ok(());
        }
        match line {
            "c6-prepare-global" | "c6-prepare-weather" => {
                let weather = line == "c6-prepare-weather";
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new(if weather {
                        "C6 public Greenwich weather"
                    } else {
                        "C6 without place"
                    });
                    ui.question =
                        cambium::TextInput::new("What detail of this occasion deserves attention?");
                    ui.tags = cambium::TextInput::default();
                    ui.additional_facts = cambium::TextInput::default();
                    ui.layout.selected = 1;
                    ui.mode.selected = 2;
                    ui.derived_seed = cambium::TextInput::new("C6 public situation");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c6/qualification");
                    ui.astrology_facts_select.selected = 0;
                    ui.set_place_for_draw(if weather {
                        Some(
                            crate::situation::ReadingPlace::new(51_476_900, 0)
                                .expect("public Greenwich coordinates"),
                        )
                    } else {
                        None
                    });
                    ui.set_weather_for_draw(weather);
                });
                self.state.scenario_stage = 1;
                Ok(())
            },
            "c5-read" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(field) = ui.catalog.fields.iter().position(|f| {
                        f.tarot
                            .as_ref()
                            .is_some_and(|d| d.id == "cleromancy.tarot/rws-complete-reflective-v1")
                    }) else {
                        error = Some("missing C5 typed deck".into());
                        return;
                    };
                    let Some(template) = ui
                        .catalog
                        .spread_templates
                        .iter()
                        .position(|t| t.label == "C2 Letting go")
                    else {
                        error = Some("missing saved custom layout".into());
                        return;
                    };
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new("C5 one shuffled deck");
                    ui.question = cambium::TextInput::new("What can I release next?");
                    ui.tags = cambium::TextInput::new("");
                    ui.additional_facts = cambium::TextInput::new("");
                    ui.field_select.selected = field;
                    ui.mode.selected = 2;
                    ui.derived_seed = cambium::TextInput::new("C5 public deck");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c5/qualification");
                    ui.layout.selected = 2;
                    ui.template_select.selected = template + 1;
                    ui.astrology_facts_select.selected = 0;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c3-read" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(index) = ui.catalog.fields.iter().position(|f| {
                        f.tarot
                            .as_ref()
                            .is_some_and(|d| d.id == "cleromancy.tarot/rws-complete-reflective-v1")
                    }) else {
                        error = Some("C3 RWS typed deck is missing".to_string());
                        return;
                    };
                    ui.context_label = cambium::TextInput::new("C3 patient courage");
                    ui.question = cambium::TextInput::new("What can I meet patiently?");
                    ui.tags = cambium::TextInput::new("courage");
                    ui.field_select.selected = index;
                    ui.mode.selected = 0;
                    ui.layout.selected = 0;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c3-install" | "c3-crosswalk" | "c3-return" => {
                let key = if line == "c3-install" {
                    "install-typed-decks"
                } else if line == "c3-crosswalk" {
                    "crosswalk:cleromancy.tarot/marseille-complete-reflective-v1"
                } else {
                    "return-drawn-card"
                };
                let node = {
                    let dom = self.ctx.runner.dom();
                    let dom = dom.borrow();
                    taproot::matching(
                        &dom,
                        &taproot::Selector::role("button").with_attr("data-key", key),
                    )
                    .into_iter()
                    .next()
                    .ok_or("C3 inspection control missing")?
                };
                let (_, _, width, height) = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("C3 inspection control is not painted")?;
                self.ctx
                    .runner
                    .dispatch_click(node, cambium::PointerClick::at((width / 2.0, height / 2.0)));
                if line == "c3-install" {
                    submit_pending(self.ctx.runner, self.state.worker.as_ref());
                }
                Ok(())
            },
            "c2-author-layout" => {
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.template_label = cambium::TextInput::new("C2 Letting go");
                    ui.template_positions = cambium::TextInput::new("foundation | What I must release\ntension | What resists\nnext_step | What I can try");
                    ui.template_relations = cambium::TextInput::new("tension | questions | foundation | presses on what I hold");
                    let action = ui.request_spread_template();
                    ui.record_action(action);
                });
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c2-read-layout" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(index) = ui
                        .catalog
                        .spread_templates
                        .iter()
                        .position(|t| t.label == "C2 Letting go")
                    else {
                        error = Some("C2 authored layout is missing".to_string());
                        return;
                    };
                    ui.context_label = cambium::TextInput::new("C2 change of habit");
                    ui.question = cambium::TextInput::new("What am I ready to set down?");
                    ui.tags = cambium::TextInput::new("change, reflection");
                    ui.additional_facts = cambium::TextInput::new("season: autumn");
                    ui.field_select.selected = 0;
                    ui.mode.selected = 1;
                    ui.layout.selected = 2;
                    ui.template_select.selected = index + 1;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            _ if line.starts_with("assert visible key ") => {
                let target = line.trim_start_matches("assert visible key ");
                let node = self.keyed(target)?;
                let painted = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("missing painted bounds")?;
                let visible = self
                    .ctx
                    .visible_rect(node)
                    .ok_or("missing visible bounds")?;
                if painted.2 <= 0.0
                    || painted.3 <= 0.0
                    || (painted.0 - visible.0).abs() > 1.0
                    || (painted.1 - visible.1).abs() > 1.0
                    || (painted.2 - visible.2).abs() > 1.0
                    || (painted.3 - visible.3).abs() > 1.0
                {
                    return Err(format!(
                        "{target} clipped: painted={painted:?}, visible={visible:?}"
                    ));
                }
                eprintln!("cleromancy: visible {target}: painted={painted:?}, visible={visible:?}");
                Ok(())
            },
            _ if line.starts_with("reveal class ") => {
                let class = line.trim_start_matches("reveal class ");
                let node = {
                    let dom = self.ctx.runner.dom();
                    let dom = dom.borrow();
                    taproot::matching(&dom, &taproot::Selector::class(class))
                        .into_iter()
                        .next()
                        .ok_or_else(|| format!("no .{class}"))?
                };
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Start);
                Ok(())
            },
            _ if line.starts_with("reveal key ") => {
                let node = self.keyed(line.trim_start_matches("reveal key "))?;
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Start);
                Ok(())
            },
            // Acceptance-only: deliver the mounted view's own hover handler,
            // as the host does when its hit target enters or leaves.
            _ if line.starts_with("hover key ") || line.starts_with("unhover key ") => {
                let (phase, key) = match line.strip_prefix("hover key ") {
                    Some(key) => (cambium::HoverPhase::Enter, key),
                    None => (
                        cambium::HoverPhase::Leave,
                        line.trim_start_matches("unhover key "),
                    ),
                };
                let node = self.keyed(key)?;
                let (_, _, width, height) = self
                    .ctx
                    .painted_rect(node)
                    .ok_or_else(|| format!("{key} has no painted bounds"))?;
                self.ctx.runner.dispatch_hover(
                    node,
                    cambium::HoverEvent::new(phase, (width / 2.0, height / 2.0), (width, height)),
                );
                Ok(())
            },
            _ if line.starts_with("reveal mere-control ") => {
                let step = line.trim_start_matches("reveal mere-control ");
                let node = self.mere_control(step)?;
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Nearest);
                Ok(())
            },
            _ if line.starts_with("assert visible mere-control ") => {
                let step = line.trim_start_matches("assert visible mere-control ");
                let node = self.mere_control(step)?;
                let painted = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("control has no painted bounds")?;
                let visible = self
                    .ctx
                    .visible_rect(node)
                    .ok_or("control is clipped out of view")?;
                let (width, height) = self.ctx.logical_size;
                let tolerance = 1.0;
                if painted.2 <= 0.0
                    || painted.3 <= 0.0
                    || visible.2 + tolerance < painted.2
                    || visible.3 + tolerance < painted.3
                    || visible.0 < -tolerance
                    || visible.1 < -tolerance
                    || visible.0 + visible.2 > width + tolerance
                    || visible.1 + visible.3 > height + tolerance
                {
                    return Err(format!(
                        "mere-control {step} not fully reachable: painted={painted:?}, visible={visible:?}, viewport={width}x{height}"
                    ));
                }
                let event = format!(
                    "visible mere-control {step}: painted={painted:?}, visible={visible:?}, viewport={width}x{height}"
                );
                // Acceptance-only: retain the exact observed bounds in native.log
                // as well as the event stream, whose successful payloads Taproot
                // does not currently include in its saved Outcome.
                eprintln!("cleromancy: {event}");
                self.state.probe_events.push(event);
                Ok(())
            },
            "advance" => self.advance(),
            "show-mere" => {
                if self.ctx.runner.state().mere.open {
                    Ok(())
                } else {
                    self.mere_step(line)
                }
            },
            _ if line.starts_with("mere-") => self.mere_step(line),
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
        let containers = taproot::matching(&dom, &container);
        taproot::matching(
            &dom,
            &taproot::Selector::role("button").with_attr("data-step", step),
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
                let key = mere::kernel::graph::Graph::node_namespace_id(&format!(
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
                .map(|selector| taproot::matching(&dom, selector));
            taproot::matching(&dom, &selector)
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
        taproot::matching(
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
