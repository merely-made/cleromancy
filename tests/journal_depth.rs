// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    CleromancyHost, Consultation, ConsultationError, ContextDraft, ReadingError, SelectionMode,
};
use muniment::RedbBackend;

#[test]
fn journal_depth_preserves_contexts_three_cards_comparisons_and_follow_ups() {
    let temporary = tempfile::tempdir().expect("temporary local store");
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).expect("open local store");
    let host = pollster::block_on(CleromancyHost::open(backend.clone())).expect("open host");
    let mut consultation = Consultation::new(host);
    let field_digest =
        pollster::block_on(consultation.install_builtin_tarot_at(1)).expect("install tarot");
    let context_digest = pollster::block_on(
        consultation.save_context_at(
            ContextDraft::new(
                "A changing structure",
                "What deserves attention now?",
                "change, reflection",
            )
            .with_additional_facts("season: late summer\nrecent_graph: Field notes"),
            2,
        ),
    )
    .expect("save disclosed context");
    let original_context = consultation
        .host()
        .context_for_digest(&context_digest)
        .expect("resolve original context");
    assert_eq!(
        original_context
            .facts
            .get("recent_graph")
            .map(String::as_str),
        Some("Field notes")
    );

    let amended_context_digest = pollster::block_on(
        consultation.save_context_at(
            ContextDraft::new(
                "A changing structure",
                "What deserves attention now?",
                "change, reflection",
            )
            .with_additional_facts("season: late summer\nrecent_graph: A harmony map"),
            3,
        ),
    )
    .expect("save amended context snapshot");
    assert_ne!(amended_context_digest, context_digest);
    assert_eq!(
        consultation
            .host()
            .context_for_digest(&context_digest)
            .expect("original context remains addressable")
            .facts
            .get("recent_graph")
            .map(String::as_str),
        Some("Field notes")
    );

    let mut three_card_entropy = FixedEntropy::new(0_u64..32);
    let three_card = pollster::block_on(consultation.read_three_card_at_with_entropy(
        &context_digest,
        &field_digest,
        1_000,
        4,
        &mut three_card_entropy,
    ))
    .expect("save fixed three-card cast");
    assert_eq!(three_card.readings.len(), 3);
    assert_eq!(
        three_card
            .session
            .placements
            .iter()
            .map(|placement| placement.position.as_str())
            .collect::<Vec<_>>(),
        ["foundation", "tension", "next_step"]
    );
    assert!(
        three_card
            .readings
            .iter()
            .all(|reading| reading.receipt.mode == SelectionMode::Cast)
    );
    let three_card_id = three_card.session.id.clone();

    let mut single_entropy = FixedEntropy::new(40_u64..56);
    let single = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        SelectionMode::Cast,
        2_000,
        5,
        &mut single_entropy,
    ))
    .expect("save comparison session");
    let comparison = consultation
        .compare_receipts(&three_card_id, &single.session.id)
        .expect("compare immutable receipts");
    assert!(comparison.same_context);
    assert!(comparison.same_field);
    assert!(!comparison.same_position_names);
    assert_eq!(
        comparison
            .entries
            .iter()
            .map(|entry| entry.position.as_str())
            .collect::<Vec<_>>(),
        ["focus", "foundation", "next_step", "tension"]
    );

    let mut reflection_entropy = FixedEntropy::new(60_u64..76);
    let once = pollster::block_on(consultation.reflect_at_with_entropy(
        &three_card_id,
        "Keep the useful constraint revisable.".to_string(),
        3_000,
        6,
        &mut reflection_entropy,
    ))
    .expect("append first reflection");
    let twice = pollster::block_on(consultation.reflect_at_with_entropy(
        &three_card_id,
        "Compare the next attempt against this receipt.".to_string(),
        4_000,
        7,
        &mut reflection_entropy,
    ))
    .expect("append second reflection");
    assert_eq!(once.reflections.len(), 1);
    assert_eq!(twice.reflections.len(), 2);
    assert_ne!(twice.reflections[0].id, twice.reflections[1].id);
    assert_eq!(
        twice
            .reflections
            .iter()
            .map(|reflection| reflection.body.as_str())
            .collect::<Vec<_>>(),
        [
            "Compare the next attempt against this receipt.",
            "Keep the useful constraint revisable."
        ]
    );

    let expected_detail = serde_json::to_vec(&twice).expect("serialize detail receipt");
    let expected_comparison = serde_json::to_vec(&comparison).expect("serialize comparison");
    drop(consultation);
    let reopened_host = pollster::block_on(CleromancyHost::open(backend)).expect("reopen host");
    let reopened = Consultation::new(reopened_host);
    assert_eq!(
        serde_json::to_vec(&reopened.detail(&three_card_id).expect("replay session"))
            .expect("serialize reopened detail"),
        expected_detail
    );
    assert_eq!(
        serde_json::to_vec(
            &reopened
                .compare_receipts(&three_card_id, &single.session.id)
                .expect("recompute comparison"),
        )
        .expect("serialize reopened comparison"),
        expected_comparison
    );
}

#[test]
fn additional_facts_reject_ambiguous_or_reserved_input() {
    for facts in [
        "season late summer",
        "question: replace the dedicated field",
        "season-name: late summer",
        "season: late summer\nseason: early autumn",
        "season: ",
    ] {
        assert!(matches!(
            ContextDraft::new("A question", "What persists?", "boundary")
                .with_additional_facts(facts)
                .into_snapshot(),
            Err(ConsultationError::InvalidContext(_))
        ));
    }
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
