// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reading region: per-position results and the collapsed workings.

use cambium::{button, detail_panel, disclosure, el, map_action, map_state};

use super::ConsultationView;
use crate::ASTROLOGY_FACTS_ROLE;
use crate::reading_scene::{position_label, position_rationale};
use crate::ui::screen::{self, ConsultationScreen};
use crate::ui::state::ConsultationUi;

pub(super) fn reading_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ()>("h2", "Reading"))];
    match ui.detail.as_ref() {
        None => children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                "Enter a context and make a reading to see its prompt and receipt.",
            )
            .attr("class", "empty-reading"),
        )),
        Some(detail) => {
            children.push(Box::new(
                el::<_, ConsultationUi, ()>(
                    "p",
                    detail
                        .context
                        .facts
                        .get("question")
                        .cloned()
                        .unwrap_or_else(|| detail.context.label.clone()),
                )
                .attr("class", "reading-question")
                .attr("data-key", "reading-question"),
            ));
            if let Some((_concurrence_id, facts_digest, placements)) = chart_concurrence(ui) {
                let link_digest = facts_digest.clone();
                children.push(Box::new(
                    el::<_, ConsultationUi, ()>(
                        "aside",
                        vec![
                            Box::new(el::<_, ConsultationUi, ()>("h3", "Chart alongside this reading"))
                                as ConsultationView,
                            Box::new(el::<_, ConsultationUi, ()>(
                                "p",
                                "This chart was saved alongside the draw. Explore its astrological reading as another perspective on the same occasion; it did not choose the cards.",
                            )) as ConsultationView,
                            Box::new(el::<_, ConsultationUi, ()>(
                                "p",
                                placements,
                            )) as ConsultationView,
                            Box::new(
                                button("Open concurrent chart", move |ui: &mut ConsultationUi, _| {
                                    ui.select_chart_facts(&link_digest);
                                    screen::select_surface(
                                        &mut ui.surface_tabs,
                                        ConsultationScreen::Chart,
                                    );
                                })
                                .attr("data-key", "open-concurrent-chart")
                                .attr("aria-label", "Open concurrent chart"),
                            ) as ConsultationView,
                        ],
                    )
                    .attr("role", "region")
                    .attr("aria-label", "Chart alongside this reading")
                    .attr("data-key", "reading-chart-concurrence"),
                ));
            }
            children.push(super::reading_scene::spread_view(ui));
            let selected = ui
                .reading_focus
                .get(&detail.session.id)
                .and_then(|position| {
                    detail
                        .session
                        .placements
                        .iter()
                        .find(|item| &item.position == position)
                })
                .or_else(|| detail.session.placements.first());
            if let Some(placement) = selected {
                if let Some(reading) = detail
                    .readings
                    .iter()
                    .find(|reading| reading.id == placement.reading_id)
                {
                    let card: Vec<ConsultationView> = vec![
                        Box::new(el::<_, ConsultationUi, ()>(
                            "h3",
                            position_label(&placement.position),
                        )),
                        Box::new(el::<_, ConsultationUi, ()>("h4", reading.title.clone())),
                        Box::new(
                            el::<_, ConsultationUi, ()>("p", reading.interpretation.clone())
                                .attr("data-key", "result-prompt"),
                        ),
                        Box::new(
                            el::<_, ConsultationUi, ()>(
                                "p",
                                position_rationale(&placement.position),
                            )
                            .attr("class", "position-rationale"),
                        ),
                    ];
                    children.push(Box::new(
                        el::<_, ConsultationUi, ()>("article", card)
                            .attr("class", "reading-card")
                            .attr("data-key", "focused-reading")
                            .attr("aria-live", "polite"),
                    ));
                }
            }
            let sections = detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| {
                    super::rationale::section(detail, &placement.position, reading)
                })
                .collect::<Vec<_>>();
            let details = detail_panel::<cambium::DisclosureState, ()>(&sections);
            // Cambium's `disclosure` reports the toggle back to the caller
            // rather than mutating its own state (mere `crates/cambium/cambium/
            // src/disclosure.rs`). This view's state *is* the disclosure state,
            // so the handler is `DisclosureState::toggle`, which is the form
            // that crate's own doc names for exactly this case.
            let workings = map_action(
                disclosure(&ui.workings, details, cambium::DisclosureState::toggle),
                never_disclosure_action,
            );
            let workings = map_state(workings, workings_state);
            children.push(Box::new(workings));
        },
    }

    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Reading")
            .attr("data-key", "region:reading"),
    )
}

/// Resolve only canonical astrology-facts addresses that remain in the
/// replay-verified catalog. A stale or unknown member is intentionally absent
/// rather than projected as a chart.
fn chart_concurrence(ui: &ConsultationUi) -> Option<(String, String, String)> {
    const FACTS_PREFIX: &str = "cleromancy://astrology/facts/";
    ui.detail
        .as_ref()?
        .concurrences
        .iter()
        .find_map(|concurrence| {
            let digest = concurrence.members.iter().find_map(|member| {
                (member.role == ASTROLOGY_FACTS_ROLE)
                    .then(|| member.address.strip_prefix(FACTS_PREFIX))
                    .flatten()
            })?;
            let stored = ui
                .catalog
                .astrology_charts
                .iter()
                .find(|stored| stored.facts.digest() == digest)?;
            let placements = stored
                .facts
                .placements
                .iter()
                .map(|placement| format!("{} {:?}", placement.body, placement.sign))
                .collect::<Vec<_>>()
                .join(", ");
            Some((concurrence.id.clone(), digest.to_string(), placements))
        })
}

fn workings_state(ui: &mut ConsultationUi) -> &mut cambium::DisclosureState {
    &mut ui.workings
}

fn never_disclosure_action(_: &mut cambium::DisclosureState, _: ()) {
    unreachable!("disclosures do not bubble unit actions")
}
