// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned physical-deck draws. A seed fixes one complete permutation and
//! every orientation; spread length selects a prefix, never another shuffle.
//! The method itself lives in `cleromancy-draw`; this module binds it to a
//! context and field and seals the receipts.
use super::{Reading, ReadingEngine, ReadingError, Receipt, SelectionMode};
use crate::{ContextSnapshot, Field, moirai::clotho::EntropySource};
use cleromancy_draw::DrawError;
pub use cleromancy_draw::{
    DeckShuffle, Orientation, SHUFFLE_ALGORITHM, SHUFFLE_SCHEMA, ShuffleSource,
};
use serde::{Deserialize, Serialize};

pub const SHUFFLED_RECEIPT_SCHEMA: &str = "cleromancy.receipt/v5";

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

impl From<DrawError> for ReadingError {
    fn from(error: DrawError) -> Self {
        invalid(error.0)
    }
}

/// Replays a shuffle against the context and field it was made for.
pub trait ReplayDeck {
    /// Reconstruct the entire deck, including undrawn cards and orientations.
    ///
    /// # Errors
    /// The record is unsupported, or the field is not a tarot deck within
    /// the method's bounds.
    fn replay(
        &self,
        context: &ContextSnapshot,
        field: &Field,
    ) -> Result<Vec<ShuffledCard>, ReadingError>;
}
impl ReplayDeck for DeckShuffle {
    fn replay(
        &self,
        context: &ContextSnapshot,
        field: &Field,
    ) -> Result<Vec<ShuffledCard>, ReadingError> {
        self.validate()?;
        validate_field(context, field, 1)?;
        Ok(self
            .order(&context.digest(), &field.digest(), field.candidates.len())?
            .into_iter()
            .map(|placed| ShuffledCard {
                candidate_index: placed.index,
                candidate_id: field.candidates[placed.index].id.clone(),
                orientation: placed.orientation,
            })
            .collect())
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
