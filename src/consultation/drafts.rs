// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bounded, line-oriented product forms parsed into immutable domain values.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::ConsultationError;
use crate::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, ContextSnapshot, SpreadPosition,
    SpreadRelation, SpreadRelationKind, SpreadTemplate,
};

pub const MANUAL_CONTEXT_SCHEMA: &str = "cleromancy.context/manual/v1";

const MAX_CONTEXT_LABEL_BYTES: usize = 256;
const MAX_QUESTION_BYTES: usize = 4 * 1024;
const MAX_TAGS: usize = 64;
const MAX_TAG_BYTES: usize = 64;
const MAX_ADDITIONAL_FACTS: usize = 64;
const MAX_FACT_NAME_BYTES: usize = 64;
const MAX_FACT_VALUE_BYTES: usize = 4 * 1024;

/// A small, line-oriented authored-layout form. It is intentionally a
/// bounded editor for reusable named positions, not a programmable spread
/// language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadTemplateDraft {
    pub label: String,
    /// `position_name | Visible position label`, one position per line.
    pub positions: String,
    /// `from | relationship | to | Visible relationship label`, one per line.
    #[serde(default)]
    pub relations: String,
}

impl SpreadTemplateDraft {
    pub fn new(label: impl Into<String>, positions: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            positions: positions.into(),
            relations: String::new(),
        }
    }

    pub fn with_relations(mut self, relations: impl Into<String>) -> Self {
        self.relations = relations.into();
        self
    }

    pub fn into_template(self) -> Result<SpreadTemplate, ConsultationError> {
        let positions = self
            .positions
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let (name, label) =
                    line.split_once('|')
                        .ok_or(ConsultationError::InvalidSpread(
                            "each position must use name | label",
                        ))?;
                Ok::<_, ConsultationError>(SpreadPosition::new(
                    name.trim().to_ascii_lowercase(),
                    label.trim(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let relations = self
            .relations
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(parse_spread_relation)
            .collect::<Result<Vec<_>, _>>()?;
        SpreadTemplate::new(self.label.trim(), positions, relations)
            .map_err(|_| ConsultationError::InvalidSpread("the layout is not valid"))
    }
}

fn parse_spread_relation(line: &str) -> Result<SpreadRelation, ConsultationError> {
    let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
    let [from, kind, to, label] = parts.as_slice() else {
        return Err(ConsultationError::InvalidSpread(
            "each relationship must use from | relationship | to | label",
        ));
    };
    let kind = match kind.to_ascii_lowercase().replace('-', "_").as_str() {
        "supports" => SpreadRelationKind::Supports,
        "contradicts" => SpreadRelationKind::Contradicts,
        "questions" => SpreadRelationKind::Questions,
        "next_step" => SpreadRelationKind::NextStep,
        "elaborates" => SpreadRelationKind::Elaborates,
        _ => {
            return Err(ConsultationError::InvalidSpread(
                "relationship is supports, contradicts, questions, next_step, or elaborates",
            ));
        }
    };
    Ok(SpreadRelation::new(
        from.to_ascii_lowercase(),
        kind,
        to.to_ascii_lowercase(),
        *label,
    ))
}

/// Inputs for the local ephemeris calculator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstrologyCalculationDraft {
    pub instant_utc: String,
    #[serde(default)]
    pub latitude_microdegrees: String,
    #[serde(default)]
    pub longitude_microdegrees: String,
    pub orb_millidegrees: String,
}

impl AstrologyCalculationDraft {
    pub fn into_moment_and_orb(self) -> Result<(AstrologyMoment, u32), ConsultationError> {
        moment_and_orb(
            &self.instant_utc,
            &self.latitude_microdegrees,
            &self.longitude_microdegrees,
            &self.orb_millidegrees,
        )
    }
}

/// A source-qualified manual chart import form. Positions are copied from an
/// identified calculation source when a local ephemeris is not used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstrologyChartDraft {
    pub algorithm: String,
    pub engine: String,
    pub ephemeris: String,
    pub instant_utc: String,
    #[serde(default)]
    pub latitude_microdegrees: String,
    #[serde(default)]
    pub longitude_microdegrees: String,
    pub orb_millidegrees: String,
    /// `body | longitude millidegrees | latitude millidegrees | retrograde`,
    /// one body per line. Retrograde is `true`, `false`, or empty.
    pub positions: String,
}

impl AstrologyChartDraft {
    pub fn into_chart_and_orb(self) -> Result<(AstrologyChart, u32), ConsultationError> {
        let (moment, orb) = moment_and_orb(
            &self.instant_utc,
            &self.latitude_microdegrees,
            &self.longitude_microdegrees,
            &self.orb_millidegrees,
        )?;
        let positions = self
            .positions
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(parse_astrology_position)
            .collect::<Result<Vec<_>, _>>()?;
        let chart = AstrologyChart::new(
            self.algorithm.trim(),
            self.engine.trim(),
            self.ephemeris.trim(),
            moment,
            positions,
        )?;
        chart.facts(orb)?;
        Ok((chart, orb))
    }
}

fn moment_and_orb(
    instant_utc: &str,
    latitude_microdegrees: &str,
    longitude_microdegrees: &str,
    orb_millidegrees: &str,
) -> Result<(AstrologyMoment, u32), ConsultationError> {
    if instant_utc.trim().is_empty() {
        return Err(ConsultationError::InvalidAstrology(
            "UTC instant is required",
        ));
    }
    let latitude = optional_number(latitude_microdegrees, "latitude microdegrees")?;
    let longitude = optional_number(longitude_microdegrees, "longitude microdegrees")?;
    let moment = match (latitude, longitude) {
        (None, None) => AstrologyMoment::global(instant_utc.trim()),
        (Some(latitude), Some(longitude)) => {
            AstrologyMoment::at(instant_utc.trim(), latitude, longitude)
        }
        _ => {
            return Err(ConsultationError::InvalidAstrology(
                "latitude and longitude are both required when either is given",
            ));
        }
    };
    let orb = orb_millidegrees
        .trim()
        .parse::<u32>()
        .map_err(|_| ConsultationError::InvalidAstrology("orb millidegrees must be a number"))?;
    if orb > 180_000 {
        return Err(ConsultationError::InvalidAstrology(
            "orb millidegrees must be at most 180000",
        ));
    }
    Ok((moment, orb))
}

fn optional_number(text: &str, name: &'static str) -> Result<Option<i32>, ConsultationError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse()
        .map(Some)
        .map_err(|_| ConsultationError::InvalidAstrology(name))
}

fn parse_astrology_position(line: &str) -> Result<AstrologyPosition, ConsultationError> {
    let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
    let [body, longitude, latitude, retrograde] = parts.as_slice() else {
        return Err(ConsultationError::InvalidAstrology(
            "each position must use body | longitude | latitude | retrograde",
        ));
    };
    let longitude = longitude.parse().map_err(|_| {
        ConsultationError::InvalidAstrology("longitude millidegrees must be a number")
    })?;
    let latitude = latitude.parse().map_err(|_| {
        ConsultationError::InvalidAstrology("latitude millidegrees must be a number")
    })?;
    let position = AstrologyPosition::new(*body, longitude, latitude);
    match *retrograde {
        "" => Ok(position),
        "true" => Ok(position.with_retrograde(true)),
        "false" => Ok(position.with_retrograde(false)),
        _ => Err(ConsultationError::InvalidAstrology(
            "retrograde is true, false, or empty",
        )),
    }
}

/// A headed context form. `additional_facts` is a deliberately small,
/// line-oriented editor for disclosed values rather than a mutable store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextDraft {
    pub label: String,
    pub question: String,
    pub tags: String,
    #[serde(default)]
    pub additional_facts: String,
}

impl ContextDraft {
    pub fn new(
        label: impl Into<String>,
        question: impl Into<String>,
        tags: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            question: question.into(),
            tags: tags.into(),
            additional_facts: String::new(),
        }
    }

    pub fn with_additional_facts(mut self, facts: impl Into<String>) -> Self {
        self.additional_facts = facts.into();
        self
    }

    pub fn into_snapshot(self) -> Result<ContextSnapshot, ConsultationError> {
        let label = self.label.trim();
        if label.is_empty() || label.len() > MAX_CONTEXT_LABEL_BYTES {
            return Err(ConsultationError::InvalidContext(
                "label must contain 1 to 256 bytes",
            ));
        }
        let question = self.question.trim();
        if question.is_empty() || question.len() > MAX_QUESTION_BYTES {
            return Err(ConsultationError::InvalidContext(
                "question must contain 1 to 4096 bytes",
            ));
        }

        let mut tags = BTreeSet::new();
        for tag in self.tags.split(',') {
            let tag = tag.trim().to_lowercase();
            if tag.is_empty() {
                continue;
            }
            if tag.len() > MAX_TAG_BYTES {
                return Err(ConsultationError::InvalidContext(
                    "each tag must contain at most 64 bytes",
                ));
            }
            tags.insert(tag);
        }
        if tags.len() > MAX_TAGS {
            return Err(ConsultationError::InvalidContext(
                "a context may contain at most 64 tags",
            ));
        }

        let facts = additional_facts(&self.additional_facts)?;
        let mut snapshot = ContextSnapshot::new(label, MANUAL_CONTEXT_SCHEMA)
            .with_fact("question", question)
            .with_tags(tags);
        snapshot.facts.extend(facts);
        Ok(snapshot)
    }
}

fn additional_facts(text: &str) -> Result<BTreeMap<String, String>, ConsultationError> {
    let mut facts = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            return Err(ConsultationError::InvalidContext(
                "each additional fact must use name: value",
            ));
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        if name.is_empty()
            || name.len() > MAX_FACT_NAME_BYTES
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(ConsultationError::InvalidContext(
                "additional fact names use lowercase letters, digits, and underscores",
            ));
        }
        if name == "question" {
            return Err(ConsultationError::InvalidContext(
                "question belongs in the question field",
            ));
        }
        if value.is_empty() || value.len() > MAX_FACT_VALUE_BYTES {
            return Err(ConsultationError::InvalidContext(
                "each additional fact value must contain 1 to 4096 bytes",
            ));
        }
        if facts.insert(name, value.to_string()).is_some() {
            return Err(ConsultationError::InvalidContext(
                "additional fact names must be unique",
            ));
        }
        if facts.len() > MAX_ADDITIONAL_FACTS {
            return Err(ConsultationError::InvalidContext(
                "a context may contain at most 64 additional facts",
            ));
        }
    }
    Ok(facts)
}
