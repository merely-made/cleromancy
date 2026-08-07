// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy-owned, device-local consent for personal graph sync.
//!
//! This is deliberately a product setting rather than Graphshell owner
//! configuration. It decides which Cleromancy truth this device may offer to
//! an already-configured personal graph. It neither identifies the graph nor
//! stores a roster, paired peer, key, or endpoint ticket.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CleromancySyncSelection;

pub const CLEROMANCY_SYNC_SETTINGS_SCHEMA: &str = "cleromancy.sync-settings/v1";
pub const CLEROMANCY_SYNC_SETTINGS_FILENAME: &str = "sync-settings.json";

/// The local consent boundary for the Cleromancy personal-sync adapter.
///
/// scope=application; movement=local-only; mutability=live;
/// security=private. The selection is not itself personal graph truth: copying
/// it to another device could silently broaden what that device publishes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CleromancySyncSettings {
    pub schema: String,
    pub selection: CleromancySyncSelection,
}

impl Default for CleromancySyncSettings {
    fn default() -> Self {
        Self {
            schema: CLEROMANCY_SYNC_SETTINGS_SCHEMA.to_string(),
            selection: CleromancySyncSelection::Off,
        }
    }
}

#[derive(Debug, Error)]
pub enum CleromancySyncSettingsError {
    #[error("Cleromancy sync settings at {path}: {message}")]
    File { path: String, message: String },
    #[error("unsupported Cleromancy sync settings schema {schema:?}")]
    Schema { schema: String },
}

impl CleromancySyncSettings {
    /// Load device-local consent. A missing file means the privacy-preserving
    /// default: no Cleromancy truth is selected for sync.
    pub fn load(path: &Path) -> Result<Self, CleromancySyncSettingsError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(file_error(path, error)),
        };
        let settings: Self =
            serde_json::from_str(&text).map_err(|error| CleromancySyncSettingsError::File {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
        settings.validate()?;
        Ok(settings)
    }

    /// Save a complete settings file through a temporary sibling. The caller
    /// owns when a live selection change should become durable.
    pub fn save(&self, path: &Path) -> Result<(), CleromancySyncSettingsError> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| file_error(path, error))?;
        }
        let mut text = serde_json::to_string_pretty(self).map_err(|error| {
            CleromancySyncSettingsError::File {
                path: path.display().to_string(),
                message: error.to_string(),
            }
        })?;
        text.push('\n');
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, text).map_err(|error| file_error(path, error))?;
        if path.exists() {
            std::fs::remove_file(path).map_err(|error| file_error(path, error))?;
        }
        std::fs::rename(&temporary, path).map_err(|error| file_error(path, error))
    }

    pub fn validate(&self) -> Result<(), CleromancySyncSettingsError> {
        if self.schema != CLEROMANCY_SYNC_SETTINGS_SCHEMA {
            return Err(CleromancySyncSettingsError::Schema {
                schema: self.schema.clone(),
            });
        }
        Ok(())
    }
}

/// Product-owned path below an already-chosen Cleromancy data root.
pub fn sync_settings_path(root: &Path) -> PathBuf {
    root.join(CLEROMANCY_SYNC_SETTINGS_FILENAME)
}

fn file_error(path: &Path, error: std::io::Error) -> CleromancySyncSettingsError {
    CleromancySyncSettingsError::File {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}
