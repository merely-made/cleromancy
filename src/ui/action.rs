// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(feature = "analytic-ephemeris")]
use crate::AstrologyCalculationDraft;
use crate::{
    AstrologyChartDraft, ContextDraft, DerivedSelection, SelectionMode, SpreadTemplateDraft,
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
    #[cfg(feature = "analytic-ephemeris")]
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
