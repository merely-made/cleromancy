// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{DisclosureState, RadioGroup, SelectState, TextInput};

#[cfg(feature = "ephemeris")]
use crate::{AstrologyCalculationDraft, EphemerisStatus};
use crate::{
    AstrologyChartDraft, ConsultationCatalog, ConsultationDetail, ContextDraft, DerivedSelection,
    ReceiptComparison, SelectionMode, SpreadTemplateDraft,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsultationContext {
    Existing(String),
    New(ContextDraft),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConsultationLayout {
    #[default]
    Single,
    ThreeCard,
    Authored(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsultationAction {
    Read {
        context: ConsultationContext,
        field_digest: String,
        mode: SelectionMode,
        derivation: Option<DerivedSelection>,
        layout: ConsultationLayout,
        astrology_facts_digest: Option<String>,
    },
    SaveSpreadTemplate {
        draft: SpreadTemplateDraft,
    },
    SaveAstrologyChart {
        draft: AstrologyChartDraft,
    },
    #[cfg(feature = "ephemeris")]
    InstallEphemeris,
    #[cfg(feature = "ephemeris")]
    CalculateAstrologyChart {
        draft: AstrologyCalculationDraft,
    },
    SaveReflection {
        session_id: String,
        body: String,
    },
    SelectSession {
        session_id: String,
    },
    CompareSessions {
        left_session_id: String,
        right_session_id: String,
    },
}

impl cambium::Action for ConsultationAction {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsultationScreen {
    #[default]
    Consultation,
    Reading,
    Journal,
}

impl ConsultationScreen {
    pub fn key(self) -> &'static str {
        match self {
            Self::Consultation => "consultation",
            Self::Reading => "reading",
            Self::Journal => "journal",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConsultationStatus {
    #[default]
    Ready,
    SavingReading,
    SavingSetup,
    #[cfg(feature = "ephemeris")]
    InstallingEphemeris,
    #[cfg(feature = "ephemeris")]
    CalculatingChart,
    #[cfg(feature = "ephemeris")]
    EphemerisReady,
    SetupSaved(String),
    ReadingSaved(String),
    SavingReflection,
    ReflectionSaved(String),
    LoadingSession,
    ViewingSession(String),
    ComparingReceipts,
    ViewingComparison(String),
}

impl ConsultationStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Ready => "Ready".to_string(),
            Self::SavingReading => "Saving reading".to_string(),
            Self::SavingSetup => "Saving setup".to_string(),
            #[cfg(feature = "ephemeris")]
            Self::InstallingEphemeris => "Installing NASA/JPL ephemeris".to_string(),
            #[cfg(feature = "ephemeris")]
            Self::CalculatingChart => "Calculating chart".to_string(),
            #[cfg(feature = "ephemeris")]
            Self::EphemerisReady => "NASA/JPL ephemeris ready".to_string(),
            Self::SetupSaved(id) => format!("Saved: {id}"),
            Self::ReadingSaved(id) => format!("Reading saved: {id}"),
            Self::SavingReflection => "Saving reflection".to_string(),
            Self::ReflectionSaved(id) => format!("Reflection saved: {id}"),
            Self::LoadingSession => "Loading session".to_string(),
            Self::ViewingSession(id) => format!("Viewing session: {id}"),
            Self::ComparingReceipts => "Comparing receipts".to_string(),
            Self::ViewingComparison(id) => format!("Comparing receipts with: {id}"),
        }
    }

    pub fn is_busy(&self) -> bool {
        match self {
            Self::SavingReading
            | Self::SavingSetup
            | Self::SavingReflection
            | Self::LoadingSession
            | Self::ComparingReceipts => true,
            #[cfg(feature = "ephemeris")]
            Self::InstallingEphemeris | Self::CalculatingChart => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsultationUi {
    pub(super) catalog: ConsultationCatalog,
    pub(super) context_select: SelectState,
    pub(super) context_label: TextInput,
    pub(super) question: TextInput,
    pub(super) tags: TextInput,
    pub(super) additional_facts: TextInput,
    pub(super) field_select: SelectState,
    pub(super) mode: RadioGroup,
    pub(super) derived_seed: TextInput,
    pub(super) derived_domain: TextInput,
    pub(super) layout: RadioGroup,
    pub(super) template_select: SelectState,
    pub(super) template_label: TextInput,
    pub(super) template_positions: TextInput,
    pub(super) template_relations: TextInput,
    pub(super) astrology_facts_select: SelectState,
    pub(super) astrology_algorithm: TextInput,
    pub(super) astrology_engine: TextInput,
    pub(super) astrology_ephemeris: TextInput,
    pub(super) astrology_instant_utc: TextInput,
    pub(super) astrology_latitude: TextInput,
    pub(super) astrology_longitude: TextInput,
    pub(super) astrology_orb: TextInput,
    pub(super) astrology_positions: TextInput,
    #[cfg(feature = "ephemeris")]
    pub(super) ephemeris_status: EphemerisStatus,
    pub(super) comparison_select: SelectState,
    pub(super) workings: DisclosureState,
    pub(super) reflection: TextInput,
    pub(super) detail: Option<ConsultationDetail>,
    pub(super) comparison: Option<ReceiptComparison>,
    pub(super) screen: ConsultationScreen,
    pub(super) status: ConsultationStatus,
    pub(super) error: Option<String>,
}

impl ConsultationUi {
    pub fn new(catalog: ConsultationCatalog) -> Self {
        Self {
            catalog,
            context_select: SelectState::new(0).with_label("Context"),
            context_label: TextInput::default(),
            question: TextInput::default(),
            tags: TextInput::default(),
            additional_facts: TextInput::default(),
            field_select: SelectState::new(0).with_label("Stored field"),
            mode: RadioGroup::new(0).with_label("Selection mode"),
            derived_seed: TextInput::default(),
            derived_domain: TextInput::default(),
            layout: RadioGroup::new(0).with_label("Reading shape"),
            template_select: SelectState::new(0).with_label("Authored layout"),
            template_label: TextInput::default(),
            template_positions: TextInput::default(),
            template_relations: TextInput::default(),
            astrology_facts_select: SelectState::new(0).with_label("Astrology facts"),
            astrology_algorithm: TextInput::default(),
            astrology_engine: TextInput::default(),
            astrology_ephemeris: TextInput::default(),
            astrology_instant_utc: TextInput::default(),
            astrology_latitude: TextInput::default(),
            astrology_longitude: TextInput::default(),
            astrology_orb: TextInput::new("1000"),
            astrology_positions: TextInput::default(),
            #[cfg(feature = "ephemeris")]
            ephemeris_status: EphemerisStatus::Missing,
            comparison_select: SelectState::new(0).with_label("Compare with"),
            workings: DisclosureState::new("cleromancy-workings", "Workings"),
            reflection: TextInput::default(),
            detail: None,
            comparison: None,
            screen: ConsultationScreen::Consultation,
            status: ConsultationStatus::Ready,
            error: None,
        }
    }

    pub fn catalog(&self) -> &ConsultationCatalog {
        &self.catalog
    }

    pub fn detail(&self) -> Option<&ConsultationDetail> {
        self.detail.as_ref()
    }

    pub fn screen(&self) -> ConsultationScreen {
        self.screen
    }

    pub fn status(&self) -> &ConsultationStatus {
        &self.status
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Replace picker and history values after the persistence worker commits a
    /// transaction. Keep the current selections where they still name a live
    /// value; the worker is the authority for what remains available.
    pub(crate) fn replace_catalog(&mut self, catalog: ConsultationCatalog) {
        self.context_select.selected = self.context_select.selected.min(catalog.contexts.len());
        self.field_select.selected = self
            .field_select
            .selected
            .min(catalog.fields.len().saturating_sub(1));
        self.comparison_select.selected = self.comparison_select.selected.min(
            catalog
                .sessions
                .len()
                .saturating_sub(if self.detail.is_some() { 1 } else { 0 }),
        );
        self.template_select.selected = self
            .template_select
            .selected
            .min(catalog.spread_templates.len());
        self.astrology_facts_select.selected = self
            .astrology_facts_select
            .selected
            .min(catalog.astrology_facts.len());
        self.catalog = catalog;
        self.error = None;
        if self.status.is_busy() {
            self.status = ConsultationStatus::Ready;
        }
    }

    pub fn present_reading(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let session_id = detail.session.id.clone();
        self.adopt_detail(catalog, detail);
        self.workings.expanded = false;
        self.reflection = TextInput::default();
        self.comparison = None;
        self.screen = ConsultationScreen::Reading;
        self.status = ConsultationStatus::ReadingSaved(session_id);
    }

    pub fn present_reflection(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let reflection_id = detail
            .reflections
            .first()
            .map(|reflection| reflection.id.clone())
            .unwrap_or_else(|| detail.session.id.clone());
        self.adopt_detail(catalog, detail);
        self.reflection = TextInput::default();
        self.screen = ConsultationScreen::Journal;
        self.status = ConsultationStatus::ReflectionSaved(reflection_id);
    }

    pub fn present_session(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        let session_id = detail.session.id.clone();
        self.adopt_detail(catalog, detail);
        self.comparison = None;
        self.screen = ConsultationScreen::Journal;
        self.status = ConsultationStatus::ViewingSession(session_id);
    }

    pub fn present_comparison(
        &mut self,
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
        comparison: ReceiptComparison,
    ) {
        let compared_session = comparison.right_session_id.clone();
        self.adopt_detail(catalog, detail);
        self.comparison_select.selected = self
            .comparison_candidates()
            .iter()
            .position(|session_id| session_id == &compared_session)
            .map_or(0, |index| index + 1);
        self.comparison = Some(comparison);
        self.screen = ConsultationScreen::Journal;
        self.status = ConsultationStatus::ViewingComparison(compared_session);
    }

    pub fn present_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.status = ConsultationStatus::Ready;
    }

    #[cfg(feature = "ephemeris")]
    pub(crate) fn present_ephemeris_status(&mut self, status: EphemerisStatus) {
        let ready = status.is_ready();
        self.ephemeris_status = status;
        self.error = None;
        self.status = if ready {
            ConsultationStatus::EphemerisReady
        } else {
            ConsultationStatus::Ready
        };
    }

    #[cfg(feature = "ephemeris")]
    pub(crate) fn present_calculated_chart(
        &mut self,
        catalog: ConsultationCatalog,
        facts_digest: String,
    ) {
        self.replace_catalog(catalog);
        self.astrology_facts_select.selected = self
            .catalog
            .astrology_facts
            .iter()
            .position(|facts| facts.digest() == facts_digest)
            .map_or(0, |index| index + 1);
        self.status = ConsultationStatus::SetupSaved(format!(
            "chart {}",
            facts_digest.get(..12).unwrap_or(&facts_digest)
        ));
    }

    pub(super) fn request_read(&mut self) -> Option<ConsultationAction> {
        let Some(field) = self.catalog.fields.get(self.field_select.selected) else {
            self.present_error("Choose a stored field before reading.");
            return None;
        };
        let context = if self.context_select.selected == 0 {
            if self.context_label.text().trim().is_empty() || self.question.text().trim().is_empty()
            {
                self.present_error("Enter a context label and question before reading.");
                return None;
            }
            ConsultationContext::New(
                ContextDraft::new(
                    self.context_label.text(),
                    self.question.text(),
                    self.tags.text(),
                )
                .with_additional_facts(self.additional_facts.text()),
            )
        } else {
            let Some(context) = self.catalog.contexts.get(self.context_select.selected - 1) else {
                self.present_error("Choose a stored context before reading.");
                return None;
            };
            ConsultationContext::Existing(context.digest())
        };
        let (mode, derivation) = match self.mode.selected {
            0 => (SelectionMode::Calculated, None),
            1 => (SelectionMode::Cast, None),
            _ => {
                match DerivedSelection::new(self.derived_seed.text(), self.derived_domain.text()) {
                    Ok(selection) => (SelectionMode::Derived, Some(selection)),
                    Err(error) => {
                        self.present_error(error.to_string());
                        return None;
                    }
                }
            }
        };
        let layout = if self.layout.selected == 0 {
            ConsultationLayout::Single
        } else if self.layout.selected == 1 {
            ConsultationLayout::ThreeCard
        } else {
            let Some(template) = self
                .catalog
                .spread_templates
                .get(self.template_select.selected.saturating_sub(1))
            else {
                self.present_error("Choose an authored layout before reading.");
                return None;
            };
            ConsultationLayout::Authored(template.id.clone())
        };
        if !matches!(layout, ConsultationLayout::Single) && mode != SelectionMode::Cast {
            self.present_error("Multi-position readings are always cast.");
            return None;
        }
        let astrology_facts_digest = self
            .catalog
            .astrology_facts
            .get(self.astrology_facts_select.selected.saturating_sub(1))
            .map(|facts| facts.digest());
        let action = ConsultationAction::Read {
            context,
            field_digest: field.digest(),
            mode,
            derivation,
            layout,
            astrology_facts_digest,
        };
        self.error = None;
        self.status = ConsultationStatus::SavingReading;
        Some(action)
    }

    pub(super) fn request_spread_template(&mut self) -> Option<ConsultationAction> {
        if self.template_label.text().trim().is_empty()
            || self.template_positions.text().trim().is_empty()
        {
            self.present_error("Enter a layout label and at least one position.");
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::SaveSpreadTemplate {
            draft: SpreadTemplateDraft::new(
                self.template_label.text(),
                self.template_positions.text(),
            )
            .with_relations(self.template_relations.text()),
        })
    }

    pub(super) fn request_astrology_chart(&mut self) -> Option<ConsultationAction> {
        if self.astrology_algorithm.text().trim().is_empty()
            || self.astrology_engine.text().trim().is_empty()
            || self.astrology_ephemeris.text().trim().is_empty()
            || self.astrology_instant_utc.text().trim().is_empty()
            || self.astrology_positions.text().trim().is_empty()
        {
            self.present_error("Enter the chart source, UTC instant, and at least one position.");
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::SaveAstrologyChart {
            draft: AstrologyChartDraft {
                algorithm: self.astrology_algorithm.text().to_string(),
                engine: self.astrology_engine.text().to_string(),
                ephemeris: self.astrology_ephemeris.text().to_string(),
                instant_utc: self.astrology_instant_utc.text().to_string(),
                latitude_microdegrees: self.astrology_latitude.text().to_string(),
                longitude_microdegrees: self.astrology_longitude.text().to_string(),
                orb_millidegrees: self.astrology_orb.text().to_string(),
                positions: self.astrology_positions.text().to_string(),
            },
        })
    }

    #[cfg(feature = "ephemeris")]
    pub(super) fn request_ephemeris_install(&mut self) -> Option<ConsultationAction> {
        if self.ephemeris_status.is_ready() {
            self.present_error("The verified NASA/JPL ephemeris is already installed.");
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::InstallingEphemeris;
        Some(ConsultationAction::InstallEphemeris)
    }

    #[cfg(feature = "ephemeris")]
    pub(super) fn request_calculated_astrology_chart(&mut self) -> Option<ConsultationAction> {
        if !self.ephemeris_status.is_ready() {
            self.present_error(
                "Install the verified NASA/JPL ephemeris before calculating a chart.",
            );
            return None;
        }
        let draft = AstrologyCalculationDraft {
            instant_utc: self.astrology_instant_utc.text().to_string(),
            latitude_microdegrees: self.astrology_latitude.text().to_string(),
            longitude_microdegrees: self.astrology_longitude.text().to_string(),
            orb_millidegrees: self.astrology_orb.text().to_string(),
        };
        if let Err(error) = draft.clone().into_moment_and_orb() {
            self.present_error(error.to_string());
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::CalculatingChart;
        Some(ConsultationAction::CalculateAstrologyChart { draft })
    }

    pub(super) fn request_reflection(&mut self) -> Option<ConsultationAction> {
        let Some(detail) = self.detail.as_ref() else {
            self.present_error("Make or select a reading before saving a reflection.");
            return None;
        };
        let body = self.reflection.text().trim();
        if body.is_empty() {
            self.present_error("Enter a reflection before saving it.");
            return None;
        }
        let action = ConsultationAction::SaveReflection {
            session_id: detail.session.id.clone(),
            body: body.to_string(),
        };
        self.error = None;
        self.status = ConsultationStatus::SavingReflection;
        Some(action)
    }

    pub(super) fn request_session(&mut self, session_id: String) -> ConsultationAction {
        self.error = None;
        self.status = ConsultationStatus::LoadingSession;
        ConsultationAction::SelectSession { session_id }
    }

    pub(super) fn request_comparison(&mut self) -> Option<ConsultationAction> {
        let Some(detail) = self.detail.as_ref() else {
            self.present_error("Open a reading before comparing its receipt.");
            return None;
        };
        let Some(right_session_id) = self
            .comparison_candidates()
            .get(self.comparison_select.selected.saturating_sub(1))
            .cloned()
        else {
            self.present_error("Choose another saved session to compare.");
            return None;
        };
        self.error = None;
        self.status = ConsultationStatus::ComparingReceipts;
        Some(ConsultationAction::CompareSessions {
            left_session_id: detail.session.id.clone(),
            right_session_id,
        })
    }

    fn adopt_detail(&mut self, catalog: ConsultationCatalog, detail: ConsultationDetail) {
        self.context_select.selected = catalog
            .contexts
            .iter()
            .position(|context| context.digest() == detail.session.context_digest)
            .map_or(0, |index| index + 1);
        self.field_select.selected = catalog
            .fields
            .iter()
            .position(|field| field.digest() == detail.session.field_digest)
            .unwrap_or(0);
        self.mode.selected = match detail.readings.first().map(|reading| reading.receipt.mode) {
            Some(SelectionMode::Cast) => 1,
            Some(SelectionMode::Derived) => 2,
            Some(SelectionMode::Calculated) | None => 0,
        };
        let derivation = detail
            .readings
            .first()
            .and_then(|reading| reading.receipt.derivation.as_ref());
        self.derived_seed = TextInput::new(
            derivation
                .map(|selection| selection.seed.clone())
                .unwrap_or_default(),
        );
        self.derived_domain = TextInput::new(
            derivation
                .map(|selection| selection.domain.clone())
                .unwrap_or_default(),
        );
        self.layout.selected = if detail
            .session
            .placements
            .iter()
            .map(|placement| placement.position.as_str())
            .eq(["foundation", "tension", "next_step"])
        {
            1
        } else {
            0
        };
        self.context_label = TextInput::new(detail.context.label.clone());
        self.question = TextInput::new(
            detail
                .context
                .facts
                .get("question")
                .cloned()
                .unwrap_or_default(),
        );
        self.tags = TextInput::new(
            detail
                .context
                .tags
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        );
        self.additional_facts = TextInput::new(
            detail
                .context
                .facts
                .iter()
                .filter(|(name, _)| name.as_str() != "question")
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        self.catalog = catalog;
        self.detail = Some(detail);
        self.comparison_select.selected = 0;
        self.error = None;
    }

    fn comparison_candidates(&self) -> Vec<String> {
        let selected_session = self.detail.as_ref().map(|detail| &detail.session.id);
        self.catalog
            .sessions
            .iter()
            .filter(|session| Some(&session.id) != selected_session)
            .map(|session| session.id.clone())
            .collect()
    }
}
