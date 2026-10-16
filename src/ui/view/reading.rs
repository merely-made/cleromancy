// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reading region: per-position results and the collapsed workings.

use cambium::{button, detail_panel, disclosure, el, map_action, map_state};

use super::ConsultationView;
use crate::narrative::{ReadingNarrative, narrate, position_meaning};
use crate::ui::screen::{self, ConsultationScreen};
use crate::ui::state::ConsultationUi;
use crate::{ASTROLOGY_FACTS_ROLE, ConsultationDetail, format_relative_date};

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
            children.push(context_with_reading(ui, detail));
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
            let narrative = narrate(detail);
            let focused = ui.focused_position(detail);
            if let Some((placement, reading)) = detail
                .session
                .placements
                .iter()
                .find(|placement| placement.position == focused)
                .and_then(|placement| {
                    detail
                        .readings
                        .iter()
                        .find(|reading| reading.id == placement.reading_id)
                        .map(|reading| (placement, reading))
                })
            {
                let meaning = position_meaning(detail, &placement.position);
                let card: Vec<ConsultationView> = vec![
                    Box::new(el::<_, ConsultationUi, ()>("h3", meaning.label.clone())),
                    Box::new(el::<_, ConsultationUi, ()>("h4", reading.title.clone())),
                    Box::new(
                        el::<_, ConsultationUi, ()>("p", reading.interpretation.clone())
                            .attr("data-key", "result-prompt"),
                    ),
                    Box::new(
                        el::<_, ConsultationUi, ()>("p", meaning.meaning.clone())
                            .attr("class", "position-rationale")
                            .attr("data-key", "focused-position-meaning"),
                    ),
                ];
                children.push(Box::new(
                    el::<_, ConsultationUi, ()>("article", card)
                        .attr("class", "reading-card")
                        .attr("data-key", "focused-reading")
                        .attr("aria-live", "polite"),
                ));
            }
            if let Some(card) = super::typed_card::card_view(ui, detail, &focused) {
                children.push(card);
            }
            children.push(narrative_view(&narrative, &focused));
            let sections = detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| {
                    super::rationale::section(
                        detail,
                        &position_meaning(detail, &placement.position),
                        reading,
                    )
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

/// The question and the context it was asked in, shown as part of the
/// reading rather than in a separate authoring region.
fn context_with_reading(ui: &ConsultationUi, detail: &ConsultationDetail) -> ConsultationView {
    let context = &detail.context;
    let question = context
        .facts
        .get("question")
        .cloned()
        .unwrap_or_else(|| context.label.clone());
    let mut facts = vec![format!(
        "{} · drawn {}",
        context.label,
        format_relative_date(detail.session.created_at_ms, ui.journal.now_ms)
    )];
    if !context.tags.is_empty() {
        facts.push(format!(
            "tags: {}",
            context.tags.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    facts.extend(
        context
            .facts
            .iter()
            .filter(|(name, _)| name.as_str() != "question")
            .map(|(name, value)| format!("{name}: {value}")),
    );
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            vec![
                Box::new(
                    el::<_, ConsultationUi, ()>("p", question)
                        .attr("class", "reading-question")
                        .attr("data-key", "reading-question"),
                ) as ConsultationView,
                Box::new(
                    el::<_, ConsultationUi, ()>("p", facts.join(" · "))
                        .attr("class", "reading-context-facts")
                        .attr("data-key", "reading-context-facts"),
                ),
            ],
        )
        .attr("class", "reading-context")
        .attr("data-key", "reading-context")
        .attr("aria-label", "Question and context"),
    )
}

/// Every card's computation in plain words, in draw order. This is the
/// reading's audit; the receipt below stays available for exact values.
fn narrative_view(narrative: &ReadingNarrative, focused: &str) -> ConsultationView {
    let items = narrative
        .cards
        .iter()
        .map(|card| {
            let current = card.position.name == focused;
            Box::new(
                el::<_, ConsultationUi, ()>(
                    "li",
                    vec![
                        Box::new(
                            el::<_, ConsultationUi, ()>(
                                "strong",
                                format!(
                                    "Card {} of {} · {} · {}",
                                    card.ordinal, card.count, card.position.label, card.title
                                ),
                            )
                            .attr("class", "narration-heading"),
                        ) as ConsultationView,
                        Box::new(
                            el::<_, ConsultationUi, ()>("p", card.text.clone())
                                .attr("class", "narration-text"),
                        ),
                    ],
                )
                .attr(
                    "class",
                    if current {
                        "narration current"
                    } else {
                        "narration"
                    },
                )
                .attr("aria-current", if current { "true" } else { "false" })
                .attr("data-key", format!("narration:{}", card.position.name)),
            ) as ConsultationView
        })
        .collect::<Vec<_>>();
    Box::new(
        el::<_, ConsultationUi, ()>(
            "section",
            vec![
                Box::new(el::<_, ConsultationUi, ()>(
                    "h3",
                    "How this reading was made",
                )) as ConsultationView,
                Box::new(
                    el::<_, ConsultationUi, ()>("p", narrative.overview.clone())
                        .attr("class", "narration-overview")
                        .attr("data-key", "narration-overview"),
                ),
                Box::new(el::<_, ConsultationUi, ()>("ol", items).attr("class", "narration-list")),
            ],
        )
        .attr("class", "reading-narrative")
        .attr("role", "region")
        .attr("aria-label", "How this reading was made")
        .attr("data-key", "reading-narrative"),
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
