// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Accessible DOM realization of the shared scene, preserving its placement
//! and source/occurrence distinction without making the saved reading mutable.
use super::ConsultationView;
use crate::reading_scene::{CARD_HEIGHT, CARD_WIDTH, position_label, reading_scene};
use crate::ui::state::ConsultationUi;
use cambium::{button_with, el};

pub(super) fn spread_view(ui: &ConsultationUi) -> ConsultationView {
    let detail = ui
        .detail
        .as_ref()
        .expect("reading scene needs a saved reading");
    let projection = reading_scene(detail);
    let selected = ui
        .reading_focus
        .get(&projection.session_id)
        .map(String::as_str)
        .unwrap_or_else(|| {
            projection
                .positions
                .first()
                .map(String::as_str)
                .unwrap_or("")
        });
    let mut cards: Vec<ConsultationView> = Vec::new();
    for (index, (item, position)) in projection
        .scene
        .items
        .iter()
        .zip(&projection.positions)
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
        let label = position_label(position);
        let content: Vec<ConsultationView> = vec![
            Box::new(
                el::<_, ConsultationUi, ()>("span", label.clone())
                    .attr("class", "scene-position")
                    .attr("data-key", format!("reading-position:{position}")),
            ),
            Box::new(
                el::<_, ConsultationUi, ()>("span", reading.title.clone())
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
        cards.push(Box::new(button_with(content, move |ui: &mut ConsultationUi, _| {
            // A queued event for an older session cannot select the current one.
            if ui.detail.as_ref().is_some_and(|detail| detail.session.id == session) {
                ui.reading_focus.insert(session.clone(), target.clone());
            }
        }).attr("class", "scene-card")
            .attr("data-key", format!("scene-card:{position}"))
            .attr("aria-label", format!("{label}: {}", reading.title))
            .attr("aria-pressed", if position == selected { "true" } else { "false" })
            .attr("style", format!("position:absolute;left:{}px;top:{}px;width:{CARD_WIDTH}px;height:{CARD_HEIGHT}px;", item.transform.translate.x - CARD_WIDTH / 2.0, item.transform.translate.y - CARD_HEIGHT / 2.0))));
    }
    let width = projection.scene.bounds.size.w;
    let height = projection.scene.bounds.size.h;
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            Box::new(el::<_, ConsultationUi, ()>("div", cards).attr(
                "style",
                format!("position:relative;width:{width}px;height:{height}px;"),
            )) as ConsultationView,
        )
        .attr("class", "reading-scene")
        .attr("role", "group")
        .attr("aria-label", "Reading spread")
        .attr("data-key", "reading-scene"),
    )
}
