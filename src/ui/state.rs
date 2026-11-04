// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::action::{ConsultationAction, ConsultationContext, ConsultationLayout};
use super::chart_state::ChartState;
use super::journal_state::JournalState;
use super::screen::{self, ConsultationScreen};
use super::sky_state::SkyState;
#[cfg(feature = "analytic-ephemeris")]
use crate::AstrologyCalculationDraft;
use crate::{
    AstrologyChartDraft, ConsultationCatalog, ConsultationDetail, ContextDraft, DerivedSelection,
    ReceiptComparison, SelectionMode, SpreadTemplateDraft,
};
use cambium::{DisclosureState, RadioGroup, SelectState, SelectionState, TextInput};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConsultationStatus {
    #[default]
    Ready,
    SavingReading,
    SavingSetup,
    #[cfg(feature = "analytic-ephemeris")]
    CalculatingChart,
    SetupSaved(String),
    SavedNotice(String),
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
            Self::SavedNotice(message) => message.clone(),
            Self::SavingReading => "Saving reading".to_string(),
            Self::SavingSetup => "Saving setup".to_string(),
            #[cfg(feature = "analytic-ephemeris")]
            Self::CalculatingChart => "Calculating chart".to_string(),
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
            #[cfg(feature = "analytic-ephemeris")]
            Self::CalculatingChart => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsultationUi {
    pub(crate) ambient: super::ambient_state::AmbientUi,
    /// Inspection of another deck never changes the saved reading.
    pub(crate) card_preview: Option<(String, String)>,
    pub(super) catalog: ConsultationCatalog,
    pub(super) context_select: SelectState,
    pub(super) context_label: TextInput,
    pub(super) question: TextInput,
    pub(super) tags: TextInput,
    pub(super) additional_facts: TextInput,
    pub(super) place_consent: RadioGroup,
    pub(super) place_latitude: TextInput,
    pub(super) place_longitude: TextInput,
    pub(super) weather_consent: RadioGroup,
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
    pub(super) comparison_select: SelectState,
    pub(super) birth_notice: String,
    pub(super) natal_details: bool,
    pub(super) natal_selected: Option<String>,
    pub(super) birth_catalog: Option<mere_resident::personal_facts::PersonalFactsCatalog>,
    pub(super) natal_reflection: TextInput,
    pub(super) birth_subject: usize,
    pub(super) birth_revision: usize,
    pub(super) natal_name: TextInput,
    pub(super) natal_date: TextInput,
    pub(super) natal_time: TextInput,
    pub(super) natal_zone: TextInput,
    pub(super) natal_place: TextInput,
    pub(super) natal_latitude: TextInput,
    pub(super) natal_longitude: TextInput,
    pub(super) natal_source: TextInput,
    pub(super) natal_question: TextInput,
    pub(super) natal_fold: TextInput,
    pub(super) natal_house_system: Option<crate::natal_cap::HouseChoice>,
    pub(super) natal_eop: TextInput,
    pub(super) natal_aspects_enabled: bool,
    pub(super) natal_aspect_orbs: [TextInput; 5],
    pub(super) chart: ChartState,
    pub(super) journal: JournalState,
    pub(super) sky: SkyState,
    pub(super) workings: DisclosureState,
    pub(super) reading_focus: std::collections::BTreeMap<String, String>,
    /// The scene position under the pointer, keyed by its session. Local
    /// presentation state only; it never selects or writes anything.
    pub(super) position_hover: Option<(String, String)>,
    pub(super) layout_editor: DisclosureState,
    pub(super) reflection: TextInput,
    pub(super) detail: Option<ConsultationDetail>,
    pub(super) comparison: Option<ReceiptComparison>,
    /// The only source of truth for which surface is rendered. The screen is
    /// derived from it by [`screen::selected_screen`]; nothing else assigns a
    /// surface, and in particular no `present_*` method does.
    pub(super) surface_tabs: SelectionState,
    pub(super) status: ConsultationStatus,
    pub(super) error: Option<String>,
    pub(super) pending_action: Option<ConsultationAction>,
    pub(super) local_model_label: String,
    pub(super) expanded_inference_inputs: std::collections::BTreeSet<String>,
}

impl ConsultationUi {
    pub fn new(catalog: ConsultationCatalog) -> Self {
        Self {
            ambient: Default::default(),
            card_preview: None,
            catalog,
            context_select: SelectState::new(0).with_label("Context"),
            context_label: TextInput::default(),
            question: TextInput::default(),
            tags: TextInput::default(),
            additional_facts: TextInput::default(),
            place_consent: RadioGroup::new(0).with_label("Place for this draw"),
            place_latitude: TextInput::default(),
            place_longitude: TextInput::default(),
            weather_consent: RadioGroup::new(0).with_label("Weather for this draw"),
            field_select: SelectState::new(0).with_label("Stored field"),
            mode: RadioGroup::new(1).with_label("Selection mode"),
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
            astrology_orb: TextInput::new("1"),
            astrology_positions: TextInput::default(),
            comparison_select: SelectState::new(0).with_label("Compare with"),
            birth_notice: String::new(),
            natal_details: false,
            natal_selected: None,
            birth_catalog: None,
            natal_reflection: TextInput::default(),
            birth_subject: 0,
            birth_revision: 0,
            natal_name: TextInput::default(),
            natal_date: TextInput::default(),
            natal_time: TextInput::default(),
            natal_zone: TextInput::default(),
            natal_place: TextInput::default(),
            natal_latitude: TextInput::default(),
            natal_longitude: TextInput::default(),
            natal_source: TextInput::default(),
            natal_question: TextInput::default(),
            natal_fold: TextInput::default(),
            natal_house_system: None,
            natal_eop: TextInput::default(),
            natal_aspects_enabled: false,
            natal_aspect_orbs: ["8", "8", "6", "6", "4"].map(TextInput::new),
            chart: ChartState::new(),
            journal: JournalState::new(),
            sky: SkyState::new(),
            workings: DisclosureState::new("cleromancy-workings", "How it was done"),
            reading_focus: Default::default(),
            position_hover: None,
            layout_editor: DisclosureState::new("cleromancy-layout-editor", "Author a layout"),
            reflection: TextInput::default(),
            detail: None,
            comparison: None,
            surface_tabs: screen::surface_tab_state(),
            status: ConsultationStatus::Ready,
            error: None,
            pending_action: None,
            expanded_inference_inputs: Default::default(),
            local_model_label: {
                #[cfg(feature = "local-inference")]
                {
                    crate::inference::local::configured_label()
                        .unwrap_or_else(|| "No local model is configured.".into())
                }
                #[cfg(not(feature = "local-inference"))]
                {
                    "This build does not include a local model runtime.".into()
                }
            },
        }
    }

    pub fn catalog(&self) -> &ConsultationCatalog {
        &self.catalog
    }

    pub fn detail(&self) -> Option<&ConsultationDetail> {
        self.detail.as_ref()
    }

    /// The active surface, derived from the tab bar's selection. There is no
    /// stored screen field to fall out of step with it.
    pub fn screen(&self) -> ConsultationScreen {
        screen::selected_screen(&self.surface_tabs)
    }

    pub fn status(&self) -> &ConsultationStatus {
        &self.status
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub(crate) fn record_action(&mut self, action: Option<ConsultationAction>) {
        if matches!(
            &action,
            Some(
                ConsultationAction::RefreshBirthFacts
                    | ConsultationAction::CreateSubject { .. }
                    | ConsultationAction::SaveBirthFacts { .. }
                    | ConsultationAction::CreateNatal { .. }
                    | ConsultationAction::ReflectNatal { .. }
            )
        ) {
            if self.status.is_busy() {
                return;
            }
            self.status = ConsultationStatus::SavingSetup;
            self.error = None;
        }

        if matches!(action, Some(ConsultationAction::CreateNatal { .. })) {
            self.natal_selected = None;
        }
        self.pending_action = action;
    }

    /// Take the product command recorded by the last retained-view dispatch.
    /// The native host sends it to Cleromancy's persistence worker only after
    /// Cambium finishes the input turn.
    pub fn take_pending_action(&mut self) -> Option<ConsultationAction> {
        self.pending_action.take()
    }

    /// Open the paired stored chart for a canonical astrology-facts receipt.
    /// This only changes retained view state; it intentionally has no worker
    /// action and does not claim the chart caused the reading.
    pub(super) fn select_chart_facts(&mut self, facts_digest: &str) {
        if let Some(index) = self
            .catalog
            .astrology_charts
            .iter()
            .position(|stored| stored.facts.digest() == facts_digest)
        {
            self.chart.selected_chart.selected = index;
        }
    }

    pub(super) fn request_read(&mut self) -> Option<ConsultationAction> {
        let Some(field) = self.catalog.fields.get(self.field_select.selected) else {
            self.present_error("Choose a stored field before reading.");
            return None;
        };
        if field.rules == crate::UNIFORM_DIE_RULE
            && (self.mode.selected == 0 || self.layout.selected != 0)
        {
            self.present_error("Choose operating-system randomness or a public seed, and a single outcome, for this die roll.");
            return None;
        }
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
                    },
                }
            },
        };
        let layout = if self.layout.selected == 0 {
            ConsultationLayout::Single
        } else if self.layout.selected == 1 {
            ConsultationLayout::ThreeCard
        } else {
            let Some(template) = self
                .template_select
                .selected
                .checked_sub(1)
                .and_then(|index| self.catalog.spread_templates.get(index))
            else {
                self.present_error("Choose an authored layout before reading.");
                return None;
            };
            ConsultationLayout::Authored(template.id.clone())
        };
        if !matches!(layout, ConsultationLayout::Single) && mode == SelectionMode::Calculated {
            self.present_error(
                "Choose operating-system randomness or a public seed for a multi-position reading.",
            );
            return None;
        }
        let astrology_facts_digest = self
            .astrology_facts_select
            .selected
            .checked_sub(1)
            .and_then(|index| self.catalog.astrology_facts.get(index))
            .map(|facts| facts.digest());
        let situation = match self.situation_preferences() {
            Ok(value) => value,
            Err(error) => {
                self.present_error(error.to_string());
                return None;
            },
        };
        let action = ConsultationAction::Read {
            context,
            field_digest: field.digest(),
            mode,
            derivation,
            layout,
            astrology_facts_digest,
            situation,
        };
        self.error = None;
        self.status = ConsultationStatus::SavingReading;
        Some(action)
    }

    pub(crate) fn present_die(&mut self, catalog: ConsultationCatalog, digest: &str) {
        self.replace_catalog(catalog);
        // Another admitted writer may withdraw an unused field after the ACK.
        let Some(index) = self
            .catalog
            .fields
            .iter()
            .position(|f| f.digest() == digest)
        else {
            self.present_error(
                "The die is no longer available in this session. Add it again before rolling.",
            );
            return;
        };
        self.clear_selection();
        self.card_preview = None;
        self.error = None;
        self.field_select.selected = index;
        self.mode.selected = 1;
        self.layout.selected = 0;
        self.status = ConsultationStatus::Ready;
    }

    pub(super) fn request_die(&mut self) -> Option<ConsultationAction> {
        if self.status.is_busy() {
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::InstallSixSidedDie)
    }

    pub(super) fn request_typed_decks(&mut self) -> Option<ConsultationAction> {
        if self.status.is_busy() {
            return None;
        }
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::InstallTarotDecks)
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

    fn chart_angles(&self) -> Result<(String, String, String), String> {
        Ok((
            super::chart_input::degrees(self.astrology_latitude.text(), 6)?,
            super::chart_input::degrees(self.astrology_longitude.text(), 6)?,
            super::chart_input::degrees(self.astrology_orb.text(), 3)?,
        ))
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
        let (latitude, longitude, orb) = match self.chart_angles() {
            Ok(values) => values,
            Err(error) => {
                self.present_error(error);
                return None;
            },
        };
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::SaveAstrologyChart {
            draft: AstrologyChartDraft {
                algorithm: self.astrology_algorithm.text().to_string(),
                engine: self.astrology_engine.text().to_string(),
                ephemeris: self.astrology_ephemeris.text().to_string(),
                instant_utc: super::chart_input::instant(self.astrology_instant_utc.text()),
                latitude_microdegrees: latitude,
                longitude_microdegrees: longitude,
                orb_millidegrees: orb,
                positions: self.astrology_positions.text().to_string(),
            },
        })
    }

    #[cfg(feature = "analytic-ephemeris")]
    pub(super) fn request_calculated_astrology_chart(&mut self) -> Option<ConsultationAction> {
        let (latitude, longitude, orb) = match self.chart_angles() {
            Ok(values) => values,
            Err(error) => {
                self.present_error(error);
                return None;
            },
        };
        let draft = AstrologyCalculationDraft {
            instant_utc: super::chart_input::instant(self.astrology_instant_utc.text()),
            latitude_microdegrees: latitude,
            longitude_microdegrees: longitude,
            orb_millidegrees: orb,
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
}

#[cfg(test)]
mod refresh_tests;

impl ConsultationUi {
    pub fn situation_preferences(
        &self,
    ) -> Result<crate::situation::SituationPreferences, crate::situation::SituationError> {
        let place = if self.place_consent.selected == 1 {
            Some(crate::situation::ReadingPlace::from_degrees(
                self.place_latitude.text(),
                self.place_longitude.text(),
            )?)
        } else {
            None
        };
        let value = crate::situation::SituationPreferences {
            place,
            weather: self.weather_consent.selected == 1,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn set_place_for_draw(&mut self, place: Option<crate::situation::ReadingPlace>) {
        self.place_consent.selected = usize::from(place.is_some());
        if let Some(place) = place {
            self.place_latitude =
                TextInput::new(format!("{:.6}", place.latitude_microdegrees as f64 / 1e6));
            self.place_longitude =
                TextInput::new(format!("{:.6}", place.longitude_microdegrees as f64 / 1e6));
        }
    }
    pub fn set_weather_for_draw(&mut self, enabled: bool) {
        self.weather_consent.selected = usize::from(enabled);
    }
}

mod presentation;
