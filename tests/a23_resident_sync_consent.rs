// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(
    feature = "graphshell-admission",
    feature = "personal-sync",
    not(target_arch = "wasm32")
))]

use cleromancy::{
    CLEROMANCY_SYNC_SETTINGS_SCHEMA, CleromancyApp, CleromancyHost, CleromancySessionAuthority,
    CleromancySyncSelection, CleromancySyncSettings, CleromancySyncSettingsError, ReadingEngine,
    a0_fixture, sync_settings_path,
};
use muniment::MemoryBackend;

#[test]
fn persisted_local_consent_controls_the_live_resident_sync_selection() {
    let temporary = tempfile::tempdir().unwrap();
    let path = sync_settings_path(temporary.path());

    let default_settings = CleromancySyncSettings::load(&path).unwrap();
    assert_eq!(default_settings.selection, CleromancySyncSelection::Off);

    let (context, field) = a0_fixture();
    let reading = ReadingEngine::calculate(&context, &field).unwrap();
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    host.insert_reading(&context, &field, &reading).unwrap();
    let authority = CleromancySessionAuthority::new(CleromancyApp::new(host));

    authority.apply_sync_settings(&default_settings).unwrap();
    assert_eq!(authority.sync_selection(), CleromancySyncSelection::Off);
    assert!(authority.export_selected_sync_batch().unwrap().is_empty());

    let selected = CleromancySyncSettings {
        schema: CLEROMANCY_SYNC_SETTINGS_SCHEMA.to_string(),
        selection: CleromancySyncSelection::ContextsAndReadings,
    };
    selected.save(&path).unwrap();
    let reloaded = CleromancySyncSettings::load(&path).unwrap();
    assert_eq!(reloaded, selected);
    authority.apply_sync_settings(&reloaded).unwrap();
    let selected_batch = authority.export_selected_sync_batch().unwrap();
    assert_eq!(
        (
            selected_batch.contexts,
            selected_batch.fields,
            selected_batch.readings,
        ),
        (1, 1, 1)
    );

    let invalid = CleromancySyncSettings {
        schema: "cleromancy.sync-settings/v0".to_string(),
        selection: CleromancySyncSelection::Off,
    };
    assert!(matches!(
        authority.apply_sync_settings(&invalid),
        Err(CleromancySyncSettingsError::Schema { .. })
    ));
    assert_eq!(
        authority.sync_selection(),
        CleromancySyncSelection::ContextsAndReadings,
        "an invalid configuration cannot silently revoke or broaden live consent"
    );

    CleromancySyncSettings::default().save(&path).unwrap();
    authority
        .apply_sync_settings(&CleromancySyncSettings::load(&path).unwrap())
        .unwrap();
    assert_eq!(authority.sync_selection(), CleromancySyncSelection::Off);
    assert!(authority.export_selected_sync_batch().unwrap().is_empty());
}
