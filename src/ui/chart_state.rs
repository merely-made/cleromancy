// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View-local choice of one stored chart. It never becomes a product action.

use cambium::SelectState;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChartState {
    pub(super) selected_chart: SelectState,
    pub(super) selected_aspect: SelectState,
}

impl ChartState {
    pub(super) fn new() -> Self {
        Self {
            selected_chart: SelectState::new(0).with_label("Stored chart"),
            selected_aspect: SelectState::new(0).with_label("Aspect relation"),
        }
    }
}
