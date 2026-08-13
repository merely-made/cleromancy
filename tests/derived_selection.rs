// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use cleromancy::{
    Candidate, CleromancyHost, Consultation, ContextDraft, DerivedSelection, Field, ReadingEngine,
    ReadingError, SelectionMode, UNIFORM_DIE_RULE, a0_fixture,
};
use muniment::RedbBackend;

#[test]
fn derived_receipt_is_byte_stable_and_replays_from_disclosed_inputs() {
    let (context, field) = a0_fixture();
    let selection =
        DerivedSelection::new("public-fixture-seed", "cleromancy.test/derived").unwrap();

    let first = ReadingEngine::derive(&context, &field, &selection).unwrap();
    let second = ReadingEngine::derive(&context, &field, &selection).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(first.receipt.schema, "cleromancy.receipt/v3");
    assert_eq!(first.receipt.mode, SelectionMode::Derived);
    assert_eq!(first.receipt.derivation.as_ref(), Some(&selection));
    assert!(first.receipt.derivation_digest.is_some());
    assert_eq!(first.receipt.event_nonce, None);
    assert_eq!(
        ReadingEngine::replay(&context, &field, &first.receipt).unwrap(),
        first
    );
}

#[test]
fn derived_uniform_die_is_replayable_and_detects_receipt_tampering() {
    let (context, _) = a0_fixture();
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
    let selection = DerivedSelection::new("roll-2026-08-08", "cleromancy.test/die").unwrap();

    assert!(matches!(
        ReadingEngine::calculate(&context, &die),
        Err(ReadingError::QualificationRequiresCast(rule)) if rule == UNIFORM_DIE_RULE
    ));
    let reading = ReadingEngine::derive(&context, &die, &selection).unwrap();
    assert!(reading.receipt.sample.unwrap() < 6);
    assert_eq!(
        ReadingEngine::replay(&context, &die, &reading.receipt).unwrap(),
        reading
    );

    let mut changed_seed = reading.receipt.clone();
    changed_seed.derivation.as_mut().unwrap().seed.push('!');
    assert!(matches!(
        ReadingEngine::replay(&context, &die, &changed_seed),
        Err(ReadingError::ReceiptMismatch(_))
    ));

    let mut changed_domain = reading.receipt.clone();
    changed_domain.derivation.as_mut().unwrap().domain.push('!');
    assert!(matches!(
        ReadingEngine::replay(&context, &die, &changed_domain),
        Err(ReadingError::ReceiptMismatch(_))
    ));

    let mut changed_sample = reading.receipt.clone();
    changed_sample.sample = Some((changed_sample.sample.unwrap() + 1) % 6);
    assert!(matches!(
        ReadingEngine::replay(&context, &die, &changed_sample),
        Err(ReadingError::ReceiptMismatch(_))
    ));

    let mut changed_nonce = reading.receipt.clone();
    changed_nonce.event_nonce = Some("not-derived-entropy".to_string());
    assert!(matches!(
        ReadingEngine::replay(&context, &die, &changed_nonce),
        Err(ReadingError::ReceiptMismatch(_))
    ));
}

#[test]
fn local_consultation_persists_and_reopens_the_derived_descriptor() {
    let temporary = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(temporary.path().join("cleromancy.redb")).unwrap();
    let mut consultation = Consultation::new(CleromancyHost::empty(backend.clone()));
    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new(
            "A replayable question",
            "What can be checked again?",
            "replay, receipt",
        ),
        2,
    ))
    .unwrap();
    let selection =
        DerivedSelection::new("public-local-seed", "cleromancy.local/one-card").unwrap();
    let detail = pollster::block_on(consultation.read_derived(
        &context_digest,
        &field_digest,
        selection.clone(),
    ))
    .unwrap();
    let session_id = detail.session.id.clone();
    assert_eq!(
        detail.readings[0].receipt.derivation.as_ref(),
        Some(&selection)
    );
    assert_eq!(detail.readings[0].receipt.event_nonce, None);
    drop(consultation);

    let reopened = Consultation::new(pollster::block_on(CleromancyHost::open(backend)).unwrap());
    let reopened_detail = reopened.detail(&session_id).unwrap();
    assert_eq!(
        reopened_detail.readings[0].receipt.derivation.as_ref(),
        Some(&selection)
    );
    assert_eq!(
        reopened
            .host()
            .replay_session(&reopened_detail.session)
            .unwrap(),
        reopened_detail.readings
    );
}
