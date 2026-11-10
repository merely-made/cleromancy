// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's native Cambium host glue.
//!
//! The shared host owns winit, retained layout, paint, input, scrolling, and
//! accessibility. This module keeps Cleromancy's worker, command drain, and
//! headed scenario policy.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};

use cambium::{AngleStrip, DimensionLine};
use cambium_genet_winit_host::{
    AppCtx, CaptionLabels, FocusedTextSlot, HostHooks, HostOptions, HostWake, Init, Runner,
    WindowFrame, inert_hooks, platform_caption_controls, run as run_host,
};
use layout_dom_api::LayoutDom as _;

use crate::ui::scenario::{self, Phase};
use crate::ui::worker::{ConsultationWorker, WorkerUpdate, spawn};
use crate::{ConsultationCatalog, ConsultationUi, ConsultationView};

mod ambient;
mod appearance;
mod appearance_host;
#[cfg(test)]
mod appearance_tests;
mod appearance_view;
mod fonts;
mod scenario_driver;

use appearance_view::{ApplicationUi, Child};

#[cfg(test)]
type Logic = fn(&ConsultationUi) -> ConsultationView;
type ApplicationLogic = Box<dyn FnMut(&ApplicationUi) -> Child>;
type ConsultationRunner = Runner<ApplicationUi, ApplicationLogic, Child>;
type ConsultationCtx<'a> = AppCtx<'a, ApplicationUi, ApplicationLogic, Child>;

/// Start the ordinary local consultation window.
pub fn run(data_root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(data_root)
        .map_err(|error| format!("create {}: {error}", data_root.display()))?;
    let state = Rc::new(RefCell::new(NativeState::new(scenario::load())));
    let init_state = state.clone();
    let (selection, library) = appearance::paths(data_root);
    let appearance = appearance::AppearanceState::load(selection, library, data_root.to_path_buf());
    let options = HostOptions {
        title: "Cleromancy".to_string(),
        initial_logical_size: (1160.0, 760.0),
        window_frame: WindowFrame::App,
        ..Default::default()
    };

    run_host(
        options,
        move |_window, commands, wake| {
            install_worker(&init_state, wake);
            let mut appearance = appearance;
            let sheet = appearance
                .take_stylesheet_change()
                .expect("initial owned sheet");
            Init {
                state: ApplicationUi {
                    consultation: ConsultationUi::new(empty_catalog()),
                    mere: Default::default(),
                    appearance,
                },
                logic: Box::new({
                    let commands = commands.clone();
                    move |state: &ApplicationUi| {
                        appearance_view::root(
                            state,
                            platform_caption_controls(&commands, &CaptionLabels::default()),
                        )
                    }
                }) as ApplicationLogic,
                sheet,
                fonts: fonts::bundled_fonts(),
                images: Vec::new(),
            }
        },
        hooks(state),
    )
    .map_err(|error| format!("run Cleromancy native host: {error}"))
}

fn empty_catalog() -> ConsultationCatalog {
    ConsultationCatalog {
        natal_readings: Vec::new(),
        natal_reflections: Vec::new(),
        ambient_memory: Vec::new(),
        contexts: Vec::new(),
        fields: Vec::new(),
        spread_templates: Vec::new(),
        astrology_facts: Vec::new(),
        astrology_charts: Vec::new(),
        sky_day_facts: Vec::new(),
        session_summaries: Vec::new(),
    }
}

fn install_worker(state: &Rc<RefCell<NativeState>>, wake: &HostWake) {
    let (updates, receiver) = mpsc::channel();
    let wake = wake.callback();
    let worker = spawn(move |update| {
        let _ = updates.send(update);
        wake();
    });
    let mut state = state.borrow_mut();
    state.worker = Some(worker);
    state.updates = Some(receiver);
}

fn hooks(state: Rc<RefCell<NativeState>>) -> HostHooks<ApplicationUi, ApplicationLogic, Child> {
    let mut hooks = inert_hooks();
    let waking = state.clone();
    hooks.after_wake = Box::new(move |ctx| drain_worker(ctx, &waking));
    let dispatching = state.clone();
    hooks.after_dispatch = Box::new(move |ctx| {
        appearance_host::after_dispatch(ctx);
        dispatch_action(ctx, &dispatching);
    });
    let mut previews = tabard_workshop::native_host::PreviewBindings::default();
    let mut ambient_runtime = None;
    hooks.frame = Box::new(move |ctx| {
        if ctx.runner.state().appearance.editor_open {
            previews.register(
                &ctx.runner.state().appearance.workshop,
                ctx.leaves,
                ctx.producers,
            );
        }
        let window_width = ctx.logical_size.0.round().max(1.0) as u32;
        let wide = window_width >= 900;
        let width = if wide { 420 } else { window_width };
        let height = if wide {
            (ctx.logical_size.1.round().max(1.0) as u32)
                .saturating_sub(42)
                .max(300)
        } else {
            300
        };
        if ctx.runner.state().mere.width != width
            || ctx.runner.state().mere.height != height
            || ctx.runner.state().mere.wide != wide
        {
            ctx.runner.update(|state| {
                state.mere.width = width;
                state.mere.height = height;
                state.mere.wide = wide;
                state.mere.view.lay_out(&state.mere.model, width, height);
            });
        }
        if ctx.runner.state().mere.open {
            let leaf = ctx
                .runner
                .state()
                .mere
                .view_of()
                .paint_leaf(|_| sprigging::ColorF {
                    r: 0.20,
                    g: 0.35,
                    b: 0.50,
                    a: 1.0,
                });
            ctx.leaves
                .insert(appearance_view::MERE_LEAF, Box::new(leaf));
        }
        sync_chart_angle_strip(ctx);
        sync_chart_aspect_dimension(ctx);
        ambient::sync(ctx, &mut ambient_runtime)
    });
    let run = state.borrow_mut().scenario.take();
    let mut lane = run.map(|run| scenario_driver::lane(state.clone(), run));
    hooks.after_frame = Box::new(move |ctx| {
        if let Some(lane) = &mut lane {
            scenario_driver::drive(lane, ctx);
        }
    });
    hooks.focused_text = Box::new(appearance_host::focused_text);
    hooks.close_request = Box::new(appearance_host::close_request);
    hooks
}

fn sync_chart_angle_strip(ctx: &mut ConsultationCtx<'_>) {
    let marks = super::view::chart::selected_angle_strip_marks(&ctx.runner.state().consultation);
    match marks {
        Some(marks) => {
            if let Some(strip) = ctx
                .leaves
                .get_mut_as::<AngleStrip>(&super::view::chart::CHART_ANGLE_STRIP_KEY)
            {
                strip.set_marks(marks);
            } else {
                ctx.leaves.insert(
                    super::view::chart::CHART_ANGLE_STRIP_KEY,
                    Box::new(AngleStrip::with_size(marks, 720.0, 36.0)),
                );
            }
        },
        // Keep the clean retained leaf across surface switches. The Chart DOM
        // element is absent elsewhere, so it cannot paint; retaining the
        // payload also avoids leaving a stale rendered-leaf cache entry behind.
        None => {},
    }
}

fn sync_chart_aspect_dimension(ctx: &mut ConsultationCtx<'_>) {
    let dimension = super::view::chart::selected_aspect_dimension(&ctx.runner.state().consultation);
    let Some(dimension) = dimension else {
        // Keep the clean retained leaf across surface switches and empty
        // charts, as with the ecliptic strip. The DOM leaf is absent then.
        return;
    };
    if let Some(line) = ctx
        .leaves
        .get_mut_as::<DimensionLine>(&super::view::chart_aspect::CHART_ASPECT_DIMENSION_KEY)
    {
        line.set_measurement(
            dimension.start,
            dimension.end,
            dimension.traversal,
            Some(dimension.target),
        );
    } else {
        ctx.leaves.insert(
            super::view::chart_aspect::CHART_ASPECT_DIMENSION_KEY,
            Box::new(DimensionLine::with_size(
                dimension.start,
                dimension.end,
                dimension.traversal,
                Some(dimension.target),
                720.0,
                36.0,
            )),
        );
    }
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

// A refresh is not an acknowledgement: publish model, catalog and selection in
// one retained-state transition, and release only the command response's wait.
fn apply_mere_publication(
    state: &mut ApplicationUi,
    model: mere_view::MereViewModel,
    catalog: crate::ConsultationCatalog,
    selection: Option<(crate::ConsultationDetail, Option<String>)>,
    complete_request: bool,
) {
    let pending_status = state
        .consultation
        .status
        .is_busy()
        .then(|| state.consultation.status.clone());
    let previous = state
        .mere
        .model
        .sessions
        .iter()
        .find(|s| s.attached)
        .map(|s| s.id);
    let next = model.sessions.iter().find(|s| s.attached).map(|s| s.id);
    if previous != next {
        state.consultation.clear_selection();
        state.mere.view.focused = None;
        state.mere.view.hovered = None;
    }
    state.mere.model = model;
    if state.mere.view.focused.as_ref().is_some_and(|key| {
        !state
            .mere
            .model
            .graph
            .nodes
            .iter()
            .any(|node| &node.key == key)
    }) {
        state.mere.view.focused = None;
    }
    if state.mere.view.hovered.as_ref().is_some_and(|key| {
        !state
            .mere
            .model
            .graph
            .nodes
            .iter()
            .any(|node| &node.key == key)
    }) {
        state.mere.view.hovered = None;
    }
    if complete_request {
        state.mere.busy = false;
    }
    state
        .mere
        .view
        .lay_out(&state.mere.model, state.mere.width, state.mere.height);
    state.consultation.replace_catalog(catalog);
    if let Some(status) = pending_status {
        state.consultation.status = status;
    }
    if let Some((detail, position)) = selection {
        let session = detail.session.id.clone();
        let focus_url = position
            .as_ref()
            .and_then(|position| {
                detail
                    .session
                    .placements
                    .iter()
                    .find(|p| &p.position == position)
            })
            .map(|p| format!("cleromancy://reading/{}", p.reading_id))
            .unwrap_or_else(|| format!("cleromancy://session/{session}"));
        state.mere.view.focused =
            Some(kernel::graph::Graph::node_namespace_id(&focus_url).to_string());
        state
            .consultation
            .present_external_session(state.consultation.catalog().clone(), detail);
        if let Some(position) = position {
            state.consultation.reading_focus.insert(session, position);
        }
    }
}

fn submit_pending(runner: &mut ConsultationRunner, worker: Option<&ConsultationWorker>) {
    let mut action = None;
    runner.update(|state| {
        let ui = &mut state.consultation;
        action = ui.take_pending_action();
    });
    let Some(action) = action else { return };
    if let Err(error) = worker
        .ok_or_else(|| "the consultation worker is unavailable".to_string())
        .and_then(|worker| worker.command(action))
    {
        runner.update(|state| {
            let ui = &mut state.consultation;
            ui.present_error(error);
        });
    }
}

fn submit_mere_pending(runner: &mut ConsultationRunner, worker: Option<&ConsultationWorker>) {
    let mut request = None;
    runner.update(|state| {
        request = state.mere.pending.take();
        if request.is_some() {
            state.mere.busy = true;
        }
    });
    if let Some(request) = request {
        if let Err(message) = worker
            .ok_or_else(|| "resident worker is unavailable".to_owned())
            .and_then(|worker| worker.mere(request))
        {
            runner.update(|state| {
                state.mere.busy = false;
                state.mere.model.notice = Some(message);
            });
        }
    }
}

fn dispatch_action(ctx: &mut ConsultationCtx<'_>, state: &Rc<RefCell<NativeState>>) {
    let state = state.borrow();
    submit_pending(ctx.runner, state.worker.as_ref());
    submit_mere_pending(ctx.runner, state.worker.as_ref());
}

/// Resolve the focused retained text control to its Cleromancy state slot.
///
/// This is public so windowless acceptance tests can exercise the same caret,
/// keyboard, and IME boundary as the native host.
pub fn consultation_focused_text(
    runner: &Runner<ConsultationUi, fn(&ConsultationUi) -> ConsultationView, ConsultationView>,
) -> Option<FocusedTextSlot<ConsultationUi>> {
    let node = runner.focus()?;
    consultation_text_slot(&runner.dom(), node)
}

fn consultation_text_slot(
    dom: &cambium::DomHandle,
    node: genet_scripted_dom::NodeId,
) -> Option<FocusedTextSlot<ConsultationUi>> {
    let dom = dom.borrow();
    let tag = &dom.element_name(node)?.local;
    let native_text_control = matches!(tag.as_ref(), "input" | "textarea");
    let marker_text_control = dom.attribute(
        node,
        &layout_dom_api::Namespace::from(""),
        &layout_dom_api::LocalName::from("role"),
    ) == Some("textbox")
        && dom
            .attribute(
                node,
                &layout_dom_api::Namespace::from(""),
                &layout_dom_api::LocalName::from("data-cambium-text-value"),
            )
            .is_some();
    if !native_text_control && !marker_text_control {
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
        "cleromancy-natal-reflection" => field!(natal_reflection),
        "cleromancy-natal-name" => field!(natal_name),
        "cleromancy-natal-date" => field!(natal_date),
        "cleromancy-natal-time" => field!(natal_time),
        "cleromancy-natal-zone" => field!(natal_zone),
        "cleromancy-natal-place" => field!(natal_place),
        "cleromancy-natal-latitude" => field!(natal_latitude),
        "cleromancy-natal-longitude" => field!(natal_longitude),
        "cleromancy-natal-source" => field!(natal_source),
        "cleromancy-natal-question" => field!(natal_question),
        "cleromancy-natal-fold" => field!(natal_fold),

        "cleromancy-context-label" => field!(context_label),
        "cleromancy-question" => field!(question),
        "cleromancy-place-latitude" => field!(place_latitude),
        "cleromancy-place-longitude" => field!(place_longitude),
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

#[cfg(test)]
mod focused_text_tests;

pub(super) struct NativeState {
    worker: Option<ConsultationWorker>,
    updates: Option<Receiver<WorkerUpdate>>,
    pub(super) catalog_ready: bool,
    pub(super) scenario: Option<scenario::Run>,
    pub(super) scenario_phase: Option<Phase>,
    pub(super) scenario_stage: u8,
    pub(super) probe_events: Vec<String>,
}

impl NativeState {
    fn new(scenario: Option<scenario::Run>) -> Self {
        let scenario_phase = scenario.as_ref().map(|run| run.phase);
        Self {
            worker: None,
            updates: None,
            catalog_ready: false,
            scenario,
            scenario_phase,
            scenario_stage: 0,
            probe_events: Vec::new(),
        }
    }
}

const SHEET: &str = include_str!("../consultation.css");

mod worker_update;

use worker_update::apply_worker_update;
