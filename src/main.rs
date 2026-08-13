// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The ordinary, private local Cleromancy window.

use std::ffi::OsString;
use std::path::PathBuf;

#[cfg(feature = "personal-sync")]
use cleromancy::{CleromancySyncSelection, CleromancySyncSettings, sync_settings_path};

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
    if !arguments.is_empty() {
        return Err(invalid_input(
            "usage: cleromancy [sync-consent ...]; use cleromancy_receipt for the A0 HTML receipt",
        ));
    }
    cleromancy::ui::native::run(&data_root()).map_err(|message| std::io::Error::other(message))?;
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
