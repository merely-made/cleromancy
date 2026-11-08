// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The natal authority through the wasm guest (cycle breaks P3). These four tests moved here
//! from Mere's `resident_mere` unit tests, where the resident linked Cleromancy's validator;
//! the resident now links none. Each serves a `divination` mere on an in-process route with the
//! owner-approved guest installed, and checks that two applications share admitted natal
//! readings and that a tampered record, a re-sealed result or a changed rule is refused.

mod common {
    #[allow(dead_code)]
    pub mod authority_pack {
        include!("common/authority_pack.rs");
    }
}
use common::authority_pack::install_divination_guest;

use std::collections::BTreeMap;
use std::sync::Arc;

use chirograph::{
    IntentInvocation, IntentResult, PortableCardV1, ProjectionSession, ProjectionSnapshot,
    ResourceRequest,
};
use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use pandect::mere_dir;
use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointSession};
use graphshell_host::lifecycle::AdmittedEndpointContext;
use mere_overmap::wire::{self as session_item, ApplyEditsV1};
use mere_resident::door::app_broker::AppEndpointCatalog;
use mere_resident::resident_mere::{MereRoutes, mere_route_id};
use mere_resident::resident_reservoir::{
    EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
};
use ostiary::app_admission::AppId;
use personae::PersonaId;
use sceno::InstanceId;
use uuid::Uuid;

const DOMAIN: &str = "divination";

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x4e41_5441_4c))
}

fn context(app: &str) -> AdmittedEndpointContext {
    AdmittedEndpointContext::new(ProjectionSession("v1:test".into()), [0x2b; 32]).with_application(app)
}

struct Fixture {
    _root: tempfile::TempDir,
    reservoir: ResidentReservoir,
    _routes: MereRoutes,
    catalog: AppEndpointCatalog,
    route: String,
}

impl Fixture {
    async fn attach(&self, app: &str) -> ResidentEndpointSession {
        self.catalog
            .update(|catalog| catalog.open(&self.route, &context(app)))
            .await
            .unwrap()
    }
}

/// A `divination` mere served by Mere's defaults plus the approved guest.
async fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    install_divination_guest(root.path(), persona());
    let reservoir = ResidentReservoir::open(root.path(), persona()).await.unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    let declaration = mere_resident::resident_reservoir::mere_client_declaration();
    mere_resident::applications::install(
        &grants,
        ["turnstone", "knot-editor"]
            .map(|app| mere_resident::applications::grant_for(&AppId::new(app), &declaration, None))
            .to_vec(),
    );
    let routes = MereRoutes::new(root.path().to_path_buf(), persona(), catalog.clone(), grants)
        .with_resident_defaults()
        .unwrap();
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes.clone())))
        .await
        .unwrap();
    let mut door = catalog
        .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context("turnstone")))
        .await
        .unwrap();
    let request = door.describe().projections.remove(0).request;
    let listed = door.snapshot(request).unwrap();
    let ensure = IntentInvocation {
        session: listed.session.clone(),
        target: InstanceId(0),
        observed_epoch: listed.scene.epoch,
        observed_revision: listed.scene.revision,
        intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
        payload: serde_json::to_vec(&EnsureMereV1::new(DOMAIN)).unwrap(),
    };
    assert_eq!(door.invoke(ensure).unwrap(), IntentResult::Accepted);
    Fixture {
        _root: root,
        reservoir,
        _routes: routes,
        catalog,
        route: mere_route_id(DOMAIN),
    }
}


fn session_tree_bytes(path: &std::path::Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        path: &std::path::Path,
        files: &mut BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(path, path, &mut files);
    files
}

fn projection(session: &mut ResidentEndpointSession, index: usize) -> ProjectionSnapshot {
    let request = session.describe().projections[index].request.clone();
    session.snapshot(request).unwrap()
}

fn graph(session: &mut ResidentEndpointSession) -> ProjectionSnapshot {
    projection(session, 1)
}

fn session_item_of(snapshot: &ProjectionSnapshot) -> InstanceId {
    snapshot
        .scene
        .active_items_in_order()
        .into_iter()
        .find_map(|(instance, item)| {
            snapshot.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .filter(|source| source.adapter == session_item::SESSION_ITEM_SOURCE)
                .map(|_| instance)
        })
        .expect("the resident graph shows the session item")
}

fn on_item(snapshot: &ProjectionSnapshot, intent: &str, payload: Vec<u8>) -> IntentInvocation {
    IntentInvocation {
        session: snapshot.session.clone(),
        target: session_item_of(snapshot),
        observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision,
        intent: intent.into(),
        payload,
    }
}

fn apply(
    snapshot: &ProjectionSnapshot,
    edits: Vec<pandect::CapturedDelta>,
) -> IntentInvocation {
    on_item(
        snapshot,
        session_item::APPLY_EDITS_INTENT,
        serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
    )
}

fn disclosed_graph(
    session: &mut impl PresentationSource<Error = String>,
    snapshot: &ProjectionSnapshot,
) -> session_item::SessionGraphV1 {
    let offer = &snapshot
        .presentation
        .offers_for(session_item_of(snapshot))
        .unwrap()[0];
    let card = session
        .resource(ResourceRequest {
            session: snapshot.session.clone(),
            resource: offer.resource,
        })
        .unwrap();
    assert!(card.has_valid_address());
    let card: PortableCardV1 = serde_json::from_slice(&card.bytes).unwrap();
    let graph = session
        .resource(ResourceRequest {
            session: snapshot.session.clone(),
            resource: card.media[0],
        })
        .unwrap();
    assert!(graph.has_valid_address());
    serde_json::from_slice(&graph.bytes).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn natal_product_authority_admits_replay_and_refuses_tampering_across_apps() {
    let fixture = fixture().await;
    let mut product = fixture.attach("turnstone").await;
    let mut other = fixture.attach("knot-editor").await;
    let initial = graph(&mut product);
    let mut candidate = disclosed_graph(&mut product, &initial)
        .into_graph()
        .unwrap();
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let held = captured.clone();
    candidate.set_recorder(Some(Arc::new(move |delta| {
        held.lock().unwrap().push(delta.clone())
    })));
    let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":null,"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
    let birth = personal_facts::birth_resource(&facts).unwrap();
    let record = cleromancy::natal::NatalReading::new(
        "public-source-persona".into(),
        "Public natal subject".into(),
        birth,
        "A synthetic natal question".into(),
        cleromancy::natal::FoldChoice::Refuse,
        1792149302000,
        "00000000000000000000000000000008".into(),
    )
    .unwrap();
    let mut host = cleromancy::CleromancyHost::candidate(candidate).unwrap();
    host.insert_natal_reading(&record).unwrap();
    assert_eq!(
        product
            .invoke(apply(&initial, captured.lock().unwrap().clone()))
            .unwrap(),
        IntentResult::Accepted
    );
    let shared = graph(&mut other);
    let current = disclosed_graph(&mut other, &shared);
    let host =
        cleromancy::CleromancyHost::candidate(current.clone().into_graph().unwrap())
            .unwrap();
    assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
    let mut bad = record.clone();
    bad.question = "Tampered under the sealed id".into();
    assert!(matches!(
        other
            .invoke(apply(
                &shared,
                vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                    node_id: mere::kernel::graph::Graph::node_namespace_id(&format!(
                        "cleromancy://natal/{}",
                        record.id
                    ))
                    .to_string(),
                    facet: cleromancy::natal::NATAL_FACET.into(),
                    value_json: serde_json::to_string(&bad).unwrap()
                }]
            ))
            .unwrap(),
        IntentResult::Rejected { .. }
    ));
    let after = graph(&mut other);
    assert_eq!(
        serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
        serde_json::to_value(current).unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn natal_houses_authority_admits_both_systems_and_refuses_cusp_tamper() {
    for system in [
        cleromancy::natal_houses::HouseSystem::WholeSign,
        cleromancy::natal_houses::HouseSystem::Equal,
    ] {
        let fixture = fixture().await;
        let mut product = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        let initial = graph(&mut product);
        let mut candidate = disclosed_graph(&mut product, &initial)
            .into_graph()
            .unwrap();
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        let held = captured.clone();
        candidate.set_recorder(Some(Arc::new(move |delta| {
            held.lock().unwrap().push(delta.clone())
        })));
        let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":{"label":"Public coordinate fixture","latitude_microdegrees":40712800,"longitude_microdegrees":-74006000,"precision":"Public synthetic"},"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
        let birth = personal_facts::birth_resource(&facts).unwrap();
        let record = cleromancy::natal::NatalReading::new(
            "public-source-persona".into(),
            "Public natal subject".into(),
            birth,
            "A synthetic natal question".into(),
            cleromancy::natal::FoldChoice::Refuse,
            1792149302000,
            "00000000000000000000000000000008".into(),
        )
        .unwrap()
        .with_houses(system)
        .unwrap();
        let mut host = cleromancy::CleromancyHost::candidate(candidate).unwrap();
        host.insert_natal_reading(&record).unwrap();
        assert_eq!(
            product
                .invoke(apply(&initial, captured.lock().unwrap().clone()))
                .unwrap(),
            IntentResult::Accepted
        );
        let shared = graph(&mut other);
        let current = disclosed_graph(&mut other, &shared);
        let host =
            cleromancy::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                .unwrap();
        assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
        let mut bad = record.clone();
        if let cleromancy::natal_houses::HouseOutcome::Calculated {
            cusps_microdegrees,
            ..
        } = &mut bad.houses.as_mut().unwrap().outcome
        {
            cusps_microdegrees[0] = (cusps_microdegrees[0] + 1) % 360_000_000;
        } else {
            panic!("expected actual houses");
        }
        assert!(matches!(
            other
                .invoke(apply(
                    &shared,
                    vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                        node_id: mere::kernel::graph::Graph::node_namespace_id(&format!(
                            "cleromancy://natal/{}",
                            record.id
                        ))
                        .to_string(),
                        facet: cleromancy::natal::NATAL_FACET.into(),
                        value_json: serde_json::to_string(&bad).unwrap()
                    }]
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        let after = graph(&mut other);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn natal_cap_authority_admits_retained_observations_and_refuses_motion_tamper() {
    for observed in [false, true] {
        let fixture = fixture().await;
        let mut product = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        let initial = graph(&mut product);
        let mut candidate = disclosed_graph(&mut product, &initial)
            .into_graph()
            .unwrap();
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        let held = captured.clone();
        candidate.set_recorder(Some(Arc::new(move |delta| {
            held.lock().unwrap().push(delta.clone())
        })));
        let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":{"label":"Public coordinate fixture","latitude_microdegrees":40712800,"longitude_microdegrees":-74006000,"precision":"Public synthetic"},"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
        let birth = personal_facts::birth_resource(&facts).unwrap();
        let record = cleromancy::natal::NatalReading::new(
            "public-source-persona".into(),
            "Public natal subject".into(),
            birth,
            "A synthetic natal question".into(),
            cleromancy::natal::FoldChoice::Refuse,
            1792149302000,
            "00000000000000000000000000000008".into(),
        )
        .unwrap()
        .with_supplement(
            Some(cleromancy::natal_cap::HouseChoice::Placidus),
            observed.then(|| cleromancy::natal_cap::OrientationSnapshot {
                finals2000a: include_str!("../tests/fixtures/natal-cap-finals2000a.txt").into(),
            }),
        )
        .unwrap();
        let mut host = cleromancy::CleromancyHost::candidate(candidate).unwrap();
        host.insert_natal_reading(&record).unwrap();
        assert_eq!(
            product
                .invoke(apply(&initial, captured.lock().unwrap().clone()))
                .unwrap(),
            IntentResult::Accepted
        );
        let shared = graph(&mut other);
        let current = disclosed_graph(&mut other, &shared);
        let host =
            cleromancy::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                .unwrap();
        assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
        let mut bad = record.clone();
        if let cleromancy::natal_cap::MotionOutcome::Calculated { planets, .. } =
            &mut bad.supplement.as_mut().unwrap().motion
        {
            planets[0].microdegrees_per_tt_day += 1;
        } else {
            panic!("expected motion")
        };
        assert!(matches!(
            other
                .invoke(apply(
                    &shared,
                    vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                        node_id: mere::kernel::graph::Graph::node_namespace_id(&format!(
                            "cleromancy://natal/{}",
                            record.id
                        ))
                        .to_string(),
                        facet: cleromancy::natal::NATAL_FACET.into(),
                        value_json: serde_json::to_string(&bad).unwrap()
                    }]
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        let after = graph(&mut other);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn natal_aspects_authority_admits_cross_app_and_refuses_resealed_results_and_rules() {
    use cleromancy::natal_aspects::{AspectOutcome, AspectRules};

    let fixture = fixture().await;
    let mut product = fixture.attach("turnstone").await;
    let mut other = fixture.attach("knot-editor").await;
    let initial = graph(&mut product);
    let mut candidate = disclosed_graph(&mut product, &initial)
        .into_graph()
        .unwrap();
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let held = captured.clone();
    candidate.set_recorder(Some(Arc::new(move |delta| {
        held.lock().unwrap().push(delta.clone())
    })));
    let facts: dramatis::birth::BirthFacts = serde_json::from_value(serde_json::json!({
        "schema": "dramatis.birth-facts/v1",
        "subject": {"Profile": {"id": "public-natal-subject"}},
        "date": {"year": 2000, "month": 1, "day": 1},
        "time": {"Known": {"hour": 12, "minute": 0, "second": 0}},
        "place": null,
        "timezone": "UTC",
        "source": {"label": "Public product-authority fixture", "reference": null}
    }))
    .unwrap();
    let record = cleromancy::natal::NatalReading::new(
        "public-source-persona".into(),
        "Public natal subject".into(),
        personal_facts::birth_resource(&facts).unwrap(),
        "A synthetic natal aspects question".into(),
        cleromancy::natal::FoldChoice::Refuse,
        1792149302000,
        "00000000000000000000000000000008".into(),
    )
    .unwrap()
    .with_aspects(AspectRules::default())
    .unwrap();
    let AspectOutcome::Calculated { hits, .. } = &record.aspects.as_ref().unwrap().outcome
    else {
        panic!("expected actual aspects");
    };
    assert!(!hits.is_empty(), "fixture must exercise calculated results");
    assert!(hits.iter().any(|hit| hit.orb_millidegrees > 0));
    let mut host = cleromancy::CleromancyHost::candidate(candidate).unwrap();
    host.insert_natal_reading(&record).unwrap();
    assert_eq!(
        product
            .invoke(apply(&initial, captured.lock().unwrap().clone()))
            .unwrap(),
        IntentResult::Accepted
    );
    let shared = graph(&mut other);
    let current = disclosed_graph(&mut other, &shared);
    let host =
        cleromancy::CleromancyHost::candidate(current.clone().into_graph().unwrap())
            .unwrap();
    assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
    let mere = fixture.reservoir.meres().await.remove(0);
    let sessions = mere_dir(fixture._root.path(), persona(), mere.id).join("sessions");
    let product_snapshot = graph(&mut product);
    let product_current = disclosed_graph(&mut product, &product_snapshot);
    let durable = session_tree_bytes(&sessions);
    assert!(durable.keys().any(|path| path.ends_with("changes.jsonl")));

    for fault in ["output", "rules"] {
        let mut value = serde_json::to_value(&record).unwrap();
        match fault {
            "output" => {
                value["aspects"]["outcome"]["hits"][0]["orb_millidegrees"] =
                    serde_json::json!(99999)
            },
            _ => {
                value["aspects"]["rules"] = serde_json::to_value(AspectRules {
                    conjunction_millidegrees: 0,
                    opposition_millidegrees: 0,
                    trine_millidegrees: 0,
                    square_millidegrees: 0,
                    sextile_millidegrees: 0,
                })
                .unwrap()
            },
        }
        // Forge a valid content seal and matching canonical node identity.
        // Refusal must come from replay, rather than an old id or URL.
        value.as_object_mut().unwrap().remove("id");
        let id = blake3::hash(&serde_json::to_vec(&value).unwrap())
            .to_hex()
            .to_string();
        value["id"] = serde_json::json!(id);
        let bad: cleromancy::natal::NatalReading =
            serde_json::from_value(value).unwrap();
        assert_eq!(
            bad.validate().unwrap_err(),
            "Natal aspects differ from their retained rules and chart",
            "{fault}"
        );
        let url = format!("cleromancy://natal/{}", bad.id);
        let node_id = mere::kernel::graph::Graph::node_namespace_id(&url).to_string();
        let before = graph(&mut other);
        let result = other
            .invoke(apply(
                &before,
                vec![
                    pandect::CapturedDelta::ReplayAddNodeWithIdIfMissing {
                        id: node_id.clone(),
                        url,
                        position: [0.0, 0.0],
                    },
                    pandect::CapturedDelta::ReplaySetNodeFacetById {
                        node_id,
                        facet: cleromancy::natal::NATAL_FACET.into(),
                        value_json: serde_json::to_string(&bad).unwrap(),
                    },
                ],
            ))
            .unwrap();
        assert!(
            matches!(&result, IntentResult::Rejected { reason }
                if reason.contains("Natal aspects differ from their retained rules and chart")),
            "{fault}: {result:?}"
        );
        for (app, expected) in [(&mut product, &product_current), (&mut other, &current)] {
            let after = graph(app);
            assert_eq!(after.scene.revision, before.scene.revision, "{fault}");
            assert_eq!(
                serde_json::to_value(disclosed_graph(app, &after)).unwrap(),
                serde_json::to_value(expected).unwrap(),
                "{fault}"
            );
        }
        assert_eq!(
            session_tree_bytes(&sessions),
            durable,
            "{fault}: durable session changed"
        );
    }
}

