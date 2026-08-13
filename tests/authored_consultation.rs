// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    AstrologyChartDraft, CleromancyHost, Consultation, ReadingError, SpreadTemplateDraft,
};
use muniment::RedbBackend;

#[test]
fn consultation_persists_an_authored_cast_and_source_qualified_chart_concurrence() {
    let temporary = tempfile::tempdir().expect("temporary local store");
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).expect("open local store");
    let host = pollster::block_on(CleromancyHost::open(backend.clone())).expect("open host");
    let mut consultation = Consultation::new(host);
    let field_digest =
        pollster::block_on(consultation.install_builtin_tarot_at(1)).expect("install tarot");
    let context_digest = pollster::block_on(consultation.save_context_at(
        cleromancy::ContextDraft::new(
            "An authored concern",
            "What does this four-part situation need from me?",
            "structure, change",
        ),
        2,
    ))
    .expect("save context");
    let template_id = pollster::block_on(consultation.save_spread_template_at(
        SpreadTemplateDraft::new(
            "Four directions",
            "north | North\neast | East\nsouth | South\nwest | West",
        )
        .with_relations(
            "east | questions | north | tests the north\nsouth | next_step | east | answers the east",
        ),
        3,
    ))
    .expect("save authored layout");
    let facts_digest = pollster::block_on(consultation.save_astrology_chart_at(
        AstrologyChartDraft {
            algorithm: "source-import/v1".to_string(),
            engine: "example-calculator 1.0".to_string(),
            ephemeris: "example ephemeris 2026.1".to_string(),
            instant_utc: "2026-08-08T12:00:00Z".to_string(),
            latitude_microdegrees: "40712800".to_string(),
            longitude_microdegrees: "-74006000".to_string(),
            orb_millidegrees: "1000".to_string(),
            positions: "Sun | 135000 | 0 | false\nMoon | 225500 | 0 | true".to_string(),
        },
        4,
    ))
    .expect("save chart facts");

    let mut entropy = FixedEntropy::new(0_u64..64);
    let detail = pollster::block_on(consultation.read_spread_at_with_entropy(
        &context_digest,
        &field_digest,
        &template_id,
        5_000,
        5,
        &mut entropy,
    ))
    .expect("save authored cast");
    assert_eq!(detail.readings.len(), 4);
    assert!(detail.concurrences.is_empty());
    let associated = pollster::block_on(consultation.associate_astrology_facts_at(
        &facts_digest,
        &detail.session.id,
        6_000,
        6,
    ))
    .expect("associate chart and reading");
    assert_eq!(associated.concurrences.len(), 1);
    assert_eq!(associated.concurrences[0].created_at_ms, 6_000);
    assert!(
        associated.concurrences[0]
            .members
            .iter()
            .any(|member| member.address == format!("cleromancy://astrology/facts/{facts_digest}"))
    );

    let expected = serde_json::to_vec(&associated).expect("serialize durable receipt");
    drop(consultation);
    let reopened =
        Consultation::new(pollster::block_on(CleromancyHost::open(backend)).expect("reopen host"));
    assert_eq!(
        serde_json::to_vec(
            &reopened
                .detail(&associated.session.id)
                .expect("replay authored receipt"),
        )
        .expect("serialize reopened receipt"),
        expected
    );
    let catalog = reopened.catalog().expect("load durable catalog");
    assert_eq!(catalog.spread_templates.len(), 1);
    assert_eq!(catalog.astrology_facts.len(), 1);
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
