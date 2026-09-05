// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's retained local consultation surface.
//!
//! This module owns view state and Cambium rendering only. Storage remains in
//! [`crate::Consultation`]; the H2 worker will execute the actions emitted by
//! this surface away from the window thread.

mod action;
mod journal_state;
pub mod native;
pub(crate) mod scenario;
mod screen;
mod state;
mod view;
pub(crate) mod worker;

pub use action::{ConsultationAction, ConsultationContext, ConsultationLayout};
pub use screen::ConsultationScreen;
pub use state::{ConsultationStatus, ConsultationUi};
pub use view::{ConsultationView, consultation_view};
