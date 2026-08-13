// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The direct local product boundary over Cleromancy's graph authority.
//!
//! Graphshell remains the portable and remote-admission adapter. A headed
//! local process owns its private store directly and uses this controller to
//! turn product commands into durable graph truth.

use std::time::{SystemTime, UNIX_EPOCH};

use muniment::Backend;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::moirai::clotho::{EntropySource, OsEntropy};
use crate::{
    AstrologyChart, AstrologyError, AstrologyFacts, CleromancyHost, Concurrence, ContextSnapshot,
    DerivedSelection, Field, HostError, Reading, ReadingEngine, ReadingError, ReadingSession,
    Reflection, SelectionMode, SpreadTemplate, TarotPack, TarotQualification,
};

mod compare;
mod drafts;

use compare::compare_details;
pub use drafts::{
    AstrologyCalculationDraft, AstrologyChartDraft, ContextDraft, MANUAL_CONTEXT_SCHEMA,
    SpreadTemplateDraft,
};

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
    #[error("ephemeris operation failed: {0}")]
    Ephemeris(String),
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
        let (chart, orb) = draft.into_chart_and_orb()?;
        self.save_calculated_astrology_chart_at(chart, orb, saved_at_secs)
            .await
    }

    pub async fn save_calculated_astrology_chart(
        &mut self,
        chart: AstrologyChart,
        orb_millidegrees: u32,
    ) -> Result<String, ConsultationError> {
        self.save_calculated_astrology_chart_at(chart, orb_millidegrees, unix_time_secs()?)
            .await
    }

    pub async fn save_calculated_astrology_chart_at(
        &mut self,
        chart: AstrologyChart,
        orb_millidegrees: u32,
        saved_at_secs: u64,
    ) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        chart.validate()?;
        let facts = chart.facts(orb_millidegrees)?;
        let digest = facts.digest();
        self.host.insert_astrology_chart(&chart, orb_millidegrees)?;
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
