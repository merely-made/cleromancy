// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The stable A0 HTML receipt writer.
//!
//! This binary is intentionally separate from the ordinary native launch. It
//! keeps fixture creation and receipt generation named, repeatable, and out of
//! a person's private local consultation store.

use std::path::PathBuf;

use cleromancy::{CleromancyApp, CleromancyHost, ReadingEngine, a0_fixture};
use muniment::RedbBackend;

fn main() {
    if let Err(error) = run() {
        eprintln!("cleromancy_receipt: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.len() > 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: cleromancy_receipt [output.html]",
        )
        .into());
    }
    let data_root = data_root();
    let store_path = data_root.join("a0-receipt.redb");
    std::fs::create_dir_all(&data_root)?;
    let backend = RedbBackend::open(&store_path)?;
    let mut host = pollster::block_on(CleromancyHost::open(backend))?;
    if host.is_empty() {
        let (context, field) = a0_fixture();
        let calculated = ReadingEngine::calculate(&context, &field)?;
        let cast = ReadingEngine::cast(&context, &field)?;
        host.insert_reading(&context, &field, &calculated)?;
        host.insert_reading(&context, &field, &cast)?;
        let saved_at_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        pollster::block_on(host.persist(saved_at_secs))?;
    }
    let output = arguments
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("receipts/a0.html"));
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut app = CleromancyApp::new(host);
    std::fs::write(&output, app.receipt_html()?)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn data_root() -> PathBuf {
    if let Some(root) = std::env::var_os("CLEROMANCY_ROOT") {
        return PathBuf::from(root);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cleromancy")
}
