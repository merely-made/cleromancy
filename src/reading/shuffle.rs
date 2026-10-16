// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned physical-deck draws. A seed fixes one complete permutation and
//! every orientation; spread length selects a prefix, never another shuffle.
use super::{Reading, ReadingEngine, ReadingError, Receipt, SelectionMode};
use crate::{ContextSnapshot, Field, context::canonical_digest, moirai::clotho::EntropySource};
use serde::{Deserialize, Serialize};

pub const SHUFFLE_ALGORITHM: &str = "cleromancy.shuffle/blake3-fisher-yates-u64-rejection/v1";
pub const SHUFFLE_SCHEMA: &str = "cleromancy.deck-shuffle/v1";
pub const SHUFFLED_RECEIPT_SCHEMA: &str = "cleromancy.receipt/v5";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Upright,
    Reversed,
}
impl Orientation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Upright => "upright",
            Self::Reversed => "reversed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShuffleSource {
    /// Recorded random seed, not the operating system's internal RNG state.
    OperatingSystem {
        seed_hex: String,
    },
    PublicSeed {
        seed: String,
        domain: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckShuffle {
    pub schema: String,
    pub algorithm: String,
    pub source: ShuffleSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckDraw {
    pub shuffle: DeckShuffle,
    /// Zero-based index into the shuffled deck.
    pub index: usize,
    pub count: usize,
    pub orientation: Orientation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShuffledCard {
    pub candidate_index: usize,
    pub candidate_id: String,
    pub orientation: Orientation,
}

fn invalid(message: &str) -> ReadingError {
    ReadingError::ReceiptMismatch(format!("deck shuffle: {message}"))
}

impl DeckShuffle {
    pub fn from_public_seed(seed: &str, domain: &str) -> Result<Self, ReadingError> {
        let record = Self {
            schema: SHUFFLE_SCHEMA.into(),
            algorithm: SHUFFLE_ALGORITHM.into(),
            source: ShuffleSource::PublicSeed {
                seed: seed.into(),
                domain: domain.into(),
            },
        };
        record.validate()?;
        Ok(record)
    }
    fn validate(&self) -> Result<(), ReadingError> {
        if self.schema != SHUFFLE_SCHEMA || self.algorithm != SHUFFLE_ALGORITHM {
            return Err(invalid("unsupported schema or method"));
        }
        match &self.source {
            ShuffleSource::OperatingSystem { seed_hex } => {
                if seed_hex.len() != 64
                    || !seed_hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err(invalid("operating-system seed"));
                }
            },
            ShuffleSource::PublicSeed { seed, domain } => {
                if seed.trim().is_empty()
                    || seed.len() > 4096
                    || domain.trim().is_empty()
                    || domain.len() > 256
                {
                    return Err(invalid("public seed/domain bounds"));
                }
            },
        }
        Ok(())
    }
    /// Reconstruct the entire deck, including undrawn cards and orientations.
    /// All integers, input encoding and stream order are fixed by the method.
    pub fn replay(
        &self,
        context: &ContextSnapshot,
        field: &Field,
    ) -> Result<Vec<ShuffledCard>, ReadingError> {
        self.validate()?;
        validate_field(context, field, 1)?;
        let digest = canonical_digest(&(SHUFFLE_ALGORITHM, context.digest(), field.digest(), self));
        let mut hasher = blake3::Hasher::new();
        hasher.update(digest.as_bytes());
        let mut stream = hasher.finalize_xof();
        let mut next = || {
            let mut bytes = [0u8; 8];
            stream.fill(&mut bytes);
            u64::from_le_bytes(bytes)
        };
        let mut indices: Vec<_> = (0..field.candidates.len()).collect();
        for end in (1..indices.len()).rev() {
            let chosen = below(&mut next, (end + 1) as u64) as usize;
            indices.swap(end, chosen);
        }
        Ok(indices
            .into_iter()
            .map(|index| ShuffledCard {
                candidate_index: index,
                candidate_id: field.candidates[index].id.clone(),
                orientation: if below(&mut next, 2) == 0 {
                    Orientation::Upright
                } else {
                    Orientation::Reversed
                },
            })
            .collect())
    }
}

// The accepted interval has exactly a multiple of upper elements in 2^64.
fn below(next: &mut impl FnMut() -> u64, upper: u64) -> u64 {
    let threshold = upper.wrapping_neg() % upper;
    loop {
        let raw = next();
        if raw >= threshold {
            return raw % upper;
        }
    }
}

fn validate_field(
    context: &ContextSnapshot,
    field: &Field,
    count: usize,
) -> Result<(), ReadingError> {
    if !field.is_tarot() {
        return Err(invalid("requires a tarot deck"));
    }
    if field.candidates.is_empty()
        || field.candidates.len() > 4096
        || count == 0
        || count > field.candidates.len()
    {
        return Err(invalid("draw exceeds available deck or supported bounds"));
    }
    // Validate the declared field and supported rule, but physical shuffling
    // gives every card one place regardless of contextual qualification weight.
    crate::moirai::lachesis::qualify(context, field)?;
    Ok(())
}

fn readings(
    context: &ContextSnapshot,
    field: &Field,
    record: &DeckShuffle,
    count: usize,
) -> Result<Vec<Reading>, ReadingError> {
    validate_field(context, field, count)?;
    let cards = record.replay(context, field)?;
    let context_digest = context.digest();
    let field_digest = field.digest();
    Ok(cards
        .into_iter()
        .take(count)
        .enumerate()
        .map(|(index, card)| {
            seal_draw(
                &context_digest,
                &field_digest,
                field,
                record,
                count,
                index,
                card,
            )
        })
        .collect())
}
fn seal_draw(
    context_digest: &str,
    field_digest: &str,
    field: &Field,
    record: &DeckShuffle,
    count: usize,
    index: usize,
    card: ShuffledCard,
) -> Reading {
    let mode = match record.source {
        ShuffleSource::OperatingSystem { .. } => SelectionMode::Cast,
        ShuffleSource::PublicSeed { .. } => SelectionMode::Derived,
    };
    let receipt = Receipt {
        schema: SHUFFLED_RECEIPT_SCHEMA.into(),
        mode,
        algorithm: SHUFFLE_ALGORITHM.into(),
        context_digest: context_digest.into(),
        field_digest: field_digest.into(),
        enrichment: None,
        derivation: None,
        derivation_digest: None,
        shuffle: Some(DeckDraw {
            shuffle: record.clone(),
            index,
            count,
            orientation: card.orientation,
        }),
        qualified_weights: vec![1; field.candidates.len()],
        total_weight: field.candidates.len() as u64,
        sample: None,
        event_nonce: None,
        selected_index: card.candidate_index,
        selected_candidate: card.candidate_id,
    };
    crate::moirai::atropos::seal(field, &field.candidates[receipt.selected_index], receipt)
}

impl ReadingEngine {
    pub fn shuffle_with(
        context: &ContextSnapshot,
        field: &Field,
        count: usize,
        entropy: &mut impl EntropySource,
    ) -> Result<Vec<Reading>, ReadingError> {
        validate_field(context, field, count)?;
        let mut seed = String::new();
        for _ in 0..4 {
            seed.push_str(&format!("{:016x}", entropy.next_u64()?));
        }
        readings(
            context,
            field,
            &DeckShuffle {
                schema: SHUFFLE_SCHEMA.into(),
                algorithm: SHUFFLE_ALGORITHM.into(),
                source: ShuffleSource::OperatingSystem { seed_hex: seed },
            },
            count,
        )
    }
    pub fn shuffle_derived(
        context: &ContextSnapshot,
        field: &Field,
        count: usize,
        seed: &str,
        domain: &str,
    ) -> Result<Vec<Reading>, ReadingError> {
        readings(
            context,
            field,
            &DeckShuffle::from_public_seed(seed, domain)?,
            count,
        )
    }
}

pub(super) fn replay(
    context: &ContextSnapshot,
    field: &Field,
    receipt: &Receipt,
) -> Result<Reading, ReadingError> {
    let draw = receipt
        .shuffle
        .as_ref()
        .ok_or_else(|| invalid("missing record"))?;
    if draw.index >= draw.count {
        return Err(invalid("draw index"));
    }
    validate_field(context, field, draw.count)?;
    let cards = draw.shuffle.replay(context, field)?;
    let reading = seal_draw(
        &context.digest(),
        &field.digest(),
        field,
        &draw.shuffle,
        draw.count,
        draw.index,
        cards[draw.index].clone(),
    );
    if reading.receipt != *receipt {
        return Err(invalid("record differs from replay"));
    }
    Ok(reading)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejection_includes_maximum_word_and_refuses_the_short_prefix() {
        let mut values = [0, 5, 6, u64::MAX].into_iter();
        assert_eq!(super::below(&mut || values.next().unwrap(), 10), 6);
        assert_eq!(super::below(&mut || values.next().unwrap(), 10), 5);
    }
}
