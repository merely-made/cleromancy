// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's retained local consultation surface.
//!
//! This module owns view state and Cambium rendering only. Storage remains in
//! [`crate::Consultation`]; the H2 worker will execute the actions emitted by
//! this surface away from the window thread.

pub mod native;
pub(crate) mod scenario;
mod state;
mod view;
pub(crate) mod worker;

pub use state::{
    ConsultationAction, ConsultationContext, ConsultationLayout, ConsultationScreen,
    ConsultationStatus, ConsultationUi,
};
pub use view::{ConsultationView, consultation_view};
