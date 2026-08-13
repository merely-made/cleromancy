// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sealed readings: selection modes, receipts, and the reading engine.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::enrichment::SealedEnrichment;

mod engine;

pub use engine::ReadingEngine;

pub const EXTERNAL_QUALIFICATION_ALGORITHM: &str =
    "cleromancy.qualification/external-term-share/v1";
pub const DERIVED_SELECTION_SCHEMA: &str = "cleromancy.derived-selection/v1";
pub const DERIVED_SELECTION_ALGORITHM: &str =
    "cleromancy.derived-selection/blake3-u64-rejection/v1";

const MAX_DERIVATION_SEED_BYTES: usize = 4 * 1024;
const MAX_DERIVATION_DOMAIN_BYTES: usize = 256;

/// Whether a reading follows the declared qualifier to its maximum, casts
/// within the same qualified field, or derives a public replayable selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    Calculated,
    Cast,
    Derived,
}

/// Public inputs to a deterministic selection. They are disclosed in the
/// receipt so another host can reproduce the same bounded sample.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedSelection {
    pub schema: String,
    pub algorithm: String,
    pub seed: String,
    pub domain: String,
}

impl DerivedSelection {
    pub fn new(seed: impl Into<String>, domain: impl Into<String>) -> Result<Self, ReadingError> {
        let selection = Self {
            schema: DERIVED_SELECTION_SCHEMA.to_string(),
            algorithm: DERIVED_SELECTION_ALGORITHM.to_string(),
            seed: seed.into(),
            domain: domain.into(),
        };
        selection.validate()?;
        Ok(selection)
    }

    fn validate(&self) -> Result<(), ReadingError> {
        if self.schema != DERIVED_SELECTION_SCHEMA {
            return Err(ReadingError::InvalidDerivation(
                "selection schema does not match the contract".to_string(),
            ));
        }
        if self.algorithm != DERIVED_SELECTION_ALGORITHM {
            return Err(ReadingError::InvalidDerivation(
                "selection algorithm does not match the contract".to_string(),
            ));
        }
        if self.seed.trim().is_empty() || self.seed.len() > MAX_DERIVATION_SEED_BYTES {
            return Err(ReadingError::InvalidDerivation(
                "seed must contain 1 to 4096 bytes".to_string(),
            ));
        }
        if self.domain.trim().is_empty() || self.domain.len() > MAX_DERIVATION_DOMAIN_BYTES {
            return Err(ReadingError::InvalidDerivation(
                "domain must contain 1 to 256 bytes".to_string(),
            ));
        }
        Ok(())
    }
}

/// The sealed source snapshot and every derived addition used to qualify the
/// candidate field. Replay recomputes these fields from `evidence`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrichmentQualification {
    pub schema: String,
    pub algorithm: String,
    pub evidence: SealedEnrichment,
    pub report_digest: String,
    pub candidate_terms: Vec<Vec<String>>,
    pub weight_additions: Vec<u64>,
}

/// The complete, replayable calculation disclosed beside a reading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema: String,
    pub mode: SelectionMode,
    pub algorithm: String,
    pub context_digest: String,
    pub field_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<EnrichmentQualification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation: Option<DerivedSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation_digest: Option<String>,
    pub qualified_weights: Vec<u64>,
    pub total_weight: u64,
    pub sample: Option<u64>,
    pub event_nonce: Option<String>,
    pub selected_index: usize,
    pub selected_candidate: String,
}

/// A sealed result plus the evidence needed to audit it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    pub schema: String,
    pub id: String,
    pub system: String,
    pub candidate_id: String,
    pub title: String,
    pub interpretation: String,
    pub receipt: Receipt,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReadingError {
    #[error("the system has no candidates")]
    EmptyField,
    #[error("qualified weight must be nonzero")]
    EmptyWeight,
    #[error("candidate id is empty or duplicated: {0}")]
    InvalidCandidate(String),
    #[error("qualification rule is unsupported: {0}")]
    UnsupportedRule(String),
    #[error("qualification rule requires sealed evidence: {0}")]
    QualificationEvidenceRequired(String),
    #[error("uniform rule requires candidate {candidate} to have base weight 1, found {weight}")]
    NonUniformCandidate { candidate: String, weight: u64 },
    #[error("qualification rule requires cast selection: {0}")]
    QualificationRequiresCast(String),
    #[error("qualified weight overflowed u64")]
    WeightOverflow,
    #[error("operating-system entropy failed: {0}")]
    Entropy(String),
    #[error("sample {sample} is outside 0..{upper}")]
    InvalidSample { sample: u64, upper: u64 },
    #[error("receipt does not match its declared inputs: {0}")]
    ReceiptMismatch(String),
    #[error("sealed enrichment does not match its declared inputs: {0}")]
    InvalidEnrichment(String),
    #[error("derived selection is invalid: {0}")]
    InvalidDerivation(String),
    #[error("derived selection did not yield a bounded sample")]
    DerivedSelectionExhausted,
}
