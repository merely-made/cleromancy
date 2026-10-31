// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(all(feature = "legacy-receipts", feature = "personal-sync"))]
use cleromancy::inference::*;
use cleromancy::{
    CleromancyHost, CleromancySyncSelection, ContextSnapshot, ReadingEngine, ReadingSession,
    TarotQualification, export_sync_batch, import_sync_projection,
};
use graphshell::personal_sync::{PersonalGraphReplica, SyncRoster};
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};

#[test]
fn signed_sync_preserves_generated_prompts_only_with_reflection_consent_and_refuses_tamper_before_mutation()
 {
    pollster::block_on(async {
        let context =
            ContextSnapshot::new("C6 signed public weather", "cleromancy.context/manual/v1");
        let field = cleromancy::cards::builtin_decks()[0]
            .field(TarotQualification::Uniform)
            .unwrap();
        let reading =
            ReadingEngine::shuffle_derived(&context, &field, 1, "C6 public", "weather-sync")
                .unwrap()
                .remove(0);
        let session = ReadingSession::single(
            1_792_149_302_000,
            "00000000000000000000000000000006",
            context.digest(),
            field.digest(),
            reading.id.clone(),
            None,
        )
        .unwrap();
        let mut source = CleromancyHost::empty(MemoryBackend::new());
        source
            .insert_session(&context, &field, &[reading], &session)
            .unwrap();
        let formula = GeneratedInference::formula(source.inference_inputs(&session.id).unwrap());
        source.insert_generated_inference(&formula).unwrap();
        source.keep_generated_inference(&formula.digest()).unwrap();
        let source_key = source
            .graph()
            .get_node_by_url(&format!("cleromancy://inference/{}", formula.digest()))
            .unwrap()
            .0;
        assert!(
            source.graph().shown_resource_id(source_key).is_some(),
            "exercise a resource-bound resident-style record"
        );
        assert!(
            !source
                .graph()
                .get_node(source_key)
                .unwrap()
                .tags
                .contains("kept"),
            "the legacy field is not the resource-tag projection"
        );
        let selection = CleromancySyncSelection::ContextsReadingsAndReflections;
        let batch = export_sync_batch(&source, selection).unwrap();
        assert_eq!(batch.schema, "cleromancy.sync-batch/v8");
        assert_eq!(batch.inferences, 1);
        assert!(batch.events.iter().any(|event| matches!(event, graphshell::personal_sync::PersonalGraphEvent::AddTag { node, tag } if *node == source.graph().get_node(source_key).unwrap().id && tag == "kept")));
        assert_eq!(
            export_sync_batch(&source, CleromancySyncSelection::Contexts)
                .unwrap()
                .inferences,
            0
        );
        assert!(
            export_sync_batch(&source, CleromancySyncSelection::Off)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            export_sync_batch(&source, CleromancySyncSelection::ContextsAndReadings)
                .unwrap()
                .inferences,
            0
        );
        let identity = InMemoryProvider::from_seed([0xc6; 32]);
        let peer = InMemoryProvider::from_seed([0xc7; 32]);
        let roster = SyncRoster::new([
            identity.master_public_key().to_bytes(),
            peer.master_public_key().to_bytes(),
        ]);
        let mut writer = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0xc6; 32],
            &identity,
            roster.clone(),
            selection.personal_graph_selection(),
        )
        .unwrap();
        let receiver = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0xc6; 32],
            &peer,
            roster,
            selection.personal_graph_selection(),
        )
        .unwrap();
        let signed = writer.author(batch.events.clone()).await.unwrap();
        assert!(receiver.accept(&signed).await.unwrap());
        let mut projection = receiver.projection().await.unwrap();
        let mut target = CleromancyHost::empty(MemoryBackend::new());
        assert_eq!(
            import_sync_projection(&mut target, &projection, selection)
                .unwrap()
                .inferences,
            1
        );
        let target_key = target
            .graph()
            .get_node_by_url(&format!("cleromancy://inference/{}", formula.digest()))
            .unwrap()
            .0;
        assert_eq!(
            pandect::memory_levels::level_of(target.graph(), target_key),
            pandect::memory_levels::MemoryLevel::LongTerm,
            "selected sync retains the explicit Keep action"
        );
        let restored = target
            .inferences_for_session(&session.id)
            .unwrap()
            .remove(0);
        assert_eq!(
            serde_json::to_vec(&restored).unwrap(),
            serde_json::to_vec(&formula).unwrap()
        );
        assert_eq!(
            export_sync_batch(&target, selection).unwrap().events,
            batch.events
        );
        let (key, _) = projection
            .graph
            .get_node_by_url(&format!("cleromancy://inference/{}", formula.digest()))
            .unwrap();
        let id = projection.graph.get_node(key).unwrap().id;
        let mut forged = serde_json::to_value(&formula).unwrap();
        forged["output"] = serde_json::json!("Invented formula output");
        projection
            .graph
            .facets_mut()
            .set(
                id,
                chartulary::FacetId::new(INFERENCE_FACET),
                forged,
                &chartulary::AcceptAll,
            )
            .unwrap();
        let mut empty = CleromancyHost::empty(MemoryBackend::new());
        let before =
            serde_json::to_vec(&(empty.graph().to_snapshot_at(0), empty.graph().facets())).unwrap();
        assert!(import_sync_projection(&mut empty, &projection, selection).is_err());
        assert_eq!(
            before,
            serde_json::to_vec(&(empty.graph().to_snapshot_at(0), empty.graph().facets())).unwrap()
        );
    });
}
