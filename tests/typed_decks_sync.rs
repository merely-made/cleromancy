// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(feature = "legacy-receipts", feature = "personal-sync"))]

use cleromancy::{
    CleromancyHost, CleromancySyncSelection, ContextSnapshot, ReadingEngine, TarotQualification,
    export_sync_batch, import_sync_projection,
};
use graphshell::personal_sync::{PersonalGraphReplica, SyncRoster};
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};

#[test]
fn signed_sync_preserves_complete_decks_and_their_content_revisions() {
    pollster::block_on(async {
        let context = ContextSnapshot::new("Typed content", "c3-sync/v1");
        let mut source = CleromancyHost::empty(MemoryBackend::new());
        for deck in cleromancy::cards::builtin_decks() {
            let field = deck.field(TarotQualification::Contextual).unwrap();
            let reading = ReadingEngine::calculate(&context, &field).unwrap();
            source.insert_reading(&context, &field, &reading).unwrap();
        }
        let selection = CleromancySyncSelection::ContextsAndReadings;
        let batch = export_sync_batch(&source, selection).unwrap();
        assert_eq!(batch.fields, 2);
        let identity = InMemoryProvider::from_seed([0xc3; 32]);
        let peer = InMemoryProvider::from_seed([0xc4; 32]);
        let roster = SyncRoster::new([
            identity.master_public_key().to_bytes(),
            peer.master_public_key().to_bytes(),
        ]);
        let mut writer = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0xc3; 32],
            &identity,
            roster.clone(),
            selection.personal_graph_selection(),
        )
        .unwrap();
        let receiver = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0xc3; 32],
            &peer,
            roster,
            selection.personal_graph_selection(),
        )
        .unwrap();
        let signed = writer.author(batch.events.clone()).await.unwrap();
        assert!(receiver.accept(&signed).await.unwrap());
        let projection = receiver.projection().await.unwrap();
        let mut target = CleromancyHost::empty(MemoryBackend::new());
        let imported = import_sync_projection(&mut target, &projection, selection).unwrap();
        assert_eq!(imported.fields, 2);
        let result = export_sync_batch(&target, selection).unwrap();
        assert_eq!(result.events, batch.events);
        assert_eq!(result.digest, batch.digest);
        for field in target.fields().unwrap() {
            field.validate_metadata().unwrap();
            assert_eq!(field.tarot.as_ref().unwrap().cards.len(), 78);
            assert!(field.position_meanings.is_some());
        }
    });
}
