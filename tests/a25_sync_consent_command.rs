// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]

use std::process::{Command, Output};

use cleromancy::{CleromancySyncSelection, CleromancySyncSettings, sync_settings_path};

fn command(root: &std::path::Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cleromancy"))
        .env("CLEROMANCY_ROOT", root)
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn sync_consent_command_edits_only_the_local_consent_file() {
    let temporary = tempfile::tempdir().unwrap();

    let shown = command(temporary.path(), &["sync-consent", "show"]);
    assert!(shown.status.success());
    assert_eq!(
        String::from_utf8(shown.stdout).unwrap().trim(),
        "sync consent: off"
    );
    assert!(!sync_settings_path(temporary.path()).exists());
    assert!(!temporary.path().join("cleromancy.redb").exists());

    let selected = command(
        temporary.path(),
        &["sync-consent", "set", "contexts-and-readings"],
    );
    assert!(selected.status.success());
    assert_eq!(
        String::from_utf8(selected.stdout).unwrap().trim(),
        "sync consent: contexts-and-readings"
    );
    assert_eq!(
        CleromancySyncSettings::load(&sync_settings_path(temporary.path()))
            .unwrap()
            .selection,
        CleromancySyncSelection::ContextsAndReadings
    );
    assert!(!temporary.path().join("cleromancy.redb").exists());

    let invalid = command(temporary.path(), &["sync-consent", "set", "everything"]);
    assert!(!invalid.status.success());
    assert!(
        String::from_utf8(invalid.stderr)
            .unwrap()
            .contains("unknown Cleromancy sync selection")
    );
    assert_eq!(
        CleromancySyncSettings::load(&sync_settings_path(temporary.path()))
            .unwrap()
            .selection,
        CleromancySyncSelection::ContextsAndReadings
    );
}
