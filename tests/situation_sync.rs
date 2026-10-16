// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(all(feature = "legacy-receipts", feature = "personal-sync"))]
use cleromancy::situation::*;
use cleromancy::{
    CleromancyHost, CleromancySyncSelection, ContextSnapshot, ReadingEngine, ReadingSession,
    TarotQualification, export_sync_batch, import_sync_projection,
};
use graphshell::personal_sync::{PersonalGraphReplica, SyncRoster};
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};

#[test]
fn signed_reading_sync_preserves_weather_evidence_and_refuses_tamper_before_importing_any_nodes() {
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
        let place = ReadingPlace::new(51_476_900, 0).unwrap();
        let weather = WeatherObservation::from_response(
            &place,
            session.created_at_ms - 1000,
            weather_url(&place).unwrap(),
            include_str!("fixtures/c6-open-meteo-current.json").into(),
        )
        .unwrap();
        let situation = ReadingSituation::capture(
            &session,
            &SituationPreferences {
                place: Some(place),
                weather: true,
            },
            Some(weather),
        )
        .unwrap();
        let mut source = CleromancyHost::empty(MemoryBackend::new());
        source
            .insert_session(&context, &field, &[reading], &session)
            .unwrap();
        source.insert_reading_situation(&situation).unwrap();
        let selection = CleromancySyncSelection::ContextsAndReadings;
        let batch = export_sync_batch(&source, selection).unwrap();
        assert_eq!(batch.schema, "cleromancy.sync-batch/v7");
        assert_eq!(batch.situations, 1);
        assert_eq!(
            export_sync_batch(&source, CleromancySyncSelection::Contexts)
                .unwrap()
                .situations,
            0
        );
        assert!(
            export_sync_batch(&source, CleromancySyncSelection::Off)
                .unwrap()
                .is_empty()
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
                .situations,
            1
        );
        let restored = target.situation_for_session(&session.id).unwrap().unwrap();
        assert_eq!(
            serde_json::to_vec(&restored).unwrap(),
            serde_json::to_vec(&situation).unwrap()
        );
        assert_eq!(
            export_sync_batch(&target, selection).unwrap().events,
            batch.events
        );
        let (key, _) = projection
            .graph
            .get_node_by_url(&format!("cleromancy://situation/{}", situation.digest()))
            .unwrap();
        let id = projection.graph.get_node(key).unwrap().id;
        let mut forged = serde_json::to_value(&situation).unwrap();
        forged["weather"]["values"]["temperature_millicelsius"] = serde_json::json!(99_999);
        projection
            .graph
            .facets_mut()
            .set(
                id,
                chartulary::FacetId::new(SITUATION_FACET),
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
