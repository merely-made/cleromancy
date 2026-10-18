// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::ConsultationCtx;
use crate::ambient_scene::NeighborhoodRuntime;

pub(super) fn sync(
    ctx: &mut ConsultationCtx<'_>,
    runtime: &mut Option<(u64, String, NeighborhoodRuntime)>,
) -> bool {
    let Some(context) = ctx.runner.state().consultation.ambient.context.clone() else {
        *runtime = None;
        return false;
    };
    if !context
        .data
        .relations
        .iter()
        .any(|r| context.view.families.contains(&r.family))
    {
        if ctx.runner.state().consultation.ambient.swatch.is_some() {
            ctx.runner.update(|s| s.consultation.ambient.swatch = None);
        }
        return false;
    }
    let width = {
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            taproot::matching_classes(&dom, &taproot::Selector::class("ambient-canvas-holder"))
                .into_iter()
                .next()
        };
        let Some(rect) = node.and_then(|node| ctx.painted_rect(node)) else {
            return false;
        };
        rect.2.floor().max(120.0) as u32
    };
    let generation = ctx.runner.state().consultation.ambient.generation;
    let signature = context.data.revision.clone();
    if runtime
        .as_ref()
        .is_none_or(|(g, s, _)| *g != generation || *s != signature)
    {
        match NeighborhoodRuntime::new(&context.data, &context.view) {
            Ok(next) => *runtime = Some((generation, signature, next)),
            Err(error) => {
                ctx.runner.update(|state| {
                    state.consultation.ambient.error = Some(error);
                    state.consultation.ambient.swatch = None;
                });
                return false;
            },
        }
    }
    let (_, _, live) = runtime.as_mut().expect("bound runtime");
    let dragging = ctx.runner.state().consultation.ambient.dragging.clone();
    let moving = !context.view.paused
        && !context.view.reduced_motion
        && dragging.is_none()
        && live.dynamics.is_running();
    if moving {
        if let Err(error) = live.tick() {
            ctx.runner.update(|state| {
                state.consultation.ambient.error = Some(error);
                if let Some(c) = &mut state.consultation.ambient.context {
                    c.view.paused = true;
                }
            });
            return false;
        }
    }
    let mut view = context.view.clone();
    live.retain_pose(&mut view);
    let mut swatch = live.swatch(&context.data, &view, width);
    swatch.viewport.pan = (
        view.pan_milli[0] as f32 / 1000.0,
        view.pan_milli[1] as f32 / 1000.0,
    );
    swatch.viewport.zoom = view.zoom_milli as f32 / 1000.0;
    if let Some((id, at)) = dragging {
        if let Some(node) = swatch.graph.nodes.iter_mut().find(|n| n.id == id) {
            node.position = at;
        }
    }
    let changed = ctx.runner.state().consultation.ambient.swatch.as_ref() != Some(&swatch);
    ctx.leaves.insert(
        crate::ambient_scene::AMBIENT_LEAF,
        Box::new(swatch.paint_leaf(|family| match family {
            crate::ambient::RelationFamily::Correspondence => sprigging::ColorF {
                r: 0.80,
                g: 0.62,
                b: 0.22,
                a: 1.0,
            },
            crate::ambient::RelationFamily::History => sprigging::ColorF {
                r: 0.35,
                g: 0.65,
                b: 0.55,
                a: 1.0,
            },
            crate::ambient::RelationFamily::Situation => sprigging::ColorF {
                r: 0.42,
                g: 0.58,
                b: 0.85,
                a: 1.0,
            },
            crate::ambient::RelationFamily::Spread => sprigging::ColorF {
                r: 0.70,
                g: 0.45,
                b: 0.60,
                a: 1.0,
            },
        })),
    );
    if changed || moving {
        ctx.runner.update(|state| {
            let ui = &mut state.consultation.ambient;
            ui.swatch = Some(swatch);
            ui.width = width;
            ui.drag_basis = [live.world_at((0.0, 0.0)), live.world_at((1.0, 1.0))];
            ui.ticks = live.dynamics.ticks();
            if let Some(c) = &mut ui.context {
                c.view.positions = view.positions;
            }
        });
    }
    moving || changed
}
