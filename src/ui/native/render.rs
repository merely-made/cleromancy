// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Frame production and input dispatch over the retained layout.

use accesskit::{Action, NodeId as AccessNodeId, Tree, TreeId, TreeUpdate};
use cambium::{PointerClick, Propagation};
use genet_layout::{IncrementalLayout, ScrollOffsets};
use genet_scripted_dom::{NodeId, ScriptedDom};
use genet_winit_host::{BridgeStatus, wheel_delta_from_winit};
use layout_dom_api::{DomMutation, LayoutDom as _, LayoutDomMut as _};
use netrender::{ColorLoad, ExternalTexturePlacement};
use paint_list_api::{DeviceIntSize, PaintList as _};

use super::{NativeApp, SHEET};
use crate::ui::scenario;

impl NativeApp {
    /// Route screen-reader clicks through the same Cambium action path as a
    /// mouse press. The previous frame's tree is precisely the tree the OS saw.
    pub(super) fn pump_a11y_actions(&mut self) {
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

    pub(super) fn redraw(&mut self) {
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
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        host.queue().present(frame);
        if let Some(path) = self.pending_capture.take()
            && !scenario::capture_frame(host, &view, pw, ph, &path)
        {
            self.capture_error = Some(format!("write {}", path.display()));
        }
    }

    pub(super) fn click(&mut self) {
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

    pub(super) fn key(&mut self, event: &winit::event::KeyEvent) {
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

    pub(super) fn ime(&mut self, event: &winit::event::Ime) {
        let Some(runner) = self.runner.as_mut() else {
            return;
        };
        let actions = runner.dispatch_key(cambium_winit::ime_event_from_winit(event));
        self.submit_actions(actions);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    pub(super) fn scroll(&mut self, delta: winit::event::MouseScrollDelta) {
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
