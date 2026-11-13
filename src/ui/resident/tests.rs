// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use scenotime::{Revision, SceneEpoch};

// The owner-approved divination guest, as the integration tests install it.
#[cfg(unix)]
#[allow(dead_code)]
mod authority_pack {
    include!("../../../tests/common/authority_pack.rs");
}

#[test]
fn installed_die_survives_owner_refresh_with_authoritative_catalog() {
    let mut c = Consultation::new(CleromancyHost::empty(MemoryBackend::new()));
    let digest = pollster::block_on(c.install_six_sided_die()).unwrap();
    let stale = Consultation::new(CleromancyHost::empty(MemoryBackend::new()))
        .catalog()
        .unwrap();
    match owner_update(
        &c,
        WorkerUpdate::Die {
            catalog: stale,
            field_digest: digest.clone(),
        },
    )
    .unwrap()
    {
        WorkerUpdate::Die {
            catalog,
            field_digest,
        } => {
            assert_eq!(field_digest, digest);
            assert_eq!(catalog.fields.len(), 1);
            assert_eq!(catalog.fields[0].digest(), digest);
        },
        other => panic!("installed field selection was lost: {other:?}"),
    }
}

fn envelope() -> session_item::SessionGraphV1 {
    let graph = Graph::new();
    session_item::SessionGraphV1 {
        schema: session_item::SESSION_GRAPH_SCHEMA.into(),
        application: "cleromancy".into(),
        read_only: false,
        views: Default::default(),
        attention: Default::default(),
        session: serde_json::from_str("\"00000000-0000-0000-0000-000000000001\"").unwrap(),
        epoch: SceneEpoch(1),
        revision: Revision(4),
        graph: graph.to_snapshot(),
        facets: graph.facets().clone(),
    }
}

#[test]
fn session_without_own_view_clears_previous_saved_layout() {
    let mut original = envelope();
    original.views.insert(
        "reading-workspace".into(),
        incipit::view::ViewIntent {
            strategy: Some("grid.default".into()),
            ..Default::default()
        },
    );
    let mut model = MereViewModel::default();
    restore_scoped_layout(&mut model, Some(&original));
    assert_eq!(model.layout, "grid.default");

    let mut next = envelope();
    next.session = serde_json::from_str("\"00000000-0000-0000-0000-000000000002\"").unwrap();
    next.views.insert(
        "graphshell.mere".into(),
        incipit::view::ViewIntent {
            strategy: Some("spiral.default".into()),
            ..Default::default()
        },
    );
    restore_scoped_layout(&mut model, Some(&next));
    assert_eq!(model.layout, MereViewModel::default().layout);

    // Returning to the original session restores its own saved choice.
    restore_scoped_layout(&mut model, Some(&original));
    assert_eq!(model.layout, "grid.default");
    next.views
        .insert("reading-workspace".into(), incipit::view::ViewIntent::default());
    restore_scoped_layout(&mut model, Some(&next));
    assert_eq!(model.layout, MereViewModel::default().layout);
    restore_scoped_layout(&mut model, Some(&original));
    restore_scoped_layout(&mut model, None);
    assert_eq!(model.layout, MereViewModel::default().layout);
}

#[cfg(unix)]
#[test]
fn readonly_archive_layout_selection_and_comparison_never_fork_or_write() {
    use super::super::action::{ConsultationContext, ConsultationLayout};
    use castellan::authority::PersonaeHost;
    use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
    use dramatis::view::VaultProtectionView;
    use mere_resident::embedded_reservoir::ReservoirAttachmentOptions;
    use mere_view::{ArchiveRequest, SessionStep};
    use ostiary::app_admission::AppId;
    use personae::{Ed25519Keypair, PersonaId, ProfileId};

    fn bytes(root: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn visit(
            root: &std::path::Path,
            dir: &std::path::Path,
            result: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
        ) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(root, &path, result);
                } else {
                    result.insert(
                        path.strip_prefix(root).unwrap().into(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut result = std::collections::BTreeMap::new();
        visit(root, root, &mut result);
        result
    }

    let root = tempfile::tempdir().unwrap();
    let address = format!(
        "/tmp/c1-readonly-{}.sock",
        root.path().file_name().unwrap().to_string_lossy()
    );
    let persona = PersonaId::from_uuid("00000000-0000-0000-0000-00c120261014".parse().unwrap());
    // The resident loads only owner-approved guests; approve the divination guest first.
    authority_pack::install_divination_guest(root.path(), persona);
    let identity = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("c1-public-readonly-fixture".into()),
                "C1 public fixture",
                Ed25519Keypair::from_seed([0xc1; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let attachment = ReservoirAttachment::open(
            ReservoirAttachmentOptions {
                endpoint: address,
                application: AppId::new("cleromancy"),
                declaration: crate::granted_ambient::declaration(),
                grants: Vec::new(),
                shared_root: root.path().into(),
                persona,
                domain: pandect::DomainId::new("divination").unwrap(),
                session_duration: std::time::Duration::from_secs(60),
            },
            || async { Ok(identity as Arc<dyn dramatis_resident::custody_host::ResidentCustody>) },
        )
        .await
        .unwrap();
        let mut client = ResidentConsultation::attach(attachment).await.unwrap();
        let original = client.envelope.as_ref().unwrap().session;
        let field = client.consultation().unwrap().catalog().unwrap().fields[0].digest();
        let mut occasions = Vec::new();
        for question in ["What is retained?", "What remains transient?"] {
            let update = client
                .action(ConsultationAction::Read {
                    context: ConsultationContext::New(crate::ContextDraft::new(
                        "C1 archive reading",
                        question,
                        "ownership",
                    )),
                    field_digest: field.clone(),
                    mode: crate::SelectionMode::Calculated,
                    derivation: None,
                    layout: ConsultationLayout::Single,
                    astrology_facts_digest: None,
                    situation: Default::default(),
                })
                .await
                .unwrap();
            let WorkerUpdate::Reading { detail, .. } = update else {
                panic!("expected reading")
            };
            occasions.push(detail.session.id);
        }
        client
            .request(MereViewRequest::Archive(ArchiveRequest::Save(original)))
            .await
            .unwrap();
        let archive = client.model.archive.as_ref().unwrap().entries[0]
            .key
            .clone();
        client
            .request(MereViewRequest::Archive(ArchiveRequest::Open(archive)))
            .await
            .unwrap();
        let thaw = client.envelope.as_ref().unwrap().session;
        assert!(client.envelope.as_ref().unwrap().read_only);
        let count = client.model.sessions.len();
        let before = bytes(root.path());
        let original_view = client.envelope.as_ref().unwrap().views.clone();

        client
            .request(MereViewRequest::Layout("grid.default".into()))
            .await
            .unwrap();
        assert_eq!(client.model.layout, "grid.default");
        client
            .action(ConsultationAction::SelectSession {
                session_id: occasions[0].clone(),
            })
            .await
            .unwrap();
        assert_eq!(
            client.restored_selection().unwrap().0.session.id,
            occasions[0]
        );
        client
            .action(ConsultationAction::CompareSessions {
                left_session_id: occasions[1].clone(),
                right_session_id: occasions[0].clone(),
            })
            .await
            .unwrap();
        client
            .select(&format!("cleromancy://session/{}", occasions[0]))
            .await
            .unwrap();
        client.refresh().await.unwrap();
        assert_eq!(client.model.layout, "grid.default");
        assert_eq!(client.envelope.as_ref().unwrap().session, thaw);
        assert!(client.envelope.as_ref().unwrap().read_only);
        assert_eq!(client.envelope.as_ref().unwrap().views, original_view);
        assert_eq!(client.model.sessions.len(), count);
        assert_eq!(
            bytes(root.path()),
            before,
            "read-only presentation must send no owner write"
        );

        // A real owner refusal must discard the transient focus and
        // layout together, without accepting any first-edit fork.
        let reading = client
            .consultation()
            .unwrap()
            .detail(&occasions[0])
            .unwrap()
            .readings[0]
            .clone();
        let node = Graph::node_namespace_id(&format!("cleromancy://reading/{}", reading.id));
        let mut damaged = serde_json::to_value(reading).unwrap();
        damaged["candidate_id"] = serde_json::json!("invented-readonly-candidate");
        let rejected = chirograph::IntentInvocation {
            session: client.graph_projection.session.clone(),
            target: sceno::InstanceId(0),
            observed_epoch: client.graph_projection.scene.epoch,
            observed_revision: client.graph_projection.scene.revision,
            intent: session_item::APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&session_item::ApplyEditsV1::checked(
                client.envelope.as_ref().unwrap().revision,
                vec![CapturedDelta::ReplaySetNodeFacetById {
                    node_id: node.to_string(),
                    facet: crate::host::READING_FACET.into(),
                    value_json: damaged.to_string(),
                }],
            ))
            .unwrap(),
        };
        assert!(client.invoke(rejected).await.is_err());
        assert!(
            !client.unavailable,
            "the route remains readable after domain refusal"
        );
        assert!(client.transient_view.is_none());
        assert_eq!(
            client.model.layout,
            original_view
                .get("reading-workspace")
                .and_then(|view| view.strategy.clone())
                .unwrap_or_default()
        );
        client.refresh().await.unwrap();
        assert_eq!(client.envelope.as_ref().unwrap().session, thaw);
        assert!(client.envelope.as_ref().unwrap().read_only);
        assert_eq!(client.model.sessions.len(), count);
        assert_eq!(
            bytes(root.path()),
            before,
            "failed first edit changes no archive session state"
        );

        client
            .request(MereViewRequest::Session(original, SessionStep::Switch))
            .await
            .unwrap();
        assert!(
            client.transient_view.is_none(),
            "a view cannot cross attachments"
        );
        assert!(!client.envelope.as_ref().unwrap().read_only);
        client
            .request(MereViewRequest::Layout("phyllotaxis.default".into()))
            .await
            .unwrap();
        assert_eq!(
            client.envelope.as_ref().unwrap().views["reading-workspace"]
                .strategy
                .as_deref(),
            Some("phyllotaxis.default")
        );
        assert_eq!(client.model.sessions.len(), count);
        assert_ne!(
            bytes(root.path()),
            before,
            "an admitted writable layout is persisted"
        );
        client.close().await.unwrap();
    });
}

#[test]
fn acknowledged_reading_survives_view_refusal_but_unavailable_still_purges() {
    let refused = postcommit_focus_notice(false, Err("stale view revision".into())).unwrap();
    assert_eq!(
        refused.as_deref(),
        Some("Reading saved; focus not saved: stale view revision")
    );
    assert_eq!(postcommit_focus_notice(false, Ok(())).unwrap(), None);
    assert_eq!(
        postcommit_focus_notice(true, Err("access revoked".into())),
        Err("access revoked".into())
    );
}

#[test]
fn complete_graph_must_match_the_attached_projection_before_cache_publication() {
    let prior = envelope();
    let before = serde_json::to_vec(&prior).unwrap();
    verify_envelope(&prior, prior.session, prior.epoch, prior.revision).unwrap();
    for (session, epoch, revision) in [
        (
            serde_json::from_str("\"00000000-0000-0000-0000-000000000002\"").unwrap(),
            prior.epoch,
            prior.revision,
        ),
        (prior.session, SceneEpoch(2), prior.revision),
        (prior.session, prior.epoch, Revision(5)),
    ] {
        assert!(verify_envelope(&prior, session, epoch, revision).is_err());
        assert_eq!(serde_json::to_vec(&prior).unwrap(), before);
    }
    let mut other_app = prior.clone();
    other_app.application = "graphshell".into();
    assert!(verify_envelope(&other_app, prior.session, prior.epoch, prior.revision).is_err());
    let mut future = prior.clone();
    future.schema = "mere.session.graph/v9".into();
    assert!(verify_envelope(&future, prior.session, prior.epoch, prior.revision).is_err());
}

#[test]
fn acknowledged_result_uses_current_owner_records_instead_of_precommit_candidate() {
    let mut owner = Consultation::new(CleromancyHost::candidate(Graph::new()).unwrap());
    let field = pollster::block_on(owner.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(owner.save_context_at(
        crate::ContextDraft::new("A boundary", "What remains visible?", "ownership"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(owner.read(&context, &field, crate::SelectionMode::Calculated)).unwrap();
    let id = detail.session.id.clone();
    let candidate = WorkerUpdate::Reading {
        catalog: owner.catalog().unwrap(),
        detail,
    };
    // A peer advances the same owner graph between commit and UI delivery.
    pollster::block_on(owner.reflect(&id, "A concurrent peer reflection".into())).unwrap();
    match owner_update(&owner, candidate).unwrap() {
        WorkerUpdate::Reading { detail, catalog } => {
            assert_eq!(detail.reflections.len(), 1);
            assert_eq!(detail.reflections[0].body, "A concurrent peer reflection");
            assert_eq!(catalog, owner.catalog().unwrap());
            assert_eq!(detail, owner.detail(&id).unwrap());
        },
        other => panic!("expected current owner reading, got {other:?}"),
    }
}

#[test]
fn mint_switches_only_to_one_new_live_identity_and_refuses_ambiguity() {
    let id =
        |n: u8| serde_json::from_str(&format!("\"00000000-0000-0000-0000-{:012}\"", n)).unwrap();
    let row = |n: u8| mere_view::SessionEntry {
        id: id(n),
        name: "Same visible name".into(),
        detail: None,
        attached: false,
        trashed: false,
        steps: vec![mere_view::SessionStep::Switch],
    };
    let mut model = MereViewModel {
        sessions: vec![row(1), row(2)],
        ..Default::default()
    };
    assert_eq!(newly_minted_session(&[id(1)], &model).unwrap(), id(2));
    model.sessions.push(row(3));
    assert!(newly_minted_session(&[id(1)], &model).is_err());
    assert!(newly_minted_session(&[id(1), id(2), id(3)], &model).is_err());
    model.sessions[2].trashed = true;
    assert_eq!(newly_minted_session(&[id(1)], &model).unwrap(), id(2));
}
