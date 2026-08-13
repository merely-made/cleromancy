// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    CleromancyHost, ContextSnapshot, ReadingError, SpreadPosition, SpreadRelation,
    SpreadRelationKind, SpreadTemplate, TarotPack, TarotQualification,
};
use muniment::RedbBackend;

#[test]
fn authored_spread_replays_its_template_session_and_relations_after_reopen() {
    let temporary = tempfile::tempdir().expect("temporary local store");
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).expect("open local store");
    let mut host = CleromancyHost::empty(backend.clone());
    let context = ContextSnapshot::new("An authored layout", "cleromancy.test-context/v1")
        .with_fact("question", "What should this four-part inquiry disclose?");
    let field = TarotPack::rws_major_arcana().field(TarotQualification::Uniform);
    let template = SpreadTemplate::new(
        "Four directions",
        [
            SpreadPosition::new("north", "North"),
            SpreadPosition::new("east", "East"),
            SpreadPosition::new("south", "South"),
            SpreadPosition::new("west", "West"),
        ],
        [
            SpreadRelation::new(
                "east",
                SpreadRelationKind::Questions,
                "north",
                "tests the north",
            ),
            SpreadRelation::new(
                "south",
                SpreadRelationKind::NextStep,
                "east",
                "answers the east",
            ),
        ],
    )
    .expect("valid authored template");
    let changed_template = SpreadTemplate::new(
        "Four directions",
        [
            SpreadPosition::new("north", "North"),
            SpreadPosition::new("east", "East"),
            SpreadPosition::new("south", "South"),
            SpreadPosition::new("west", "West"),
        ],
        [SpreadRelation::new(
            "south",
            SpreadRelationKind::Supports,
            "east",
            "supports the east",
        )],
    )
    .expect("different authored template");
    assert_ne!(template.id, changed_template.id);

    let mut entropy = FixedEntropy::new(0_u64..128);
    let (session, spread, readings) = host
        .record_spread_at_with_entropy(&context, &field, &template, 1_000, None, &mut entropy)
        .expect("record authored spread");
    assert_eq!(readings.len(), 4);
    assert_eq!(
        session
            .placements
            .iter()
            .map(|placement| placement.position.as_str())
            .collect::<Vec<_>>(),
        ["north", "east", "south", "west"]
    );
    assert_eq!(
        host.replay_spread(&spread).expect("replay spread"),
        readings
    );
    pollster::block_on(host.persist(2)).expect("persist authored spread");
    let expected_template = serde_json::to_vec(&template).expect("serialize template");
    let expected_spread = serde_json::to_vec(&spread).expect("serialize spread");
    let expected_readings = serde_json::to_vec(&readings).expect("serialize readings");
    drop(host);

    let reopened = pollster::block_on(CleromancyHost::open(backend)).expect("reopen host");
    assert_eq!(
        serde_json::to_vec(
            &reopened
                .spread_template_for_id(&template.id)
                .expect("resolve template"),
        )
        .expect("serialize reopened template"),
        expected_template
    );
    let reopened_spread = reopened.spread_for_id(&spread.id).expect("resolve spread");
    assert_eq!(
        serde_json::to_vec(&reopened_spread).expect("serialize reopened spread"),
        expected_spread
    );
    assert_eq!(
        serde_json::to_vec(
            &reopened
                .replay_spread(&reopened_spread)
                .expect("replay reopened")
        )
        .expect("serialize reopened readings"),
        expected_readings
    );
}

#[test]
fn authored_template_rejects_unknown_or_repeated_positions() {
    assert!(
        SpreadTemplate::new(
            "Broken layout",
            [SpreadPosition::new("focus", "Focus")],
            [SpreadRelation::new(
                "focus",
                SpreadRelationKind::Questions,
                "missing",
                "asks the missing position",
            )],
        )
        .is_err()
    );
    assert!(
        SpreadTemplate::new(
            "Broken layout",
            [
                SpreadPosition::new("focus", "Focus"),
                SpreadPosition::new("focus", "Another focus"),
            ],
            [],
        )
        .is_err()
    );
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
