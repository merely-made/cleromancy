// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::context::ContextSnapshot;
use crate::enrichment::SealedEnrichment;
use crate::field::{
    CONTEXTUAL_WEIGHT_RULE, EXTERNAL_TERM_WEIGHT_RULE, Field, UNIFORM_DIE_RULE, UNIFORM_RULE,
};
use crate::moirai::{atropos, clotho, lachesis};

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

#[derive(Clone, Copy, Debug, Default)]
pub struct ReadingEngine;

impl ReadingEngine {
    pub fn calculate(context: &ContextSnapshot, field: &Field) -> Result<Reading, ReadingError> {
        let qualified = lachesis::qualify(context, field)?;
        require_calculable(field)?;
        let index = lachesis::calculated_index(&qualified)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::calculated(),
                None,
            ),
        ))
    }

    pub fn calculate_enriched(
        context: &ContextSnapshot,
        field: &Field,
        evidence: &SealedEnrichment,
    ) -> Result<Reading, ReadingError> {
        let (qualified, enrichment) = prepare_enrichment(context, field, evidence)?;
        let index = lachesis::calculated_index(&qualified)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::calculated(),
                Some(enrichment),
            ),
        ))
    }

    pub fn cast(context: &ContextSnapshot, field: &Field) -> Result<Reading, ReadingError> {
        Self::cast_with(context, field, &mut clotho::OsEntropy)
    }

    pub fn cast_with(
        context: &ContextSnapshot,
        field: &Field,
        entropy: &mut impl clotho::EntropySource,
    ) -> Result<Reading, ReadingError> {
        let qualified = lachesis::qualify(context, field)?;
        let draw = clotho::draw_below(entropy, qualified.total)?;
        let index = lachesis::index_for_sample(&qualified, draw.sample)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::cast(draw.sample, draw.event_nonce),
                None,
            ),
        ))
    }

    pub fn cast_enriched(
        context: &ContextSnapshot,
        field: &Field,
        evidence: &SealedEnrichment,
    ) -> Result<Reading, ReadingError> {
        Self::cast_enriched_with(context, field, evidence, &mut clotho::OsEntropy)
    }

    pub fn cast_enriched_with(
        context: &ContextSnapshot,
        field: &Field,
        evidence: &SealedEnrichment,
        entropy: &mut impl clotho::EntropySource,
    ) -> Result<Reading, ReadingError> {
        let (qualified, enrichment) = prepare_enrichment(context, field, evidence)?;
        let draw = clotho::draw_below(entropy, qualified.total)?;
        let index = lachesis::index_for_sample(&qualified, draw.sample)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::cast(draw.sample, draw.event_nonce),
                Some(enrichment),
            ),
        ))
    }

    pub fn derive(
        context: &ContextSnapshot,
        field: &Field,
        selection: &DerivedSelection,
    ) -> Result<Reading, ReadingError> {
        selection.validate()?;
        let qualified = lachesis::qualify(context, field)?;
        let draw = derived_draw(context, field, &qualified, selection)?;
        let index = lachesis::index_for_sample(&qualified, draw.sample)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::derived(draw.sample, draw.digest, selection.clone()),
                None,
            ),
        ))
    }

    pub fn derive_enriched(
        context: &ContextSnapshot,
        field: &Field,
        evidence: &SealedEnrichment,
        selection: &DerivedSelection,
    ) -> Result<Reading, ReadingError> {
        selection.validate()?;
        let (qualified, enrichment) = prepare_enrichment(context, field, evidence)?;
        let draw = derived_draw(context, field, &qualified, selection)?;
        let index = lachesis::index_for_sample(&qualified, draw.sample)?;
        Ok(atropos::seal(
            field,
            &field.candidates[index],
            make_receipt(
                context,
                field,
                &qualified,
                index,
                SelectionProof::derived(draw.sample, draw.digest, selection.clone()),
                Some(enrichment),
            ),
        ))
    }

    /// Recompute every derivable field. Cast replay uses the already-bounded
    /// sample disclosed by the receipt and never pretends to recover entropy.
    pub fn replay(
        context: &ContextSnapshot,
        field: &Field,
        receipt: &Receipt,
    ) -> Result<Reading, ReadingError> {
        check(receipt.context_digest == context.digest(), "context digest")?;
        check(receipt.field_digest == field.digest(), "field digest")?;
        let qualified = match &receipt.enrichment {
            Some(enrichment) => replay_enrichment(context, field, enrichment)?,
            None => lachesis::qualify(context, field)?,
        };
        check(
            receipt.qualified_weights == qualified.weights,
            "qualified weights",
        )?;
        check(receipt.total_weight == qualified.total, "total weight")?;
        let index = match receipt.mode {
            SelectionMode::Calculated => {
                check(receipt.sample.is_none(), "calculated sample")?;
                check(receipt.event_nonce.is_none(), "calculated nonce")?;
                check(receipt.derivation.is_none(), "calculated derivation")?;
                check(
                    receipt.derivation_digest.is_none(),
                    "calculated derivation digest",
                )?;
                require_calculable(field)?;
                lachesis::calculated_index(&qualified)?
            }
            SelectionMode::Cast => {
                let sample = receipt
                    .sample
                    .ok_or_else(|| ReadingError::ReceiptMismatch("cast sample".to_string()))?;
                check(receipt.event_nonce.is_some(), "cast nonce")?;
                check(receipt.derivation.is_none(), "cast derivation")?;
                check(
                    receipt.derivation_digest.is_none(),
                    "cast derivation digest",
                )?;
                lachesis::index_for_sample(&qualified, sample)?
            }
            SelectionMode::Derived => {
                let sample = receipt
                    .sample
                    .ok_or_else(|| ReadingError::ReceiptMismatch("derived sample".to_string()))?;
                let selection = receipt.derivation.as_ref().ok_or_else(|| {
                    ReadingError::ReceiptMismatch("derived selection".to_string())
                })?;
                selection.validate()?;
                check(receipt.event_nonce.is_none(), "derived nonce")?;
                let draw = derived_draw(context, field, &qualified, selection)?;
                check(sample == draw.sample, "derived sample")?;
                check(
                    receipt.derivation_digest.as_deref() == Some(draw.digest.as_str()),
                    "derived digest",
                )?;
                lachesis::index_for_sample(&qualified, sample)?
            }
        };
        check(receipt.selected_index == index, "selected index")?;
        check(
            receipt.selected_candidate == field.candidates[index].id,
            "selected candidate",
        )?;
        let rebuilt = make_receipt(
            context,
            field,
            &qualified,
            index,
            SelectionProof {
                mode: receipt.mode,
                sample: receipt.sample,
                event_nonce: receipt.event_nonce.clone(),
                derivation: receipt.derivation.clone(),
                derivation_digest: receipt.derivation_digest.clone(),
            },
            receipt.enrichment.clone(),
        );
        check(rebuilt == *receipt, "algorithm or schema")?;
        Ok(atropos::seal(field, &field.candidates[index], rebuilt))
    }
}

struct SelectionProof {
    mode: SelectionMode,
    sample: Option<u64>,
    event_nonce: Option<String>,
    derivation: Option<DerivedSelection>,
    derivation_digest: Option<String>,
}

impl SelectionProof {
    fn calculated() -> Self {
        Self {
            mode: SelectionMode::Calculated,
            sample: None,
            event_nonce: None,
            derivation: None,
            derivation_digest: None,
        }
    }

    fn cast(sample: u64, event_nonce: String) -> Self {
        Self {
            mode: SelectionMode::Cast,
            sample: Some(sample),
            event_nonce: Some(event_nonce),
            derivation: None,
            derivation_digest: None,
        }
    }

    fn derived(sample: u64, digest: String, selection: DerivedSelection) -> Self {
        Self {
            mode: SelectionMode::Derived,
            sample: Some(sample),
            event_nonce: None,
            derivation: Some(selection),
            derivation_digest: Some(digest),
        }
    }
}

fn make_receipt(
    context: &ContextSnapshot,
    field: &Field,
    qualified: &lachesis::QualifiedField,
    index: usize,
    selection: SelectionProof,
    enrichment: Option<EnrichmentQualification>,
) -> Receipt {
    let enriched = enrichment.is_some();
    let derived = selection.derivation.is_some();
    let mode = selection.mode;
    Receipt {
        schema: receipt_schema(enriched, derived).to_string(),
        mode,
        algorithm: receipt_algorithm(field, mode, enriched).to_string(),
        context_digest: context.digest(),
        field_digest: field.digest(),
        enrichment,
        derivation: selection.derivation,
        derivation_digest: selection.derivation_digest,
        qualified_weights: qualified.weights.clone(),
        total_weight: qualified.total,
        sample: selection.sample,
        event_nonce: selection.event_nonce,
        selected_index: index,
        selected_candidate: field.candidates[index].id.clone(),
    }
}

fn receipt_schema(enriched: bool, derived: bool) -> &'static str {
    match (enriched, derived) {
        (false, false) => "cleromancy.receipt/v1",
        (true, false) => "cleromancy.receipt/v2",
        (false, true) => "cleromancy.receipt/v3",
        (true, true) => "cleromancy.receipt/v4",
    }
}

fn receipt_algorithm(field: &Field, mode: SelectionMode, enriched: bool) -> &'static str {
    match (field.rules.as_str(), mode, enriched) {
        (CONTEXTUAL_WEIGHT_RULE, SelectionMode::Calculated, false) => "contextual-weight/max/v1",
        (CONTEXTUAL_WEIGHT_RULE, SelectionMode::Cast, false) => {
            "os-csprng/rejection-u64+contextual-weight/v1"
        }
        (CONTEXTUAL_WEIGHT_RULE, SelectionMode::Derived, false) => {
            "derived-blake3/rejection-u64+contextual-weight/v1"
        }
        (UNIFORM_RULE, SelectionMode::Cast, false) => "os-csprng/rejection-u64+uniform/v1",
        (UNIFORM_RULE, SelectionMode::Derived, false) => "derived-blake3/rejection-u64+uniform/v1",
        (UNIFORM_DIE_RULE, SelectionMode::Cast, false) => "os-csprng/rejection-u64+uniform-die/v1",
        (UNIFORM_DIE_RULE, SelectionMode::Derived, false) => {
            "derived-blake3/rejection-u64+uniform-die/v1"
        }
        (EXTERNAL_TERM_WEIGHT_RULE, SelectionMode::Calculated, true) => {
            "contextual-weight+external-term-share/max/v1"
        }
        (EXTERNAL_TERM_WEIGHT_RULE, SelectionMode::Cast, true) => {
            "os-csprng/rejection-u64+contextual-weight+external-term-share/v1"
        }
        (EXTERNAL_TERM_WEIGHT_RULE, SelectionMode::Derived, true) => {
            "derived-blake3/rejection-u64+contextual-weight+external-term-share/v1"
        }
        _ => unreachable!("qualification succeeded only for a supported rule and evidence mode"),
    }
}

struct DerivedDraw {
    sample: u64,
    digest: String,
}

fn derived_draw(
    context: &ContextSnapshot,
    field: &Field,
    qualified: &lachesis::QualifiedField,
    selection: &DerivedSelection,
) -> Result<DerivedDraw, ReadingError> {
    if qualified.total == 0 {
        return Err(ReadingError::EmptyWeight);
    }
    let limit = u64::MAX - (u64::MAX % qualified.total);
    let context_digest = context.digest();
    let field_digest = field.digest();
    for attempt in 0..u64::MAX {
        let mut hasher = blake3::Hasher::new();
        hasher.update(DERIVED_SELECTION_ALGORITHM.as_bytes());
        hash_component(&mut hasher, selection.seed.as_bytes());
        hash_component(&mut hasher, selection.domain.as_bytes());
        hash_component(&mut hasher, context_digest.as_bytes());
        hash_component(&mut hasher, field_digest.as_bytes());
        hash_component(&mut hasher, &(qualified.weights.len() as u64).to_le_bytes());
        for weight in &qualified.weights {
            hash_component(&mut hasher, &weight.to_le_bytes());
        }
        hash_component(&mut hasher, &qualified.total.to_le_bytes());
        hash_component(&mut hasher, &attempt.to_le_bytes());
        let hash = hasher.finalize();
        let raw = u64::from_le_bytes(
            hash.as_bytes()[..8]
                .try_into()
                .expect("BLAKE3 output has a 64-bit prefix"),
        );
        if raw < limit {
            return Ok(DerivedDraw {
                sample: raw % qualified.total,
                digest: hash.to_hex().to_string(),
            });
        }
    }
    Err(ReadingError::DerivedSelectionExhausted)
}

fn hash_component(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn require_calculable(field: &Field) -> Result<(), ReadingError> {
    match field.rules.as_str() {
        UNIFORM_RULE | UNIFORM_DIE_RULE => {
            Err(ReadingError::QualificationRequiresCast(field.rules.clone()))
        }
        _ => Ok(()),
    }
}

fn prepare_enrichment(
    context: &ContextSnapshot,
    field: &Field,
    evidence: &SealedEnrichment,
) -> Result<(lachesis::QualifiedField, EnrichmentQualification), ReadingError> {
    let report = evidence
        .verify(context)
        .map_err(|error| ReadingError::InvalidEnrichment(error.to_string()))?;
    let enriched = lachesis::qualify_enriched(context, field, &report)?;
    let qualification = EnrichmentQualification {
        schema: "cleromancy.enrichment-qualification/v1".to_string(),
        algorithm: EXTERNAL_QUALIFICATION_ALGORITHM.to_string(),
        evidence: evidence.clone(),
        report_digest: report.digest(),
        candidate_terms: enriched.candidate_terms,
        weight_additions: enriched.weight_additions,
    };
    Ok((enriched.qualified, qualification))
}

fn replay_enrichment(
    context: &ContextSnapshot,
    field: &Field,
    qualification: &EnrichmentQualification,
) -> Result<lachesis::QualifiedField, ReadingError> {
    check(
        qualification.schema == "cleromancy.enrichment-qualification/v1",
        "enrichment qualification schema",
    )?;
    check(
        qualification.algorithm == EXTERNAL_QUALIFICATION_ALGORITHM,
        "enrichment qualification algorithm",
    )?;
    let (qualified, rebuilt) = prepare_enrichment(context, field, &qualification.evidence)?;
    check(
        rebuilt.report_digest == qualification.report_digest,
        "enrichment report digest",
    )?;
    check(
        rebuilt.candidate_terms == qualification.candidate_terms,
        "enrichment candidate terms",
    )?;
    check(
        rebuilt.weight_additions == qualification.weight_additions,
        "enrichment weight additions",
    )?;
    Ok(qualified)
}

fn check(condition: bool, field: &str) -> Result<(), ReadingError> {
    condition
        .then_some(())
        .ok_or_else(|| ReadingError::ReceiptMismatch(field.to_string()))
}
