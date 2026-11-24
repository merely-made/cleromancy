// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{ConsultationUi, ConsultationView};
use crate::cards::{Correspondence, CorrespondenceTerm, TarotDeck};
use cambium::{button, el};

pub(super) fn append_terms(
    rows: &mut Vec<ConsultationView>,
    ui: &ConsultationUi,
    deck: &TarotDeck,
    citation: &Correspondence,
    session: &str,
    field_digest: &str,
) {
    use CorrespondenceTerm::{Element, Planet, Sign};
    let terms = [
        (
            "Element",
            citation
                .elements
                .iter()
                .map(|v| (Element(*v), format!("{v:?}")))
                .collect::<Vec<_>>(),
        ),
        (
            "Planet",
            citation
                .planets
                .iter()
                .map(|v| (Planet(*v), format!("{v:?}")))
                .collect(),
        ),
        (
            "Sign",
            citation
                .signs
                .iter()
                .map(|v| (Sign(*v), format!("{v:?}")))
                .collect(),
        ),
    ];
    for (axis, values) in terms {
        if values.is_empty() {
            rows.push(Box::new(el::<_, ConsultationUi, ()>(
                "p",
                format!("{axis}: not transcribed"),
            )));
            continue;
        }
        for (term, label) in values {
            let selection = (citation.tradition.clone(), term);
            let selected = ui.correspondence.as_ref() == Some(&selection);
            let key = format!("correspondence-term:{}:{axis}:{label}", citation.source_id);
            let current_session = session.to_owned();
            rows.push(Box::new(
                button(
                    format!("{axis}: {label}"),
                    move |ui: &mut ConsultationUi, _| {
                        if ui
                            .detail
                            .as_ref()
                            .is_some_and(|d| d.session.id == current_session)
                        {
                            ui.correspondence = (ui.correspondence.as_ref() != Some(&selection))
                                .then(|| selection.clone());
                        }
                    },
                )
                .attr("aria-expanded", if selected { "true" } else { "false" })
                .attr("data-key", key),
            ));
            if selected {
                let mut results: Vec<ConsultationView> =
                    vec![Box::new(el::<_, ConsultationUi, ()>(
                        "p",
                        format!(
                            "{} cards with {label} in {}",
                            citation.tradition, deck.title
                        ),
                    ))];
                for card in deck.cards_with_correspondence(citation, term) {
                    let target = (field_digest.to_owned(), card.id.clone());
                    let current_session = session.to_owned();
                    results.push(Box::new(
                        button(card.title.clone(), move |ui: &mut ConsultationUi, _| {
                            if ui
                                .detail
                                .as_ref()
                                .is_some_and(|d| d.session.id == current_session)
                            {
                                ui.set_card_preview(Some(target.clone()));
                            }
                        })
                        .attr("data-key", format!("correspondence-card:{}", card.id)),
                    ));
                }
                rows.push(Box::new(
                    el::<_, ConsultationUi, ()>("section", results)
                        .attr("data-key", "correspondence-results"),
                ));
            }
        }
    }
}
