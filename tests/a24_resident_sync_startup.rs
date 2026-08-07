// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(all(
    feature = "graphshell-admission",
    feature = "personal-sync",
    not(target_arch = "wasm32")
))]

use cleromancy::{
    CLEROMANCY_SYNC_SETTINGS_SCHEMA, CleromancyHost, CleromancyResidentOpenError,
    CleromancySessionAuthority, CleromancySyncSelection, CleromancySyncSettings, ReadingEngine,
    a0_fixture, sync_settings_path,
};
use muniment::RedbBackend;

#[test]
fn resident_startup_loads_local_sync_consent_before_exposing_readings() {
    let temporary = tempfile::tempdir().unwrap();
    let data_root = temporary.path().join("cleromancy");
    let backend = RedbBackend::open(temporary.path().join("cleromancy.redb")).unwrap();
    let (context, field) = a0_fixture();
    let reading = ReadingEngine::calculate(&context, &field).unwrap();
    let mut host = CleromancyHost::empty(backend.clone());
    host.insert_reading(&context, &field, &reading).unwrap();
    pollster::block_on(host.persist(1)).unwrap();

    let default_authority = pollster::block_on(
        CleromancySessionAuthority::open_with_local_sync_settings(backend.clone(), &data_root),
    )
    .unwrap();
    assert_eq!(
        default_authority.sync_selection(),
        CleromancySyncSelection::Off
    );
    assert!(
        default_authority
            .export_selected_sync_batch()
            .unwrap()
            .is_empty()
    );
    drop(default_authority);

    CleromancySyncSettings {
        schema: CLEROMANCY_SYNC_SETTINGS_SCHEMA.to_string(),
        selection: CleromancySyncSelection::ContextsAndReadings,
    }
    .save(&sync_settings_path(&data_root))
    .unwrap();
    let selected_authority = pollster::block_on(
        CleromancySessionAuthority::open_with_local_sync_settings(backend.clone(), &data_root),
    )
    .unwrap();
    let selected = selected_authority.export_selected_sync_batch().unwrap();
    assert_eq!(
        selected_authority.sync_selection(),
        CleromancySyncSelection::ContextsAndReadings
    );
    assert_eq!(
        (selected.contexts, selected.fields, selected.readings),
        (1, 1, 1)
    );
    drop(selected_authority);

    std::fs::write(
        sync_settings_path(&data_root),
        r#"{"schema":"cleromancy.sync-settings/v0","selection":"off"}"#,
    )
    .unwrap();
    assert!(matches!(
        pollster::block_on(CleromancySessionAuthority::open_with_local_sync_settings(
            backend, &data_root
        )),
        Err(CleromancyResidentOpenError::SyncSettings(_))
    ));
}
