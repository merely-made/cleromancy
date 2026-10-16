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

fn apply_worker_update(
    runner: &mut ConsultationRunner,
    state: &mut NativeState,
    update: WorkerUpdate,
) {
    match update {
        WorkerUpdate::GrantedContext {
            cards,
            notice,
            complete_request,
        } => runner.update(move |state| {
            state.consultation.ambient.granted_cards = cards;
            state.consultation.ambient.granted_notice = notice;
            if complete_request {
                state.consultation.status = super::ConsultationStatus::Ready;
            }
        }),
        WorkerUpdate::Neighborhood {
            catalog,
            detail,
            context,
        } => {
            runner.update(move |state| {
                state
                    .consultation
                    .present_neighborhood(catalog, detail, context)
            });
        },
        WorkerUpdate::Unavailable { message } => {
            state.catalog_ready = true;
            state
                .probe_events
                .push(format!("resident unavailable: {message}"));
            runner.update(|state| {
                state.mere.busy = false;
                state.mere.view.focused = None;
                state.mere.view.hovered = None;
                state.mere.model.graph = Default::default();
                state.mere.model.sessions.clear();
                state.mere.model.can_mint = false;
                state.mere.model.actions.clear();
                state.mere.pending = None;
                state.mere.model.archive = None;
                state.consultation.clear_selection();
                state.consultation.replace_catalog(empty_catalog());
                state.mere.model.status =
                    mere_view::ViewStatus::Unavailable(mere_view::StatusNote {
                        message: message.clone(),
                        action: None,
                    });
                state.consultation.present_error(message);
            });
        },
        WorkerUpdate::Mere { model, catalog } => {
            state.catalog_ready = true;
            runner.update(|state| {
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
                state.mere.busy = false;
                state
                    .mere
                    .view
                    .lay_out(&state.mere.model, state.mere.width, state.mere.height);
                state.consultation.replace_catalog(catalog);
            });
        },
        WorkerUpdate::GraphSelection {
            catalog,
            detail,
            position,
        } => {
            state
                .probe_events
                .push(format!("shared graph selected {}", detail.session.id));
            runner.update(|state| {
                state.mere.busy = false;
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
                    Some(mere::kernel::graph::Graph::node_namespace_id(&focus_url).to_string());
                state.consultation.present_session(catalog, detail);
                if let Some(position) = position {
                    state.consultation.reading_focus.insert(session, position);
                }
            });
        },
        WorkerUpdate::Catalog(catalog) => {
            state.catalog_ready = true;
            runner.update(|state| {
                let ui = &mut state.consultation;
                ui.replace_catalog(catalog);
                if ui.status.is_busy() {
                    ui.status = crate::ui::state::ConsultationStatus::Ready;
                }
            });
        },
        #[cfg(feature = "analytic-ephemeris")]
        WorkerUpdate::AstrologyChart {
            catalog,
            facts_digest,
        } => {
            state
                .probe_events
                .push(format!("durable astrology chart saved {facts_digest}"));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_calculated_chart(catalog, facts_digest);
            });
        },
        WorkerUpdate::Reading { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reading saved {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_reading(catalog, detail);
            });
        },
        WorkerUpdate::Reflection { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reflection saved {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_reflection(catalog, detail);
            });
        },
        WorkerUpdate::Session { catalog, detail } => {
            runner.update(|state| state.mere.busy = false);
            state
                .probe_events
                .push(format!("durable session recovered {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_session(catalog, detail);
            });
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
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_comparison(catalog, detail, comparison);
            });
        },
        WorkerUpdate::SavedNotice(message) => runner.update(move |state| {
            state.consultation.status = super::ConsultationStatus::SavedNotice(message);
        }),
        WorkerUpdate::Error { catalog, message } => runner.update(move |state| {
            state.mere.busy = false;
            let ui = &mut state.consultation;
            if let Some(catalog) = catalog {
                ui.replace_catalog(catalog);
            }
            ui.present_error(message);
        }),
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

#[cfg(test)]
mod focused_text_tests {
    use super::*;
    use cambium::{TextFieldMode, TextInput, caret_field_children, el, on_key};
    use cambium_genet_winit_host::{Harness, inert_hooks};
    use taproot::Selector;

    fn marked_question_view(ui: &ConsultationUi) -> ConsultationView {
        let field = el::<_, ConsultationUi, ()>(
            "div",
            caret_field_children::<ConsultationUi, ()>(&ui.question, &[]),
        )
        .attr("role", "textbox")
        .attr("aria-label", "Question")
        .attr("aria-multiline", "true")
        .attr("data-cambium-text-value", ui.question.text())
        .attr("style", "display:block;width:240px;height:40px");
        let field = on_key(field, |ui: &mut ConsultationUi, event| {
            ui.question.apply_key(&event, TextFieldMode::Multiline);
        });
        Box::new(
            el::<_, ConsultationUi, ()>("label", (el("span", "Question"), field))
                .attr("id", "cleromancy-question"),
        )
    }

    #[test]
    fn marked_div_field_routes_text_into_its_question_slot() {
        let hooks: HostHooks<ConsultationUi, Logic, ConsultationView> = HostHooks {
            focused_text: Box::new(consultation_focused_text),
            ..inert_hooks()
        };
        let mut state = ConsultationUi::new(empty_catalog());
        state.question = TextInput::default();
        let mut harness = Harness::with_hooks(
            Init {
                state,
                logic: marked_question_view as Logic,
                sheet: String::new(),
                fonts: Vec::new(),
                images: Vec::new(),
            },
            hooks,
        );
        harness.layout_at(320.0, 120.0);
        assert!(
            harness.click_on(&Selector::role("textbox").with_attr("aria-label", "Question")),
            "the marked textbox has a painted click target",
        );

        let focused = consultation_focused_text(harness.runner())
            .expect("the marked DIV resolves to the Question slot");
        assert_eq!(
            focused.node,
            harness.focus().expect("the textbox receives focus")
        );
        assert_eq!((focused.get)(harness.state()).text(), "");
        assert_eq!((focused.get)(harness.state()).caret_position().byte, 0);
        harness.key_injected("x");

        let focused = consultation_focused_text(harness.runner())
            .expect("the marked DIV remains routed after input");
        assert_eq!((focused.get)(harness.state()).text(), "x");
        assert_eq!((focused.get)(harness.state()).caret_position().byte, 1);
    }
}

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
