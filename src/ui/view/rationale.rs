// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::field::{
    CONTEXTUAL_WEIGHT_RULE, EXTERNAL_TERM_WEIGHT_RULE, UNIFORM_DIE_RULE, UNIFORM_RULE,
};
use crate::reading_scene::{position_label, position_rationale};
use crate::{ConsultationDetail, Reading, SelectionMode};
use cambium::{DetailRow, DetailSection};

pub(super) fn section(
    detail: &ConsultationDetail,
    position: &str,
    reading: &Reading,
) -> DetailSection {
    let receipt = &reading.receipt;
    let selection = match receipt.mode {
        SelectionMode::Cast => {
            "A fresh chance draw chose this result from the available possibilities, using their relative chances."
        },
        SelectionMode::Calculated => {
            "This result had the highest score after applying the collection's rules to your saved context. When scores tie, the first result in the collection wins."
        },
        SelectionMode::Derived => {
            "A repeatable draw chose this result using your saved context, the collection, and your chosen seed and domain. The same inputs reproduce the same result."
        },
    };
    let qualification = match detail.field.rules.as_str() {
        UNIFORM_RULE | UNIFORM_DIE_RULE => "Every possibility had an equal chance. Your question and tags were saved as context; they did not change the odds.".to_string(),
        CONTEXTUAL_WEIGHT_RULE | EXTERNAL_TERM_WEIGHT_RULE => {
            let tags = detail.field.candidates.iter().find(|candidate| candidate.id == reading.candidate_id)
                .map(|candidate| candidate.tags.intersection(&detail.context.tags).cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            let matched = if tags.is_empty() { "This result shared no tags with your saved context.".into() }
                else { format!("This result matched these saved tags: {}.", tags.join(", ")) };
            format!("Each possibility starts with a weight set by the collection. Each matching context tag adds another share of that starting weight. {matched}")
        }
        _ => "The saved collection defines which possibilities were eligible and how they were scored.".to_string(),
    };
    let mut rows = vec![
        DetailRow::new("The position", position_rationale(position)),
        DetailRow::new("The choice", selection),
        DetailRow::new("What influenced it", qualification),
    ];
    if let Some(enrichment) = &receipt.enrichment {
        let terms = enrichment
            .candidate_terms
            .get(receipt.selected_index)
            .cloned()
            .unwrap_or_default();
        rows.push(DetailRow::new(
            "Additional context",
            if terms.is_empty() {
                "Saved external context was considered, but added no matching terms to this result."
                    .into()
            } else {
                format!(
                    "These terms in the saved external context increased this result's weight: {}.",
                    terms.join(", ")
                )
            },
        ));
    }
    if receipt.mode != SelectionMode::Calculated && receipt.total_weight > 0 {
        if let Some(weight) = receipt.qualified_weights.get(receipt.selected_index) {
            rows.push(DetailRow::new("Its chance", format!("About {:.1}% for this draw. Each position is drawn separately, so a result can appear more than once.", *weight as f64 / receipt.total_weight as f64 * 100.0)));
        }
    }
    rows.push(DetailRow::new("The interpretation", "The words come from the selected entry in the saved collection. Read them through your question and this position; they were not newly written from your question."));
    DetailSection::new(
        format!("{}: {}", position_label(position), reading.title),
        rows,
    )
}
