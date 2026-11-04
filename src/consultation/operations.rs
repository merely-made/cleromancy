// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

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

    /// Return the owned candidate; persistence remains the caller's authority.
    pub fn into_host(self) -> CleromancyHost<B> {
        self.host
    }

    pub(crate) fn host_mut_candidate(&mut self) -> &mut CleromancyHost<B> {
        &mut self.host
    }

    pub fn is_faulted(&self) -> bool {
        self.faulted
    }

    pub fn catalog(&self) -> Result<ConsultationCatalog, ConsultationError> {
        let astrology_facts = self.host.astrology_facts()?;
        let astrology_charts = astrology_facts
            .iter()
            .map(|facts| {
                Ok(StoredAstrologyChart {
                    facts: facts.clone(),
                    chart: self.host.astrology_chart_for_digest(&facts.chart_digest)?,
                })
            })
            .collect::<Result<Vec<_>, ConsultationError>>()?;
        Ok(ConsultationCatalog {
            natal_readings: self.host.natal_readings()?,
            natal_reflections: self
                .host
                .natal_readings()?
                .iter()
                .map(|r| self.host.natal_reflections(&r.id))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),
            ambient_memory: self.host.ambient_memories()?,
            contexts: self.host.contexts()?,
            fields: self.host.fields()?,
            spread_templates: self.host.spread_templates()?,
            astrology_facts,
            astrology_charts,
            sky_day_facts: self.host.sky_day_facts()?,
            session_summaries: self.host.session_summaries()?,
        })
    }

    pub fn detail(&self, session_id: &str) -> Result<ConsultationDetail, ConsultationError> {
        self.detail_projection(session_id, true)
    }

    /// C4 consumes reading facts, not generated C7 records. Keep its source
    /// projection independent; complete resident validation verifies every C7
    /// record separately before any candidate is admitted.
    pub(crate) fn ambient_detail(
        &self,
        session_id: &str,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.detail_projection(session_id, false)
    }
    pub(super) fn detail_projection(
        &self,
        session_id: &str,
        include_inferences: bool,
    ) -> Result<ConsultationDetail, ConsultationError> {
        let session = self.host.reading_session_for_id(session_id)?;
        let context = self.host.context_for_digest(&session.context_digest)?;
        let field = self.host.field_for_digest(&session.field_digest)?;
        let readings = self.host.replay_session(&session)?;
        let reflections = self.host.reflections_for_session(&session.id)?;
        let concurrences = self.host.concurrences_for_session(&session.id)?;
        let spread_template = self.host.spread_template_for_session(&session.id)?;
        let situation = self.host.situation_for_session(&session.id)?;
        let inferences = if include_inferences {
            self.host.inferences_for_session(session_id)?
        } else {
            Vec::new()
        };
        let kept_inferences = inferences
            .iter()
            .filter_map(|r| {
                let key = self
                    .host
                    .graph()
                    .get_node_by_url(&format!("cleromancy://inference/{}", r.digest()))?
                    .0;
                matches!(
                    pandect::memory_levels::level_of(self.host.graph(), key),
                    pandect::memory_levels::MemoryLevel::LongTerm
                )
                .then(|| r.digest())
            })
            .collect();
        Ok(ConsultationDetail {
            session,
            context,
            field,
            readings,
            reflections,
            concurrences,
            spread_template,
            situation,
            inferences,
            kept_inferences,
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

    /// Install complete immutable deck snapshots through the existing owner action.
    pub async fn install_tarot_deck(&mut self, id: &str) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let deck = crate::cards::builtin_decks()
            .iter()
            .find(|deck| deck.id == id)
            .ok_or_else(|| HostError::Reading(ReadingError::InvalidDeck(id.into())))?;
        let field = deck
            .field(TarotQualification::Contextual)
            .map_err(HostError::from)?;
        let digest = field.digest();
        self.host.insert_field(&field)?;
        self.persist(unix_time_secs()?).await?;
        Ok(digest)
    }

    pub async fn install_typed_tarot(&mut self) -> Result<Vec<String>, ConsultationError> {
        self.install_typed_tarot_at(unix_time_secs()?).await
    }

    pub async fn install_typed_tarot_at(
        &mut self,
        saved_at_secs: u64,
    ) -> Result<Vec<String>, ConsultationError> {
        self.ensure_writable()?;
        let mut digests = Vec::new();
        for deck in crate::cards::builtin_decks() {
            let field = deck
                .field(TarotQualification::Contextual)
                .map_err(HostError::from)?;
            digests.push(field.digest());
            self.host.insert_field(&field)?;
        }
        self.persist(saved_at_secs).await?;
        Ok(digests)
    }

    /// Add the first dice surface using the existing uniform roll field and replay rules.
    pub async fn install_six_sided_die(&mut self) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let mut field = crate::RollIntentPayload::new(6)
            .with_label("Six-sided die")
            .field();
        field.position_meanings = Some(
            serde_json::from_str(include_str!("../content/die-position-v1.json"))
                .expect("bundled die position"),
        );
        let digest = field.digest();
        self.host.insert_field(&field)?;
        self.persist(unix_time_secs()?).await?;
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
            SelectionMode::Cast if field.is_tarot() => {
                ReadingEngine::shuffle_with(&context, &field, 1, entropy)?.remove(0)
            },
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
        selection.validate()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let reading = if field.is_tarot() {
            ReadingEngine::shuffle_derived(&context, &field, 1, &selection.seed, &selection.domain)?
                .remove(0)
        } else {
            ReadingEngine::derive(&context, &field, selection)?
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

    /// Public-seed physical-deck spread; the session nonce identifies the
    /// occasion independently of its deterministic card order.
    pub async fn read_seeded_spread(
        &mut self,
        context_digest: &str,
        field_digest: &str,
        template_id: Option<&str>,
        selection: &DerivedSelection,
    ) -> Result<ConsultationDetail, ConsultationError> {
        self.ensure_writable()?;
        selection.validate()?;
        let context = self.host.context_for_digest(context_digest)?;
        let field = self.host.field_for_digest(field_digest)?;
        let at = unix_time_ms()?;
        let mut entropy = OsEntropy;
        let session = if let Some(id) = template_id {
            let template = self.host.spread_template_for_id(id)?;
            self.host
                .record_spread_derived_at_with_entropy(
                    &context,
                    &field,
                    &template,
                    &selection.seed,
                    &selection.domain,
                    at,
                    None,
                    &mut entropy,
                )?
                .0
        } else {
            self.host
                .record_three_card_spread_derived_at_with_entropy(
                    &context,
                    &field,
                    &selection.seed,
                    &selection.domain,
                    at,
                    None,
                    &mut entropy,
                )?
                .0
        };
        self.persist(at / 1000).await?;
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

    pub(crate) fn ensure_writable(&self) -> Result<(), ConsultationError> {
        if self.faulted {
            Err(ConsultationError::Faulted)
        } else {
            Ok(())
        }
    }

    pub(super) async fn persist(&mut self, saved_at_secs: u64) -> Result<(), ConsultationError> {
        if let Err(error) = self.host.persist(saved_at_secs).await {
            self.faulted = true;
            return Err(error.into());
        }
        Ok(())
    }
}
