// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reading region: per-position results and the collapsed workings.

use cambium::{
    DetailRow, DetailSection, button, detail_panel, disclosure, el, map_action, map_state,
};

use super::{ConsultationView, mode_label};
use crate::ui::screen::{self, ConsultationScreen};
use crate::ui::state::ConsultationUi;
use crate::{ASTROLOGY_FACTS_ROLE, Reading};

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
            if let Some((concurrence_id, facts_digest, placements)) = chart_concurrence(ui) {
                let link_digest = facts_digest.clone();
                children.push(Box::new(
                    el::<_, ConsultationUi, ()>(
                        "aside",
                        vec![
                            Box::new(el::<_, ConsultationUi, ()>("h3", "Chart concurrence"))
                                as ConsultationView,
                            Box::new(el::<_, ConsultationUi, ()>(
                                "p",
                                "This chart was recorded with this reading as a concurrence. It does not claim that the chart caused or interpreted the reading.",
                            )) as ConsultationView,
                            Box::new(el::<_, ConsultationUi, ()>(
                                "p",
                                format!("Receipt {concurrence_id}: {placements}"),
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
                    .attr("aria-label", "Chart concurrence")
                    .attr("data-key", "reading-chart-concurrence"),
                ));
            }
            let mut cards: Vec<ConsultationView> = Vec::new();
            for (index, (placement, reading)) in detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .enumerate()
            {
                let position_label = match placement.position.as_str() {
                    "foundation" => "Foundation",
                    "tension" => "Tension",
                    "next_step" => "Next step",
                    other => other,
                };
                let card = vec![
                    Box::new(
                        el::<_, ConsultationUi, ()>("h3", position_label.to_string()).attr(
                            "data-key",
                            format!("reading-position:{}", placement.position),
                        ),
                    ) as ConsultationView,
                    Box::new(
                        el::<_, ConsultationUi, ()>("h4", reading.title.clone()).attr(
                            "data-key",
                            if index == 0 {
                                "result-title".to_string()
                            } else {
                                format!("result-title:{}", placement.position)
                            },
                        ),
                    ) as ConsultationView,
                    Box::new(
                        el::<_, ConsultationUi, ()>("p", reading.interpretation.clone()).attr(
                            "data-key",
                            if index == 0 {
                                "result-prompt".to_string()
                            } else {
                                format!("result-prompt:{}", placement.position)
                            },
                        ),
                    ) as ConsultationView,
                ];
                cards.push(Box::new(
                    el::<_, ConsultationUi, ()>("article", card)
                        .attr("class", "reading-card")
                        .attr(
                            "aria-label",
                            format!("{}: {}", position_label, reading.title),
                        ),
                ));
            }
            children.push(Box::new(
                el::<_, ConsultationUi, ()>("div", cards).attr("class", "reading-cards"),
            ));
            let sections = detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| receipt_section(&placement.position, reading))
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

fn receipt_section(position: &str, reading: &Reading) -> DetailSection {
    let receipt = &reading.receipt;
    let mut rows = vec![
        DetailRow::new("Mode", mode_label(receipt.mode)),
        DetailRow::new("Algorithm", receipt.algorithm.clone()),
        DetailRow::new("Context digest", receipt.context_digest.clone()),
        DetailRow::new("Field digest", receipt.field_digest.clone()),
        DetailRow::new(
            "Qualified weights",
            receipt
                .qualified_weights
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        DetailRow::new("Total weight", receipt.total_weight.to_string()),
        DetailRow::new(
            "Bounded sample",
            receipt
                .sample
                .map_or_else(|| "not used".to_string(), |sample| sample.to_string()),
        ),
    ];
    if let Some(derivation) = &receipt.derivation {
        rows.push(DetailRow::new("Derived seed", derivation.seed.clone()));
        rows.push(DetailRow::new("Derived domain", derivation.domain.clone()));
    }
    if let Some(digest) = &receipt.derivation_digest {
        rows.push(DetailRow::new("Derivation digest", digest.clone()));
    }
    rows.push(DetailRow::new(
        "Selected index",
        receipt.selected_index.to_string(),
    ));
    DetailSection::new(format!("{} receipt", position), rows)
}

fn workings_state(ui: &mut ConsultationUi) -> &mut cambium::DisclosureState {
    &mut ui.workings
}

fn never_disclosure_action(_: &mut cambium::DisclosureState, _: ()) {
    unreachable!("disclosures do not bubble unit actions")
}
