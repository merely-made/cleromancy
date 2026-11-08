// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sign the divination authority guest into the pack the owner reviews.
//!
//! `build_authority_pack --seed-file FILE --out PACK.json [--component FILE.wasm]` where the
//! seed file holds the 32-byte author seed as 64 hex digits. The author's key is the
//! author's to hold; this tool never generates or stores one. Without `--component` it builds
//! `authority-guest/` first. The pack asks for `power:domain-authority` and nothing else.
//! Install it with Mere's `authority_guest::AuthorityReview::prepare(..).approve(..)`.

#[allow(dead_code)]
#[path = "../tests/common/authority_pack.rs"]
mod authority_pack;

fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let seed_file = arg("--seed-file").expect("usage: build_authority_pack --seed-file F --out F [--component F]");
    let out = arg("--out").expect("--out");
    let component_path = arg("--component")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(authority_pack::build_guest);
    let component = std::fs::read(&component_path).expect("read the component");
    let hex = std::fs::read_to_string(seed_file).expect("read the seed file");
    let hex = hex.trim();
    assert_eq!(hex.len(), 64, "the seed is 32 bytes as 64 hex digits");
    let mut seed = [0u8; 32];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).expect("hex seed");
    }
    let (pack, hash) = authority_pack::sign(&component, seed, env!("CARGO_PKG_VERSION"));
    std::fs::write(&out, &pack).expect("write the pack");
    println!(
        "pack {} bytes at {out}, address {hash}; component {} bytes",
        pack.len(),
        component.len()
    );
}
