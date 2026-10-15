// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C2 acceptance: a reading narrates its own computation card by card, keeps
//! an authored layout's labels and meanings, shows position meanings on
//! hover, presents its context with it, and offers reflection examples taken
//! from the reading.

#[path = "support/cleromancy_dom.rs"]
mod support;

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::narrative::{MeaningSource, narrate, reading_layout};
use cleromancy::{
    Consultation, ConsultationAction, ContextDraft, DerivedSelection, ReadingError,
    SpreadTemplateDraft,
};
use muniment::MemoryBackend;
use support::{attr_at_key, click_key, harness, has_key, one, rect_at_key, text_at_key};

/// Draw words for three casts over the 22 untagged Major Arcana: The Fool
/// (sample 0), The Fool again (22 % 22), then The Empress (3). Each cast also
/// consumes two words for its event nonce; the tail feeds session nonces.
fn repeating_draw() -> FixedEntropy {
    FixedEntropy::new([0, 1, 2, 22, 3, 4, 3, 5, 6].into_iter().chain(100..160))
}

fn authored_reading() -> (
    Consultation<MemoryBackend>,
    cleromancy::ConsultationCatalog,
    cleromancy::ConsultationDetail,
) {
    let host = cleromancy::CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(
        consultation.save_context_at(
            ContextDraft::new("A long habit", "What am I ready to set down?", "archive")
                .with_additional_facts("season: autumn"),
            2,
        ),
    )
    .unwrap();
    // A custom label on the built-in `foundation` id: the saved label and the
    // author's relation must win over Cleromancy's own three-card meanings.
    let template_id = pollster::block_on(consultation.save_spread_template_at(
        SpreadTemplateDraft::new(
            "Letting go",
            "foundation | What I must release\ntension | What resists\nnext_step | What I can try",
        )
        .with_relations("tension | questions | foundation | presses on what I hold"),
        3,
    ))
    .unwrap();
    let detail = pollster::block_on(consultation.read_spread_at_with_entropy(
        &context_digest,
        &field_digest,
        &template_id,
        1_000,
        4,
        &mut repeating_draw(),
    ))
    .unwrap();
    let catalog = consultation.catalog().unwrap();
    (consultation, catalog, detail)
}

#[test]
fn narration_reports_method_deck_chance_orientation_position_and_order() {
    let (consultation, _, detail) = authored_reading();
    assert_eq!(
        detail
            .spread_template
            .as_ref()
            .map(|template| template.label.as_str()),
        Some("Letting go")
    );
    // The resolved detail survives the resident's ordinary reopen path.
    assert_eq!(consultation.detail(&detail.session.id).unwrap(), detail);

    let narrative = narrate(&detail);
    assert_eq!(narrative.cards.len(), 3);
    assert!(
        narrative
            .overview
            .contains("3 cards from Major Arcana: reflective prompts")
    );
    assert!(narrative.overview.contains("in “Letting go”"));

    let first = &narrative.cards[0];
    assert_eq!((first.ordinal, first.count), (1, 3));
    assert_eq!(first.title, "The Fool");
    assert_eq!(first.position.label, "What I must release");
    for expected in [
        "A chance draw using your computer's operating-system randomness chose The Fool",
        "from the 22 cards of Major Arcana: reflective prompts",
        "It shared none of your saved tags, so it kept its starting weight: 1 in 22, about 4.5%.",
        "The Fool is upright; this deck has no reversed cards.",
        "It sits in “What I must release”, position 1 of 3 in “Letting go”.",
        "Each position draws from the full deck separately, so a card can come up more than once.",
    ] {
        assert!(
            first.text.contains(expected),
            "missing {expected:?} in {:?}",
            first.text
        );
    }

    let second = &narrative.cards[1];
    assert_eq!(second.title, "The Fool");
    assert!(
        second
            .text
            .contains("The same card already came up in “What I must release” in this reading."),
        "{}",
        second.text
    );
    assert!(!narrative.cards[2].text.contains("already came up"));

    let layout = reading_layout(&detail);
    let foundation = &layout.positions[0];
    assert!(matches!(foundation.source, MeaningSource::Authored { .. }));
    assert!(foundation.meaning.contains("“Letting go”"));
    assert!(
        foundation
            .meaning
            .contains("“What resists” questions it: presses on what I hold.")
    );
    assert!(!foundation.meaning.contains("ground you are starting from"));
    assert!(
        layout.positions[2]
            .meaning
            .contains("gave it no further explanation")
    );
}

#[test]
fn derived_and_built_in_readings_narrate_their_own_inputs() {
    let host = cleromancy::CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("Seeded", "Which card follows this seed?", "beginning"),
        2,
    ))
    .unwrap();
    let selection = DerivedSelection::new("river stone", "morning").unwrap();
    let derived = pollster::block_on(consultation.read_derived_at_with_entropy(
        &context_digest,
        &field_digest,
        &selection,
        1_000,
        3,
        &mut FixedEntropy::new(0..16),
    ))
    .unwrap();
    let card = &narrate(&derived).cards[0];
    assert!(card.text.contains(
        "A repeatable draw from the seed “river stone” in the domain “morning”, which gives the same result every time"
    ));
    assert!(
        card.text
            .contains("It is the only card in a single-card reading, in the position “Focus”.")
    );
    assert!(!card.text.contains("separately"));

    let three = pollster::block_on(consultation.read_three_card_at_with_entropy(
        &context_digest,
        &field_digest,
        2_000,
        4,
        &mut FixedEntropy::new(7..80),
    ))
    .unwrap();
    assert!(three.spread_template.is_none());
    let narrative = narrate(&three);
    assert_eq!(narrative.cards[1].position.label, "Tension");
    assert!(
        narrative.cards[1]
            .text
            .contains("position 2 of 3 in the three-card layout")
    );
    assert!(matches!(
        narrative.cards[1].position.source,
        MeaningSource::BuiltIn
    ));
}

#[test]
fn reader_shows_saved_labels_narration_context_hover_meaning_and_examples() {
    let (_, catalog, detail) = authored_reading();
    let session_id = detail.session.id.clone();
    let mut h = harness(catalog.clone());
    h.update(move |ui| ui.present_reading(catalog, detail));

    assert!(support::has_attr(&h, "aria-label", "Authored layout"));
    assert!(!text_at_key(&h, "reading-context-facts").contains("in the future"));
    // The authored label reaches the scene, the focused card and the receipt.
    assert_eq!(
        text_at_key(&h, "reading-position:foundation"),
        "What I must release"
    );
    assert_eq!(text_at_key(&h, "reading-position:tension"), "What resists");
    let focused = text_at_key(&h, "focused-reading");
    assert!(focused.contains("What I must release"), "{focused}");
    assert!(!focused.contains("Foundation"), "{focused}");
    let meaning = text_at_key(&h, "focused-position-meaning");
    assert!(meaning.contains("Letting go"), "{meaning}");
    assert!(
        !meaning.contains("ground you are starting from"),
        "{meaning}"
    );

    // Every card is narrated in accessible text, in draw order.
    for (position, heading) in [
        ("foundation", "Card 1 of 3 · What I must release · The Fool"),
        ("tension", "Card 2 of 3 · What resists · The Fool"),
        ("next_step", "Card 3 of 3 · What I can try · The Empress"),
    ] {
        let text = text_at_key(&h, &format!("narration:{position}"));
        assert!(text.starts_with(heading), "{text}");
        assert!(text.contains("operating-system randomness"), "{text}");
        assert!(text.contains("upright"), "{text}");
    }
    assert_eq!(
        attr_at_key(&h, "narration:foundation", "aria-current").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr_at_key(&h, "reading-narrative", "aria-label").as_deref(),
        Some("How this reading was made")
    );

    // The context is part of the reading, not a separate region.
    assert_eq!(
        text_at_key(&h, "reading-question"),
        "What am I ready to set down?"
    );
    let facts = text_at_key(&h, "reading-context-facts");
    for expected in ["A long habit", "drawn", "tags: archive", "season: autumn"] {
        assert!(facts.contains(expected), "missing {expected:?} in {facts}");
    }

    // Hovering a position shows its meaning; leaving returns to the focus.
    h.layout_at(1160.0, 760.0);
    assert_eq!(
        attr_at_key(&h, "position-meaning", "data-position").as_deref(),
        Some("foundation")
    );
    let (x, y, width, height) = rect_at_key(&h, "scene-card:tension");
    h.move_to(x + width / 2.0, y + height / 2.0);
    h.relayout();
    assert_eq!(
        attr_at_key(&h, "position-meaning", "data-position").as_deref(),
        Some("tension")
    );
    let hovered = text_at_key(&h, "position-meaning");
    assert!(hovered.starts_with("What resists"), "{hovered}");
    assert!(
        hovered.contains("It questions “What I must release”"),
        "{hovered}"
    );
    assert_eq!(
        attr_at_key(&h, "scene-card:tension", "aria-pressed").as_deref(),
        Some("false"),
        "hover never selects"
    );
    h.move_to(2.0, 2.0);
    h.relayout();
    assert_eq!(
        attr_at_key(&h, "position-meaning", "data-position").as_deref(),
        Some("foundation")
    );

    // The reflection editor opens with examples taken from this reading.
    h.layout_at(1600.0, 4000.0);
    let example = text_at_key(&h, "reflection-example:0");
    assert!(
        example.starts_with("The Fool in “What I must release”: where I notice it"),
        "{example}"
    );
    let repeat = text_at_key(&h, "reflection-example:1");
    assert!(
        repeat.starts_with("The Fool came up twice, in “What I must release” and “What resists”."),
        "{repeat}"
    );
    assert!(click_key(&mut h, "use-reflection-example:0").is_none());
    match one(click_key(&mut h, "save-reflection")) {
        ConsultationAction::SaveReflection {
            session_id: id,
            body,
        } => {
            assert_eq!(id, session_id);
            assert!(
                body.starts_with("The Fool in “What I must release”"),
                "{body}"
            );
        },
        other => panic!("expected reflection action, found {other:?}"),
    }
    assert!(has_key(&h, "reading-scene"));
}

struct FixedEntropy {
    words: VecDeque<u64>,
}

impl FixedEntropy {
    fn new(words: impl IntoIterator<Item = u64>) -> Self {
        Self {
            words: words.into_iter().collect(),
        }
    }
}

impl EntropySource for FixedEntropy {
    fn next_u64(&mut self) -> Result<u64, ReadingError> {
        self.words
            .pop_front()
            .ok_or_else(|| ReadingError::Entropy("fixed source exhausted".to_string()))
    }
}
