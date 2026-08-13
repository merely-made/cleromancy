// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    CleromancyHost, Consultation, ConsultationError, ContextDraft, HostError, ReadingError,
    SelectionMode, TarotPack, TarotQualification,
};
use muniment::{Backend, MemoryBackend, RedbBackend, StoreError, WriteOp};

#[test]
fn saved_tarot_consultation_and_reflection_reopen_exactly() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).unwrap();
    let host = pollster::block_on(CleromancyHost::open(backend.clone())).unwrap();
    let mut consultation = Consultation::new(host);

    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new(
            "A changing structure",
            "What deserves attention now?",
            " Change, reflection, change ",
        ),
        2,
    ))
    .unwrap();

    let context = consultation
        .host()
        .context_for_digest(&context_digest)
        .unwrap();
    assert_eq!(
        context.facts.get("question").map(String::as_str),
        Some("What deserves attention now?")
    );
    assert_eq!(
        context.tags.into_iter().collect::<Vec<_>>(),
        ["change", "reflection"]
    );

    let mut cast_entropy = FixedEntropy::new([7, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
    let cast = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        SelectionMode::Cast,
        1_000,
        3,
        &mut cast_entropy,
    ))
    .unwrap();
    assert_eq!(cast.readings.len(), 1);
    assert_eq!(cast.readings[0].receipt.mode, SelectionMode::Cast);
    assert_eq!(cast.readings[0].receipt.sample, Some(7));
    let cast_session_id = cast.session.id.clone();

    let mut calculated_entropy = FixedEntropy::new([0x77, 0x88]);
    let calculated = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        SelectionMode::Calculated,
        2_000,
        4,
        &mut calculated_entropy,
    ))
    .unwrap();
    assert_eq!(
        calculated.readings[0].receipt.mode,
        SelectionMode::Calculated
    );
    assert_eq!(calculated.readings[0].receipt.sample, None);

    let reflected = pollster::block_on(consultation.reflect_at_with_entropy(
        &cast_session_id,
        "The structure is useful when it remains revisable.".to_string(),
        3_000,
        5,
        &mut cast_entropy,
    ))
    .unwrap();
    assert_eq!(reflected.reflections.len(), 1);

    let expected_catalog = consultation.catalog().unwrap();
    assert_eq!(expected_catalog.contexts.len(), 1);
    assert_eq!(expected_catalog.fields.len(), 1);
    assert_eq!(expected_catalog.sessions.len(), 2);
    assert_eq!(expected_catalog.sessions[0].id, calculated.session.id);
    assert_eq!(expected_catalog.sessions[1].id, cast_session_id);
    let expected_detail_bytes = serde_json::to_vec(&reflected).unwrap();
    let expected_catalog_bytes = serde_json::to_vec(&expected_catalog).unwrap();
    let stored_before_reopen = pollster::block_on(backend.get(cleromancy::host::HOST_SLOT))
        .unwrap()
        .unwrap();
    drop(consultation);

    let reopened_host = pollster::block_on(CleromancyHost::open(backend.clone())).unwrap();
    assert!(reopened_host.was_reopened());
    let reopened = Consultation::new(reopened_host);
    let reopened_catalog = reopened.catalog().unwrap();
    let reopened_detail = reopened.detail(&cast_session_id).unwrap();
    assert_eq!(
        serde_json::to_vec(&reopened_catalog).unwrap(),
        expected_catalog_bytes
    );
    assert_eq!(
        serde_json::to_vec(&reopened_detail).unwrap(),
        expected_detail_bytes
    );
    assert_eq!(
        reopened
            .host()
            .replay_session(&reopened_detail.session)
            .unwrap(),
        reopened_detail.readings
    );
    assert_eq!(
        pollster::block_on(backend.get(cleromancy::host::HOST_SLOT))
            .unwrap()
            .unwrap(),
        stored_before_reopen,
        "read-only reopen changed durable graph truth"
    );
}

#[test]
fn rejected_commands_leave_persisted_truth_unchanged() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("cleromancy.redb");
    let backend = RedbBackend::open(&path).unwrap();
    let mut host = CleromancyHost::empty(backend.clone());
    let uniform_field = TarotPack::rws_major_arcana().field(TarotQualification::Uniform);
    let uniform_digest = uniform_field.digest();
    host.insert_field(&uniform_field).unwrap();
    pollster::block_on(host.persist(1)).unwrap();
    let mut consultation = Consultation::new(host);
    let contextual_digest = pollster::block_on(consultation.install_builtin_tarot_at(2)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("A question", "What is the useful boundary?", "boundary"),
        3,
    ))
    .unwrap();
    let mut session_entropy = FixedEntropy::new([0x10, 0x20]);
    let valid = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &contextual_digest,
        SelectionMode::Calculated,
        1_000,
        4,
        &mut session_entropy,
    ))
    .unwrap();
    let baseline_catalog = consultation.catalog().unwrap();
    let baseline = pollster::block_on(backend.get(cleromancy::host::HOST_SLOT))
        .unwrap()
        .unwrap();

    assert!(matches!(
        pollster::block_on(
            consultation.save_context_at(ContextDraft::new("", "Still a question", "boundary"), 5,)
        ),
        Err(ConsultationError::InvalidContext(_))
    ));

    let mut unused_entropy = FixedEntropy::new([]);
    assert!(matches!(
        pollster::block_on(consultation.read_at_with_entropy(
            "missing-context",
            &contextual_digest,
            SelectionMode::Cast,
            2_000,
            5,
            &mut unused_entropy,
        )),
        Err(ConsultationError::Host(
            HostError::MissingReadingDependency {
                kind: "context",
                ..
            }
        ))
    ));
    assert!(matches!(
        pollster::block_on(consultation.read_at_with_entropy(
            &context_digest,
            "missing-field",
            SelectionMode::Cast,
            2_000,
            5,
            &mut unused_entropy,
        )),
        Err(ConsultationError::Host(
            HostError::MissingReadingDependency { kind: "field", .. }
        ))
    ));
    assert!(matches!(
        pollster::block_on(consultation.read_at_with_entropy(
            &context_digest,
            &uniform_digest,
            SelectionMode::Calculated,
            2_000,
            5,
            &mut unused_entropy,
        )),
        Err(ConsultationError::Reading(
            ReadingError::QualificationRequiresCast(_)
        ))
    ));
    assert!(matches!(
        pollster::block_on(consultation.reflect_at_with_entropy(
            "missing-session",
            "A note".to_string(),
            2_000,
            5,
            &mut unused_entropy,
        )),
        Err(ConsultationError::Host(
            HostError::MissingReadingDependency {
                kind: "session",
                ..
            } | HostError::MissingSessionDependency {
                kind: "session",
                ..
            }
        ))
    ));
    let mut reflection_entropy = FixedEntropy::new([0x30, 0x40]);
    assert!(matches!(
        pollster::block_on(consultation.reflect_at_with_entropy(
            &valid.session.id,
            "   ".to_string(),
            2_000,
            5,
            &mut reflection_entropy,
        )),
        Err(ConsultationError::Host(HostError::Session(_)))
    ));

    assert!(!consultation.is_faulted());
    assert_eq!(consultation.catalog().unwrap(), baseline_catalog);
    assert_eq!(
        pollster::block_on(backend.get(cleromancy::host::HOST_SLOT))
            .unwrap()
            .unwrap(),
        baseline
    );
}

#[test]
fn storage_failure_faults_the_controller_until_reopen() {
    let backend = SwitchBackend::new(true);
    let durable = backend.inner.clone();
    let mut consultation = Consultation::new(CleromancyHost::empty(backend.clone()));

    let error = pollster::block_on(consultation.save_context_at(
        ContextDraft::new("A question", "What persists?", "persistence"),
        1,
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        ConsultationError::Host(HostError::Store(StoreError::Backend(_)))
    ));
    assert!(consultation.is_faulted());
    backend.fail_put.store(false, Ordering::SeqCst);
    assert!(matches!(
        pollster::block_on(consultation.install_builtin_tarot_at(2)),
        Err(ConsultationError::Faulted)
    ));
    assert!(
        pollster::block_on(durable.get(cleromancy::host::HOST_SLOT))
            .unwrap()
            .is_none()
    );

    let reopened = Consultation::new(pollster::block_on(CleromancyHost::open(durable)).unwrap());
    assert!(reopened.catalog().unwrap().contexts.is_empty());
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

#[derive(Clone)]
struct SwitchBackend {
    inner: MemoryBackend,
    fail_put: Arc<AtomicBool>,
}

impl SwitchBackend {
    fn new(fail_put: bool) -> Self {
        Self {
            inner: MemoryBackend::new(),
            fail_put: Arc::new(AtomicBool::new(fail_put)),
        }
    }
}

#[async_trait]
impl Backend for SwitchBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.inner.get(key).await
    }

    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        if self.fail_put.load(Ordering::SeqCst) {
            Err(StoreError::Backend("injected write failure".to_string()))
        } else {
            self.inner.put(key, bytes).await
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.inner.delete(key).await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.inner.list(prefix).await
    }

    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.inner.scan(start, end).await
    }

    async fn apply(&self, operations: &[WriteOp]) -> Result<(), StoreError> {
        self.inner.apply(operations).await
    }
}
