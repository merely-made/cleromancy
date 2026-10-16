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
