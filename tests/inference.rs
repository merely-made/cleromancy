// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use cleromancy::inference::*;
use cleromancy::{
    CleromancyHost, ContextSnapshot, ReadingEngine, ReadingSession, TarotQualification,
};
fn fixture() -> (CleromancyHost<muniment::MemoryBackend>, ReadingSession) {
    let context = ContextSnapshot::new("Synthetic C7 reading", "cleromancy.c7-fixture/v1")
        .with_fact("question", "What should I reconsider?");
    let field = cleromancy::cards::builtin_decks()[0]
        .field(TarotQualification::Uniform)
        .unwrap();
    let reading =
        ReadingEngine::shuffle_derived(&context, &field, 1, "C7 public seed", "inference")
            .unwrap()
            .remove(0);
    let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
    let mut latest = None;
    for i in 0..3 {
        let session = ReadingSession::single(
            1_792_149_302_000 + i,
            format!("{i:032x}"),
            context.digest(),
            field.digest(),
            reading.id.clone(),
            None,
        )
        .unwrap();
        host.insert_session(&context, &field, &[reading.clone()], &session)
            .unwrap();
        latest = Some(session);
    }
    (host, latest.unwrap())
}
fn bytes(host: &CleromancyHost<muniment::MemoryBackend>) -> Vec<u8> {
    serde_json::to_vec(&(host.graph().to_snapshot_at(0), host.graph().facets())).unwrap()
}
fn identity() -> ModelIdentity {
    ModelIdentity {
        model_id: "test-only/record-contract".into(),
        revision: "fixture-v1".into(),
        loader: "esp-burn-ndarray".into(),
        artifacts: ["config.json", "tokenizer.json", "model.safetensors"]
            .map(|name| (name.into(), "a".repeat(64)))
            .into_iter()
            .collect(),
    }
}
#[test]
fn formula_replays_exactly_and_preserves_draws_and_earlier_details() {
    let (mut host, session) = fixture();
    let draws = host.replay_session(&session).unwrap();
    let inputs = host.inference_inputs(&session.id).unwrap();
    assert!(inputs.context_text.contains("What should I reconsider?"));
    assert_eq!(inputs.history_session_ids.len(), 2);
    assert_eq!(inputs.repeated_cards.len(), 1);
    let record = GeneratedInference::formula(inputs.clone());
    assert_eq!(
        serde_json::to_vec(&record).unwrap(),
        serde_json::to_vec(&GeneratedInference::formula(inputs)).unwrap()
    );
    host.insert_generated_inference(&record).unwrap();
    cleromancy::host::validate_divination_graph(host.graph()).unwrap();
    assert_eq!(draws, host.replay_session(&session).unwrap());
    let before = bytes(&host);
    host.insert_generated_inference(&record).unwrap();
    assert_eq!(before, bytes(&host));
}
#[test]
fn formula_tamper_and_fabricated_sources_are_refused_before_any_change() {
    let (mut host, session) = fixture();
    let original = GeneratedInference::formula(host.inference_inputs(&session.id).unwrap());
    for kind in 0..5 {
        let mut bad = original.clone();
        match kind {
            0 => bad.output.push('!'),
            1 => bad.inputs.context_text.push('!'),
            2 => bad.inputs.history_session_ids.push(session.id.clone()),
            3 => bad.inputs.reflection_ids.push("missing".into()),
            _ => bad.label = LABEL_MODEL.into(),
        }
        let before = bytes(&host);
        assert!(host.insert_generated_inference(&bad).is_err());
        assert_eq!(before, bytes(&host));
    }
}
#[test]
fn stored_model_output_round_trips_without_model_execution_and_requires_disclosures() {
    // Synthetic output tests the record contract only, not real model execution.
    let (mut host, session) = fixture();
    let record = GeneratedInference::model(
        host.inference_inputs(&session.id).unwrap(),
        identity(),
        "Synthetic contract output: what changed?".into(),
    );
    host.insert_generated_inference(&record).unwrap();
    let encoded = serde_json::to_vec(&record).unwrap();
    assert_eq!(
        encoded,
        serde_json::to_vec(&host.inferences_for_session(&session.id).unwrap()[0]).unwrap()
    );
    let decoded: GeneratedInference = serde_json::from_slice(&encoded).unwrap();
    decoded.validate().unwrap();
    for kind in 0..4 {
        let mut bad = record.clone();
        if let Generation::Model { model, prompt, .. } = &mut bad.generation {
            match kind {
                0 => model.artifacts.clear(),
                1 => prompt.push('!'),
                2 => model.model_id.clear(),
                _ => model.loader = "stub".into(),
            }
        }
        let before = bytes(&host);
        assert!(host.insert_generated_inference(&bad).is_err());
        assert_eq!(before, bytes(&host));
    }
    cleromancy::host::validate_divination_graph(host.graph()).unwrap();
    let key = host
        .graph()
        .get_node_by_url(&format!("cleromancy://inference/{}", record.digest()))
        .unwrap()
        .0;
    assert_ne!(
        alembic::memory_levels::level_of(host.graph(), key),
        alembic::memory_levels::MemoryLevel::LongTerm
    );
    host.keep_generated_inference(&record.digest()).unwrap();
    assert_eq!(
        alembic::memory_levels::level_of(host.graph(), key),
        alembic::memory_levels::MemoryLevel::LongTerm
    );
    assert_eq!(record, host.inferences_for_session(&session.id).unwrap()[0]);
}
#[test]
fn later_reflections_do_not_rewrite_frozen_inputs() {
    let (mut host, session) = fixture();
    let record = GeneratedInference::formula(host.inference_inputs(&session.id).unwrap());
    host.insert_generated_inference(&record).unwrap();
    let reflection = cleromancy::Reflection::new(
        &session.id,
        session.created_at_ms + 1,
        "00000000000000000000000000000077",
        "A later reflection",
    )
    .unwrap();
    host.insert_reflection(&session, &reflection).unwrap();
    assert_eq!(record, host.inferences_for_session(&session.id).unwrap()[0]);
    assert_ne!(record.inputs, host.inference_inputs(&session.id).unwrap());
}

#[cfg(all(feature = "local-inference", not(feature = "portable-core")))]
#[test]
#[ignore = "requires an explicitly supplied real local model artifact directory"]
fn real_local_esp_generation_records_actual_output_and_artifact_identity() {
    let (mut host, session) = fixture();
    let directory =
        std::env::var_os("CLEROMANCY_MODEL_DIR").expect("explicit local model directory");
    let record = cleromancy::inference::local::generate(
        host.inference_inputs(&session.id).unwrap(),
        std::path::Path::new(&directory),
    )
    .unwrap();
    assert!(!record.output.trim().is_empty());
    host.insert_generated_inference(&record).unwrap();
    assert_eq!(record, host.inferences_for_session(&session.id).unwrap()[0]);
    if let Some(path) = std::env::var_os("CLEROMANCY_INFERENCE_PROOF") {
        std::fs::write(path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    }
}

#[test]
fn generated_records_do_not_change_existing_card_neighborhoods() {
    let (mut host, session) = fixture();
    let position = &session.placements[0].position;
    let before = host.card_neighborhood(&session.id, position).unwrap();
    let record = GeneratedInference::formula(host.inference_inputs(&session.id).unwrap());
    host.insert_generated_inference(&record).unwrap();
    let after = host.card_neighborhood(&session.id, position).unwrap();
    assert_eq!(
        serde_json::to_vec(&before).unwrap(),
        serde_json::to_vec(&after).unwrap()
    );
    cleromancy::host::validate_divination_graph(host.graph()).unwrap();
}
