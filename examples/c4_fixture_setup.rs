// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Public C4 acceptance fixture. Domain edits use the ordinary admitted route;
//! this helper owns no graph files and refuses an unmarked or nonempty profile.
#[allow(dead_code)]
#[path = "../tests/support/ambient_fixture.rs"]
mod fixture;
use chirograph::{CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult};
use djinn::embedded_reservoir::{ApplicationReservoirOptions, ReservoirAttachment};
use graphshell_endpoint::door::app_admission::AppId;
use mere_overmap::{route, wire};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("expected public fixture root")?,
    );
    if std::fs::read(root.join(".c1-public-synthetic"))?
        != b"cleromancy-c1 public synthetic fixture/v1\n"
    {
        return Err("not the marked public fixture".into());
    }
    let inspect = std::env::args().nth(2).as_deref() == Some("inspect");
    let mut prompt = djinn::startup_vault::NativeOrTerminal {
        native: djinn::identity_ui::SystemNativeIdentityUi::default(),
    };
    let mut attachment = ReservoirAttachment::open_application(
        ApplicationReservoirOptions {
            endpoint: "/tmp/c4-20261016.sock".into(),
            application: AppId::new("cleromancy"),
            domain: pandect::DomainId::new("divination")?,
            shared_root: root.join("shared"),
            vault_dir: root.join("ordinary-vault"),
            profile: None,
            persona: None,
            session_duration: Duration::from_secs(3600),
        },
        &mut prompt,
        |routes| async { Ok(routes.with_divination_authority()) },
    )
    .await?;
    let opened = attachment.client_mut().open_session().await?;
    let (sessions, graph) =
        route::requests(&opened.descriptor).ok_or("missing route projections")?;
    let sessions = attachment.client_mut().snapshot(sessions).await?;
    if !inspect {
        let intent =
            route::intent(&mere_view::MereViewRequest::Mint, &sessions).ok_or("missing mint")?;
        let response = attachment
            .client_mut()
            .request_body(CarrierRequestBody::Intent(intent))
            .await?;
        if !matches!(
            response,
            CarrierResponseBody::Intent(IntentResult::Accepted)
        ) {
            return Err(format!("mint refused: {response:?}").into());
        }
    }
    let (mut graph, mut envelope) = read_graph(&mut attachment, graph.clone()).await?;
    let mut host = cleromancy::CleromancyHost::candidate(envelope.clone().into_graph()?)?;
    if inspect {
        let c = cleromancy::Consultation::new(host);
        let catalog = c.catalog()?;
        let details = catalog
            .session_summaries
            .iter()
            .map(|s| c.detail(&s.session_id))
            .collect::<Result<Vec<_>, _>>()?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"details":details,"views":c.host().ambient_views()?,"memory":catalog.ambient_memory})
            )?
        );
    } else {
        if !host.is_empty() {
            return Err("fixture setup refuses a nonempty session".into());
        }
        let edits = Arc::new(Mutex::new(Vec::new()));
        host.record_edits(edits.clone());
        let (_c, detail) = fixture::fixture_on(host);
        let edits = std::mem::take(&mut *edits.lock().unwrap());
        // Bound the carrier message without weakening domain validation. Each
        // prefix admitted below is itself a complete replay-valid candidate.
        let mut offset = 0;
        let mut batches = Vec::new();
        while offset < edits.len() {
            let mut candidate = envelope.clone().into_graph()?;
            let mut bytes = 0;
            let mut accepted = 0;
            for (index, edit) in edits[offset..].iter().enumerate() {
                bytes += serde_json::to_vec(edit)?.len() + 1;
                if bytes > 180_000 {
                    break;
                }
                mere::kernel::graph::replay_captured_deltas_onto(
                    &mut candidate,
                    vec![edit.clone()],
                );
                if cleromancy::host::validate_divination_graph(&candidate).is_ok() {
                    accepted = index + 1;
                }
            }
            if accepted == 0 {
                return Err("one valid fixture stage exceeds the bounded carrier batch".into());
            }
            let batch = edits[offset..offset + accepted].to_vec();
            let intent = IntentInvocation {
                session: graph.session.clone(),
                target: sceno::InstanceId(0),
                observed_epoch: graph.scene.epoch,
                observed_revision: graph.scene.revision,
                intent: wire::APPLY_EDITS_INTENT.into(),
                payload: serde_json::to_vec(&wire::ApplyEditsV1::checked(
                    envelope.revision,
                    batch,
                ))?,
            };
            let body = CarrierRequestBody::Intent(intent);
            let encoded = serde_json::to_vec(&body)?.len();
            if encoded > 900_000 {
                return Err("fixture carrier exceeds the 900k acceptance bound".into());
            }
            let response = attachment.client_mut().request_body(body).await?;
            if !matches!(
                response,
                CarrierResponseBody::Intent(IntentResult::Accepted)
            ) {
                return Err(format!("fixture stage refused: {response:?}").into());
            }
            offset += accepted;
            batches.push(serde_json::json!({"deltas":accepted,"encoded_body_bytes":encoded}));
            let opened = attachment.client_mut().open_session().await?;
            let request = route::requests(&opened.descriptor)
                .ok_or("missing graph after fixture stage")?
                .1;
            (graph, envelope) = read_graph(&mut attachment, request).await?;
        }
        let installed = cleromancy::Consultation::new(cleromancy::CleromancyHost::candidate(
            envelope.into_graph()?,
        )?);
        if installed.detail(&detail.session.id)? != detail {
            return Err("admitted fixture differs from its calculated source".into());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"session":detail.session,"detail":detail,"batches":batches,"public_fixture":"calculated readings with repeated position occurrences; not an OS-random cast"})
            )?
        );
    }
    attachment.close().await?;
    Ok(())
}

async fn read_graph(
    attachment: &mut ReservoirAttachment,
    request: chirograph::ProjectionRequest,
) -> Result<(chirograph::ProjectionSnapshot, wire::SessionGraphV1), Box<dyn std::error::Error>> {
    let graph = attachment.client_mut().snapshot(request).await?;
    let mut cards = HashMap::new();
    for request in route::card_requests(&graph) {
        let bytes = attachment
            .client_mut()
            .resource(request.session, request.resource)
            .await?;
        cards.insert(request.resource, bytes);
    }
    let request = route::session_graph_request(&graph, &cards).ok_or("missing complete graph")?;
    let bytes = attachment
        .client_mut()
        .resource(request.session, request.resource)
        .await?;
    let envelope: wire::SessionGraphV1 = serde_json::from_slice(&bytes)?;
    Ok((graph, envelope))
}
