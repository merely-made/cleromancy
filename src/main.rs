// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use cleromancy::{CleromancyApp, CleromancyHost, ReadingEngine, a0_fixture};
#[cfg(feature = "personal-sync")]
use cleromancy::{CleromancySyncSelection, CleromancySyncSettings, sync_settings_path};
use muniment::RedbBackend;

fn main() {
    if let Err(error) = run() {
        eprintln!("cleromancy: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments
        .first()
        .is_some_and(|argument| argument == "sync-consent")
    {
        return sync_consent(&arguments);
    }

    let data_root = data_root();
    let store_path = store_path(&data_root);
    if let Some(parent) = store_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
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

#[cfg(feature = "personal-sync")]
fn sync_consent(arguments: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let data_root = data_root();
    let path = sync_settings_path(&data_root);
    match arguments.get(1).and_then(|argument| argument.to_str()) {
        None | Some("show") if arguments.len() <= 2 => {
            println!(
                "sync consent: {}",
                CleromancySyncSettings::load(&path)?.selection
            );
            Ok(())
        }
        Some("set") if arguments.len() == 3 => {
            let selection = arguments[2]
                .to_str()
                .ok_or_else(|| invalid_input("sync consent selection must be valid Unicode"))?
                .parse::<CleromancySyncSelection>()?;
            let mut settings = CleromancySyncSettings::load(&path)?;
            settings.selection = selection;
            settings.save(&path)?;
            println!("sync consent: {selection}");
            Ok(())
        }
        _ => Err(consent_usage()),
    }
}

#[cfg(not(feature = "personal-sync"))]
fn sync_consent(_arguments: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    Err(invalid_input(
        "sync-consent requires the personal-sync feature; rebuild with --features personal-sync",
    ))
}

#[cfg(feature = "personal-sync")]
fn consent_usage() -> Box<dyn std::error::Error> {
    invalid_input(
        "usage: cleromancy sync-consent [show | set off|contexts|contexts-and-readings|contexts-readings-and-reflections]",
    )
}

fn invalid_input(message: &str) -> Box<dyn std::error::Error> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into()
}

fn data_root() -> PathBuf {
    if let Some(root) = std::env::var_os("CLEROMANCY_ROOT") {
        return PathBuf::from(root);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cleromancy")
}

fn store_path(data_root: &Path) -> PathBuf {
    data_root.join("cleromancy.redb")
}
