// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned deck shuffles. A seed fixes one complete permutation and every
//! orientation; a spread takes a prefix, never another shuffle. Nothing here
//! knows what a deck holds: callers pass the digests that bind a shuffle to
//! its context and deck.
use serde::{Deserialize, Serialize};

pub const SHUFFLE_ALGORITHM: &str = "cleromancy.shuffle/blake3-fisher-yates-u64-rejection/v1";
pub const SHUFFLE_SCHEMA: &str = "cleromancy.deck-shuffle/v1";
/// Largest deck the method accepts.
pub const MAX_DECK: usize = 4096;

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

/// One place in the shuffled deck: which card of the original order, and how
/// it lies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub index: usize,
    pub orientation: Orientation,
}

/// Why a shuffle record or deck size was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawError(pub &'static str);
impl core::fmt::Display for DrawError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for DrawError {}

impl DeckShuffle {
    pub fn from_public_seed(seed: &str, domain: &str) -> Result<Self, DrawError> {
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
    pub fn validate(&self) -> Result<(), DrawError> {
        if self.schema != SHUFFLE_SCHEMA || self.algorithm != SHUFFLE_ALGORITHM {
            return Err(DrawError("unsupported schema or method"));
        }
        match &self.source {
            ShuffleSource::OperatingSystem { seed_hex } => {
                if seed_hex.len() != 64
                    || !seed_hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err(DrawError("operating-system seed"));
                }
            },
            ShuffleSource::PublicSeed { seed, domain } => {
                if seed.trim().is_empty()
                    || seed.len() > 4096
                    || domain.trim().is_empty()
                    || domain.len() > 256
                {
                    return Err(DrawError("public seed/domain bounds"));
                }
            },
        }
        Ok(())
    }
    /// The entire deck of `len` cards in shuffled order, undrawn cards and
    /// orientations included. All integers, input encoding and stream order
    /// are fixed by the method.
    pub fn order(
        &self,
        context_digest: &str,
        deck_digest: &str,
        len: usize,
    ) -> Result<Vec<Placed>, DrawError> {
        self.validate()?;
        if len == 0 || len > MAX_DECK {
            return Err(DrawError("deck size outside supported bounds"));
        }
        let digest = canonical_digest(&(SHUFFLE_ALGORITHM, context_digest, deck_digest, self));
        let mut hasher = blake3::Hasher::new();
        hasher.update(digest.as_bytes());
        let mut stream = hasher.finalize_xof();
        let mut next = || {
            let mut bytes = [0u8; 8];
            stream.fill(&mut bytes);
            u64::from_le_bytes(bytes)
        };
        let mut indices: Vec<_> = (0..len).collect();
        for end in (1..indices.len()).rev() {
            let chosen = below(&mut next, (end + 1) as u64) as usize;
            indices.swap(end, chosen);
        }
        Ok(indices
            .into_iter()
            .map(|index| Placed {
                index,
                orientation: if below(&mut next, 2) == 0 {
                    Orientation::Upright
                } else {
                    Orientation::Reversed
                },
            })
            .collect())
    }
}

/// BLAKE3 over the value's compact JSON, as lowercase hex.
pub fn canonical_digest(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("versioned Cleromancy values always serialize");
    blake3::hash(&bytes).to_hex().to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejection_includes_maximum_word_and_refuses_the_short_prefix() {
        let mut values = [0, 5, 6, u64::MAX].into_iter();
        assert_eq!(below(&mut || values.next().unwrap(), 10), 6);
        assert_eq!(below(&mut || values.next().unwrap(), 10), 5);
    }

    #[test]
    fn a_seed_fixes_one_whole_permutation() {
        let shuffle = DeckShuffle::from_public_seed("2026-11-11", "test").unwrap();
        let order = shuffle.order("context", "deck", 78).unwrap();
        let mut seen: Vec<_> = order.iter().map(|placed| placed.index).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..78).collect::<Vec<_>>());
        assert_eq!(order, shuffle.order("context", "deck", 78).unwrap());
        assert_ne!(order, shuffle.order("context", "other deck", 78).unwrap());
    }

    #[test]
    fn bad_records_and_sizes_are_refused() {
        assert!(DeckShuffle::from_public_seed(" ", "test").is_err());
        let shuffle = DeckShuffle::from_public_seed("seed", "test").unwrap();
        assert!(shuffle.order("c", "d", 0).is_err());
        assert!(shuffle.order("c", "d", MAX_DECK + 1).is_err());
    }
}
