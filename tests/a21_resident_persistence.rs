// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "graphshell-admission")]

use std::collections::VecDeque;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::servitor::{Cap, Grant, Mode, Subject};
use cleromancy::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, CONCURRENCE_FACET,
    CREATE_CONCURRENCE_INTENT, CleromancyApp, CleromancyHost, CleromancySessionAuthority,
    Concurrence, ReadingEngine, ReadingError, a0_fixture,
};
use graphshell::client::RetainedEndpointSession;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell_local::LocalCarrier;
use chirograph::{
    CapabilityProfile, IntentResult, PresentationCapability, ProjectionSession,
};
use muniment::RedbBackend;

#[test]
fn admitted_write_flushes_as_one_reopenable_local_graph() {
    let temp = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(temp.path().join("cleromancy.redb")).unwrap();
    let (context, field) = a0_fixture();
    let chart = fixture_chart("2026-08-07T12:00:00Z", 181_000);
    let facts = chart.facts(1_000).unwrap();
    let reading = ReadingEngine::calculate(&context, &field).unwrap();
    let mut entropy = FixedEntropy::new([0x21, 0x22]);
    let mut host = CleromancyHost::empty(backend.clone());
    host.insert_astrology_chart(&chart, 1_000).unwrap();
    let reading_session = host
        .record_reading_session_at_with_entropy(
            &context,
            &field,
            &reading,
            1_786_104_000_000,
            Some("a21-resident-persistence".to_string()),
            &mut entropy,
        )
        .unwrap();

    let writer = AdmittedEndpointContext::new(
        ProjectionSession("admitted:cleromancy-a21-writer".to_string()),
        [0x21; 32],
    );
    let mut app = CleromancyApp::new(host);
    app.servitors_mut()
        .grant(Grant::new(
            Subject::new(writer.subject()),
            Cap::scope("cleromancy/intents").unwrap(),
            Mode::Write,
        ))
        .unwrap();
    let authority = CleromancySessionAuthority::new(app);
    let mut catalog = ResidentEndpointCatalog::new();
    authority
        .register_catalog(&mut catalog, "cleromancy", "Local Cleromancy readings")
        .unwrap();

    let endpoint = catalog.open("cleromancy", &writer).unwrap();
    let mut session = RetainedEndpointSession::over(
        Box::new(LocalCarrier::new(endpoint, |_, _| {
            Err("A21 requests a fresh snapshot after a write".to_string())
        })),
        CapabilityProfile::new([PresentationCapability::PortableCard]),
    )
    .unwrap();
    let mounted = session.mount(0).unwrap();
    let (target, action) = session
        .client()
        .accessibility_tree(&mounted, session.profile())
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
    let (mut draft, invocation) = session
        .open_action_draft(&mounted, target, &action.intent.0)
        .unwrap();
    draft
        .choose("astrology_facts_digest", facts.digest())
        .unwrap();
    draft
        .choose("reading_session_id", &reading_session.id)
        .unwrap();
    assert_eq!(
        session
            .submit_action_draft(&invocation, &mut draft)
            .unwrap(),
        IntentResult::Accepted
    );

    pollster::block_on(authority.persist(321)).unwrap();
    drop(session);
    drop(catalog);
    drop(authority);

    let host = pollster::block_on(CleromancyHost::open(backend)).unwrap();
    assert!(host.was_reopened());
    let concurrences = host
        .graph()
        .nodes()
        .filter_map(|(key, _)| {
            host.facet_value(key, CONCURRENCE_FACET)
                .and_then(|value| serde_json::from_value::<Concurrence>(value.clone()).ok())
        })
        .collect::<Vec<_>>();
    assert_eq!(concurrences.len(), 1);
    assert_eq!(
        host.replay_concurrence(&concurrences[0]).unwrap(),
        concurrences[0]
    );
    assert!(concurrences[0].members.iter().any(|member| {
        member.address == format!("cleromancy://astrology/facts/{}", facts.digest())
    }));
    assert!(
        concurrences[0]
            .members
            .iter()
            .any(|member| member.address == format!("cleromancy://session/{}", reading_session.id))
    );
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
