// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The winit application lifecycle: window boot, event routing, and the
//! scenario tick.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use accesskit::{NodeId as AccessNodeId, Tree, TreeId, TreeUpdate};
use cambium::DomHandle;
use genet_layout::IncrementalLayout;
use genet_scripted_dom::{NodeId, ScriptedDom};
use genet_winit_host::{AccessKitBridge, SurfaceHost};
use layout_dom_api::LayoutDom as _;
use netrender::NetrenderOptions;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use super::{HostEvent, NativeApp, Runner, SCENARIO_TICK, SHEET};
use crate::ui::scenario;
use crate::{ConsultationCatalog, ConsultationUi, ConsultationView, consultation_view};

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
