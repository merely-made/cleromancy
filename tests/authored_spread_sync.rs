// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    CleromancyHost, CleromancySyncSelection, ContextSnapshot, ReadingError, SpreadPosition,
    SpreadRelation, SpreadRelationKind, SpreadTemplate, TarotPack, TarotQualification,
    export_sync_batch, import_sync_projection,
};
use graphshell::personal_sync::{PersonalGraphReplica, SyncRoster};
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};

#[test]
fn selected_sync_round_trips_the_generic_template_before_its_bound_cast() {
    pollster::block_on(async {
        let context = ContextSnapshot::new("A four-part turn", "cleromancy.sync-context/v1");
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
        .expect("valid authored layout");
        let mut source = CleromancyHost::empty(MemoryBackend::new());
        let mut entropy = FixedEntropy::new(0_u64..64);
        let (session, spread, readings) = source
            .record_spread_at_with_entropy(
                &context,
                &field,
                &template,
                1_753_000_001_000,
                Some("authored-layout-sync".to_string()),
                &mut entropy,
            )
            .expect("record authored spread");
        let selection = CleromancySyncSelection::ContextsAndReadings;
        let batch = export_sync_batch(&source, selection).expect("export selected truth");
        assert_eq!(
            (batch.sessions, batch.spreads, batch.spread_templates),
            (1, 1, 1)
        );

        let alice_identity = InMemoryProvider::from_seed([0x91; 32]);
        let bob_identity = InMemoryProvider::from_seed([0x92; 32]);
        let roster = SyncRoster::new([
            alice_identity.master_public_key().to_bytes(),
            bob_identity.master_public_key().to_bytes(),
        ]);
        let mut alice = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0x93; 32],
            &alice_identity,
            roster.clone(),
            selection.personal_graph_selection(),
        )
        .expect("open source replica");
        let bob = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            [0x93; 32],
            &bob_identity,
            roster,
            selection.personal_graph_selection(),
        )
        .expect("open target replica");
        let operation = alice
            .author(batch.events.clone())
            .await
            .expect("author sync operation");
        assert!(
            bob.accept(&operation)
                .await
                .expect("accept signed operation")
        );
        let projection = bob.projection().await.expect("project replica");
        let mut target = CleromancyHost::empty(MemoryBackend::new());
        let imported = import_sync_projection(&mut target, &projection, selection)
            .expect("import selected projection");
        assert_eq!(
            (
                imported.sessions,
                imported.spreads,
                imported.spread_templates
            ),
            (1, 1, 1)
        );
        assert_eq!(
            target
                .replay_spread(&spread)
                .expect("replay imported spread"),
            readings
        );
        assert_eq!(
            target
                .spread_template_for_id(&template.id)
                .expect("imported template"),
            template
        );
        assert_eq!(
            target
                .replay_session(&session)
                .expect("imported session")
                .len(),
            4
        );
        let round_trip = export_sync_batch(&target, selection).expect("re-export selected truth");
        assert_eq!(round_trip.events, batch.events);
        assert_eq!(round_trip.digest, batch.digest);
    });
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
            .ok_or_else(|| ReadingError::Entropy("fixture exhausted".to_string()))
    }
}
