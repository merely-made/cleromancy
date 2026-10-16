// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Versioned, cited tarot data. A saved field owns its complete deck snapshot.
use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{Candidate, Field, ReadingError, TarotQualification, ZodiacSign};

pub const DECK_SCHEMA: &str = "cleromancy.typed-tarot-deck/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arcana {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Wands,
    Cups,
    Swords,
    Coins,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Page,
    Knight,
    Queen,
    King,
}

impl Rank {
    pub fn number(self) -> Option<u8> {
        match self {
            Self::Ace => Some(1),
            Self::Two => Some(2),
            Self::Three => Some(3),
            Self::Four => Some(4),
            Self::Five => Some(5),
            Self::Six => Some(6),
            Self::Seven => Some(7),
            Self::Eight => Some(8),
            Self::Nine => Some(9),
            Self::Ten => Some(10),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trump {
    Fool,
    Magician,
    HighPriestess,
    Empress,
    Emperor,
    Hierophant,
    Lovers,
    Chariot,
    Strength,
    Hermit,
    WheelOfFortune,
    Justice,
    HangedMan,
    Death,
    Temperance,
    Devil,
    Tower,
    Star,
    Moon,
    Sun,
    Judgement,
    World,
}

/// Cross-deck identity never depends on a displayed number or title.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "arcana", rename_all = "snake_case", deny_unknown_fields)]
pub enum CardKey {
    Major { trump: Trump },
    Minor { suit: Suit, rank: Rank },
}

impl CardKey {
    pub fn arcana(self) -> Arcana {
        match self {
            Self::Major { .. } => Arcana::Major,
            Self::Minor { .. } => Arcana::Minor,
        }
    }
    pub fn suit(self) -> Option<Suit> {
        match self {
            Self::Minor { suit, .. } => Some(suit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Fire,
    Water,
    Air,
    Earth,
    Spirit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Planet {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    Earth,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentSource {
    pub id: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub url: String,
    pub locator: String,
    pub consulted: String,
}

impl ContentSource {
    pub fn validate(&self) -> bool {
        if !self.url.starts_with("https://")
            || self
                .url
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
        {
            return false;
        }
        [
            &self.id,
            &self.title,
            &self.author,
            &self.date,
            &self.url,
            &self.locator,
            &self.consulted,
        ]
        .iter()
        .all(|s| !s.trim().is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredText {
    pub id: String,
    pub version: String,
    pub author: String,
    pub date: String,
    pub text: String,
    pub provenance: TextProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOrigin {
    ExistingAuthored,
    ModelAssistedEditorial,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextProvenance {
    pub origin: TextOrigin,
    pub model: Option<String>,
    pub method: String,
    pub inputs: Vec<String>,
}

impl AuthoredText {
    pub fn validate(&self) -> bool {
        if self.provenance.method.trim().is_empty()
            || self.provenance.inputs.is_empty()
            || self.provenance.inputs.iter().any(|s| s.trim().is_empty())
            || (self.provenance.origin == TextOrigin::ModelAssistedEditorial
                && self
                    .provenance
                    .model
                    .as_ref()
                    .is_none_or(|m| m.trim().is_empty()))
        {
            return false;
        }
        [
            &self.id,
            &self.version,
            &self.author,
            &self.date,
            &self.text,
        ]
        .iter()
        .all(|s| !s.trim().is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correspondence {
    pub tradition: String,
    pub version: String,
    pub source_id: String,
    pub elements: Vec<Element>,
    pub planets: Vec<Planet>,
    pub signs: Vec<ZodiacSign>,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedCard {
    pub id: String,
    pub title: String,
    pub key: CardKey,
    /// Printed number; unnumbered Fool and court cards have no number.
    pub number: Option<u8>,
    pub interpretation: AuthoredText,
    pub tags: BTreeSet<String>,
    pub correspondences: Vec<Correspondence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TarotDeck {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub sources: Vec<ContentSource>,
    pub cards: Vec<TypedCard>,
}

impl TarotDeck {
    pub fn card(&self, id: &str) -> Option<&TypedCard> {
        self.cards.iter().find(|c| c.id == id)
    }
    pub fn crosswalk(&self, key: CardKey) -> Option<&TypedCard> {
        self.cards.iter().find(|c| c.key == key)
    }
    pub fn source(&self, id: &str) -> Option<&ContentSource> {
        self.sources.iter().find(|s| s.id == id)
    }
    pub fn digest(&self) -> String {
        crate::context::canonical_digest(self)
    }

    pub fn validate(&self) -> Result<(), ReadingError> {
        let invalid = || ReadingError::InvalidDeck(self.id.clone());
        if self.schema != DECK_SCHEMA
            || self.id.trim().is_empty()
            || self.title.trim().is_empty()
            || self.version.trim().is_empty()
            || self.description.trim().is_empty()
            || self.cards.len() != 78
        {
            return Err(invalid());
        }
        let mut sources = BTreeSet::new();
        for s in &self.sources {
            if !s.validate() || !sources.insert(&s.id) {
                return Err(invalid());
            }
        }
        let mut ids = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        for c in &self.cards {
            if c.id.trim().is_empty()
                || c.title.trim().is_empty()
                || !ids.insert(&c.id)
                || !keys.insert(c.key)
                || !c.interpretation.validate()
            {
                return Err(invalid());
            }
            match c.key {
                CardKey::Major { trump } => {
                    if (trump != Trump::Fool && c.number.is_none_or(|n| n == 0))
                        || c.number.is_some_and(|n| n > 21)
                        || (trump == Trump::Fool && c.number.is_some_and(|n| n != 0))
                    {
                        return Err(invalid());
                    }
                    if let Some(n) = c.number {
                        if !numbers.insert(n) {
                            return Err(invalid());
                        }
                    }
                },
                CardKey::Minor { rank, .. } => {
                    if c.number != rank.number() {
                        return Err(invalid());
                    }
                },
            }
            let mut traditions = BTreeSet::new();
            if c.correspondences.is_empty() {
                return Err(invalid());
            }
            for r in &c.correspondences {
                if r.tradition.trim().is_empty()
                    || r.version.trim().is_empty()
                    || r.note.trim().is_empty()
                    || !sources.contains(&r.source_id)
                    || !traditions.insert(&r.tradition)
                {
                    return Err(invalid());
                }
            }
        }
        if keys.iter().filter(|k| k.arcana() == Arcana::Major).count() != 22 {
            return Err(invalid());
        }
        Ok(())
    }

    pub fn field(&self, qualification: TarotQualification) -> Result<Field, ReadingError> {
        self.validate()?;
        let candidates = self.cards.iter().map(|c| {
            Candidate::new(&c.id, &c.title, &c.interpretation.text)
                .with_tags(c.tags.iter().cloned())
        });
        let mut field = Field::new(&self.id, qualification.rule(), candidates);
        field.tarot = Some(self.clone());
        field.position_meanings = Some(crate::content::position_pack().clone());
        Ok(field)
    }
}

pub fn builtin_decks() -> &'static [TarotDeck] {
    static DECKS: OnceLock<Vec<TarotDeck>> = OnceLock::new();
    DECKS.get_or_init(|| {
        let decks: Vec<TarotDeck> =
            serde_json::from_str(include_str!("content/tarot-decks-v1.json"))
                .expect("bundled tarot data");
        for deck in &decks {
            deck.validate().expect("valid bundled deck");
        }
        decks
    })
}
