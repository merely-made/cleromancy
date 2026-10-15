// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A saved reading told in plain words: which computation ran on which data,
//! which deck it drew from, which card it placed in which orientation and
//! position, and where that card falls in the draw.
//!
//! Every sentence is derived from the sealed receipts, the stored field and
//! context, and the authored layout bound to the session. Nothing here
//! recomputes a draw or claims more than those records support. The receipt
//! remains inspectable; this narration is the reading's audit.

use crate::field::{
    CONTEXTUAL_WEIGHT_RULE, EXTERNAL_TERM_WEIGHT_RULE, UNIFORM_DIE_RULE, UNIFORM_RULE,
};
use crate::reading_scene::position_label;
use crate::tarot::{RWS_MAJOR_ARCANA_ID, RWS_MAJOR_ARCANA_TITLE};
use crate::{
    ConsultationDetail, Field, Reading, SelectionMode, SpreadRelationKind, SpreadTemplate,
};

/// Built-in layouts identify themselves by their exact ordered position
/// names. An authored template bound to the session always takes precedence,
/// even when it reuses one of these names.
const THREE_CARD_POSITIONS: [&str; 3] = ["foundation", "tension", "next_step"];

/// Where a position's label and meaning came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeaningSource {
    /// The saved template bound to this session.
    Authored { template_id: String },
    /// Cleromancy's own built-in layout for these position names.
    BuiltIn,
    /// No saved layout and no built-in meaning for this position name.
    Unnamed,
}

/// One position resolved for presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionMeaning {
    /// The stable position name stored in the session.
    pub name: String,
    pub label: String,
    pub meaning: String,
    pub source: MeaningSource,
}

/// The layout a session was read through.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingLayout {
    /// A phrase that fits after "in": "the three-card layout", "“Crossroads”".
    pub name: String,
    pub positions: Vec<PositionMeaning>,
}

/// One card's narration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardNarration {
    /// One-based place in the draw.
    pub ordinal: usize,
    pub count: usize,
    pub position: PositionMeaning,
    pub title: String,
    pub text: String,
}

/// The whole reading's narration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingNarrative {
    pub overview: String,
    pub cards: Vec<CardNarration>,
}

/// Resolve every placement's label and meaning, preferring the saved template.
pub fn reading_layout(detail: &ConsultationDetail) -> ReadingLayout {
    let names = detail
        .session
        .placements
        .iter()
        .map(|placement| placement.position.as_str())
        .collect::<Vec<_>>();
    if let Some(template) = &detail.spread_template {
        return ReadingLayout {
            name: format!("“{}”", template.label),
            positions: names
                .iter()
                .map(|name| authored_position(template, name))
                .collect(),
        };
    }
    let name = if names == THREE_CARD_POSITIONS {
        "the three-card layout".to_string()
    } else if names == ["focus"] {
        "a single-card reading".to_string()
    } else {
        "this reading".to_string()
    };
    ReadingLayout {
        name,
        positions: names.iter().map(|name| built_in_position(name)).collect(),
    }
}

/// Narrate every card in draw order.
pub fn narrate(detail: &ConsultationDetail) -> ReadingNarrative {
    let layout = reading_layout(detail);
    let deck = Deck::of(&detail.field);
    let count = detail.session.placements.len();
    let cards = detail
        .session
        .placements
        .iter()
        .zip(&layout.positions)
        .enumerate()
        .filter_map(|(index, (placement, position))| {
            let reading = detail
                .readings
                .iter()
                .find(|reading| reading.id == placement.reading_id)?;
            let earlier = detail.session.placements[..index]
                .iter()
                .zip(&layout.positions)
                .find_map(|(earlier, earlier_position)| {
                    detail
                        .readings
                        .iter()
                        .find(|candidate| candidate.id == earlier.reading_id)
                        .filter(|candidate| candidate.candidate_id == reading.candidate_id)
                        .map(|_| earlier_position.label.as_str())
                });
            Some(CardNarration {
                ordinal: index + 1,
                count,
                position: position.clone(),
                title: reading.title.clone(),
                text: card_text(detail, &deck, &layout, reading, position, index, earlier),
            })
        })
        .collect();
    ReadingNarrative {
        overview: overview(detail, &deck, &layout),
        cards,
    }
}

/// The position meaning for one stored position name in this detail.
pub fn position_meaning(detail: &ConsultationDetail, name: &str) -> PositionMeaning {
    reading_layout(detail)
        .positions
        .into_iter()
        .find(|position| position.name == name)
        .unwrap_or_else(|| built_in_position(name))
}

/// Starting points for a reflection, taken from the reading itself: the
/// focused card in its position, and how two neighbouring positions meet.
/// They are prompts to edit, not interpretations, and are never saved unless
/// the person adds them.
pub fn reflection_examples(detail: &ConsultationDetail, focused: &str) -> Vec<String> {
    let narrative = narrate(detail);
    let Some(index) = narrative
        .cards
        .iter()
        .position(|card| card.position.name == focused)
        .or_else(|| (!narrative.cards.is_empty()).then_some(0))
    else {
        return Vec::new();
    };
    let card = &narrative.cards[index];
    let mut examples = vec![format!(
        "{} in “{}”: where I notice it in my situation is ",
        card.title, card.position.label
    )];
    let other = narrative.cards.get(index + 1).or_else(|| {
        index
            .checked_sub(1)
            .and_then(|before| narrative.cards.get(before))
    });
    if let Some(other) = other {
        examples.push(if other.title == card.title {
            format!(
                "{} came up twice, in “{}” and “{}”. Seeing it twice makes me think ",
                card.title, card.position.label, other.position.label
            )
        } else {
            format!(
                "{} in “{}” beside {} in “{}”: together they make me think ",
                card.title, card.position.label, other.title, other.position.label
            )
        });
    }
    examples
}

fn authored_position(template: &SpreadTemplate, name: &str) -> PositionMeaning {
    let label_of = |name: &str| {
        template
            .positions
            .iter()
            .find(|position| position.name == name)
            .map(|position| position.label.clone())
            .unwrap_or_else(|| position_label(name))
    };
    let label = label_of(name);
    let mut sentences = Vec::new();
    for relation in &template.relations {
        if relation.from == name {
            sentences.push(format!(
                "It {} “{}”: {}.",
                relation_verb(relation.kind),
                label_of(&relation.to),
                relation.label.trim_end_matches('.')
            ));
        } else if relation.to == name {
            sentences.push(format!(
                "“{}” {} it: {}.",
                label_of(&relation.from),
                relation_verb(relation.kind),
                relation.label.trim_end_matches('.')
            ));
        }
    }
    let meaning = if sentences.is_empty() {
        format!(
            "Named “{label}” in your layout “{}”. Its author gave it no further explanation, so read the card through that name.",
            template.label
        )
    } else {
        format!(
            "Named “{label}” in your layout “{}”. {}",
            template.label,
            sentences.join(" ")
        )
    };
    PositionMeaning {
        name: name.to_string(),
        label,
        meaning,
        source: MeaningSource::Authored {
            template_id: template.id.clone(),
        },
    }
}

fn built_in_position(name: &str) -> PositionMeaning {
    let built_in = match name {
        "foundation" => Some((
            "Foundation",
            "The ground you are starting from: what supports the situation, or what you are taking for granted.",
        )),
        "tension" => Some((
            "Tension",
            "Read against the foundation: what complicates it, challenges it, or asks for a different response.",
        )),
        "next_step" => Some((
            "Next step",
            "A possible response to that tension: something to try or attend to, rather than a promised outcome.",
        )),
        "focus" => Some((
            "Focus",
            "A single lens on your question. Notice what fits your experience and what does not.",
        )),
        _ => None,
    };
    match built_in {
        Some((label, meaning)) => PositionMeaning {
            name: name.to_string(),
            label: label.to_string(),
            meaning: meaning.to_string(),
            source: MeaningSource::BuiltIn,
        },
        None => PositionMeaning {
            name: name.to_string(),
            label: position_label(name),
            meaning:
                "No saved layout gives this position a meaning; read the card through its name."
                    .to_string(),
            source: MeaningSource::Unnamed,
        },
    }
}

fn relation_verb(kind: SpreadRelationKind) -> &'static str {
    match kind {
        SpreadRelationKind::Supports => "supports",
        SpreadRelationKind::Contradicts => "contradicts",
        SpreadRelationKind::Questions => "questions",
        SpreadRelationKind::NextStep => "follows on from",
        SpreadRelationKind::Elaborates => "elaborates",
    }
}

/// What the stored field says about the collection drawn from.
struct Deck {
    name: String,
    size: usize,
    noun: &'static str,
    singular: &'static str,
    /// Whether the collection is a tarot deck, where orientation is meaningful.
    tarot: bool,
}

impl Deck {
    fn of(field: &Field) -> Self {
        let tarot = field.system.starts_with("cleromancy.tarot/");
        let (noun, singular) = if tarot {
            ("cards", "card")
        } else if field.rules == UNIFORM_DIE_RULE {
            ("faces", "face")
        } else {
            ("possibilities", "possibility")
        };
        let name = if field.system == RWS_MAJOR_ARCANA_ID {
            RWS_MAJOR_ARCANA_TITLE.to_string()
        } else {
            field.system.clone()
        };
        Self {
            name,
            size: field.candidates.len(),
            noun,
            singular,
            tarot,
        }
    }
}

fn overview(detail: &ConsultationDetail, deck: &Deck, layout: &ReadingLayout) -> String {
    let count = detail.session.placements.len();
    let drawn = if count == 1 {
        format!("One {} from {}", deck.singular, deck.name)
    } else {
        format!(
            "{count} {} from {}, in {}",
            deck.noun, deck.name, layout.name
        )
    };
    let mut text = format!("{drawn}. ");
    if deck.tarot {
        text.push_str("Each card's words are this deck's own prompt for that card; nothing was newly written for your question.");
    } else {
        text.push_str("Each result's words come from the saved collection; nothing was newly written for your question.");
    }
    text
}

fn card_text(
    detail: &ConsultationDetail,
    deck: &Deck,
    layout: &ReadingLayout,
    reading: &Reading,
    position: &PositionMeaning,
    index: usize,
    earlier: Option<&str>,
) -> String {
    let receipt = &reading.receipt;
    let count = detail.session.placements.len();
    let mut sentences = Vec::new();
    let method = match receipt.mode {
        SelectionMode::Cast => {
            "A chance draw using your computer's operating-system randomness".to_string()
        },
        SelectionMode::Derived => match &receipt.derivation {
            Some(derivation) => format!(
                "A repeatable draw from the seed “{}” in the domain “{}”, which gives the same result every time",
                derivation.seed, derivation.domain
            ),
            None => "A repeatable draw from a disclosed seed".to_string(),
        },
        SelectionMode::Calculated => "A calculation with no randomness".to_string(),
    };
    sentences.push(format!(
        "{method} chose {} from the {} {} of {}.",
        reading.title, deck.size, deck.noun, deck.name
    ));
    sentences.push(weighting(detail, deck, reading));
    if let Some(enrichment) = &receipt.enrichment {
        let terms = enrichment
            .candidate_terms
            .get(receipt.selected_index)
            .cloned()
            .unwrap_or_default();
        sentences.push(if terms.is_empty() {
            "Saved outside context was considered but matched nothing about this result."
                .to_string()
        } else {
            format!(
                "Saved outside context raised its weight through these terms: {}.",
                terms.join(", ")
            )
        });
    }
    if deck.tarot {
        sentences.push(format!(
            "{} is upright; this deck has no reversed cards.",
            reading.title
        ));
    }
    if count == 1 {
        sentences.push(format!(
            "It is the only {} in {}, in the position “{}”.",
            deck.singular, layout.name, position.label
        ));
    } else {
        sentences.push(format!(
            "It sits in “{}”, position {} of {count} in {}.",
            position.label,
            index + 1,
            layout.name
        ));
        if receipt.mode == SelectionMode::Calculated {
            sentences.push(
                "Every position is calculated from the same inputs, so each gets the same result."
                    .to_string(),
            );
        } else {
            sentences.push(format!(
                "Each position draws from the full {} separately, so a {} can come up more than once.",
                if deck.tarot { "deck" } else { "collection" },
                deck.singular
            ));
        }
    }
    if let Some(earlier) = earlier {
        sentences.push(format!(
            "The same {} already came up in “{earlier}” in this reading.",
            deck.singular
        ));
    }
    sentences.join(" ")
}

fn weighting(detail: &ConsultationDetail, deck: &Deck, reading: &Reading) -> String {
    let receipt = &reading.receipt;
    let weight = receipt
        .qualified_weights
        .get(receipt.selected_index)
        .copied()
        .unwrap_or_default();
    let total = receipt.total_weight;
    if receipt.mode == SelectionMode::Calculated {
        return format!(
            "It had the highest score, {weight} of {total}, after the collection's rules were applied to your saved context; a tie goes to the first in the collection."
        );
    }
    let chance = chance(weight, total);
    match detail.field.rules.as_str() {
        UNIFORM_RULE | UNIFORM_DIE_RULE => format!(
            "Every {} was equally likely: {chance}. Your question and tags were saved with the reading but did not change the odds.",
            deck.singular
        ),
        CONTEXTUAL_WEIGHT_RULE | EXTERNAL_TERM_WEIGHT_RULE => {
            let tags = detail
                .field
                .candidates
                .iter()
                .find(|candidate| candidate.id == reading.candidate_id)
                .map(|candidate| {
                    candidate
                        .tags
                        .intersection(&detail.context.tags)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if tags.is_empty() {
                format!(
                    "It shared none of your saved tags, so it kept its starting weight: {chance}."
                )
            } else {
                format!(
                    "Your saved tags {} raised its weight: {chance}.",
                    tags.iter()
                        .map(|tag| format!("“{tag}”"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        },
        _ => format!("The saved collection's rules gave it {chance}."),
    }
}

fn chance(weight: u64, total: u64) -> String {
    if total == 0 {
        return "no recorded chance".to_string();
    }
    let percent = weight as f64 / total as f64 * 100.0;
    if weight == 1 {
        format!("1 in {total}, about {percent:.1}%")
    } else {
        format!("{weight} of {total} shares, about {percent:.1}%")
    }
}
