// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Headed, semantic consultation receipts.
//!
//! `CLEROMANCY_SCENARIO` names a genet-probe scenario. Its companion phase is
//! deliberately explicit: `first` authors a consultation and reflection;
//! `reopen` selects the same durable session from a fresh process. Captures are
//! composed from the presented Genet scene, not from an occluded desktop.

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use genet_probe::{Outcome, Scenario};

/// Which half of the close/reopen receipt a process is executing.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase {
    First,
    Reopen,
}

impl Phase {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "first" => Ok(Self::First),
            "reopen" => Ok(Self::Reopen),
            _ => Err(format!(
                "CLEROMANCY_SCENARIO_PHASE must be first or reopen, got {value:?}"
            )),
        }
    }
}

/// A loaded scenario plus its receipt destination and required phase.
pub(crate) struct Run {
    pub(crate) scenario: Scenario,
    pub(crate) dir: PathBuf,
    pub(crate) phase: Phase,
}

/// Load a self-drive scenario, or return `None` for a normal interactive run.
pub(crate) fn load() -> Option<Run> {
    let path = PathBuf::from(std::env::var_os("CLEROMANCY_SCENARIO")?);
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read scenario {path:?}: {error}"));
    let scenario =
        Scenario::parse(&body).unwrap_or_else(|error| panic!("parse scenario {path:?}: {error}"));
    let phase = match std::env::var("CLEROMANCY_SCENARIO_PHASE") {
        Ok(value) => {
            Phase::parse(&value).unwrap_or_else(|error| panic!("load Cleromancy scenario: {error}"))
        }
        Err(_) => panic!("load Cleromancy scenario: CLEROMANCY_SCENARIO_PHASE is required"),
    };
    let dir = std::env::var_os("CLEROMANCY_CAPTURE_DIR")
        .map(PathBuf::from)
        .or_else(|| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("create scenario receipt directory {dir:?}: {error}"));
    Some(Run {
        scenario,
        dir,
        phase,
    })
}

/// The durable identities the relaunch harness compares without scraping text.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReportedIds {
    pub(crate) session_id: String,
    pub(crate) reading_id: String,
    pub(crate) reflection_id: String,
}

/// App state observed when a headed scenario completes.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Observation {
    pub(crate) status: String,
    pub(crate) catalog_ready: bool,
    pub(crate) sessions: usize,
    pub(crate) readings: usize,
    pub(crate) reflections: usize,
    pub(crate) ids: Option<ReportedIds>,
}

#[derive(Serialize)]
struct Receipt {
    schema: &'static str,
    phase: Phase,
    ok: bool,
    status: String,
    catalog_ready: bool,
    sessions: usize,
    readings: usize,
    reflections: usize,
    ids: Option<ReportedIds>,
    log: Vec<String>,
}

/// Write both a concise greppable result and a typed JSON observation.
pub(crate) fn write_done(
    dir: &Path,
    phase: Phase,
    outcome: &Outcome,
    observation: Observation,
    capture_error: Option<&str>,
) {
    let mut ok = outcome.ok && observation.catalog_ready && observation.ids.is_some();
    let mut log = outcome.log.clone();
    if !observation.catalog_ready {
        log.push("FAIL: local catalog did not become ready".to_string());
    }
    if observation.ids.is_none() {
        log.push(
            "FAIL: scenario completed without durable session, reading, and reflection ids"
                .to_string(),
        );
    }
    if let Some(error) = capture_error {
        ok = false;
        log.push(format!("FAIL: scenario capture: {error}"));
    }

    let mut done = format!("RESULT {}\n", if ok { "ok" } else { "fail" });
    for line in &log {
        done.push_str(line);
        done.push('\n');
    }
    std::fs::write(dir.join("scenario.done"), done)
        .unwrap_or_else(|error| panic!("write scenario.done in {dir:?}: {error}"));

    let receipt = Receipt {
        schema: "cleromancy.headed-scenario/v1",
        phase,
        ok,
        status: observation.status,
        catalog_ready: observation.catalog_ready,
        sessions: observation.sessions,
        readings: observation.readings,
        reflections: observation.reflections,
        ids: observation.ids,
        log,
    };
    let json = serde_json::to_vec_pretty(&receipt).expect("serialize headed scenario receipt");
    std::fs::write(dir.join("receipt.json"), json)
        .unwrap_or_else(|error| panic!("write receipt.json in {dir:?}: {error}"));
}
