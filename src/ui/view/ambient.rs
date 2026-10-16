// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::ConsultationView;
use crate::{
    ambient::*,
    ui::{ConsultationAction, ConsultationUi},
};
use cambium::{GraphCanvasEvent, PointerPhase, button, el, graph_canvas};
use seiche::Role;

pub(super) fn neighborhood(ui: &ConsultationUi) -> ConsultationView {
    let Some(detail) = ui.detail.as_ref() else {
        return Box::new(el::<_, ConsultationUi, ()>("div", ()));
    };
    let position = ui.focused_position(detail);
    let matching = ui
        .ambient
        .context
        .as_ref()
        .filter(|c| c.data.session_id == detail.session.id && c.data.position == position);
    let Some(context) = matching else {
        let session = detail.session.id.clone();
        return Box::new(
            button(
                "Explore this card’s neighborhood",
                move |ui: &mut ConsultationUi, _| {
                    ui.neighborhood_action(ConsultationAction::OpenNeighborhood {
                        session_id: session.clone(),
                        position: position.clone(),
                    })
                },
            )
            .attr("data-key", "open-neighborhood"),
        );
    };
    let data = &context.data;
    let view = &context.view;
    let mut children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Card neighborhood")),
        Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "Focus and foreground emphasis are view choices. Examine records an item in Recent memory; Keep promotes it to Saved memory. Position pins and anchors do not keep an item.",
        )),
    ];
    let mut controls: Vec<ConsultationView> = Vec::new();
    controls.push(Box::new(
        button(
            if view.paused { "Play" } else { "Pause" },
            |ui: &mut ConsultationUi, _| {
                if let Some(c) = &mut ui.ambient.context {
                    c.view.paused = !c.view.paused;
                }
            },
        )
        .attr("data-key", "ambient-pause"),
    ));
    controls.push(Box::new(
        button(
            if view.reduced_motion {
                "Allow motion"
            } else {
                "Still / reduced motion"
            },
            |ui: &mut ConsultationUi, _| {
                if let Some(c) = &mut ui.ambient.context {
                    c.view.reduced_motion = !c.view.reduced_motion;
                }
            },
        )
        .attr("data-key", "ambient-still"),
    ));
    for (label, kind) in [("Constellation", "spiral"), ("Grid scene", "grid.ordered")] {
        controls.push(Box::new(
            button(label, move |ui: &mut ConsultationUi, _| {
                if let Some(c) = &mut ui.ambient.context {
                    let mut draft = scenograph::relationship::RelationshipRecipeDraft::new(
                        c.view.snapshot.recipe.clone(),
                    );
                    let mut arrangement = draft.recipe().definition.arrangement.clone();
                    arrangement.kind = kind.into();
                    draft.apply(scenograph::relationship::RecipeEdit::SetArrangement(
                        arrangement,
                    ));
                    c.view.snapshot.recipe = draft.to_recipe().expect("supported arrangement edit");
                    c.view.positions.clear();
                    c.view.paused = kind == "grid.ordered";
                    ui.ambient.generation += 1;
                }
            })
            .attr("data-key", format!("ambient-scene:{kind}")),
        ));
    }
    for (label, id) in [
        ("Settling · Springs", "spring.rapier"),
        ("Living · Flock", "flock.boids"),
    ] {
        controls.push(Box::new(
            button(label, move |ui: &mut ConsultationUi, _| {
                if let Some(c) = &mut ui.ambient.context {
                    c.view.snapshot.recipe.definition.dynamics =
                        Some(supported_motion(id).expect("shared supported preset"));
                    ui.ambient.generation += 1;
                }
            })
            .attr("data-key", format!("ambient-motion:{id}")),
        ));
    }
    controls.push(Box::new(
        button("Save neighborhood view", |ui: &mut ConsultationUi, _| {
            if let Some(c) = &ui.ambient.context {
                let view = c.view.clone();
                ui.neighborhood_action(ConsultationAction::SaveNeighborhood { view });
            }
        })
        .attr("data-key", "save-neighborhood"),
    ));
    controls.push(Box::new(
        button("Close neighborhood", |ui: &mut ConsultationUi, _| {
            ui.ambient.close()
        })
        .attr("data-key", "close-neighborhood"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("div", controls)
            .attr("class", "ambient-controls")
            .attr("role", "group")
            .attr("aria-label", "Neighborhood scene and motion"),
    ));
    children.push(super::labelled_text(
        "Neighbor mere domain",
        "neighbor-mere-domain",
        false,
        &ui.ambient.neighbor_domain,
        neighbor_domain,
    ));
    let grant_controls: Vec<ConsultationView> = vec![
        Box::new(
            button(
                "Read granted neighboring context",
                |ui: &mut ConsultationUi, _| {
                    let domain = ui.ambient.neighbor_domain.text().trim().to_string();
                    ui.neighborhood_action(ConsultationAction::ReadGrantedContext { domain });
                },
            )
            .attr("data-key", "ambient-read-granted"),
        ),
        Box::new(
            button("Clear neighboring context", |ui: &mut ConsultationUi, _| {
                ui.neighborhood_action(ConsultationAction::ClearGrantedContext)
            })
            .attr("data-key", "ambient-clear-granted"),
        ),
    ];
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("div", grant_controls).attr("class", "ambient-controls"),
    ));
    if !ui.ambient.granted_notice.is_empty() {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("p", ui.ambient.granted_notice.clone())
                .attr("data-key", "ambient-grant-notice"),
        ));
    }
    for granted in &ui.ambient.granted_cards {
        let mut values: Vec<ConsultationView> = vec![Box::new(el::<_, ConsultationUi, ()>(
            "h4",
            granted.card.title.clone(),
        ))];
        values.push(Box::new(el::<_,ConsultationUi,()>("p",format!("Granted neighboring context · {} · cleromancy.granted-context v1 · resident-ambient-route/v1. Read-only preview; keeping remains with its owning application.",granted.domain))));
        for v in &granted.card.values {
            values.push(Box::new(el::<_, ConsultationUi, ()>(
                "p",
                format!("{}: {}", v.label, v.value),
            )));
        }
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("article", values)
                .attr("data-key", format!("granted:{}", granted.resource)),
        ));
    }
    let mut filters: Vec<ConsultationView> = Vec::new();
    for family in RelationFamily::ALL {
        let count = data.relations.iter().filter(|r| r.family == family).count();
        filters.push(Box::new(
            button(
                format!("{} ({count})", family.label()),
                move |ui: &mut ConsultationUi, _| {
                    if let Some(c) = &mut ui.ambient.context {
                        if !c.view.families.remove(&family) {
                            c.view.families.insert(family);
                        }
                        ui.ambient.generation += 1;
                    }
                },
            )
            .attr("data-key", format!("ambient-lens:{family:?}"))
            .attr(
                "aria-pressed",
                if view.families.contains(&family) {
                    "true"
                } else {
                    "false"
                },
            ),
        ));
    }
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("div", filters)
            .attr("class", "ambient-controls")
            .attr("role", "group")
            .attr("aria-label", "Relation families"),
    ));
    if let Some(error) = &ui.ambient.error {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("p", error.clone()).attr("role", "alert"),
        ));
    }
    let canvas: ConsultationView = if let Some(swatch) = &ui.ambient.swatch {
        Box::new(graph_canvas(
            swatch,
            |ui: &mut ConsultationUi, event| match event {
                GraphCanvasEvent::Activate(id) => ui.focus_neighbor(id),
                GraphCanvasEvent::RelationActivate(id) => {
                    if let Some(c) = &mut ui.ambient.context {
                        c.view.snapshot.selected_relationship = Some(id);
                    }
                },
                GraphCanvasEvent::Pan { delta } => {
                    if let Some(c) = &mut ui.ambient.context {
                        c.view.pan_milli[0] = (c.view.pan_milli[0] as f32 + delta.0 * 1000.0)
                            .round()
                            .clamp(-100000.0, 100000.0)
                            as i32;
                        c.view.pan_milli[1] = (c.view.pan_milli[1] as f32 + delta.1 * 1000.0)
                            .round()
                            .clamp(-100000.0, 100000.0)
                            as i32;
                    }
                },
                GraphCanvasEvent::Zoom { factor } => {
                    if let Some(c) = &mut ui.ambient.context {
                        c.view.zoom_milli = (c.view.zoom_milli as f32 * factor)
                            .round()
                            .clamp(100.0, 8000.0)
                            as u32;
                    }
                },
                GraphCanvasEvent::Drag(drag) => {
                    if matches!(drag.phase, PointerPhase::Up | PointerPhase::Cancel) {
                        if let Some(c) = &mut ui.ambient.context {
                            if drag.phase != PointerPhase::Cancel
                                && c.view.roles.get(&drag.id) != Some(&Role::Anchored)
                            {
                                let basis = ui.ambient.drag_basis;
                                let at = [
                                    (basis[0][0] as f64
                                        + (basis[1][0] - basis[0][0]) as f64
                                            * f64::from(drag.position.0))
                                    .round() as i32,
                                    (basis[0][1] as f64
                                        + (basis[1][1] - basis[0][1]) as f64
                                            * f64::from(drag.position.1))
                                    .round() as i32,
                                ];
                                c.view.positions.insert(drag.id, at);
                            }
                        }
                        ui.ambient.dragging = None;
                        ui.ambient.generation += 1;
                    } else {
                        ui.ambient.dragging = Some((drag.id, drag.position));
                    }
                },
                GraphCanvasEvent::Expand => {},
            },
        ))
    } else {
        Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "Choose a relation family with saved material to show its scene.",
        ))
    };
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("div", canvas)
            .attr("class", "ambient-canvas-holder")
            .attr("data-key", "ambient-canvas-holder"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            format!(
                "{} items · {} disclosed relations · {}. {}",
                data.items.len(),
                data.relations.len(),
                if view.paused || view.reduced_motion {
                    "Still"
                } else {
                    "Motion enabled"
                },
                data.disclosures.join(" ")
            ),
        )
        .attr("data-key", "ambient-disclosure"),
    ));
    if let Some(back) = &ui.ambient.back {
        let (session, position) = back.clone();
        children.push(Box::new(
            button(
                "Return to previous reading",
                move |ui: &mut ConsultationUi, _| {
                    ui.ambient.back = None;
                    ui.neighborhood_action(ConsultationAction::OpenNeighborhood {
                        session_id: session.clone(),
                        position: position.clone(),
                    });
                },
            )
            .attr("data-key", "ambient-back"),
        ));
    }
    let selected = view
        .snapshot
        .selected_occurrence
        .as_ref()
        .and_then(|id| data.item(id))
        .unwrap_or(&data.subject);
    let selected_id = selected.id.clone();
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("h4", selected.title.clone())
            .attr("data-key", "ambient-selected-title"),
    ));
    let memory = ui.catalog.ambient_memory.iter().find(|m| {
        m.memory.session_id == data.session_id
            && m.memory.position == data.position
            && m.memory.item.id == selected.id
    });
    if selected.id != data.subject.id {
        let mut actions: Vec<ConsultationView> = Vec::new();
        for (label, keep) in [("Examine · Recent", false), ("Keep · Saved", true)] {
            let session = data.session_id.clone();
            let position = data.position.clone();
            let id = selected_id.clone();
            actions.push(Box::new(
                button(label, move |ui: &mut ConsultationUi, _| {
                    ui.neighborhood_action(ConsultationAction::ExamineNeighbor {
                        session_id: session.clone(),
                        position: position.clone(),
                        item_id: id.clone(),
                        keep,
                    })
                })
                .attr(
                    "data-key",
                    if keep {
                        "ambient-keep"
                    } else {
                        "ambient-examine"
                    },
                ),
            ));
        }
        let id = selected_id.clone();
        actions.push(Box::new(
            button(
                if view.foreground.contains(&selected.id) {
                    "Move to background"
                } else {
                    "Emphasize in foreground"
                },
                move |ui: &mut ConsultationUi, _| {
                    if let Some(c) = &mut ui.ambient.context {
                        if !c.view.foreground.remove(&id) {
                            c.view.foreground.insert(id.clone());
                        }
                    }
                },
            )
            .attr("data-key", "ambient-foreground"),
        ));
        for role in Role::ALL {
            let id = selected_id.clone();
            actions.push(Box::new(
                button(
                    format!("Position: {}", role.label()),
                    move |ui: &mut ConsultationUi, _| {
                        if let Some(c) = &mut ui.ambient.context {
                            c.view.roles.insert(id.clone(), role);
                            ui.ambient.generation += 1;
                        }
                    },
                )
                .attr("data-key", format!("ambient-role:{}", role.id()))
                .attr(
                    "aria-pressed",
                    if view.roles.get(&selected.id).copied().unwrap_or_default() == role {
                        "true"
                    } else {
                        "false"
                    },
                ),
            ));
        }
        if let (Some(session), Some(position)) = (&selected.session_id, &selected.position) {
            let session = session.clone();
            let position = position.clone();
            actions.push(Box::new(
                button("Open that reading", move |ui: &mut ConsultationUi, _| {
                    if let Some(c) = &ui.ambient.context {
                        ui.ambient.back =
                            Some((c.data.session_id.clone(), c.data.position.clone()));
                    }
                    ui.neighborhood_action(ConsultationAction::OpenNeighborhood {
                        session_id: session.clone(),
                        position: position.clone(),
                    });
                })
                .attr("data-key", "ambient-open-reading"),
            ));
        }
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("div", actions).attr("class", "ambient-controls"),
        ));
    }
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            match memory {
                Some(m) if m.kept => "Saved memory",
                Some(_) => "Recent memory",
                None => "Ambient · focus has not kept this item",
            },
        )
        .attr("data-key", "ambient-memory-level"),
    ));
    if memory.is_some() || selected.id == data.subject.id {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("p", selected.body.clone())
                .attr("data-key", "ambient-inspection"),
        ));
    }
    for reason in data.relations_for(&selected.id) {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "{} · {} v{} · {}. {}",
                    reason.family.label(),
                    reason.engine,
                    reason.version,
                    reason.method,
                    reason.reason
                ),
            )
            .attr("data-key", format!("ambient-reason:{}", reason.id)),
        ));
    }
    let mut list: Vec<ConsultationView> = Vec::new();
    for family in RelationFamily::ALL {
        if !view.families.contains(&family) {
            continue;
        }
        list.push(Box::new(el::<_, ConsultationUi, ()>("h4", family.label())));
        let ids: std::collections::BTreeSet<_> = data
            .relations
            .iter()
            .filter(|r| r.family == family)
            .map(|r| r.item_id.as_str())
            .collect();
        for id in ids {
            if let Some(item) = data.item(id) {
                let id = item.id.clone();
                list.push(Box::new(
                    button(item.title.clone(), move |ui: &mut ConsultationUi, _| {
                        ui.focus_neighbor(id.clone())
                    })
                    .attr("data-key", format!("ambient-item:{}", item.id)),
                ));
            }
        }
    }
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("div", list)
            .attr("class", "ambient-item-list")
            .attr("role", "group")
            .attr(
                "aria-label",
                "Neighborhood items; accessible alternative to the canvas",
            ),
    ));
    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("class", "ambient-neighborhood")
            .attr("data-key", "ambient-neighborhood")
            .attr("aria-label", "Card neighborhood"),
    )
}

fn neighbor_domain(ui: &mut ConsultationUi) -> &mut cambium::TextInput {
    &mut ui.ambient.neighbor_domain
}
