// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Render saved typed content; inspection never writes or casts.
use super::ConsultationView;
use crate::{ConsultationDetail, ui::state::ConsultationUi};
use cambium::{button, el};

pub(super) fn card_view(
    ui: &ConsultationUi,
    detail: &ConsultationDetail,
    focused: &str,
) -> Option<ConsultationView> {
    let placement = detail
        .session
        .placements
        .iter()
        .find(|p| p.position == focused)?;
    let reading = detail
        .readings
        .iter()
        .find(|r| r.id == placement.reading_id)?;
    let field = ui
        .card_preview
        .as_ref()
        .and_then(|(digest, _)| ui.catalog.fields.iter().find(|f| f.digest() == *digest))
        .unwrap_or(&detail.field);
    let deck = field.tarot.as_ref()?;
    let id = ui
        .card_preview
        .as_ref()
        .map(|(_, id)| id.as_str())
        .unwrap_or(&reading.candidate_id);
    let card = deck.card(id)?;
    let mut items: Vec<ConsultationView> = vec![
        Box::new(
            el::<_, ConsultationUi, ()>("h3", format!("{} · {}", card.title, deck.title))
                .attr("data-key", "typed-card-heading"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "{:?} arcana · number: {} · suit: {}",
                    card.key.arcana(),
                    card.number
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "unnumbered".into()),
                    card.key
                        .suit()
                        .map(|s| format!("{s:?}"))
                        .unwrap_or_else(|| "none".into())
                ),
            )
            .attr("data-key", "typed-card-axes"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>("p", card.interpretation.text.clone())
                .attr("data-key", "typed-card-prompt"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "{} · version {} · {} · {}",
                    card.interpretation.id,
                    card.interpretation.version,
                    card.interpretation.author,
                    card.interpretation.date
                ),
            )
            .attr("class", "content-credit"),
        ),
    ];
    if let Some(model) = &card.interpretation.provenance.model {
        items.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Fixed model-assisted editorial prompt · {model}. {}",
                    card.interpretation.provenance.method
                ),
            )
            .attr("data-key", "prompt-origin"),
        ));
    }
    if ui.card_preview.is_some() {
        items.push(Box::new(
            button("Return to drawn card", |ui: &mut ConsultationUi, _| {
                ui.card_preview = None;
            })
            .attr("data-key", "return-drawn-card")
            .attr("class", "crosswalk"),
        ));
    }
    let mut columns: Vec<ConsultationView> = Vec::new();
    for correspondence in &card.correspondences {
        let source = deck.source(&correspondence.source_id)?;
        let element = correspondence
            .elements
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let planets = correspondence
            .planets
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let signs = correspondence
            .signs
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let shown = |s: String| {
            if s.is_empty() {
                "not transcribed".to_string()
            } else {
                s
            }
        };
        let rows: Vec<ConsultationView> = vec![
            Box::new(el::<_, ConsultationUi, ()>(
                "h4",
                correspondence.tradition.clone(),
            )),
            Box::new(el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Element: {} · Planet: {} · Sign: {}",
                    shown(element),
                    shown(planets),
                    shown(signs)
                ),
            )),
            Box::new(el::<_, ConsultationUi, ()>(
                "p",
                correspondence.note.clone(),
            )),
            Box::new(el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "{} · {} · {} · consulted {} · {}",
                    source.author,
                    source.title,
                    source.date,
                    source.consulted,
                    correspondence.version
                ),
            )),
            Box::new(el::<_, ConsultationUi, ()>("p", source.locator.clone())),
            Box::new(
                el::<_, ConsultationUi, ()>("a", source.url.clone())
                    .attr("href", source.url.clone()),
            ),
        ];
        columns.push(Box::new(
            el::<_, ConsultationUi, ()>("article", rows)
                .attr("class", "correspondence")
                .attr(
                    "data-key",
                    format!("correspondence:{}", correspondence.source_id),
                ),
        ));
    }
    items.push(Box::new(
        el::<_, ConsultationUi, ()>("div", columns)
            .attr("class", "correspondence-columns")
            .attr("data-key", "card-correspondences"),
    ));
    let mut targets = std::collections::BTreeSet::new();
    for other_field in &ui.catalog.fields {
        let Some(other) = &other_field.tarot else {
            continue;
        };
        if other.id == deck.id || !targets.insert(other.id.clone()) {
            continue;
        }
        let Some(target) = other.crosswalk(card.key) else {
            continue;
        };
        let digest = other_field.digest();
        let id = target.id.clone();
        let session = detail.session.id.clone();
        items.push(Box::new(
            button(
                format!("Inspect {} in {}", target.title, other.title),
                move |ui: &mut ConsultationUi, _| {
                    if ui.detail.as_ref().is_some_and(|d| d.session.id == session) {
                        ui.card_preview = Some((digest.clone(), id.clone()));
                    }
                },
            )
            .attr("data-key", format!("crosswalk:{}", other.id))
            .attr("class", "crosswalk"),
        ));
    }
    Some(Box::new(
        el::<_, ConsultationUi, ()>("section", items)
            .attr("data-key", "typed-card")
            .attr("data-deck-digest", deck.digest())
            .attr("class", "typed-card"),
    ))
}
