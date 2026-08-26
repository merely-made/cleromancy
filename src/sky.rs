// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Durable, source-qualified sky facts and authored interpretations.
//!
//! This module stores astronomical facts without making a visibility, causal,
//! or astrological claim. Numerical adapters normalize their output here;
//! authored rule packs remain independently inspectable and replaceable.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::context::canonical_digest;

#[cfg(feature = "sky-timeline")]
pub mod turquet;

pub const SKY_DAY_FACTS_SCHEMA: &str = "cleromancy.sky-day-facts/v1";
pub const SKY_RULE_PACK_SCHEMA: &str = "cleromancy.sky-rule-pack/v1";
pub const SKY_INTERPRETATION_SCHEMA: &str = "cleromancy.sky-interpretation/v1";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SkyError {
    #[error("sky {0} is empty")]
    Empty(&'static str),
    #[error("sky civil day is invalid")]
    InvalidCivilDay,
    #[error("sky observer latitude is outside WGS84 bounds: {0}")]
    InvalidLatitude(i32),
    #[error("sky observer longitude is outside WGS84 bounds: {0}")]
    InvalidLongitude(i32),
    #[error("sky search controls are invalid: {0}")]
    InvalidSearch(&'static str),
    #[error("sky twilight altitude is outside -90..90 degrees: {0}")]
    InvalidTwilightAltitude(i32),
    #[error("sky TT interval is invalid")]
    InvalidInterval,
    #[error("sky facts use an unexpected schema")]
    InvalidFactsSchema,
    #[error("sky rule pack uses an unexpected schema")]
    InvalidPackSchema,
    #[error("sky interpretation uses an unexpected schema")]
    InvalidInterpretationSchema,
    #[error("sky facts are not in canonical order")]
    NonCanonicalFacts,
    #[error("sky rules are not in canonical order")]
    NonCanonicalRules,
    #[error("sky value is duplicated: {0}")]
    Duplicate(String),
    #[error("sky interpretation does not bind to the supplied facts: {0}")]
    FactsMismatch(&'static str),
    #[error("sky interpretation does not match the supplied rule pack: {0}")]
    PackMismatch(&'static str),
}

/// A calendar day explicitly identified as UTC, without a local timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtcCivilDay {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl UtcCivilDay {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, SkyError> {
        let value = Self { year, month, day };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        let days = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(self.year) => 29,
            2 => 28,
            _ => return Err(SkyError::InvalidCivilDay),
        };
        if self.day == 0 || self.day > days {
            return Err(SkyError::InvalidCivilDay);
        }
        Ok(())
    }
}

/// A WGS84 geodetic observer; longitude is east-positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wgs84Observer {
    pub latitude_microdegrees: i32,
    pub longitude_microdegrees: i32,
    pub height_millimeters: i32,
}

impl Wgs84Observer {
    pub fn new(
        latitude_microdegrees: i32,
        longitude_microdegrees: i32,
        height_millimeters: i32,
    ) -> Result<Self, SkyError> {
        let value = Self {
            latitude_microdegrees,
            longitude_microdegrees,
            height_millimeters,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        if !(-90_000_000..=90_000_000).contains(&self.latitude_microdegrees) {
            return Err(SkyError::InvalidLatitude(self.latitude_microdegrees));
        }
        if !(-180_000_000..=180_000_000).contains(&self.longitude_microdegrees) {
            return Err(SkyError::InvalidLongitude(self.longitude_microdegrees));
        }
        Ok(())
    }
}

/// Validated sampling controls, expressed without an implicit default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkySearchControls {
    pub step_seconds: u32,
    pub tolerance_milliseconds: u32,
}

impl SkySearchControls {
    pub fn new(step_seconds: u32, tolerance_milliseconds: u32) -> Result<Self, SkyError> {
        let value = Self {
            step_seconds,
            tolerance_milliseconds,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        if self.step_seconds == 0 || self.step_seconds > 86_400 {
            return Err(SkyError::InvalidSearch("step_seconds"));
        }
        if self.tolerance_milliseconds == 0
            || self.tolerance_milliseconds > self.step_seconds.saturating_mul(1_000)
        {
            return Err(SkyError::InvalidSearch("tolerance_milliseconds"));
        }
        Ok(())
    }
}

/// The only twilight measurement this model recognizes: an airless solar
/// center-altitude crossing. It conveys no human visibility meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkyTwilightMeasurement {
    AirlessSolarCenter,
}

/// Caller-selected controls for the T4l twilight solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyTwilightPolicy {
    pub measurement: SkyTwilightMeasurement,
    pub altitude_millidegrees: i32,
    pub search: SkySearchControls,
}

impl SkyTwilightPolicy {
    pub fn validate(&self) -> Result<(), SkyError> {
        if !(-90_000..=90_000).contains(&self.altitude_millidegrees) {
            return Err(SkyError::InvalidTwilightAltitude(
                self.altitude_millidegrees,
            ));
        }
        self.search.validate()?;
        if self.search.step_seconds > 3_600 {
            return Err(SkyError::InvalidSearch("twilight step_seconds"));
        }
        Ok(())
    }
}

/// Disclosed EOP approximation retained as part of the effective policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkyEarthOrientationApproximation {
    /// A fixed UT1-minus-UTC offset with zero polar motion. This is an
    /// explicit bounded approximation, not an observation-free default.
    ConstantUt1MinusUtcZeroPolarMotion { ut1_minus_utc_milliseconds: i32 },
}

/// Disclosed EOP approximation retained as part of the effective policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyEarthOrientationPolicy {
    pub authority: String,
    pub snapshot: String,
    pub approximation: SkyEarthOrientationApproximation,
}

impl SkyEarthOrientationPolicy {
    fn validate(&self) -> Result<(), SkyError> {
        nonempty("earth_orientation.authority", &self.authority)?;
        nonempty("earth_orientation.snapshot", &self.snapshot)?;
        Ok(())
    }
}

/// All numerical controls effective for one daily timeline calculation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyNumericalPolicy {
    pub phase_search: SkySearchControls,
    pub twilight: SkyTwilightPolicy,
    pub earth_orientation: SkyEarthOrientationPolicy,
}

impl SkyNumericalPolicy {
    pub fn validate(&self) -> Result<(), SkyError> {
        self.phase_search.validate()?;
        self.twilight.validate()?;
        self.earth_orientation.validate()
    }
}

/// A TT interval containing a fact. It is intentionally not a UTC timestamp.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyTtInterval {
    pub start_julian_day: f64,
    pub end_julian_day: f64,
}

impl SkyTtInterval {
    pub fn new(start_julian_day: f64, end_julian_day: f64) -> Result<Self, SkyError> {
        let value = Self {
            start_julian_day: normalize_zero(start_julian_day),
            end_julian_day: normalize_zero(end_julian_day),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        if !self.start_julian_day.is_finite()
            || !self.end_julian_day.is_finite()
            || self.end_julian_day <= self.start_julian_day
        {
            return Err(SkyError::InvalidInterval);
        }
        Ok(())
    }
}

/// Source identity carried by one normalized fact. These values are strings so
/// persisted Cleromancy data never embeds a Turquet Rust type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyProvenance {
    pub model: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_snapshot: Option<String>,
    /// Transform identity, or a disclosed `not applicable` reason for a
    /// geocentric event that has no observer transform.
    pub transform: String,
    /// EOP identity, or a disclosed `not applicable` reason for an event that
    /// does not use Earth rotation.
    pub earth_orientation: String,
}

impl SkyProvenance {
    pub fn validate(&self) -> Result<(), SkyError> {
        nonempty("provenance.model", &self.model)?;
        nonempty("provenance.provider", &self.provider)?;
        optional_nonempty("provenance.provider_snapshot", &self.provider_snapshot)?;
        nonempty("provenance.transform", &self.transform)?;
        nonempty("provenance.earth_orientation", &self.earth_orientation)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkyFactKind {
    NewMoon,
    Dawn,
    Dusk,
}

/// One numerical fact. The interval and provenance remain facts; an authored
/// prompt is deliberately not stored here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyFact {
    pub kind: SkyFactKind,
    pub interval: SkyTtInterval,
    pub provenance: SkyProvenance,
}

impl SkyFact {
    pub fn validate(&self) -> Result<(), SkyError> {
        self.interval.validate()?;
        self.provenance.validate()
    }
}

/// The canonical, persisted factual record for one UTC day and observer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyDayFacts {
    pub schema: String,
    pub civil_day: UtcCivilDay,
    pub observer: Wgs84Observer,
    pub policy: SkyNumericalPolicy,
    pub facts: Vec<SkyFact>,
}

impl SkyDayFacts {
    pub fn new(
        civil_day: UtcCivilDay,
        observer: Wgs84Observer,
        policy: SkyNumericalPolicy,
        facts: impl IntoIterator<Item = SkyFact>,
    ) -> Result<Self, SkyError> {
        let mut value = Self {
            schema: SKY_DAY_FACTS_SCHEMA.to_string(),
            civil_day,
            observer,
            policy,
            facts: facts.into_iter().collect(),
        };
        value.facts.sort_by(compare_fact);
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        if self.schema != SKY_DAY_FACTS_SCHEMA {
            return Err(SkyError::InvalidFactsSchema);
        }
        self.civil_day.validate()?;
        self.observer.validate()?;
        self.policy.validate()?;
        if self
            .facts
            .windows(2)
            .any(|pair| compare_fact(&pair[0], &pair[1]) != Ordering::Less)
        {
            return Err(SkyError::NonCanonicalFacts);
        }
        for fact in &self.facts {
            fact.validate()?;
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        canonical_digest(self)
    }
}

/// One authored matching rule. Prompts are authored text, not generated prose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyRule {
    pub id: String,
    pub fact_kind: SkyFactKind,
    pub prompt: String,
}

impl SkyRule {
    fn validate(&self) -> Result<(), SkyError> {
        nonempty("rule.id", &self.id)?;
        nonempty("rule.prompt", &self.prompt)
    }
}

/// Independently versioned authored interpretation rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyRulePack {
    pub schema: String,
    pub id: String,
    pub author: String,
    pub version: String,
    pub rules: Vec<SkyRule>,
}

impl SkyRulePack {
    pub fn new(
        id: impl Into<String>,
        author: impl Into<String>,
        version: impl Into<String>,
        rules: impl IntoIterator<Item = SkyRule>,
    ) -> Result<Self, SkyError> {
        let mut value = Self {
            schema: SKY_RULE_PACK_SCHEMA.to_string(),
            id: id.into(),
            author: author.into(),
            version: version.into(),
            rules: rules.into_iter().collect(),
        };
        value.rules.sort_by(|left, right| left.id.cmp(&right.id));
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SkyError> {
        if self.schema != SKY_RULE_PACK_SCHEMA {
            return Err(SkyError::InvalidPackSchema);
        }
        nonempty("pack.id", &self.id)?;
        nonempty("pack.author", &self.author)?;
        nonempty("pack.version", &self.version)?;
        if self.rules.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(SkyError::NonCanonicalRules);
        }
        let mut ids = BTreeSet::new();
        let mut kinds = BTreeSet::new();
        for rule in &self.rules {
            rule.validate()?;
            if !ids.insert(rule.id.as_str()) {
                return Err(SkyError::Duplicate(rule.id.clone()));
            }
            if !kinds.insert(rule.fact_kind) {
                return Err(SkyError::Duplicate(format!(
                    "rule kind {:?}",
                    rule.fact_kind
                )));
            }
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        canonical_digest(self)
    }

    pub fn rule_for(&self, kind: SkyFactKind) -> Option<&SkyRule> {
        self.rules.iter().find(|rule| rule.fact_kind == kind)
    }
}

/// A disclosed application of one authored rule to one durable sky fact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkyInterpretation {
    pub schema: String,
    pub sky_facts_digest: String,
    pub fact_kind: SkyFactKind,
    pub fact_interval: SkyTtInterval,
    pub pack_id: String,
    pub pack_author: String,
    pub pack_version: String,
    pub pack_digest: String,
    pub rule_id: String,
    pub prompt: String,
}

impl SkyInterpretation {
    pub fn validate(&self) -> Result<(), SkyError> {
        if self.schema != SKY_INTERPRETATION_SCHEMA {
            return Err(SkyError::InvalidInterpretationSchema);
        }
        nonempty("interpretation.sky_facts_digest", &self.sky_facts_digest)?;
        self.fact_interval.validate()?;
        nonempty("interpretation.pack_id", &self.pack_id)?;
        nonempty("interpretation.pack_author", &self.pack_author)?;
        nonempty("interpretation.pack_version", &self.pack_version)?;
        nonempty("interpretation.pack_digest", &self.pack_digest)?;
        nonempty("interpretation.rule_id", &self.rule_id)?;
        nonempty("interpretation.prompt", &self.prompt)
    }

    pub fn digest(&self) -> String {
        canonical_digest(self)
    }

    pub fn verify_binding(&self, facts: &SkyDayFacts) -> Result<(), SkyError> {
        self.validate()?;
        facts.validate()?;
        if self.sky_facts_digest != facts.digest() {
            return Err(SkyError::FactsMismatch("digest"));
        }
        if !facts
            .facts
            .iter()
            .any(|fact| fact.kind == self.fact_kind && fact.interval == self.fact_interval)
        {
            return Err(SkyError::FactsMismatch("fact"));
        }
        Ok(())
    }

    pub fn verify_with_pack(
        &self,
        facts: &SkyDayFacts,
        pack: &SkyRulePack,
    ) -> Result<(), SkyError> {
        self.verify_binding(facts)?;
        pack.validate()?;
        if self.pack_id != pack.id
            || self.pack_author != pack.author
            || self.pack_version != pack.version
            || self.pack_digest != pack.digest()
        {
            return Err(SkyError::PackMismatch("identity"));
        }
        let Some(rule) = pack.rule_for(self.fact_kind) else {
            return Err(SkyError::PackMismatch("matching rule"));
        };
        if self.rule_id != rule.id || self.prompt != rule.prompt {
            return Err(SkyError::PackMismatch("rule"));
        }
        Ok(())
    }
}

/// Apply a pack exactly. This never changes, infers, or explains sky facts.
pub fn interpret_sky_day(
    facts: &SkyDayFacts,
    pack: &SkyRulePack,
) -> Result<Vec<SkyInterpretation>, SkyError> {
    facts.validate()?;
    pack.validate()?;
    let facts_digest = facts.digest();
    let pack_digest = pack.digest();
    Ok(facts
        .facts
        .iter()
        .filter_map(|fact| {
            pack.rule_for(fact.kind).map(|rule| SkyInterpretation {
                schema: SKY_INTERPRETATION_SCHEMA.to_string(),
                sky_facts_digest: facts_digest.clone(),
                fact_kind: fact.kind,
                fact_interval: fact.interval,
                pack_id: pack.id.clone(),
                pack_author: pack.author.clone(),
                pack_version: pack.version.clone(),
                pack_digest: pack_digest.clone(),
                rule_id: rule.id.clone(),
                prompt: rule.prompt.clone(),
            })
        })
        .collect())
}

fn compare_fact(left: &SkyFact, right: &SkyFact) -> Ordering {
    left.interval
        .start_julian_day
        .total_cmp(&right.interval.start_julian_day)
        .then_with(|| {
            left.interval
                .end_julian_day
                .total_cmp(&right.interval.end_julian_day)
        })
        .then_with(|| left.kind.cmp(&right.kind))
}

fn is_leap_year(year: i32) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn nonempty(field: &'static str, value: &str) -> Result<(), SkyError> {
    if value.trim().is_empty() {
        Err(SkyError::Empty(field))
    } else {
        Ok(())
    }
}

fn optional_nonempty(field: &'static str, value: &Option<String>) -> Result<(), SkyError> {
    if let Some(value) = value {
        nonempty(field, value)?;
    }
    Ok(())
}
