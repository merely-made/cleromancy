// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use castellan::{
    authority::PersonaeHost,
    custody::{IdentityVault, InMemoryStorage, Profile},
};
use cleromancy::granted_ambient::GrantedAmbient;
use djinn::{
    door::app_broker::{AppEndpointCatalog, serve_app_broker},
    resident_mere::MereRoutes,
    resident_reservoir::ResidentReservoir,
};
use graphshell_endpoint::door::{
    app_admission::{AllowedAppRoutes, AppId, AppRouteGrants},
    identity::VaultProtectionView,
    tasks::ResidentTasks,
};
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use pandect::{
    Author, DomainId, MereSessions, ReservoirStore, open_mere_backend, open_reservoir_backend,
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
#[tokio::test(flavor = "multi_thread")]
async fn neighboring_preview_requires_its_own_live_grant_and_clears_on_revocation() {
    let root = tempfile::Builder::new()
        .prefix("c4-grant-")
        .tempdir_in("/tmp")
        .unwrap();
    let endpoint = root.path().join("door.sock").to_string_lossy().to_string();
    let persona = PersonaId::from_uuid("00000000-0000-0000-0000-0000c4202610".parse().unwrap());
    let owner = AppId::new("owner-fixture");
    let app = AppId::new("cleromancy");
    let author = Author::person(persona.as_uuid().to_string()).via("owner-fixture");
    let mere = {
        let mut store = ReservoirStore::open(
            open_reservoir_backend(root.path(), persona).unwrap(),
            persona,
        )
        .await
        .unwrap();
        let (mere, _) = store
            .ensure(DomainId::new("notes").unwrap(), 1)
            .await
            .unwrap();
        let sessions = MereSessions::new(open_mere_backend(root.path(), persona, mere.id).unwrap());
        let mut session = sessions.begin(author.clone(), None);
        session
            .edit_now(author.clone(), |graph| {
                use mere::kernel::{
                    geometry::PortablePoint,
                    graph::{
                        Graph,
                        apply::{GraphDelta, add_node, apply_graph_delta},
                    },
                };
                let key = add_node(
                    graph,
                    Some(Graph::node_namespace_id("note:public-c4")),
                    "note:public-c4".into(),
                    PortablePoint::new(0.0, 0.0),
                );
                apply_graph_delta(
                    graph,
                    GraphDelta::SetNodeTitle {
                        key,
                        title: "Permission-scoped public prompt".into(),
                    },
                );
                let large = add_node(
                    graph,
                    Some(Graph::node_namespace_id("note:large-c4")),
                    "note:large-c4".into(),
                    PortablePoint::new(1.0, 1.0),
                );
                apply_graph_delta(
                    graph,
                    GraphDelta::SetNodeFacet {
                        key: large,
                        facet: "public-fixture.large/v1".into(),
                        value: serde_json::json!({"body":"z".repeat(400_000)}),
                    },
                );
            })
            .unwrap();
        session.flush(SystemTime::now()).await.unwrap();
        mere
    };
    let reservoir = ResidentReservoir::open(root.path(), Some(persona))
        .await
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([
        (app.clone(), ResidentReservoir::route()),
        (owner.clone(), ResidentReservoir::route()),
    ]));
    let routes = MereRoutes::new(
        root.path().into(),
        persona,
        catalog.clone(),
        grants.clone(),
        vec![app.clone(), owner.clone()],
    );
    routes.load_access(&reservoir).await.unwrap();
    routes.serve(&mere).await.unwrap();
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes.clone())))
        .await
        .unwrap();
    let identity = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("c4-public".into()),
                "C4 public fixture",
                Ed25519Keypair::from_seed([0xc4; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ));
    let tasks = ResidentTasks::new();
    let scope = tasks.clone();
    let address = endpoint.clone();
    let server = tokio::spawn(async move {
        scope
            .scope(serve_app_broker(
                &address,
                Arc::new(djinn::keeper::Keeper::new(identity)),
                grants,
                60000,
                None,
                catalog,
            ))
            .await
    });
    for _ in 0..100 {
        if graphshell_endpoint::door::local_endpoint::connect_local(&endpoint)
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    routes
        .set_access(
            &reservoir,
            mere.id,
            owner.clone(),
            false,
            false,
            owner.clone(),
        )
        .await
        .unwrap();
    let mut explicit = graphshell_endpoint::door::app_client::AppBrokerClient::open_route_at(
        &endpoint,
        owner.clone(),
        graphshell_endpoint::door::app_admission::AppRouteId::new(
            djinn::resident_mere::mere_route_id("notes"),
        )
        .unwrap(),
    )
    .await
    .unwrap();
    let opened = explicit.open_session().await.unwrap();
    let request = mere_overmap::route::requests(&opened.descriptor).unwrap().1;
    let graph = explicit.snapshot(request).await.unwrap();
    let mut cards = std::collections::HashMap::new();
    for r in mere_overmap::route::card_requests(&graph) {
        cards.insert(
            r.resource,
            explicit.resource(r.session, r.resource).await.unwrap(),
        );
    }
    let r = mere_overmap::route::session_graph_request(&graph, &cards).unwrap();
    let bytes = explicit
        .resource(r.session.clone(), r.resource)
        .await
        .unwrap();
    let old_frame = chirograph::ResourceResponse::new(r.session, bytes.clone());
    assert!(
        serde_json::to_vec(&old_frame).unwrap().len() > 1024 * 1024,
        "the real admitted graph requires chunking"
    );
    let envelope: mere_overmap::wire::SessionGraphV1 = serde_json::from_slice(&bytes).unwrap();
    let graph = envelope.into_graph().unwrap();
    let (_, node) = graph.get_node_by_url("note:large-c4").unwrap();
    assert_eq!(
        graph
            .facets()
            .get(
                &node.id,
                &chartulary::FacetId::new("public-fixture.large/v1")
            )
            .unwrap()["body"],
        "z".repeat(400_000)
    );
    explicit.close().await.unwrap();
    assert!(
        GrantedAmbient::open_at(&endpoint, "notes").await.is_err(),
        "explicit access alone does not grant ambient crossing"
    );
    routes
        .set_access(&reservoir, mere.id, app.clone(), false, true, owner.clone())
        .await
        .unwrap();
    let mut reader = GrantedAmbient::open_at(&endpoint, "notes").await.unwrap();
    assert!(reader.refresh().await.unwrap());
    assert!(
        reader
            .cards()
            .iter()
            .any(|c| c.card.title == "Permission-scoped public prompt")
    );
    assert!(
        reader
            .cards()
            .iter()
            .all(|c| c.domain == "notes" && c.card.media.is_empty())
    );
    routes
        .set_access(&reservoir, mere.id, app, false, false, owner)
        .await
        .unwrap();
    assert!(reader.refresh().await.is_err());
    assert!(
        reader.cards().is_empty(),
        "revocation withdraws the previous cards immediately on the next guarded read"
    );
    drop(reader);
    server.abort();
    let _ = server.await;
}
