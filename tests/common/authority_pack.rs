// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

// The divination authority guest as a reviewed, installed component: build it, sign it
// into the pack the owner reviews, and approve it into a persona's guest store. Shared by
// the conformance tests and `examples/build_authority_pack.rs`.
//
// The author's key is the author's to hold. The tests sign with a fixed public seed; the
// example takes the seed from a file and never generates or stores one.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use eidetic::pack::{PackManifest, PackPart, PackPartRole, sign_pack};
use eidetic::{ManifestId, TrustEnvelope};
use mere_resident::authority_guest::{AuthorityReview, GuestStore, POWER, Pin};
use ostiary::app_admission::AppId;
use personae::{IdentityProvider, InMemoryProvider, PersonaId};

/// The seed the tests sign with: a public, fixed test key, never a real author's.
pub const TEST_SEED: [u8; 32] = [0xc1; 32];

pub fn guest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("authority-guest")
}

/// Build the guest (`cargo build --locked --release --target wasm32-wasip2` in its own
/// lock root and toolchain) once per process and return the component's path.
pub fn build_guest() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let dir = guest_dir();
            let mut cmd = Command::new("cargo");
            cmd.current_dir(&dir)
                .args(["build", "--locked", "--release", "--target", "wasm32-wasip2"]);
            for (key, _) in std::env::vars() {
                if key == "CARGO_HOME" || key == "CARGO_BUILD_JOBS" {
                    continue;
                }
                if key.starts_with("CARGO")
                    || key.starts_with("RUSTC")
                    || key.starts_with("RUSTUP")
                    || key == "RUSTFLAGS"
                    || key == "RUSTDOCFLAGS"
                    || key == "TARGET"
                {
                    cmd.env_remove(key);
                }
            }
            let status = cmd.status().expect("run cargo for the authority guest");
            assert!(
                status.success(),
                "the authority guest did not build; run by hand: cd authority-guest && cargo build --locked --release --target wasm32-wasip2"
            );
            let wasm = dir.join("target/wasm32-wasip2/release/cleromancy_authority_guest.wasm");
            assert!(wasm.exists(), "{} is missing", wasm.display());
            wasm
        })
        .clone()
}

/// The signed pack for `component`: its bytes and its address. It asks for the one power
/// an authority pack may ask for.
pub fn sign(component: &[u8], seed: [u8; 32], version: &str) -> (Vec<u8>, String) {
    let author = InMemoryProvider::from_seed(seed);
    let manifest = PackManifest {
        name: "cleromancy-divination-authority".into(),
        version: version.into(),
        author: author
            .master_public_key()
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        requested_scopes: vec![POWER.into()],
        parts: vec![PackPart {
            name: "cleromancy-divination-authority.wasm".into(),
            role: PackPartRole::WasmComponent,
            blob: ManifestId::of_blob(component),
            bytes: component.len() as u64,
        }],
    };
    let mut trust = TrustEnvelope::self_asserted();
    trust.signatures.push(sign_pack(&manifest, author.master_keypair()));
    let bytes = serde_json::to_vec(&serde_json::json!({ "manifest": manifest, "trust": trust })).expect("encode the pack");
    let hash = ManifestId::of_blob(&bytes).0.to_hex();
    (bytes, hash)
}

/// The owner's review and approval of the divination guest into `persona`'s store under
/// `shared_root`, signed with the test seed.
pub fn install_divination_guest(shared_root: &Path, persona: PersonaId) -> Pin {
    let component = std::fs::read(build_guest()).expect("read the guest component");
    let (pack, hash) = sign(&component, TEST_SEED, "test");
    let review = AuthorityReview::prepare(&pack, &hash, &component).expect("the guest verifies");
    assert_eq!(review.domain, "divination");
    review
        .approve(&GuestStore::at(shared_root, persona), AppId::new("cleromancy"), false)
        .expect("the owner approves the guest")
}
