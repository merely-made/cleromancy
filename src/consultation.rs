// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The direct local product boundary over Cleromancy's graph authority.
//!
//! Graphshell remains the portable and remote-admission adapter. A headed
//! local process owns its private store directly and uses this controller to
//! turn product commands into durable graph truth.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use muniment::Backend;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::moirai::clotho::{EntropySource, OsEntropy};
use crate::{
    AstrologyChart, AstrologyError, AstrologyFacts, AstrologyMoment, AstrologyPosition,
    CleromancyHost, Concurrence, ContextSnapshot, DerivedSelection, Field, HostError, Reading,
    ReadingEngine, ReadingError, ReadingSession, Reflection, SelectionMode, SpreadPosition,
    SpreadRelation, SpreadRelationKind, SpreadTemplate, TarotPack, TarotQualification,
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

/// A source-qualified chart import form. Positions are copied from an
/// identified calculation source; this crate does not calculate an ephemeris.
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
        let latitude = optional_number(&self.latitude_microdegrees, "latitude microdegrees")?;
        let longitude = optional_number(&self.longitude_microdegrees, "longitude microdegrees")?;
        let moment = match (latitude, longitude) {
            (None, None) => AstrologyMoment::global(self.instant_utc.trim()),
            (Some(latitude), Some(longitude)) => {
                AstrologyMoment::at(self.instant_utc.trim(), latitude, longitude)
            }
            _ => {
                return Err(ConsultationError::InvalidAstrology(
                    "latitude and longitude are both required when either is given",
                ));
            }
        };
        let orb = self.orb_millidegrees.trim().parse::<u32>().map_err(|_| {
            ConsultationError::InvalidAstrology("orb millidegrees must be a number")
        })?;
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

/// Stable picker/history values derived from graph truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultationCatalog {
    pub contexts: Vec<ContextSnapshot>,
    pub fields: Vec<Field>,
    pub spread_templates: Vec<SpreadTemplate>,
    pub astrology_facts: Vec<AstrologyFacts>,
    pub sessions: Vec<ReadingSession>,
}

/// One saved occasion with every value required by the first reading and
/// journal surfaces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultationDetail {
    pub session: ReadingSession,
    pub context: ContextSnapshot,
    pub field: Field,
    pub readings: Vec<Reading>,
    pub reflections: Vec<Reflection>,
    pub concurrences: Vec<Concurrence>,
}

/// A replay-derived comparison of two sealed session receipts. It never writes
/// graph truth and is intentionally silent about a card's interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptComparison {
    pub left_session_id: String,
    pub right_session_id: String,
    pub same_context: bool,
    pub same_field: bool,
    pub same_position_names: bool,
    pub entries: Vec<ReceiptComparisonEntry>,
}

/// Comparison values for one explicitly named placement in a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptComparisonEntry {
    pub position: String,
    pub left_candidate: Option<String>,
    pub right_candidate: Option<String>,
    pub same_candidate: Option<bool>,
    pub left_mode: Option<SelectionMode>,
    pub right_mode: Option<SelectionMode>,
    pub same_mode: Option<bool>,
    pub left_algorithm: Option<String>,
    pub right_algorithm: Option<String>,
    pub same_receipt: Option<bool>,
}

#[derive(Debug, Error)]
pub enum ConsultationError {
    #[error(transparent)]
    Astrology(#[from] AstrologyError),
    #[error(transparent)]
    Host(#[from] HostError),
    #[error(transparent)]
    Reading(#[from] ReadingError),
    #[error("manual context is invalid: {0}")]
    InvalidContext(&'static str),
    #[error("authored spread is invalid: {0}")]
    InvalidSpread(&'static str),
    #[error("astrology chart import is invalid: {0}")]
    InvalidAstrology(&'static str),
    #[error("derived selection requires a disclosed seed and domain")]
    DerivedSelectionRequired,
    #[error("a seed and domain are only valid for derived selection")]
    UnexpectedDerivation,
    #[error("the local consultation authority must be reopened after a storage failure")]
    Faulted,
    #[error("the system clock precedes the Unix epoch")]
    Clock,
}

/// Product transactions over one Cleromancy graph authority.
pub struct Consultation<B> {
    host: CleromancyHost<B>,
    faulted: bool,
}

impl<B: Backend> Consultation<B> {
    pub fn new(host: CleromancyHost<B>) -> Self {
        Self {
            host,
            faulted: false,
        }
    }

    pub fn host(&self) -> &CleromancyHost<B> {
        &self.host
    }

    pub fn is_faulted(&self) -> bool {
        self.faulted
    }

    pub fn catalog(&self) -> Result<ConsultationCatalog, ConsultationError> {
        Ok(ConsultationCatalog {
            contexts: self.host.contexts()?,
            fields: self.host.fields()?,
            spread_templates: self.host.spread_templates()?,
            astrology_facts: self.host.astrology_facts()?,
            sessions: self.host.sessions()?,
        })
    }

    pub fn detail(&self, session_id: &str) -> Result<ConsultationDetail, ConsultationError> {
        let session = self.host.reading_session_for_id(session_id)?;
        let context = self.host.context_for_digest(&session.context_digest)?;
        let field = self.host.field_for_digest(&session.field_digest)?;
        let readings = self.host.replay_session(&session)?;
        let reflections = self.host.reflections_for_session(&session.id)?;
        let concurrences = self.host.concurrences_for_session(&session.id)?;
        Ok(ConsultationDetail {
            session,
            context,
            field,
            readings,
            reflections,
            concurrences,
        })
    }

    pub fn compare_receipts(
        &self,
        left_session_id: &str,
        right_session_id: &str,
    ) -> Result<ReceiptComparison, ConsultationError> {
        let left = self.detail(left_session_id)?;
        let right = self.detail(right_session_id)?;
        Ok(compare_details(&left, &right))
    }

    pub async fn install_builtin_tarot(&mut self) -> Result<String, ConsultationError> {
        self.install_builtin_tarot_at(unix_time_secs()?).await
    }

    pub async fn install_builtin_tarot_at(
        &mut self,
        saved_at_secs: u64,
    ) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let field = TarotPack::rws_major_arcana().field(TarotQualification::Contextual);
        let digest = field.digest();
        self.host.insert_field(&field)?;
        self.persist(saved_at_secs).await?;
        Ok(digest)
    }

    pub async fn save_context(&mut self, draft: ContextDraft) -> Result<String, ConsultationError> {
        self.save_context_at(draft, unix_time_secs()?).await
    }

    pub async fn save_context_at(
        &mut self,
        draft: ContextDraft,
        saved_at_secs: u64,
    ) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let context = draft.into_snapshot()?;
        let digest = context.digest();
        self.host.insert_context(&context)?;
        self.persist(saved_at_secs).await?;
        Ok(digest)
    }

    pub async fn save_spread_template(
        &mut self,
        draft: SpreadTemplateDraft,
    ) -> Result<String, ConsultationError> {
        self.save_spread_template_at(draft, unix_time_secs()?).await
    }

    pub async fn save_spread_template_at(
        &mut self,
        draft: SpreadTemplateDraft,
        saved_at_secs: u64,
    ) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let template = draft.into_template()?;
        let id = template.id.clone();
        self.host.insert_spread_template(&template)?;
        self.persist(saved_at_secs).await?;
        Ok(id)
    }

    pub async fn save_astrology_chart(
        &mut self,
        draft: AstrologyChartDraft,
    ) -> Result<String, ConsultationError> {
        self.save_astrology_chart_at(draft, unix_time_secs()?).await
    }

    pub async fn save_astrology_chart_at(
        &mut self,
        draft: AstrologyChartDraft,
        saved_at_secs: u64,
    ) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let (chart, orb) = draft.into_chart_and_orb()?;
        let facts = chart.facts(orb)?;
        let digest = facts.digest();
        self.host.insert_astrology_chart(&chart, orb)?;
        self.persist(saved_at_secs).await?;
        Ok(digest)
    }

    pub async fn read(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        mode: SelectionMode,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        let mut entropy = OsEntropy;
        self.read_at_with_entropy(
            context_digest,
            field_digest,
            mode,
            created_at_ms,
            created_at_ms / 1000,
            &mut entropy,
        )
        .await
    }

    pub async fn read_at_with_entropy(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        mode: SelectionMode,
        created_at_ms: u64,
        saved_at_secs: u64,
        entropy: &mut impl EntropySource,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let reading = match mode {
            SelectionMode::Calculated => ReadingEngine::calculate(&context, &field)?,
            SelectionMode::Cast => ReadingEngine::cast_with(&context, &field, entropy)?,
            SelectionMode::Derived => return Err(ConsultationError::DerivedSelectionRequired),
        };
        let session = self.host.record_reading_session_at_with_entropy(
            &context,
            &field,
            &reading,
            created_at_ms,
            None,
            entropy,
        )?;
        self.persist(saved_at_secs).await?;
        self.detail(&session.id)
    }

    pub async fn read_derived(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        selection: DerivedSelection,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        let mut entropy = OsEntropy;
        self.read_derived_at_with_entropy(
            context_digest,
            field_digest,
            &selection,
            created_at_ms,
            created_at_ms / 1_000,
            &mut entropy,
        )
        .await
    }

    pub async fn read_derived_at_with_entropy(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        selection: &DerivedSelection,
        created_at_ms: u64,
        saved_at_secs: u64,
        entropy: &mut impl EntropySource,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let reading = ReadingEngine::derive(&context, &field, selection)?;
        let session = self.host.record_reading_session_at_with_entropy(
            &context,
            &field,
            &reading,
            created_at_ms,
            None,
            entropy,
        )?;
        self.persist(saved_at_secs).await?;
        self.detail(&session.id)
    }

    /// Save the one authored A8 three-card cast. There is no configurable
    /// layout or calculated mode at this product boundary.
    pub async fn read_three_card(
        &mut self,
        context_digest: &str,
        field_digest: &str,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        let mut entropy = OsEntropy;
        self.read_three_card_at_with_entropy(
            context_digest,
            field_digest,
            created_at_ms,
            created_at_ms / 1000,
            &mut entropy,
        )
        .await
    }

    pub async fn read_three_card_at_with_entropy(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        created_at_ms: u64,
        saved_at_secs: u64,
        entropy: &mut impl EntropySource,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let (session, _, _) = self.host.record_three_card_spread_at_with_entropy(
            &context,
            &field,
            created_at_ms,
            None,
            entropy,
        )?;
        self.persist(saved_at_secs).await?;
        self.detail(&session.id)
    }

    pub async fn read_spread(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        template_id: &str,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        let mut entropy = OsEntropy;
        self.read_spread_at_with_entropy(
            context_digest,
            field_digest,
            template_id,
            created_at_ms,
            created_at_ms / 1_000,
            &mut entropy,
        )
        .await
    }

    pub async fn read_spread_at_with_entropy(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        template_id: &str,
        created_at_ms: u64,
        saved_at_secs: u64,
        entropy: &mut impl EntropySource,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let template = self.host.spread_template_for_id(template_id)?;
        let (session, _, _) = self.host.record_spread_at_with_entropy(
            &context,
            &field,
            &template,
            created_at_ms,
            None,
            entropy,
        )?;
        self.persist(saved_at_secs).await?;
        self.detail(&session.id)
    }

    pub async fn associate_astrology_facts(
        &mut self,
        astrology_facts_digest: &str,
        session_id: &str,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        self.associate_astrology_facts_at(
            astrology_facts_digest,
            session_id,
            created_at_ms,
            created_at_ms / 1_000,
        )
        .await
    }

    pub async fn associate_astrology_facts_at(
        &mut self,
        astrology_facts_digest: &str,
        session_id: &str,
        created_at_ms: u64,
        saved_at_secs: u64,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        self.host.create_astrology_reading_concurrence_at(
            astrology_facts_digest,
            session_id,
            created_at_ms,
        )?;
        self.persist(saved_at_secs).await?;
        self.detail(session_id)
    }

    pub async fn reflect(
        &mut self,
        session_id: &str,
        body: String,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let created_at_ms = unix_time_ms()?;
        let mut entropy = OsEntropy;
        self.reflect_at_with_entropy(
            session_id,
            body,
            created_at_ms,
            created_at_ms / 1000,
            &mut entropy,
        )
        .await
    }

    pub async fn reflect_at_with_entropy(
        &mut self,
        session_id: &str,
        body: String,
        created_at_ms: u64,
        saved_at_secs: u64,
        entropy: &mut impl EntropySource,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        let session = self.host.reading_session_for_id(session_id)?;
        self.host
            .record_reflection_at_with_entropy(&session, created_at_ms, body, entropy)?;
        self.persist(saved_at_secs).await?;
        self.detail(session_id)
    }

    fn ensure_writable(&self) -> Result<(), ConsultationError> {
        if self.faulted {
            Err(ConsultationError::Faulted)
        } else {
            Ok(())
        }
    }

    async fn persist(&mut self, saved_at_secs: u64) -> Result<(), ConsultationError> {
        if let Err(error) = self.host.persist(saved_at_secs).await {
            self.faulted = true;
            return Err(error.into());
        }
        Ok(())
    }
}

fn compare_details(left: &ConsultationDetail, right: &ConsultationDetail) -> ReceiptComparison {
    let left_readings = readings_by_position(left);
    let right_readings = readings_by_position(right);
    let positions = left_readings
        .keys()
        .chain(right_readings.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let entries = positions
        .into_iter()
        .map(|position| {
            let left_reading = left_readings.get(&position).copied();
            let right_reading = right_readings.get(&position).copied();
            ReceiptComparisonEntry {
                position,
                left_candidate: left_reading.map(|reading| reading.candidate_id.clone()),
                right_candidate: right_reading.map(|reading| reading.candidate_id.clone()),
                same_candidate: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.candidate_id == right.candidate_id),
                left_mode: left_reading.map(|reading| reading.receipt.mode),
                right_mode: right_reading.map(|reading| reading.receipt.mode),
                same_mode: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.receipt.mode == right.receipt.mode),
                left_algorithm: left_reading.map(|reading| reading.receipt.algorithm.clone()),
                right_algorithm: right_reading.map(|reading| reading.receipt.algorithm.clone()),
                same_receipt: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.receipt == right.receipt),
            }
        })
        .collect();
    ReceiptComparison {
        left_session_id: left.session.id.clone(),
        right_session_id: right.session.id.clone(),
        same_context: left.session.context_digest == right.session.context_digest,
        same_field: left.session.field_digest == right.session.field_digest,
        same_position_names: left
            .session
            .placements
            .iter()
            .map(|placement| &placement.position)
            .eq(right
                .session
                .placements
                .iter()
                .map(|placement| &placement.position)),
        entries,
    }
}

fn readings_by_position<'a>(detail: &'a ConsultationDetail) -> BTreeMap<String, &'a Reading> {
    detail
        .session
        .placements
        .iter()
        .zip(&detail.readings)
        .map(|(placement, reading)| (placement.position.clone(), reading))
        .collect()
}

fn unix_time_ms() -> Result<u64, ConsultationError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConsultationError::Clock)?
        .as_millis();
    u64::try_from(millis).map_err(|_| ConsultationError::Clock)
}

fn unix_time_secs() -> Result<u64, ConsultationError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConsultationError::Clock)?
        .as_secs())
}
