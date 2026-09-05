// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! R0 of the legible reader plan: session summaries are a faithful, cheap
//! projection of the same graph truth `detail()` replays.

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    CleromancyHost, Consultation, ContextDraft, ReadingError, SelectionMode, summarize,
};
use muniment::RedbBackend;

#[test]
fn summaries_are_faithful_cheap_and_ordered_like_sessions() {
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
            .with_additional_facts("season: late summer"),
            2,
        ),
    )
    .expect("save disclosed context");

    let mut three_card_entropy = FixedEntropy::new(0_u64..32);
    let three_card = pollster::block_on(consultation.read_three_card_at_with_entropy(
        &context_digest,
        &field_digest,
        1_000,
        4,
        &mut three_card_entropy,
    ))
    .expect("save three-card cast");
    let three_card_id = three_card.session.id.clone();

    let mut single_entropy = FixedEntropy::new(40_u64..56);
    let single = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        SelectionMode::Calculated,
        2_000,
        5,
        &mut single_entropy,
    ))
    .expect("save single calculated reading");

    let catalog = consultation.catalog().expect("read catalog");

    // The summary catalog has no replaying session list. Its ordering is the
    // host's canonical session order, verified through the explicit host API.
    let sessions = consultation
        .host()
        .sessions()
        .expect("read full sessions for receipt");
    assert_eq!(catalog.session_summaries.len(), sessions.len());
    assert_eq!(
        catalog
            .session_summaries
            .iter()
            .map(|summary| summary.session_id.as_str())
            .collect::<Vec<_>>(),
        sessions
            .iter()
            .map(|session| session.id.as_str())
            .collect::<Vec<_>>(),
        "summary ordering must match sessions() exactly"
    );

    // Newest first: the single reading was saved at 2_000, the cast at 1_000.
    assert_eq!(catalog.session_summaries[0].session_id, single.session.id);
    assert_eq!(catalog.session_summaries[1].session_id, three_card_id);

    // Every value a legible row shows must equal the replayed truth.
    for summary in &catalog.session_summaries {
        let detail = consultation
            .detail(&summary.session_id)
            .expect("replay the same session");
        assert_eq!(summary.created_at_ms, detail.session.created_at_ms);
        assert_eq!(summary.context_label, detail.context.label);
        assert_eq!(
            summary.question.as_deref(),
            detail.context.facts.get("question").map(String::as_str)
        );
        assert_eq!(
            summary.tags,
            detail.context.tags.iter().cloned().collect::<Vec<_>>()
        );
        assert_eq!(summary.context_digest, detail.session.context_digest);
        assert_eq!(summary.field_digest, detail.session.field_digest);
        assert_eq!(summary.placement_count(), detail.session.placements.len());
        assert_eq!(
            summary
                .placements
                .iter()
                .map(|placement| placement.position.as_str())
                .collect::<Vec<_>>(),
            detail
                .session
                .placements
                .iter()
                .map(|placement| placement.position.as_str())
                .collect::<Vec<_>>(),
            "placements keep the session's declared order"
        );
        assert_eq!(
            summary
                .placements
                .iter()
                .map(|placement| placement.title.as_str())
                .collect::<Vec<_>>(),
            detail
                .readings
                .iter()
                .map(|reading| reading.title.as_str())
                .collect::<Vec<_>>(),
            "a row shows the cards actually drawn"
        );
        assert_eq!(
            summary
                .placements
                .iter()
                .map(|placement| placement.candidate_id.as_str())
                .collect::<Vec<_>>(),
            detail
                .readings
                .iter()
                .map(|reading| reading.candidate_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    let cast = catalog
        .session_summaries
        .iter()
        .find(|summary| summary.session_id == three_card_id)
        .expect("the cast is summarized");
    assert_eq!(cast.placement_count(), 3);
    assert_eq!(cast.modes(), vec![SelectionMode::Cast]);
    assert_eq!(cast.context_label, "A changing structure");
    assert_eq!(
        cast.question.as_deref(),
        Some("What deserves attention now?")
    );
    assert_eq!(cast.tags, ["change", "reflection"]);

    let calculated = catalog
        .session_summaries
        .iter()
        .find(|summary| summary.session_id == single.session.id)
        .expect("the calculated reading is summarized");
    assert_eq!(calculated.placement_count(), 1);
    assert_eq!(calculated.modes(), vec![SelectionMode::Calculated]);

    // The projection is reachable without a field and without the engine: the
    // caller supplies only the session, its context, and its readings. This is
    // what makes the summary path structurally incapable of replaying a
    // receipt, and therefore cheap on an archive that grows without bound.
    let detail = consultation
        .detail(&three_card_id)
        .expect("replay the cast");
    let rebuilt = summarize(&detail.session, &detail.context, &detail.readings);
    assert_eq!(&rebuilt, cast);

    // Summaries survive a close and reopen unchanged, because they are
    // recomputed from graph truth rather than stored.
    let expected = serde_json::to_vec(&catalog.session_summaries).expect("serialize summaries");
    drop(consultation);
    let reopened_host = pollster::block_on(CleromancyHost::open(backend)).expect("reopen host");
    let reopened = Consultation::new(reopened_host);
    assert_eq!(
        serde_json::to_vec(
            &reopened
                .catalog()
                .expect("read reopened catalog")
                .session_summaries
        )
        .expect("serialize reopened summaries"),
        expected
    );
}

#[test]
fn summarize_skips_a_placement_whose_reading_is_absent() {
    // `summarize` is a total function over what it is handed: it never
    // fabricates a row for a reading it was not given. The host path resolves
    // every placement before calling it, so an absent reading surfaces there
    // as a missing-dependency error rather than as a short summary.
    let temporary = tempfile::tempdir().expect("temporary local store");
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).expect("open local store");
    let host = pollster::block_on(CleromancyHost::open(backend)).expect("open host");
    let mut consultation = Consultation::new(host);

    let field_digest =
        pollster::block_on(consultation.install_builtin_tarot_at(1)).expect("install tarot");
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("Bounded", "What is bounded here?", "bounds"),
        2,
    ))
    .expect("save context");
    let mut entropy = FixedEntropy::new(0_u64..32);
    let three_card = pollster::block_on(consultation.read_three_card_at_with_entropy(
        &context_digest,
        &field_digest,
        1_000,
        4,
        &mut entropy,
    ))
    .expect("save three-card cast");

    let detail = consultation
        .detail(&three_card.session.id)
        .expect("replay the cast");
    let partial = summarize(&detail.session, &detail.context, &detail.readings[..1]);
    assert_eq!(partial.placement_count(), 1);
    assert_eq!(partial.placements[0].position, "foundation");

    let complete = summarize(&detail.session, &detail.context, &detail.readings);
    assert_eq!(complete.placement_count(), 3);
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
