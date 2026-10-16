// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod support;
use cleromancy::{
    CleromancyHost, Consultation, ConsultationAction, ContextSnapshot, ReadingEngine,
    ReadingSession, TarotQualification,
};
#[test]
fn saved_prompt_inputs_start_collapsed_and_inspection_emits_no_owner_action() {
    let context = ContextSnapshot::new("C7 DOM question", "cleromancy.c7-dom/v1");
    let field = cleromancy::cards::builtin_decks()[0]
        .field(TarotQualification::Uniform)
        .unwrap();
    let reading = ReadingEngine::shuffle_derived(&context, &field, 1, "C7 DOM seed", "inference")
        .unwrap()
        .remove(0);
    let session = ReadingSession::single(
        1000,
        "00000000000000000000000000000007",
        context.digest(),
        field.digest(),
        reading.id.clone(),
        None,
    )
    .unwrap();
    let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
    host.insert_session(&context, &field, &[reading], &session)
        .unwrap();
    let record = cleromancy::inference::GeneratedInference::formula(
        host.inference_inputs(&session.id).unwrap(),
    );
    host.insert_generated_inference(&record).unwrap();
    let consultation = Consultation::new(host);
    let catalog = consultation.catalog().unwrap();
    let detail = consultation.detail(&session.id).unwrap();
    let mut h = support::harness(catalog.clone());
    h.update(move |ui| ui.present_reading(catalog, detail));
    let text = support::text_at_key(&h, "reading-inferences");
    assert!(text.contains(&record.label));
    assert!(!text.contains("Current occasion"));
    let key = format!("inference-inspect:{}", record.digest());
    assert!(support::click_key(&mut h, &key).is_none());
    assert!(support::text_at_key(&h, "reading-inferences").contains("Current occasion"));
    assert!(support::click_key(&mut h, &key).is_none());
    assert!(!support::text_at_key(&h, "reading-inferences").contains("Current occasion"));
    match support::one(support::click_key(
        &mut h,
        &format!("inference-keep:{}", record.digest()),
    )) {
        ConsultationAction::KeepInference { session_id, digest } => {
            assert_eq!(session_id, session.id);
            assert_eq!(digest, record.digest());
        },
        other => panic!("unexpected owner action {other:?}"),
    }
}
