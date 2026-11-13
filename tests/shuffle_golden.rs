// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pins the public-seed shuffle to exact cards, so a change to the method,
//! its inputs or their encoding cannot pass unnoticed.
use cleromancy::{ContextSnapshot, ReadingEngine, TarotQualification};

#[test]
fn a_public_seed_gives_these_exact_cards() {
    let context = ContextSnapshot::new("Golden", "cleromancy.shuffle-golden/v1")
        .with_fact("question", "What stays the same?");
    let field = cleromancy::cards::builtin_decks()[0]
        .field(TarotQualification::Uniform)
        .unwrap();
    let drawn: Vec<String> = ReadingEngine::shuffle_derived(&context, &field, 5, "golden", "pin")
        .unwrap()
        .iter()
        .map(|reading| format!("{} {}", reading.candidate_id, reading.orientation().label()))
        .collect();
    assert_eq!(drawn, GOLDEN);
}

// Captured 2026-11-11 from the shuffle as it stood before the draw crate.
const GOLDEN: [&str; 5] = [
    "coins-ace reversed",
    "major-judgement upright",
    "coins-page reversed",
    "wands-ace reversed",
    "wands-five upright",
];
