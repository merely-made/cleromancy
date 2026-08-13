// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::context::canonical_digest;
use crate::session::ReadingPlacement;

pub const THREE_CARD_SPREAD_SCHEMA: &str = "cleromancy.three-card-spread/v1";
pub const SPREAD_TEMPLATE_SCHEMA: &str = "cleromancy.spread-template/v1";
pub const SPREAD_SCHEMA: &str = "cleromancy.spread/v1";

const MAX_SPREAD_POSITIONS: usize = 12;
const MAX_SPREAD_RELATIONS: usize = 24;
const MAX_POSITION_NAME_BYTES: usize = 64;
const MAX_LABEL_BYTES: usize = 256;

/// The one authored layout in A8. These names are deliberately concrete: the
/// crate exposes a useful spread, not a general spread language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreeCardPosition {
    Foundation,
    Tension,
    NextStep,
}

impl ThreeCardPosition {
    pub const ALL: [Self; 3] = [Self::Foundation, Self::Tension, Self::NextStep];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Tension => "tension",
            Self::NextStep => "next_step",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreeCardRelationKind {
    Questions,
    NextStep,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeCardPlacement {
    pub position: ThreeCardPosition,
    pub reading_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeCardRelation {
    pub from: ThreeCardPosition,
    pub to: ThreeCardPosition,
    pub kind: ThreeCardRelationKind,
    pub label: String,
}

/// An authored three-card interpretation frame attached to a saved session.
/// The session remains the reusable ordered record; this node adds the
/// position names and the two explicit relationships shown in the graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeCardSpread {
    pub schema: String,
    pub id: String,
    pub session_id: String,
    pub placements: Vec<ThreeCardPlacement>,
    pub relations: Vec<ThreeCardRelation>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpreadError {
    #[error("three-card spread is invalid: {0}")]
    InvalidSpread(String),
}

/// One named, ordered place in a reusable authored spread layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadPosition {
    pub name: String,
    pub label: String,
}

/// The finite semantic relationship kinds a spread author may state between
/// positions. They describe the layout, never a discovered card meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpreadRelationKind {
    Supports,
    Contradicts,
    Questions,
    NextStep,
    Elaborates,
}

/// One authored relationship between two named positions in a layout.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadRelation {
    pub from: String,
    pub to: String,
    pub kind: SpreadRelationKind,
    pub label: String,
}

/// A reusable, content-addressed layout. It contains only explicit authorial
/// structure, so a later session can replay without a live editor or pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadTemplate {
    pub schema: String,
    pub id: String,
    pub label: String,
    pub positions: Vec<SpreadPosition>,
    pub relations: Vec<SpreadRelation>,
}

/// The immutable attachment of one authored template to one saved cast
/// session. Position-to-reading bindings remain in the session itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spread {
    pub schema: String,
    pub id: String,
    pub template_id: String,
    pub session_id: String,
}

impl ThreeCardSpread {
    pub fn new(
        session_id: impl Into<String>,
        session_placements: &[ReadingPlacement],
    ) -> Result<Self, SpreadError> {
        let session_id = session_id.into();
        let placements = ThreeCardPosition::ALL
            .into_iter()
            .map(|position| {
                let reading_id = session_placements
                    .iter()
                    .find(|placement| placement.position == position.as_str())
                    .map(|placement| placement.reading_id.clone())
                    .ok_or_else(|| SpreadError::InvalidSpread("session positions".to_string()))?;
                Ok(ThreeCardPlacement {
                    position,
                    reading_id,
                })
            })
            .collect::<Result<Vec<_>, SpreadError>>()?;
        let relations = vec![
            ThreeCardRelation {
                from: ThreeCardPosition::Tension,
                to: ThreeCardPosition::Foundation,
                kind: ThreeCardRelationKind::Questions,
                label: "tests the foundation".to_string(),
            },
            ThreeCardRelation {
                from: ThreeCardPosition::NextStep,
                to: ThreeCardPosition::Tension,
                kind: ThreeCardRelationKind::NextStep,
                label: "answers the tension".to_string(),
            },
        ];
        let id = three_card_spread_id(&session_id, &placements, &relations);
        let spread = Self {
            schema: THREE_CARD_SPREAD_SCHEMA.to_string(),
            id,
            session_id,
            placements,
            relations,
        };
        spread.validate()?;
        Ok(spread)
    }

    pub fn validate(&self) -> Result<(), SpreadError> {
        if self.schema != THREE_CARD_SPREAD_SCHEMA {
            return Err(SpreadError::InvalidSpread("schema".to_string()));
        }
        if !is_digest(&self.session_id) {
            return Err(SpreadError::InvalidSpread("session id".to_string()));
        }
        if self.placements.len() != 3
            || self
                .placements
                .iter()
                .map(|placement| placement.position)
                .collect::<Vec<_>>()
                != ThreeCardPosition::ALL
        {
            return Err(SpreadError::InvalidSpread("placements".to_string()));
        }
        for placement in &self.placements {
            if !is_digest(&placement.reading_id) {
                return Err(SpreadError::InvalidSpread(
                    "placement reading id".to_string(),
                ));
            }
        }
        let expected_relations = vec![
            ThreeCardRelation {
                from: ThreeCardPosition::Tension,
                to: ThreeCardPosition::Foundation,
                kind: ThreeCardRelationKind::Questions,
                label: "tests the foundation".to_string(),
            },
            ThreeCardRelation {
                from: ThreeCardPosition::NextStep,
                to: ThreeCardPosition::Tension,
                kind: ThreeCardRelationKind::NextStep,
                label: "answers the tension".to_string(),
            },
        ];
        if self.relations != expected_relations {
            return Err(SpreadError::InvalidSpread("authored relations".to_string()));
        }
        if self.id != three_card_spread_id(&self.session_id, &self.placements, &self.relations) {
            return Err(SpreadError::InvalidSpread("identity".to_string()));
        }
        Ok(())
    }
}

impl SpreadPosition {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
        }
    }
}

impl SpreadRelation {
    pub fn new(
        from: impl Into<String>,
        kind: SpreadRelationKind,
        to: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
            label: label.into(),
        }
    }
}

impl SpreadTemplate {
    pub fn new(
        label: impl Into<String>,
        positions: impl IntoIterator<Item = SpreadPosition>,
        relations: impl IntoIterator<Item = SpreadRelation>,
    ) -> Result<Self, SpreadError> {
        let mut relations = relations.into_iter().collect::<Vec<_>>();
        relations.sort();
        let label = label.into();
        let positions = positions.into_iter().collect::<Vec<_>>();
        let id = spread_template_id(&label, &positions, &relations);
        let template = Self {
            schema: SPREAD_TEMPLATE_SCHEMA.to_string(),
            id,
            label,
            positions,
            relations,
        };
        template.validate()?;
        Ok(template)
    }

    pub fn validate(&self) -> Result<(), SpreadError> {
        if self.schema != SPREAD_TEMPLATE_SCHEMA {
            return Err(SpreadError::InvalidSpread("template schema".to_string()));
        }
        if self.label.trim().is_empty() || self.label.len() > MAX_LABEL_BYTES {
            return Err(SpreadError::InvalidSpread("template label".to_string()));
        }
        if self.positions.is_empty() || self.positions.len() > MAX_SPREAD_POSITIONS {
            return Err(SpreadError::InvalidSpread("template positions".to_string()));
        }
        let mut positions = BTreeSet::new();
        for position in &self.positions {
            if !valid_position_name(&position.name) {
                return Err(SpreadError::InvalidSpread("position name".to_string()));
            }
            if position.label.trim().is_empty() || position.label.len() > MAX_LABEL_BYTES {
                return Err(SpreadError::InvalidSpread("position label".to_string()));
            }
            if !positions.insert(position.name.as_str()) {
                return Err(SpreadError::InvalidSpread(
                    "duplicate position name".to_string(),
                ));
            }
        }
        if self.relations.len() > MAX_SPREAD_RELATIONS {
            return Err(SpreadError::InvalidSpread("template relations".to_string()));
        }
        let mut seen_relations = BTreeSet::new();
        for relation in &self.relations {
            if !positions.contains(relation.from.as_str())
                || !positions.contains(relation.to.as_str())
                || relation.from == relation.to
            {
                return Err(SpreadError::InvalidSpread("relation positions".to_string()));
            }
            if relation.label.trim().is_empty() || relation.label.len() > MAX_LABEL_BYTES {
                return Err(SpreadError::InvalidSpread("relation label".to_string()));
            }
            if !seen_relations.insert(relation) {
                return Err(SpreadError::InvalidSpread("duplicate relation".to_string()));
            }
        }
        if self.relations.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err(SpreadError::InvalidSpread(
                "relation order is not canonical".to_string(),
            ));
        }
        if self.id != spread_template_id(&self.label, &self.positions, &self.relations) {
            return Err(SpreadError::InvalidSpread("template identity".to_string()));
        }
        Ok(())
    }
}

impl Spread {
    pub fn new(
        template: &SpreadTemplate,
        session: &crate::ReadingSession,
    ) -> Result<Self, SpreadError> {
        template.validate()?;
        session
            .validate()
            .map_err(|error| SpreadError::InvalidSpread(error.to_string()))?;
        if !template
            .positions
            .iter()
            .map(|position| position.name.as_str())
            .eq(session
                .placements
                .iter()
                .map(|placement| placement.position.as_str()))
        {
            return Err(SpreadError::InvalidSpread(
                "session positions do not match template".to_string(),
            ));
        }
        let id = spread_id(&template.id, &session.id);
        let spread = Self {
            schema: SPREAD_SCHEMA.to_string(),
            id,
            template_id: template.id.clone(),
            session_id: session.id.clone(),
        };
        spread.validate()?;
        Ok(spread)
    }

    pub fn validate(&self) -> Result<(), SpreadError> {
        if self.schema != SPREAD_SCHEMA {
            return Err(SpreadError::InvalidSpread("spread schema".to_string()));
        }
        if !is_digest(&self.template_id) || !is_digest(&self.session_id) {
            return Err(SpreadError::InvalidSpread(
                "spread dependency id".to_string(),
            ));
        }
        if self.id != spread_id(&self.template_id, &self.session_id) {
            return Err(SpreadError::InvalidSpread("spread identity".to_string()));
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct ThreeCardSpreadIdentity<'a> {
    schema: &'static str,
    session_id: &'a str,
    placements: &'a [ThreeCardPlacement],
    relations: &'a [ThreeCardRelation],
}

#[derive(Serialize)]
struct SpreadTemplateIdentity<'a> {
    schema: &'static str,
    label: &'a str,
    positions: &'a [SpreadPosition],
    relations: &'a [SpreadRelation],
}

#[derive(Serialize)]
struct SpreadIdentity<'a> {
    schema: &'static str,
    template_id: &'a str,
    session_id: &'a str,
}

fn three_card_spread_id(
    session_id: &str,
    placements: &[ThreeCardPlacement],
    relations: &[ThreeCardRelation],
) -> String {
    canonical_digest(&ThreeCardSpreadIdentity {
        schema: THREE_CARD_SPREAD_SCHEMA,
        session_id,
        placements,
        relations,
    })
}

fn spread_template_id(
    label: &str,
    positions: &[SpreadPosition],
    relations: &[SpreadRelation],
) -> String {
    canonical_digest(&SpreadTemplateIdentity {
        schema: SPREAD_TEMPLATE_SCHEMA,
        label,
        positions,
        relations,
    })
}

fn spread_id(template_id: &str, session_id: &str) -> String {
    canonical_digest(&SpreadIdentity {
        schema: SPREAD_SCHEMA,
        template_id,
        session_id,
    })
}

fn valid_position_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_POSITION_NAME_BYTES
        && value.bytes().enumerate().all(|(index, byte)| {
            (byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
                && (index > 0 || byte.is_ascii_lowercase())
        })
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
