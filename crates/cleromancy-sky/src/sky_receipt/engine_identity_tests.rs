// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn engine_identity_preserves_historical_pairs_and_rejects_relabelling() {
    let current = SkyEngineV1::turquet();
    assert_eq!(current.version, "0.17.0");
    assert_eq!(current.source_revision, TURQUET_SOURCE_REVISION);
    current.validate().unwrap();
    let historical = SkyEngineV1 {
        package: TURQUET_PACKAGE.to_owned(),
        version: HISTORICAL_TURQUET_VERSION.to_owned(),
        source_revision: HISTORICAL_TURQUET_SOURCE_REVISION.to_owned(),
    };
    historical.validate().unwrap();
    let mut mixed = historical;
    mixed.version = current.version;
    assert!(mixed.validate().is_err());
    mixed.source_revision = TURQUET_SOURCE_REVISION.to_owned();
    mixed.package = "another-engine".to_owned();
    assert!(mixed.validate().is_err());
}
