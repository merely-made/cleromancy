// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};

use crate::context::canonical_digest;
use crate::{CONTEXTUAL_WEIGHT_RULE, Candidate, Field, UNIFORM_RULE};

pub const MAJOR_ARCANA_PACK_SCHEMA: &str = "cleromancy.tarot-pack/v1";
pub const RWS_MAJOR_ARCANA_ID: &str = "cleromancy.tarot/rws-major-reflective-v1";
pub const RWS_MAJOR_ARCANA_TITLE: &str = "Major Arcana: reflective prompts";

/// The user-visible way a Tarot field is qualified. Uniform ignores context;
/// contextual gives each matching context tag one additional base-weight
/// share. Both choices remain explicit in the field and reading receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TarotQualification {
    Uniform,
    Contextual,
}

impl TarotQualification {
    pub fn rule(self) -> &'static str {
        match self {
            Self::Uniform => UNIFORM_RULE,
            Self::Contextual => CONTEXTUAL_WEIGHT_RULE,
        }
    }
}

/// A bounded built-in data pack, not a general catalog or plug-in contract.
/// It uses the familiar Rider-Waite-Smith major ordering and original
/// Cleromancy prompts. It deliberately declares neither reversals nor
/// astrology correspondences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TarotPack {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub ordering: String,
    pub content: String,
    pub candidates: Vec<Candidate>,
}

impl TarotPack {
    pub fn rws_major_arcana() -> Self {
        Self {
            schema: MAJOR_ARCANA_PACK_SCHEMA.to_string(),
            id: RWS_MAJOR_ARCANA_ID.to_string(),
            title: RWS_MAJOR_ARCANA_TITLE.to_string(),
            ordering: "Rider-Waite-Smith order; Strength VIII and Justice XI".to_string(),
            content: "Traditional card titles with original upright reflective prompts".to_string(),
            candidates: major_arcana(),
        }
    }

    pub fn field(&self, qualification: TarotQualification) -> Field {
        Field::new(
            self.id.clone(),
            qualification.rule(),
            self.candidates.clone(),
        )
    }

    pub fn digest(&self) -> String {
        canonical_digest(self)
    }
}

fn major_arcana() -> Vec<Candidate> {
    serde_json::from_str(include_str!("content/rws-major-v1.json"))
        .expect("bundled legacy tarot candidates")
}
