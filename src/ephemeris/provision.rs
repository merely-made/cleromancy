// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit, checksum-bound acquisition of the DE440s kernel.

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};
use thiserror::Error;

use super::DE440S_SHA256;

pub const DE440S_DOWNLOAD_URL: &str =
    "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp";
pub const DE440S_BYTES: u64 = 32_726_016;

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EphemerisStatus {
    Missing,
    Ready { path: PathBuf },
    Invalid { path: PathBuf, detail: String },
}

impl EphemerisStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EphemerisInstall {
    pub path: PathBuf,
    pub downloaded: bool,
    pub quarantined: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum EphemerisProvisionError {
    #[error("could not prepare ephemeris storage at {path}: {source}")]
    Storage {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not download DE440s from {url}: {source}")]
    Download {
        url: &'static str,
        #[source]
        source: ureq::Error,
    },
    #[error("DE440s download was {actual} bytes; expected {expected}")]
    Size { expected: u64, actual: u64 },
    #[error("DE440s digest mismatch: expected {expected}, got {actual}")]
    Digest {
        expected: &'static str,
        actual: String,
    },
}

/// Owns the local path and the only network source accepted by Cleromancy.
/// Acquisition occurs only after an explicit product action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EphemerisProvisioner {
    kernel_path: PathBuf,
}

impl EphemerisProvisioner {
    pub fn for_data_root(data_root: impl AsRef<Path>) -> Self {
        Self {
            kernel_path: data_root.as_ref().join("ephemeris").join("de440s.bsp"),
        }
    }

    pub fn at_path(kernel_path: impl Into<PathBuf>) -> Self {
        Self {
            kernel_path: kernel_path.into(),
        }
    }

    pub fn kernel_path(&self) -> &Path {
        &self.kernel_path
    }

    pub fn status(&self) -> EphemerisStatus {
        if !self.kernel_path.exists() {
            return EphemerisStatus::Missing;
        }
        match verified_digest(&self.kernel_path) {
            Ok(()) => EphemerisStatus::Ready {
                path: self.kernel_path.clone(),
            },
            Err(error) => EphemerisStatus::Invalid {
                path: self.kernel_path.clone(),
                detail: error.to_string(),
            },
        }
    }

    pub fn download(&self) -> Result<EphemerisInstall, EphemerisProvisionError> {
        if self.status().is_ready() {
            return Ok(EphemerisInstall {
                path: self.kernel_path.clone(),
                downloaded: false,
                quarantined: None,
            });
        }
        let config = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(Duration::from_secs(120)))
            .build();
        let agent = ureq::Agent::new_with_config(config);
        let response = agent.get(DE440S_DOWNLOAD_URL).call().map_err(|source| {
            EphemerisProvisionError::Download {
                url: DE440S_DOWNLOAD_URL,
                source,
            }
        })?;
        let (_, body) = response.into_parts();
        self.install_from_reader(body.into_reader())
    }

    pub fn install_from_reader(
        &self,
        reader: impl Read,
    ) -> Result<EphemerisInstall, EphemerisProvisionError> {
        let directory = self.kernel_path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(directory).map_err(|source| EphemerisProvisionError::Storage {
            path: directory.to_path_buf(),
            source,
        })?;
        let temporary = adjacent_path(&self.kernel_path, "download");
        let result = self.write_verified(reader, &temporary);
        if let Err(error) = result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }

        let quarantined = if self.kernel_path.exists() {
            let quarantined = adjacent_path(&self.kernel_path, "rejected");
            fs::rename(&self.kernel_path, &quarantined).map_err(|source| {
                let _ = fs::remove_file(&temporary);
                EphemerisProvisionError::Storage {
                    path: self.kernel_path.clone(),
                    source,
                }
            })?;
            Some(quarantined)
        } else {
            None
        };

        if let Err(source) = fs::rename(&temporary, &self.kernel_path) {
            if let Some(quarantined) = &quarantined {
                let _ = fs::rename(quarantined, &self.kernel_path);
            }
            let _ = fs::remove_file(&temporary);
            return Err(EphemerisProvisionError::Storage {
                path: self.kernel_path.clone(),
                source,
            });
        }
        Ok(EphemerisInstall {
            path: self.kernel_path.clone(),
            downloaded: true,
            quarantined,
        })
    }

    fn write_verified(
        &self,
        reader: impl Read,
        temporary: &Path,
    ) -> Result<(), EphemerisProvisionError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temporary)
            .map_err(|source| EphemerisProvisionError::Storage {
                path: temporary.to_path_buf(),
                source,
            })?;
        let mut reader = BufReader::new(reader).take(DE440S_BYTES + 1);
        let mut writer = BufWriter::new(file);
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1_024];
        let mut bytes = 0_u64;
        loop {
            let count =
                reader
                    .read(&mut buffer)
                    .map_err(|source| EphemerisProvisionError::Storage {
                        path: temporary.to_path_buf(),
                        source,
                    })?;
            if count == 0 {
                break;
            }
            bytes += count as u64;
            digest.update(&buffer[..count]);
            writer.write_all(&buffer[..count]).map_err(|source| {
                EphemerisProvisionError::Storage {
                    path: temporary.to_path_buf(),
                    source,
                }
            })?;
        }
        if bytes != DE440S_BYTES {
            return Err(EphemerisProvisionError::Size {
                expected: DE440S_BYTES,
                actual: bytes,
            });
        }
        let actual = format!("{:x}", digest.finalize());
        if actual != DE440S_SHA256 {
            return Err(EphemerisProvisionError::Digest {
                expected: DE440S_SHA256,
                actual,
            });
        }
        writer
            .flush()
            .map_err(|source| EphemerisProvisionError::Storage {
                path: temporary.to_path_buf(),
                source,
            })?;
        writer
            .into_inner()
            .map_err(|error| EphemerisProvisionError::Storage {
                path: temporary.to_path_buf(),
                source: error.into_error(),
            })?
            .sync_all()
            .map_err(|source| EphemerisProvisionError::Storage {
                path: temporary.to_path_buf(),
                source,
            })
    }
}

fn verified_digest(path: &Path) -> Result<(), EphemerisProvisionError> {
    let metadata = fs::metadata(path).map_err(|source| EphemerisProvisionError::Storage {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() != DE440S_BYTES {
        return Err(EphemerisProvisionError::Size {
            expected: DE440S_BYTES,
            actual: metadata.len(),
        });
    }
    let file = File::open(path).map_err(|source| EphemerisProvisionError::Storage {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1_024];
    loop {
        let count =
            reader
                .read(&mut buffer)
                .map_err(|source| EphemerisProvisionError::Storage {
                    path: path.to_path_buf(),
                    source,
                })?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let actual = format!("{:x}", digest.finalize());
    if actual != DE440S_SHA256 {
        return Err(EphemerisProvisionError::Digest {
            expected: DE440S_SHA256,
            actual,
        });
    }
    Ok(())
}

fn adjacent_path(destination: &Path, role: &str) -> PathBuf {
    let serial = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("de440s.bsp");
    destination.with_file_name(format!(".{name}.{role}.{}.{}", std::process::id(), serial))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_status_does_not_create_storage() {
        let temp = tempfile::tempdir().unwrap();
        let provisioner = EphemerisProvisioner::for_data_root(temp.path());
        assert_eq!(provisioner.status(), EphemerisStatus::Missing);
        assert!(!provisioner.kernel_path().exists());
    }

    #[test]
    fn undersized_input_is_refused_and_temporary_file_is_removed() {
        let temp = tempfile::tempdir().unwrap();
        let provisioner = EphemerisProvisioner::for_data_root(temp.path());
        assert!(matches!(
            provisioner.install_from_reader(&b"not a kernel"[..]),
            Err(EphemerisProvisionError::Size { .. })
        ));
        assert_eq!(provisioner.status(), EphemerisStatus::Missing);
        assert_eq!(
            fs::read_dir(temp.path().join("ephemeris")).unwrap().count(),
            0
        );
    }

    #[test]
    #[ignore = "requires CLEROMANCY_DE440S to name the canonical 31 MiB kernel"]
    fn canonical_kernel_installs_and_reinstallation_is_a_no_op() {
        let source = PathBuf::from(std::env::var_os("CLEROMANCY_DE440S").unwrap());
        let temp = tempfile::tempdir().unwrap();
        let provisioner = EphemerisProvisioner::for_data_root(temp.path());
        let first = provisioner
            .install_from_reader(File::open(source).unwrap())
            .unwrap();
        assert!(first.downloaded);
        assert!(provisioner.status().is_ready());
        let second = provisioner.download().unwrap();
        assert!(!second.downloaded);
    }

    #[test]
    #[ignore = "downloads the canonical 31 MiB kernel from NASA/NAIF"]
    fn canonical_kernel_download_is_verified_before_installation() {
        let temp = tempfile::tempdir().unwrap();
        let provisioner = EphemerisProvisioner::for_data_root(temp.path());
        let install = provisioner.download().unwrap();
        assert!(install.downloaded);
        assert_eq!(install.path, provisioner.kernel_path());
        assert!(install.quarantined.is_none());
        assert!(provisioner.status().is_ready());
    }
}
