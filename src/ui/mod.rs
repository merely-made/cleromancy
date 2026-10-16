// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's retained local consultation surface.
//!
//! This module owns view state and Cambium rendering only. The worker sends
//! product transformations and shared lifecycle actions through the admitted
//! resident transaction boundary away from the window thread.

mod action;
mod ambient_state;
mod chart_state;
mod journal_state;
pub mod native;
pub(crate) mod resident;
pub(crate) mod scenario;
mod screen;
mod sky_state;
mod state;
mod view;
pub(crate) mod worker;

pub use action::{ConsultationAction, ConsultationContext, ConsultationLayout};
pub use screen::ConsultationScreen;
pub use state::{ConsultationStatus, ConsultationUi};
pub use view::{ConsultationView, consultation_view};

mod chart_input;
