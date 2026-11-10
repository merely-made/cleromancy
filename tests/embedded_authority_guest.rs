// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The seven owner tests of the divination authority, through the wasm guest (cycle breaks
//! P3). They moved here from Mere's `embedded_reservoir_validation`, where the resident
//! linked Cleromancy's validator; the resident now links none, and the owner approves the
//! guest (`tests/common/authority_pack.rs`) into the persona's store before any owner
//! process starts. Real process ownership changes (embedded, daemon, embedded) retain the
//! valid graph; a forbidden complete batch changes no projection, session file or journal.
//! The test harness reads durable bytes as evidence; the attached application uses only
//! admitted carrier requests.

mod common {
    #[allow(dead_code)]
    pub mod authority_pack {
        include!("common/authority_pack.rs");
    }
}
use common::authority_pack::install_divination_guest;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use armillary::tasks::ResidentTasks;
use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, ProjectionSnapshot,
};
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use mere_resident::embedded_reservoir::{
    ApplicationReservoirOptions, ReservoirAttachment, ReservoirAttachmentOptions,
};
use mere_resident::resident_mere::MereRoutes;
use mere_resident::resident_reservoir::ResidentReservoir;
use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use ostiary::app_admission::AppId;
use graphshell_door::projection::AppBrokerClient;
use dramatis::view::VaultProtectionView;
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use mere_overmap::wire::{APPLY_EDITS_INTENT, ApplyEditsV1};
use pandect::{CapturedDelta, DomainId, MereId, mere_dir};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use sceno::InstanceId;
use uuid::Uuid;

const CHILD: &str = "child_serves_the_divination_domain";
const ROOT: &str = "DJINN_V35_ROOT";
const ENDPOINT: &str = "DJINN_V35_ENDPOINT";
const MODE: &str = "DJINN_V35_MODE";
// Each owner process compiles the guest once (about a second optimised); a loaded machine
// running the seven tests together needs more than the 30 s the fixture tests allowed.
const PATIENCE: Duration = Duration::from_secs(120);

struct NoLocalPrompt;
impl dramatis_resident::startup_vault::Prompt for NoLocalPrompt {
    fn ask(
        &mut self,
        _: &dramatis_resident::startup_vault::Ask,
    ) -> Result<dramatis_resident::startup_vault::Answer, String> {
        panic!("live resident attachment must not prompt for local custody");
    }
}

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5633_5635))
}
fn endpoint() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-v35-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return format!("/tmp/dv35-{}.sock", Uuid::new_v4());
}
fn options(root: &Path, address: &str) -> ReservoirAttachmentOptions {
    ReservoirAttachmentOptions {
        endpoint: address.into(),
        application: AppId::new("validator-fixture"),
        declaration: mere_resident::resident_reservoir::mere_client_declaration(),
        grants: Vec::new(),
        shared_root: root.into(),
        persona: persona(),
        domain: DomainId::new("divination").unwrap(),
        session_duration: Duration::from_secs(60),
    }
}
fn identity() -> Arc<dyn dramatis_resident::custody_host::ResidentCustody> {
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("v35-fixture".into()),
                "V3/V5 fixture",
                Ed25519Keypair::from_seed([0x35; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ))
}
struct Process {
    child: Child,
    lines: Receiver<String>,
}
impl Process {
    fn start(root: &Path, address: &str, mode: &str) -> Self {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                CHILD,
                "--ignored",
                "--nocapture",
                "--test-threads",
                "1",
            ])
            .env(ROOT, root)
            .env(ENDPOINT, address)
            .env(MODE, mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let (send, lines) = channel();
        let stdout = child.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = send.send(line);
            }
        });
        let process = Self { child, lines };
        process.wait_for("READY");
        process
    }
    fn wait_for(&self, marker: &str) {
        loop {
            let line = self.lines.recv_timeout(PATIENCE).unwrap();
            if line.contains(marker) {
                println!("{line}");
                break;
            }
        }
    }
    fn stop(mut self) {
        writeln!(self.child.stdin.as_mut().unwrap(), "stop").unwrap();
        self.wait_for("RELEASED");
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn snapshots(client: &mut AppBrokerClient) -> Vec<ProjectionSnapshot> {
    let opened = client.open_session().await.unwrap();
    assert_eq!(
        opened.descriptor.projections.len(),
        4,
        "sessions, graph, V3 archive and notes"
    );
    let mut snapshots = Vec::new();
    for offer in opened.descriptor.projections {
        snapshots.push(client.snapshot(offer.request).await.unwrap());
    }
    snapshots
}
fn node(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.validator.test/"),
        position: [0.0, 0.0],
    }
}
fn title(id: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(id).to_string(),
        title: title.into(),
    }
}
async fn apply(
    client: &mut AppBrokerClient,
    snapshot: &ProjectionSnapshot,
    edits: Vec<CapturedDelta>,
) -> IntentResult {
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
        }))
        .await
        .unwrap();
    let CarrierResponseBody::Intent(result) = response else {
        panic!("{response:?}");
    };
    result
}
fn durable_files_for(root: &Path, domain: &str) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(base: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let domain = DomainId::new(domain).unwrap();
    let sessions = mere_dir(root, persona(), MereId::derive(persona(), &domain)).join("sessions");
    let mut files = BTreeMap::new();
    collect(&sessions, &sessions, &mut files);
    files
}

#[tokio::test(flavor = "multi_thread")]
async fn published_divination_authority_survives_three_real_resident_owner_processes() {
    let root = tempfile::tempdir().unwrap();
    install_divination_guest(root.path(), persona());
    let unusable = root.path().join("no-client-store");
    std::fs::write(&unusable, b"client cannot open a private store").unwrap();
    for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
        let address = endpoint();
        let process = Process::start(root.path(), &address, mode);
        let mut attached = ReservoirAttachment::open_application(
            ApplicationReservoirOptions {
                endpoint: address,
                application: AppId::new("validator-fixture"),
                declaration: mere_resident::resident_reservoir::mere_client_declaration(),
                domain: DomainId::new("divination").unwrap(),
                shared_root: unusable.clone(),
                vault_dir: unusable.clone(),
                profile: None,
                persona: Some(persona()),
                session_duration: Duration::from_secs(60),
            },
            &mut NoLocalPrompt,
            |_| async {
                panic!("client must not compose local authority");
                #[allow(unreachable_code)]
                Ok::<MereRoutes, String>(unreachable!())
            },
        )
        .await
        .unwrap();
        let initial = snapshots(attached.client_mut()).await;
        if phase == 0 {
            assert_eq!(
                apply(attached.client_mut(), &initial[1], vec![node(1)]).await,
                IntentResult::Accepted
            );
        }
        let before = snapshots(attached.client_mut()).await;
        let bytes = durable_files_for(root.path(), "divination");
        let invalid = CapturedDelta::ReplaySetNodeFacetById {
            node_id: Uuid::from_u128(1).to_string(),
            facet: "cleromancy.reading/v999".into(),
            value_json: "{}".into(),
        };
        assert!(
            matches!(apply(attached.client_mut(), &before[1], vec![node(2), invalid]).await, IntentResult::Rejected { reason } if reason.contains("unsupported domain facet"))
        );
        let after = snapshots(attached.client_mut()).await;
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        assert_eq!(durable_files_for(root.path(), "divination"), bytes);
        assert_eq!(after[1].scene.active_items_in_order().len(), 2);
        attached.close().await.unwrap();
        println!(
            "RECEIPT DIVINATION mode={mode} phase={phase} published-product-validator=true complete-batch-refused=true projection-and-durable-bytes-unchanged=true"
        );
        process.stop();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs only as the real process owner launched by the regression"]
async fn child_serves_the_divination_domain() {
    let (Ok(root), Ok(address), Ok(mode)) = (
        std::env::var(ROOT),
        std::env::var(ENDPOINT),
        std::env::var(MODE),
    ) else {
        return;
    };
    let config = options(Path::new(&root), &address);
    let registered = Arc::new(AtomicBool::new(false));
    let mut daemon = None;
    let mut attached = if mode == "embedded" {
        let registered = registered.clone();
        ReservoirAttachment::open_with(
            config.clone(),
            || async { Ok(identity()) },
            move |routes| async move {
                registered.store(true, Ordering::SeqCst);
                Ok(routes.with_resident_defaults().unwrap())
            },
        )
        .await
        .unwrap()
    } else {
        let reservoir = ResidentReservoir::open(&config.shared_root, config.persona)
            .await
            .unwrap();
        let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        mere_resident::applications::install(
            &grants,
            vec![mere_resident::applications::grant_for(&config.application, &config.declaration, None)],
        );
        grants.update(|routes| routes.serve(ResidentReservoir::route()));
        let routes = MereRoutes::new(
            config.shared_root.clone(),
            config.persona,
            catalog.clone(),
            grants.clone(),
        );
        let routes = routes.with_resident_defaults().unwrap();
        registered.store(true, Ordering::SeqCst);
        routes.load_access(&reservoir).await.unwrap();
        for mere in reservoir.meres().await {
            routes.serve(&mere).await.unwrap();
        }
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes)))
            .await
            .unwrap();
        let tasks = ResidentTasks::new();
        let (scope, door, address) = (tasks.clone(), catalog.clone(), address.clone());
        let server = tokio::spawn(async move {
            scope
                .scope(serve_app_broker(
                    &address,
                    std::sync::Arc::new(dramatis_resident::keeper::Keeper::new(identity())),
                    grants,
                    60_000,
                    None,
                    door,
                ))
                .await
        });
        for _ in 0..100 {
            if ostiary::local_endpoint::connect_local(&config.endpoint)
                .await
                .is_ok()
            {
                break;
            }
            assert!(!server.is_finished());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        daemon = Some((server, tasks, catalog, reservoir));
        ReservoirAttachment::open(config.clone(), || async {
            panic!("live daemon must not request identity");
            #[allow(unreachable_code)]
            Ok::<_, String>(identity())
        })
        .await
        .unwrap()
    };
    assert_eq!(attached.is_embedded(), mode == "embedded");
    snapshots(attached.client_mut()).await;
    assert!(registered.load(Ordering::SeqCst));
    println!(
        "READY mode={mode} pid={} guest-installed=true",
        std::process::id()
    );
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    attached.close().await.unwrap();
    if let Some((server, tasks, catalog, reservoir)) = daemon {
        server.abort();
        let _ = server.await;
        assert!(
            tasks.close_and_join().await.is_clean(),
            "task shutdown failed"
        );
        catalog
            .update(|catalog| *catalog = ResidentEndpointCatalog::new())
            .await;
        drop(reservoir);
    }
    ResidentReservoir::open(&config.shared_root, config.persona)
        .await
        .unwrap();
    println!(
        "RELEASED mode={mode} pid={}",
        std::process::id()
    );
}

#[path = "embedded_authority_guest/published_replay_regression.rs"]
mod published_replay_regression;

