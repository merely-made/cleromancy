// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    Candidate, ContextSnapshot, DeckShuffle, DerivedSelection, Field, Orientation, ReadingEngine,
    ReadingPlacement, ReadingSession, TarotQualification, UNIFORM_DIE_RULE,
};
use std::collections::BTreeSet;
struct Words {
    used: usize,
}
impl EntropySource for Words {
    fn next_u64(&mut self) -> Result<u64, cleromancy::ReadingError> {
        self.used += 1;
        Ok(self.used as u64)
    }
}
fn fixture() -> (ContextSnapshot, Field) {
    (
        ContextSnapshot::new("One deck", "cleromancy.c5-public/v1")
            .with_fact("question", "What can be released?"),
        cleromancy::cards::builtin_decks()[0]
            .field(TarotQualification::Uniform)
            .unwrap(),
    )
}
#[test]
fn one_full_shuffle_has_no_repeats_and_all_prefixes_share_order_and_orientations() {
    let (context, field) = fixture();
    let whole = ReadingEngine::shuffle_derived(&context, &field, 78, "public C5", "deck").unwrap();
    let short = ReadingEngine::shuffle_derived(&context, &field, 10, "public C5", "deck").unwrap();
    assert_eq!(
        whole
            .iter()
            .map(|r| &r.candidate_id)
            .collect::<BTreeSet<_>>()
            .len(),
        78
    );
    assert!(
        whole
            .iter()
            .any(|r| r.orientation() == Orientation::Reversed)
    );
    assert!(
        whole
            .iter()
            .any(|r| r.orientation() == Orientation::Upright)
    );
    let plan = DeckShuffle::from_public_seed("public C5", "deck")
        .unwrap()
        .replay(&context, &field)
        .unwrap();
    for (i, reading) in whole.iter().enumerate() {
        assert_eq!(plan[i].candidate_id, reading.candidate_id);
        assert_eq!(plan[i].orientation, reading.orientation());
        assert_eq!(
            ReadingEngine::replay(&context, &field, &reading.receipt).unwrap(),
            *reading
        );
        if i < 10 {
            assert_eq!(short[i].candidate_id, reading.candidate_id);
            assert_eq!(short[i].orientation(), reading.orientation());
        }
    }
    let mut entropy = Words { used: 0 };
    let os = ReadingEngine::shuffle_with(&context, &field, 10, &mut entropy).unwrap();
    assert_eq!(
        entropy.used, 4,
        "one 256-bit seed, not entropy per position"
    );
    assert_eq!(
        os.iter()
            .map(|r| &r.candidate_id)
            .collect::<BTreeSet<_>>()
            .len(),
        10
    );
    for r in os {
        assert_eq!(
            ReadingEngine::replay(&context, &field, &r.receipt).unwrap(),
            r
        );
    }
    let used = entropy.used;
    assert!(ReadingEngine::shuffle_with(&context, &field, 79, &mut entropy).is_err());
    assert!(ReadingEngine::shuffle_with(&context, &field, 0, &mut entropy).is_err());
    assert_eq!(entropy.used, used, "invalid counts consume no entropy");
}
#[test]
fn receipt_tampering_and_mixed_or_reordered_session_draws_are_refused_before_mutation() {
    let (context, field) = fixture();
    let readings =
        ReadingEngine::shuffle_derived(&context, &field, 3, "public C5", "deck").unwrap();
    let original = &readings[0].receipt;
    let mut variants = Vec::new();
    let mut r = original.clone();
    r.shuffle.as_mut().unwrap().orientation = if readings[0].orientation() == Orientation::Reversed
    {
        Orientation::Upright
    } else {
        Orientation::Reversed
    };
    variants.push(r);
    let mut r = original.clone();
    r.shuffle.as_mut().unwrap().index = 1;
    variants.push(r);
    let mut r = original.clone();
    r.shuffle.as_mut().unwrap().shuffle.algorithm.push('!');
    variants.push(r);
    let mut r = original.clone();
    r.context_digest.push('!');
    variants.push(r);
    let mut r = original.clone();
    r.qualified_weights[0] += 1;
    variants.push(r);
    let mut r = original.clone();
    r.shuffle = None;
    variants.push(r);
    let mut r = original.clone();
    r.sample = Some(0);
    variants.push(r);
    for r in variants {
        assert!(ReadingEngine::replay(&context, &field, &r).is_err());
    }
    let make_session = |rs: &[cleromancy::Reading]| {
        ReadingSession::with_placements(
            1000,
            "00000000000000000000000000000001",
            context.digest(),
            field.digest(),
            rs.iter()
                .enumerate()
                .map(|(i, r)| ReadingPlacement {
                    position: format!("position-{i}"),
                    reading_id: r.id.clone(),
                })
                .collect(),
            None,
        )
        .unwrap()
    };
    let mut host = cleromancy::CleromancyHost::empty(muniment::MemoryBackend::new());
    let before = serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap();
    let mut mixed = readings.clone();
    mixed[1] = ReadingEngine::shuffle_derived(&context, &field, 3, "other seed", "deck").unwrap()
        [1]
    .clone();
    assert!(
        host.insert_session(&context, &field, &mixed, &make_session(&mixed))
            .is_err()
    );
    let mut reordered = readings.clone();
    reordered.swap(0, 1);
    assert!(
        host.insert_session(&context, &field, &reordered, &make_session(&reordered))
            .is_err()
    );
    assert_eq!(
        serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    let session = make_session(&readings);
    host.insert_session(&context, &field, &readings, &session)
        .unwrap();
    let reopened = cleromancy::CleromancyHost::candidate(host.graph().clone()).unwrap();
    assert_eq!(reopened.replay_session(&session).unwrap(), readings);
    let detail = cleromancy::Consultation::new(reopened)
        .detail(&session.id)
        .unwrap();
    let narration = cleromancy::narrative::narrate(&detail);
    assert!(
        narration
            .cards
            .iter()
            .all(|c| c.text.contains("Fisher–Yates")
                && c.text.contains("public C5")
                && c.text.contains("cannot repeat"))
    );
    assert!(
        narration
            .cards
            .iter()
            .all(|c| !c.text.contains("Each position draws from the full"))
    );
}
#[test]
fn pre_c5_scalar_calculation_and_derived_receipts_remain_byte_equal() {
    let (context, field) = cleromancy::a0_fixture();
    let die = Field::new(
        "fixture.derived-die/v1",
        UNIFORM_DIE_RULE,
        (1..=6).map(|face| {
            Candidate::new(
                format!("face-{face}"),
                format!("Face {face}"),
                format!("Face {face} is the disclosed result."),
            )
        }),
    );
    let mut readings = vec![ReadingEngine::calculate(&context, &field).unwrap()];
    for (seed, domain, f) in [
        ("public-fixture-seed", "cleromancy.test/derived", &field),
        ("roll-2026-08-08", "cleromancy.test/die", &die),
        ("isocosm-compatibility", "generator-choice", &field),
    ] {
        readings.push(
            ReadingEngine::derive(&context, f, &DerivedSelection::new(seed, domain).unwrap())
                .unwrap(),
        );
    }
    assert_eq!(
        serde_json::to_string_pretty(&readings).unwrap() + "\n",
        include_str!("fixtures/pre-c5-selections.json")
    );
    for (i, r) in readings.iter().enumerate() {
        assert_eq!(
            ReadingEngine::replay(&context, if i == 2 { &die } else { &field }, &r.receipt)
                .unwrap(),
            *r
        );
    }
}

#[test]
fn a_typed_spread_captures_one_field_snapshot_within_the_carrier_bound() {
    let (context, field) = fixture();
    let mut host = cleromancy::CleromancyHost::empty(muniment::MemoryBackend::new());
    let edits = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    host.record_edits(edits.clone());
    host.record_three_card_spread_at_with_entropy(
        &context,
        &field,
        1000,
        None,
        &mut Words { used: 0 },
    )
    .unwrap();
    let captured = edits.lock().unwrap();
    let encoded = serde_json::to_string(&*captured).unwrap();
    assert_eq!(encoded.matches("cleromancy.field/v1").count(), 1);
    assert!(
        encoded.len() < 220_000,
        "one typed-deck batch is bounded before envelope expansion: {}",
        encoded.len()
    );
}
