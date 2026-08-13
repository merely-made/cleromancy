// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's small native Genet host.
//!
//! This file deliberately owns only the window, retained DOM/layout/render
//! pipeline, and event routing. The graph controller and Redb persistence live
//! in [`super::worker`], which publishes durable results back to this loop.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use accesskit::{Action, NodeId as AccessNodeId, Tree, TreeId, TreeUpdate};
use cambium::{DomHandle, GenetAppRunner, Modifiers, PointerClick, Propagation};
use genet_layout::{IncrementalLayout, ScrollOffsets};
use genet_scripted_dom::{NodeId, ScriptedDom};
use genet_winit_host::{AccessKitBridge, BridgeStatus, SurfaceHost, wheel_delta_from_winit};
use layout_dom_api::{DomMutation, LayoutDom as _, LayoutDomMut as _};
use netrender::{ColorLoad, ExternalTexturePlacement, NetrenderOptions};
use paint_list_api::{DeviceIntSize, PaintList as _};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::{
    ConsultationAction, ConsultationCatalog, ConsultationUi, ConsultationView, consultation_view,
};

use super::scenario::{self, Observation, Phase, ReportedIds};
use super::worker::{ConsultationWorker, WorkerUpdate, spawn};

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
    /// An optional genet-probe run, advanced by `new_events` below.
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

    /// Route screen-reader clicks through the same Cambium action path as a
    /// mouse press. The previous frame's tree is precisely the tree the OS saw.
    fn pump_a11y_actions(&mut self) {
        let requests = match self.a11y.as_mut() {
            Some(bridge) => bridge.drain_actions(),
            None => return,
        };
        let Some(runner) = self.runner.as_mut() else {
            return;
        };
        let mut actions = Vec::new();
        for request in requests {
            if request.action == Action::Click
                && let Some(&node) = self.a11y_route.get(&request.target_node)
            {
                actions.extend(runner.dispatch_click(
                    node,
                    PointerClick {
                        local: (0.0, 0.0),
                        prop: Propagation::new(),
                    },
                ));
            }
        }
        self.submit_actions(actions);
    }

    fn redraw(&mut self) {
        self.pump_a11y_actions();
        let (Some(window), Some(host), Some(runner)) = (
            self.window.as_ref(),
            self.host.as_ref(),
            self.runner.as_ref(),
        ) else {
            return;
        };
        let physical = window.inner_size();
        let (pw, ph) = (physical.width.max(1), physical.height.max(1));
        let scale = window.scale_factor() as f32;
        let (lw, lh) = (pw as f32 / scale, ph as f32 / scale);

        let scene = {
            let dom = runner.dom();
            let mut mutations: Vec<DomMutation<NodeId>> = Vec::new();
            dom.borrow_mut().drain_mutations(&mut mutations);
            let dom_ref = dom.borrow();
            let sheets = [SHEET];
            let structural = mutations
                .iter()
                .any(|mutation| !matches!(mutation, DomMutation::AttributeChanged { .. }));
            let size_changed = self.layout_size != (lw, lh);
            match self.layout.as_mut() {
                Some(layout) if !structural && !size_changed => {
                    if !mutations.is_empty() {
                        let _ = layout.apply(&*dom_ref, &sheets, &mutations);
                    }
                }
                _ => {
                    self.layout = Some(IncrementalLayout::new(&*dom_ref, &sheets, lw, lh));
                    self.layout_size = (lw, lh);
                }
            }
            let layout = self.layout.as_ref().expect("layout was just installed");
            let list = layout.emit_paint_list(
                &*dom_ref,
                &ScrollOffsets::default(),
                DeviceIntSize::new(lw as i32, lh as i32),
            );
            let translated = paint_list_render::translate_paint_cmd_stream(
                list.viewport(),
                list.commands(),
                list.fonts(),
                list.images(),
            );

            if let Some(bridge) = self.a11y.as_mut() {
                let needs_tree = structural
                    || size_changed
                    || !mutations.is_empty()
                    || bridge.status() == BridgeStatus::Unavailable;
                if needs_tree {
                    let (nodes, root_id, actionable) = genet_layout::build_subtree(
                        &*dom_ref,
                        layout.fragments(),
                        dom_ref.document(),
                        &|dom: &ScriptedDom, node: NodeId| AccessNodeId(dom.opaque_id(node)),
                        &|_dom: &ScriptedDom, _node: NodeId| false,
                    );
                    self.a11y_route = actionable
                        .into_iter()
                        .map(|node| (AccessNodeId(dom_ref.opaque_id(node)), node))
                        .collect();
                    let tree = TreeUpdate {
                        nodes,
                        tree: Some(Tree::new(root_id)),
                        tree_id: TreeId::ROOT,
                        focus: root_id,
                    };
                    match bridge.status() {
                        BridgeStatus::Installed => bridge.update(tree),
                        BridgeStatus::Unavailable => {
                            let _ = bridge.install(window, tree);
                        }
                    }
                }
            }
            translated.scene
        };

        let (_texture, view) = host.core().rasterize_scaled(
            &scene,
            pw,
            ph,
            ColorLoad::Clear(wgpu::Color::BLACK),
            scale,
        );
        let Some(frame) = host.acquire() else {
            return;
        };
        let target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        host.renderer().compose_external_texture(
            &view,
            &target,
            host.format(),
            pw,
            ph,
            ExternalTexturePlacement::new([0.0, 0.0, pw as f32, ph as f32]),
        );
        frame.present();
        if let Some(path) = self.pending_capture.take()
            && !scenario::capture_frame(host, &view, pw, ph, &path)
        {
            self.capture_error = Some(format!("write {}", path.display()));
        }
    }

    fn click(&mut self) {
        let (Some(runner), Some(layout)) = (self.runner.as_mut(), self.layout.as_ref()) else {
            return;
        };
        let (x, y) = self.cursor;
        let target = {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            layout.hit_test(&*dom_ref, x, y, &ScrollOffsets::default())
        };
        let Some(target) = target else {
            return;
        };
        let actions = runner.dispatch_click(
            target,
            PointerClick {
                local: (0.0, 0.0),
                prop: Propagation::new(),
            },
        );
        self.submit_actions(actions);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn key(&mut self, event: &winit::event::KeyEvent) {
        let Some(key) = cambium_winit::key_event_from_winit(&event.logical_key, self.modifiers)
        else {
            return;
        };
        let Some(runner) = self.runner.as_mut() else {
            return;
        };
        let actions = runner.dispatch_key(key);
        self.submit_actions(actions);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn ime(&mut self, event: &winit::event::Ime) {
        let Some(runner) = self.runner.as_mut() else {
            return;
        };
        let actions = runner.dispatch_key(cambium_winit::ime_event_from_winit(event));
        self.submit_actions(actions);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn scroll(&mut self, delta: winit::event::MouseScrollDelta) {
        let (Some(runner), Some(layout)) = (self.runner.as_ref(), self.layout.as_mut()) else {
            return;
        };
        let (dx, dy) = wheel_delta_from_winit(delta);
        let moved = {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            layout.scroll_at(&*dom_ref, self.cursor.0, self.cursor.1, dx, dy)
        };
        if moved && let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl genet_probe::Automatable for NativeApp {
    fn with_surfaces<R>(&self, f: impl FnOnce(&[genet_probe::ProbeSurface<'_>]) -> R) -> R {
        let Some(runner) = self.runner.as_ref() else {
            return f(&[]);
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let (width, height) = self.layout_size;
        let surfaces = [genet_probe::ProbeSurface {
            name: "cleromancy",
            dom: &dom_ref,
            rect: [0.0, 0.0, width, height],
            sheet: SHEET,
        }];
        f(&surfaces)
    }

    fn snapshot(&self) -> genet_probe::ProbeSnapshot {
        let observation = self.scenario_observation();
        let mut snapshot = genet_probe::ProbeSnapshot::default()
            .with_field("status", observation.status)
            .with_field("catalog-ready", observation.catalog_ready.to_string())
            .with_field("sessions", observation.sessions.to_string())
            .with_field("readings", observation.readings.to_string())
            .with_field("reflections", observation.reflections.to_string());
        if let Some(ids) = observation.ids {
            snapshot = snapshot
                .with_field("session-id", ids.session_id)
                .with_field("reading-id", ids.reading_id)
                .with_field("reflection-id", ids.reflection_id);
        }
        snapshot
    }

    fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.probe_events)
    }

    fn act(&mut self, _label: &str) -> bool {
        false
    }

    fn press(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
        self.click();
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
    }

    fn release(&mut self, _x: f32, _y: f32) {}

    fn busy(&mut self) -> Option<bool> {
        if !self.catalog_ready {
            return Some(true);
        }
        Some(self.runner.as_ref().is_some_and(|runner| {
            matches!(
                runner.state().status(),
                crate::ConsultationStatus::SavingReading
                    | crate::ConsultationStatus::SavingSetup
                    | crate::ConsultationStatus::SavingReflection
                    | crate::ConsultationStatus::LoadingSession
            )
        }))
    }
}

impl genet_probe::Driveable for NativeApp {
    fn capture(&mut self, name: &str) -> bool {
        let Some(dir) = self.capture_dir.as_ref() else {
            return false;
        };
        if name.is_empty() || name.contains(['/', '\\']) {
            return false;
        }
        self.pending_capture = Some(dir.join(format!("{name}.png")));
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        true
    }

    fn app_step(&mut self, line: &str) -> Result<(), String> {
        match line {
            "advance" => self.scenario_advance(),
            _ => Err(format!("unknown Cleromancy scenario verb: {line}")),
        }
    }
}

impl ApplicationHandler<HostEvent> for NativeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Cleromancy")
                        .with_inner_size(winit::dpi::LogicalSize::new(1160.0, 760.0))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(760.0, 520.0))
                        .with_visible(false),
                )
                .expect("create Cleromancy window"),
        );
        let wake_window = window.clone();
        let mut a11y = AccessKitBridge::new(move || wake_window.request_redraw());
        let physical = window.inner_size();
        let host = SurfaceHost::boot(
            window.clone(),
            physical.width.max(1),
            physical.height.max(1),
            NetrenderOptions {
                tile_cache_size: Some(512),
                enable_vello: true,
                ..Default::default()
            },
        )
        .expect("boot Cleromancy native host");
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            consultation_view as fn(&ConsultationUi) -> ConsultationView,
            ConsultationUi::new(ConsultationCatalog {
                contexts: Vec::new(),
                fields: Vec::new(),
                spread_templates: Vec::new(),
                astrology_facts: Vec::new(),
                sessions: Vec::new(),
            }),
        );

        // Windows must receive an initial tree before the visible window. A
        // hidden window may not be redrawn, so this bootstrap cannot wait for
        // the ordinary render path below.
        let scale = window.scale_factor() as f32;
        let (lw, lh) = (
            physical.width as f32 / scale,
            physical.height as f32 / scale,
        );
        let (layout, tree, actionable) = {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            let sheets = [SHEET];
            let layout = IncrementalLayout::new(&*dom_ref, &sheets, lw, lh);
            let (nodes, root_id, actionable) = genet_layout::build_subtree(
                &*dom_ref,
                layout.fragments(),
                dom_ref.document(),
                &|dom: &ScriptedDom, node: NodeId| AccessNodeId(dom.opaque_id(node)),
                &|_dom: &ScriptedDom, _node: NodeId| false,
            );
            (
                layout,
                TreeUpdate {
                    nodes,
                    tree: Some(Tree::new(root_id)),
                    tree_id: TreeId::ROOT,
                    focus: root_id,
                },
                actionable,
            )
        };
        self.a11y_route = {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            actionable
                .into_iter()
                .map(|node| (AccessNodeId(dom_ref.opaque_id(node)), node))
                .collect()
        };
        a11y.install(&window, tree)
            .expect("install initial Cleromancy accessibility tree");
        self.window = Some(window);
        self.host = Some(host);
        self.runner = Some(runner);
        self.layout = Some(layout);
        self.layout_size = (lw, lh);
        self.a11y = Some(a11y);
        self.flush_queued_updates();
        self.window
            .as_ref()
            .expect("window just installed")
            .set_visible(true);
        self.window
            .as_ref()
            .expect("window just installed")
            .request_redraw();
        if self.scenario.is_some() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + SCENARIO_TICK));
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if !matches!(cause, StartCause::ResumeTimeReached { .. }) || self.scenario.is_none() {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        if let Some(mut run) = self.scenario.take() {
            match run.scenario.tick(self) {
                genet_probe::Progress::Done if self.pending_capture.is_none() => {
                    let outcome = run.scenario.finish();
                    scenario::write_done(
                        &run.dir,
                        run.phase,
                        &outcome,
                        self.scenario_observation(),
                        self.capture_error.as_deref(),
                    );
                    event_loop.exit();
                    return;
                }
                genet_probe::Progress::Done | genet_probe::Progress::Running => {
                    self.scenario = Some(run);
                }
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + SCENARIO_TICK));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: HostEvent) {
        match event {
            HostEvent::Worker(update) => self.apply_worker_update(update),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(host) = self.host.as_mut() {
                    host.resize(size.width.max(1), size.height.max(1));
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(host), Some(window)) = (self.host.as_mut(), self.window.as_ref()) {
                    let size = window.inner_size();
                    host.resize(size.width.max(1), size.height.max(1));
                    window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = cambium_winit::modifiers_from_winit(modifiers.state());
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self
                    .window
                    .as_ref()
                    .map_or(1.0, |window| window.scale_factor());
                self.cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.click(),
            WindowEvent::MouseWheel { delta, .. } => self.scroll(delta),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.key(&event)
            }
            WindowEvent::Ime(event) => self.ime(&event),
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
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
