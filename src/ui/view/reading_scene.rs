// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Accessible DOM realization of the shared scene, preserving its placement
//! and source/occurrence distinction without making the saved reading mutable.
use super::ConsultationView;
use crate::narrative::reading_layout;
use crate::reading_scene::reading_scene;
use crate::ui::state::ConsultationUi;
use cambium::{HoverPhase, button_with, el, on_hover};

pub(super) fn spread_view(ui: &ConsultationUi) -> ConsultationView {
    let detail = ui
        .detail
        .as_ref()
        .expect("reading scene needs a saved reading");
    let projection = reading_scene(detail);
    let layout = reading_layout(detail);
    let selected = ui.focused_position(detail);
    let mut cards: Vec<ConsultationView> = Vec::new();
    for (index, ((_item, position), meaning)) in projection
        .scene
        .items
        .iter()
        .zip(&projection.positions)
        .zip(&layout.positions)
        .enumerate()
    {
        let Some(reading) = detail
            .readings
            .iter()
            .find(|reading| reading.id == detail.session.placements[index].reading_id)
        else {
            continue;
        };
        let session = projection.session_id.clone();
        let target = position.clone();
        let label = meaning.label.clone();
        let content: Vec<ConsultationView> = vec![
            Box::new(
                el::<_, ConsultationUi, ()>("span", label.clone())
                    .attr("class", "scene-position")
                    .attr("data-key", format!("reading-position:{position}")),
            ),
            Box::new(
                el::<_, ConsultationUi, ()>("span", reading.display_title())
                    .attr("class", "scene-title")
                    .attr(
                        "data-key",
                        if index == 0 {
                            "result-title".into()
                        } else {
                            format!("result-title:{position}")
                        },
                    ),
            ),
        ];
        let card = button_with(content, move |ui: &mut ConsultationUi, _| {
            // A queued event for an older session cannot select the current one.
            if ui
                .detail
                .as_ref()
                .is_some_and(|detail| detail.session.id == session)
            {
                if ui
                    .ambient
                    .context
                    .as_ref()
                    .is_some_and(|c| c.data.position != target)
                {
                    ui.ambient.close();
                }
                ui.card_preview = None;
                ui.reading_focus.insert(session.clone(), target.clone());
            }
        })
        .attr("class", "scene-card")
        .attr("data-orientation", reading.orientation().label())
        .attr("data-key", format!("scene-card:{position}"))
        .attr(
            "aria-label",
            format!("{label}: {}", reading.display_title()),
        )
        .attr("title", meaning.meaning.clone())
        .attr(
            "aria-pressed",
            if *position == selected {
                "true"
            } else {
                "false"
            },
        );
        let session = projection.session_id.clone();
        let hovered = position.clone();
        cards.push(Box::new(on_hover(
            card,
            move |ui: &mut ConsultationUi, event| match event.phase {
                HoverPhase::Enter => {
                    ui.position_hover = Some((session.clone(), hovered.clone()));
                },
                HoverPhase::Leave => {
                    if ui
                        .position_hover
                        .as_ref()
                        .is_some_and(|(_, position)| position == &hovered)
                    {
                        ui.position_hover = None;
                    }
                },
                HoverPhase::Move => {},
            },
        )));
    }

    // The meaning of the hovered position, or of the focused one when the
    // pointer is elsewhere. Keyboard and assistive users reach the same text
    // through focus and the narration below.
    let shown = ui
        .position_hover
        .as_ref()
        .filter(|(session, _)| *session == projection.session_id)
        .map(|(_, position)| position.clone())
        .unwrap_or(selected);
    let meaning = layout
        .positions
        .iter()
        .find(|position| position.name == shown)
        .map(|position| {
            Box::new(
                el::<_, ConsultationUi, ()>(
                    "p",
                    vec![
                        Box::new(
                            el::<_, ConsultationUi, ()>("strong", position.label.clone())
                                .attr("class", "position-meaning-label"),
                        ) as ConsultationView,
                        Box::new(el::<_, ConsultationUi, ()>(
                            "span",
                            position.meaning.clone(),
                        )),
                    ],
                )
                .attr("class", "position-meaning")
                .attr("data-key", "position-meaning")
                .attr("data-position", position.name.clone()),
            ) as ConsultationView
        });

    let mut children: Vec<ConsultationView> = vec![Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            Box::new(
                el::<_, ConsultationUi, ()>("div", cards).attr("class", "reading-spread-cards"),
            ) as ConsultationView,
        )
        .attr("class", "reading-scene")
        .attr("role", "group")
        .attr("aria-label", "Reading spread")
        .attr("data-key", "reading-scene"),
    )];
    children.extend(meaning);
    Box::new(el::<_, ConsultationUi, ()>("div", children).attr("class", "reading-spread"))
}
