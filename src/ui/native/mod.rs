// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's native Cambium host glue.
//!
//! The shared host owns winit, retained layout, paint, input, scrolling, and
//! accessibility. This module keeps Cleromancy's worker, command drain, and
//! headed scenario policy.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};

use cambium_genet_winit_host::{
    AppCtx, FocusedTextSlot, HostHooks, HostOptions, HostWake, Init, Runner, inert_hooks,
    run as run_host,
};
use layout_dom_api::LayoutDom as _;

use crate::ui::scenario::{self, Phase};
use crate::ui::worker::{ConsultationWorker, WorkerUpdate, spawn};
use crate::{ConsultationCatalog, ConsultationUi, ConsultationView, consultation_view};

mod scenario_driver;

type Logic = fn(&ConsultationUi) -> ConsultationView;
type ConsultationRunner = Runner<ConsultationUi, Logic, ConsultationView>;
type ConsultationCtx<'a> = AppCtx<'a, ConsultationUi, Logic, ConsultationView>;

/// Start the ordinary local consultation window.
pub fn run(data_root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(data_root)
        .map_err(|error| format!("create {}: {error}", data_root.display()))?;
    let state = Rc::new(RefCell::new(NativeState::new(scenario::load())));
    let init_state = state.clone();
    let store_path = data_root.join("cleromancy.redb");
    let options = HostOptions {
        title: "Cleromancy".to_string(),
        initial_logical_size: (1160.0, 760.0),
        ..Default::default()
    };

    run_host(
        options,
        move |_window, _commands, wake| {
            install_worker(&init_state, store_path.clone(), wake);
            Init {
                state: ConsultationUi::new(empty_catalog()),
                logic: consultation_view as Logic,
                sheet: SHEET.to_string(),
            }
        },
        hooks(state),
    )
    .map_err(|error| format!("run Cleromancy native host: {error}"))
}

fn empty_catalog() -> ConsultationCatalog {
    ConsultationCatalog {
        contexts: Vec::new(),
        fields: Vec::new(),
        spread_templates: Vec::new(),
        astrology_facts: Vec::new(),
        sky_day_facts: Vec::new(),
        session_summaries: Vec::new(),
    }
}

fn install_worker(state: &Rc<RefCell<NativeState>>, store_path: PathBuf, wake: &HostWake) {
    let (updates, receiver) = mpsc::channel();
    let wake = wake.callback();
    let worker = spawn(store_path, move |update| {
        let _ = updates.send(update);
        wake();
    });
    let mut state = state.borrow_mut();
    state.worker = Some(worker);
    state.updates = Some(receiver);
}

fn hooks(state: Rc<RefCell<NativeState>>) -> HostHooks<ConsultationUi, Logic, ConsultationView> {
    let mut hooks = inert_hooks();
    let waking = state.clone();
    hooks.after_wake = Box::new(move |ctx| drain_worker(ctx, &waking));
    let dispatching = state.clone();
    hooks.after_dispatch = Box::new(move |ctx| dispatch_action(ctx, &dispatching));
    let framing = state.clone();
    hooks.frame = Box::new(move |ctx| {
        scenario_driver::arm_capture(ctx, &framing);
        false
    });
    hooks.after_frame = Box::new(move |ctx| scenario_driver::after_frame(ctx, &state));
    hooks.focused_text = Box::new(consultation_focused_text);
    hooks
}

fn drain_worker(ctx: &mut ConsultationCtx<'_>, state: &Rc<RefCell<NativeState>>) {
    let updates: Vec<_> = state
        .borrow()
        .updates
        .as_ref()
        .map(|receiver| receiver.try_iter().collect())
        .unwrap_or_default();
    if updates.is_empty() {
        return;
    }
    let mut state = state.borrow_mut();
    for update in updates {
        apply_worker_update(ctx.runner, &mut state, update);
    }
    if let Some(window) = ctx.window {
        window.request_redraw();
    }
}

pub(super) fn apply_worker_update(
    runner: &mut ConsultationRunner,
    state: &mut NativeState,
    update: WorkerUpdate,
) {
    match update {
        WorkerUpdate::Catalog(catalog) => {
            state.catalog_ready = true;
            runner.update(|ui| ui.replace_catalog(catalog));
        },
        #[cfg(feature = "analytic-ephemeris")]
        WorkerUpdate::AstrologyChart {
            catalog,
            facts_digest,
        } => {
            state
                .probe_events
                .push(format!("durable astrology chart saved {facts_digest}"));
            runner.update(move |ui| ui.present_calculated_chart(catalog, facts_digest));
        },
        WorkerUpdate::Reading { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reading saved {}", detail.session.id));
            runner.update(move |ui| ui.present_reading(catalog, detail));
        },
        WorkerUpdate::Reflection { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reflection saved {}", detail.session.id));
            runner.update(move |ui| ui.present_reflection(catalog, detail));
        },
        WorkerUpdate::Session { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable session recovered {}", detail.session.id));
            runner.update(move |ui| ui.present_session(catalog, detail));
        },
        WorkerUpdate::Comparison {
            catalog,
            detail,
            comparison,
        } => {
            state.probe_events.push(format!(
                "durable receipts compared {} {}",
                comparison.left_session_id, comparison.right_session_id
            ));
            runner.update(move |ui| ui.present_comparison(catalog, detail, comparison));
        },
        WorkerUpdate::Error { catalog, message } => runner.update(move |ui| {
            if let Some(catalog) = catalog {
                ui.replace_catalog(catalog);
            }
            ui.present_error(message);
        }),
    }
}

pub(super) fn submit_pending(runner: &mut ConsultationRunner, worker: Option<&ConsultationWorker>) {
    let mut action = None;
    runner.update(|ui| action = ui.take_pending_action());
    let Some(action) = action else { return };
    if let Err(error) = worker
        .ok_or_else(|| "the consultation worker is unavailable".to_string())
        .and_then(|worker| worker.command(action))
    {
        runner.update(|ui| ui.present_error(error));
    }
}

fn dispatch_action(ctx: &mut ConsultationCtx<'_>, state: &Rc<RefCell<NativeState>>) {
    let state = state.borrow();
    submit_pending(ctx.runner, state.worker.as_ref());
}

/// Resolve the focused retained text control to its Cleromancy state slot.
///
/// This is public so windowless acceptance tests can exercise the same caret,
/// keyboard, and IME boundary as the native host.
pub fn consultation_focused_text(
    runner: &Runner<ConsultationUi, fn(&ConsultationUi) -> ConsultationView, ConsultationView>,
) -> Option<FocusedTextSlot<ConsultationUi>> {
    let node = runner.focus()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    let tag = &dom.element_name(node)?.local;
    if tag.as_ref() != "input" && tag.as_ref() != "textarea" {
        return None;
    }
    let marker = dom.parent(node).and_then(|parent| {
        dom.attribute(
            parent,
            &layout_dom_api::Namespace::from(""),
            &layout_dom_api::LocalName::from("id"),
        )
    })?;
    macro_rules! field {
        ($member:ident) => {
            Some(FocusedTextSlot {
                node,
                get: Box::new(|ui| &ui.$member),
                get_mut: Box::new(|ui| &mut ui.$member),
            })
        };
    }
    match marker {
        "cleromancy-context-label" => field!(context_label),
        "cleromancy-question" => field!(question),
        "cleromancy-tags" => field!(tags),
        "cleromancy-additional-facts" => field!(additional_facts),
        "cleromancy-derived-seed" => field!(derived_seed),
        "cleromancy-derived-domain" => field!(derived_domain),
        "cleromancy-template-label" => field!(template_label),
        "cleromancy-template-positions" => field!(template_positions),
        "cleromancy-template-relations" => field!(template_relations),
        "cleromancy-astrology-instant" => field!(astrology_instant_utc),
        "cleromancy-astrology-latitude" => field!(astrology_latitude),
        "cleromancy-astrology-longitude" => field!(astrology_longitude),
        "cleromancy-astrology-orb" => field!(astrology_orb),
        "cleromancy-astrology-algorithm" => field!(astrology_algorithm),
        "cleromancy-astrology-engine" => field!(astrology_engine),
        "cleromancy-astrology-ephemeris" => field!(astrology_ephemeris),
        "cleromancy-astrology-positions" => field!(astrology_positions),
        "cleromancy-journal-tag-filter" => Some(FocusedTextSlot {
            node,
            get: Box::new(|ui| &ui.journal.tag_filter),
            get_mut: Box::new(|ui| &mut ui.journal.tag_filter),
        }),
        "cleromancy-reflection" => field!(reflection),
        _ => None,
    }
}

pub(super) struct NativeState {
    worker: Option<ConsultationWorker>,
    updates: Option<Receiver<WorkerUpdate>>,
    pub(super) catalog_ready: bool,
    pub(super) scenario: Option<scenario::Run>,
    pub(super) scenario_phase: Option<Phase>,
    pub(super) scenario_stage: u8,
    pub(super) capture_dir: Option<PathBuf>,
    pub(super) pending_capture: Option<PathBuf>,
    pub(super) capture_error: Option<String>,
    pub(super) probe_events: Vec<String>,
}

impl NativeState {
    fn new(scenario: Option<scenario::Run>) -> Self {
        let capture_dir = scenario.as_ref().map(|run| run.dir.clone());
        let scenario_phase = scenario.as_ref().map(|run| run.phase);
        Self {
            worker: None,
            updates: None,
            catalog_ready: false,
            scenario,
            scenario_phase,
            scenario_stage: 0,
            capture_dir,
            pending_capture: None,
            capture_error: None,
            probe_events: Vec::new(),
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
  /* Surfaces carry one region or three, so the track count follows the
     content rather than assuming the old three-column shell. */
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 16px;
}
.selection-bar { display: flex; gap: 8px; margin: 12px 0; }
.selection-item {
  display: block; padding: 8px 12px; border: 1px solid #625d50; border-radius: 6px;
  color: #d7c9a9; cursor: pointer;
}
.selection-item.selected { color: #f7f2e7; background: #6e522a; }
.selection-item[aria-disabled='true'] { color: #8b8474; cursor: default; }
.selection-disabled-reason { display: block; font-size: 12px; }
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
