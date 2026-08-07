// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(
    feature = "graphshell-admission",
    feature = "personal-sync",
    not(target_arch = "wasm32")
))]

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::servitor::{Cap, Grant, Mode, Subject};
use cleromancy::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, CREATE_CONCURRENCE_INTENT, CleromancyApp,
    CleromancyHost, CleromancySessionAuthority, CleromancySyncSelection, ReadingEngine,
    ReadingError, a0_fixture,
};
use graphshell::client::RetainedEndpointSession;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell::personal_sync::{PersonalGraphReplica, SyncRoster};
use graphshell_endpoint::{ProjectionCatalog, ProjectionSource};
use graphshell_local::LocalCarrier;
use graphshell_protocol::{
    CapabilityProfile, IntentResult, PresentationCapability, ProjectionSession,
};
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};

const GRAPH: [u8; 32] = [0x62; 32];

#[test]
fn admitted_pattern_sync_rehydrates_through_another_resident_authority() {
    pollster::block_on(async {
        let (context, field) = a0_fixture();
        let chart = fixture_chart("2026-08-07T12:00:00Z", 181_000);
        let facts = chart.facts(1_000).unwrap();
        let reading = ReadingEngine::calculate(&context, &field).unwrap();
        let mut entropy = FixedEntropy::new([0x22, 0x23]);
        let mut host = CleromancyHost::empty(MemoryBackend::new());
        host.insert_astrology_chart(&chart, 1_000).unwrap();
        host.insert_astrology_chart(&chart, 2_000).unwrap();
        let reading_session = host
            .record_reading_session_at_with_entropy(
                &context,
                &field,
                &reading,
                1_786_104_000_000,
                Some("a22-resident-pattern-sync".to_string()),
                &mut entropy,
            )
            .unwrap();

        let writer = AdmittedEndpointContext::new(
            ProjectionSession("admitted:cleromancy-a22-writer".to_string()),
            [0x22; 32],
        );
        let mut app = CleromancyApp::new(host);
        app.servitors_mut()
            .grant(Grant::new(
                Subject::new(writer.subject()),
                Cap::scope("cleromancy/intents").unwrap(),
                Mode::Write,
            ))
            .unwrap();
        let source_authority = CleromancySessionAuthority::new(app);
        let mut source_catalog = ResidentEndpointCatalog::new();
        source_authority
            .register_catalog(
                &mut source_catalog,
                "cleromancy",
                "Local Cleromancy readings",
            )
            .unwrap();

        let writer_endpoint = source_catalog.open("cleromancy", &writer).unwrap();
        let mut writer_session = RetainedEndpointSession::over(
            Box::new(LocalCarrier::new(writer_endpoint, |_, _| {
                Err("A22 requests a fresh snapshot after a write".to_string())
            })),
            CapabilityProfile::new([PresentationCapability::PortableCard]),
        )
        .unwrap();
        let mounted = writer_session.mount(0).unwrap();
        let (target, action) = writer_session
            .client()
            .accessibility_tree(&mounted, writer_session.profile())
            .unwrap()
            .children
            .into_iter()
            .find_map(|item| {
                item.actions
                    .into_iter()
                    .find(|action| action.intent.0 == CREATE_CONCURRENCE_INTENT)
                    .map(|action| (item.instance, action))
            })
            .expect("the admitted writer sees the concurrence action");
        let (mut draft, invocation) = writer_session
            .open_action_draft(&mounted, target, &action.intent.0)
            .unwrap();
        draft
            .choose("astrology_facts_digest", facts.digest())
            .unwrap();
        draft
            .choose("reading_session_id", &reading_session.id)
            .unwrap();
        assert_eq!(
            writer_session
                .submit_action_draft(&invocation, &mut draft)
                .unwrap(),
            IntentResult::Accepted
        );

        let selection = CleromancySyncSelection::ContextsAndReadings;
        let batch = source_authority.export_sync_batch(selection).unwrap();
        assert_eq!(
            (
                batch.contexts,
                batch.fields,
                batch.readings,
                batch.sessions,
                batch.charts,
                batch.facts,
                batch.concurrences,
            ),
            (1, 1, 1, 1, 1, 2, 1)
        );

        let alice_identity = InMemoryProvider::from_seed([0x62; 32]);
        let bob_identity = InMemoryProvider::from_seed([0x63; 32]);
        let roster = SyncRoster::new([
            alice_identity.master_public_key().to_bytes(),
            bob_identity.master_public_key().to_bytes(),
        ]);
        let personal_selection = selection.personal_graph_selection();
        let mut alice = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            GRAPH,
            &alice_identity,
            roster.clone(),
            personal_selection.clone(),
        )
        .unwrap();
        let bob = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            GRAPH,
            &bob_identity,
            roster,
            personal_selection,
        )
        .unwrap();
        let operation = alice.author(batch.events.clone()).await.unwrap();
        assert!(bob.accept(&operation).await.unwrap());
        let projection = bob.projection().await.unwrap();
        assert!(projection.pending.is_empty());
        assert!(projection.conflicts.is_empty());

        let target_authority = CleromancySessionAuthority::new(CleromancyApp::new(
            CleromancyHost::empty(MemoryBackend::new()),
        ));
        let imported = target_authority
            .import_sync_projection(&projection, selection)
            .unwrap();
        assert_eq!(
            (
                imported.contexts,
                imported.fields,
                imported.readings,
                imported.sessions,
                imported.charts,
                imported.facts,
                imported.concurrences,
            ),
            (1, 1, 1, 1, 1, 2, 1)
        );
        let round_trip = target_authority.export_sync_batch(selection).unwrap();
        assert_eq!(round_trip.events, batch.events);
        assert_eq!(round_trip.digest, batch.digest);

        let reader = AdmittedEndpointContext::new(
            ProjectionSession("admitted:cleromancy-a22-reader".to_string()),
            [0x24; 32],
        );
        let mut target_catalog = ResidentEndpointCatalog::new();
        target_authority
            .register_catalog(
                &mut target_catalog,
                "cleromancy",
                "Local Cleromancy readings",
            )
            .unwrap();
        let mut reader_endpoint = target_catalog.open("cleromancy", &reader).unwrap();
        let mut descriptor = reader_endpoint.describe();
        let request = descriptor.projections.remove(0).request;
        let snapshot = reader_endpoint.snapshot(request).unwrap();
        assert!(
            snapshot
                .presentation
                .offers
                .values()
                .flatten()
                .any(|offer| offer.semantics.label == "Pattern occasion"),
            "the imported resident graph projects the saved pattern"
        );
    });
}

fn fixture_chart(moment: &str, moon_longitude_millidegrees: u32) -> AstrologyChart {
    AstrologyChart::new(
        "fixture-positions/v1",
        "fixture-engine",
        "fixture-ephemeris",
        AstrologyMoment::global(moment),
        [
            AstrologyPosition::new("moon", moon_longitude_millidegrees, 1_000),
            AstrologyPosition::new("sun", 0, 0),
        ],
    )
    .unwrap()
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
