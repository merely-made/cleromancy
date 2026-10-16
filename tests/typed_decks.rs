// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use cleromancy::cards::{CardKey, Element, Planet, Rank, Suit, Trump, builtin_decks};
use cleromancy::{
    ContextSnapshot, Field, ReadingEngine, TarotPack, TarotQualification, ZodiacSign,
};

#[test]
fn complete_decks_crosswalk_typed_identity_and_preserve_disagreement() {
    let decks = builtin_decks();
    assert_eq!(decks.len(), 2);
    for deck in decks {
        deck.validate().unwrap();
        assert_eq!(deck.cards.len(), 78);
        for card in &deck.cards {
            assert!(decks.iter().all(|d| d.crosswalk(card.key).is_some()));
            for r in &card.correspondences {
                let source = deck.source(&r.source_id).unwrap();
                assert!(source.validate());
                assert!(!source.date.is_empty());
                assert!(!r.version.is_empty());
            }
        }
    }
    let strength = CardKey::Major {
        trump: Trump::Strength,
    };
    assert_eq!(decks[0].crosswalk(strength).unwrap().number, Some(8));
    assert_eq!(decks[1].crosswalk(strength).unwrap().number, Some(11));
    assert_eq!(
        decks[1]
            .crosswalk(CardKey::Major { trump: Trump::Fool })
            .unwrap()
            .number,
        None
    );
    let magician = decks[0]
        .crosswalk(CardKey::Major {
            trump: Trump::Magician,
        })
        .unwrap();
    assert_eq!(magician.correspondences[0].planets, [Planet::Mercury]);
    assert_eq!(magician.correspondences[1].signs, [ZodiacSign::Aries]);
    let two = decks[0]
        .crosswalk(CardKey::Minor {
            suit: Suit::Wands,
            rank: Rank::Two,
        })
        .unwrap();
    assert_eq!(two.correspondences[0].elements, [Element::Fire]);
    assert_eq!(two.correspondences[1].elements, [Element::Air]);
    assert_eq!(two.correspondences[0].planets, [Planet::Mars]);
    assert_eq!(two.correspondences[0].signs, [ZodiacSign::Aries]);
}

#[test]
fn stored_typed_field_replays_and_refuses_unbound_or_malformed_content() {
    let field = builtin_decks()[0]
        .field(TarotQualification::Contextual)
        .unwrap();
    let context = ContextSnapshot::new("A bounded choice", "c3-fixture/v1");
    let reading = ReadingEngine::calculate(&context, &field).unwrap();
    let stored: Field = serde_json::from_slice(&serde_json::to_vec(&field).unwrap()).unwrap();
    assert_eq!(field, stored);
    assert_eq!(
        ReadingEngine::replay(&context, &stored, &reading.receipt).unwrap(),
        reading
    );
    let mut altered = field.clone();
    altered.candidates[0].interpretation.push_str(" changed");
    assert!(ReadingEngine::calculate(&context, &altered).is_err());
    let mut unlabelled = field.clone();
    unlabelled.tarot.as_mut().unwrap().cards[22]
        .interpretation
        .provenance
        .model = None;
    assert!(unlabelled.validate_metadata().is_err());
    let mut active_link = field.clone();
    active_link.tarot.as_mut().unwrap().sources[0].url = "javascript:alert(1)".into();
    assert!(active_link.validate_metadata().is_err());
    let mut missing_source = field.clone();
    missing_source.tarot.as_mut().unwrap().sources.clear();
    assert!(missing_source.validate_metadata().is_err());
    let mut duplicate = field.clone();
    duplicate.tarot.as_mut().unwrap().cards[1].key = duplicate.tarot.as_ref().unwrap().cards[0].key;
    assert!(duplicate.validate_metadata().is_err());
    let mut future = field.clone();
    future.tarot.as_mut().unwrap().schema = "cleromancy.typed-tarot-deck/v9".into();
    assert!(future.validate_metadata().is_err());
    let mut wrong_rank = field;
    wrong_rank.tarot.as_mut().unwrap().cards[22].number = Some(99);
    assert!(wrong_rank.validate_metadata().is_err());
}

#[test]
fn historical_field_encoding_and_original_pack_are_unchanged() {
    let old = TarotPack::rws_major_arcana().field(TarotQualification::Uniform);
    let bytes = serde_json::to_vec(&old).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("\"tarot\""));
    assert!(!String::from_utf8_lossy(&bytes).contains("position_meanings"));
    let expected: Field =
        serde_json::from_str(include_str!("fixtures/c3-legacy-field.json")).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&expected).unwrap());
    let context = ContextSnapshot::new("Original field", "c3-fixture/v1");
    let old_reading = ReadingEngine::derive(
        &context,
        &old,
        &cleromancy::DerivedSelection::new("legacy seed", "c3").unwrap(),
    )
    .unwrap();
    assert_eq!(
        ReadingEngine::replay(&context, &expected, &old_reading.receipt).unwrap(),
        old_reading
    );
}
