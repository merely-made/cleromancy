// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View-local selection and disclosure state for stored sky timelines.

use cambium::{DisclosureState, SelectState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SkyState {
    pub(super) selected_day: SelectState,
    pub(super) policy_and_provenance: DisclosureState,
}

impl SkyState {
    pub(super) fn new() -> Self {
        Self {
            selected_day: SelectState::new(0).with_label("Stored civil day"),
            policy_and_provenance: DisclosureState::new(
                "cleromancy-sky-policy-and-provenance",
                "Stored numerical policy and event provenance",
            ),
        }
    }
}
