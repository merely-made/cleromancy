// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reading region: per-position results and the collapsed workings.

use cambium::{DetailRow, DetailSection, detail_panel, disclosure, el, map_action, map_state};

use super::{ConsultationView, mode_label};
use crate::Reading;
use crate::ui::state::{ConsultationAction, ConsultationUi};

pub(super) fn reading_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "h2", "Reading",
        ))];
    match ui.detail.as_ref() {
        None => children.push(Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "Enter a context and make a reading to see its prompt and receipt.",
            )
            .attr("class", "empty-reading"),
        )),
        Some(detail) => {
            for (index, (placement, reading)) in detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .enumerate()
            {
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("h3", placement.position.clone())
                        .attr(
                            "data-key",
                            format!("reading-position:{}", placement.position),
                        ),
                ));
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("h4", reading.title.clone()).attr(
                        "data-key",
                        if index == 0 {
                            "result-title".to_string()
                        } else {
                            format!("result-title:{}", placement.position)
                        },
                    ),
                ));
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>(
                        "p",
                        reading.interpretation.clone(),
                    )
                    .attr(
                        "data-key",
                        if index == 0 {
                            "result-prompt".to_string()
                        } else {
                            format!("result-prompt:{}", placement.position)
                        },
                    ),
                ));
            }
            let sections = detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| receipt_section(&placement.position, reading))
                .collect::<Vec<_>>();
            let details = detail_panel::<cambium::DisclosureState, ()>(&sections);
            let workings = map_action(disclosure(&ui.workings, details), never_disclosure_action);
            let workings = map_state(workings, workings_state);
            children.push(Box::new(workings));
        }
    }

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Reading")
            .attr("data-key", "region:reading"),
    )
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

fn never_disclosure_action(_: &mut cambium::DisclosureState, _: ()) -> ConsultationAction {
    unreachable!("disclosures do not bubble unit actions")
}
