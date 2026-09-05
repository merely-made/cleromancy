// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View-local controls for the journal reader.

use std::time::{SystemTime, UNIX_EPOCH};

use cambium::{SelectState, TextInput};

use crate::JournalDateRange;

/// Filter and paging state. It is deliberately outside `state.rs`: archive
/// policy is local UI state, never a product action or a graph mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JournalState {
    pub(super) tag_filter: TextInput,
    pub(super) date_range: SelectState,
    pub(super) page_index: usize,
    pub(super) now_ms: u64,
}

impl JournalState {
    pub(super) fn new() -> Self {
        Self {
            tag_filter: TextInput::default(),
            date_range: SelectState::new(0).with_label("Date range"),
            page_index: 0,
            now_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| {
                    duration.as_millis().try_into().unwrap_or(u64::MAX)
                }),
        }
    }

    pub(super) fn selected_date_range(&self) -> JournalDateRange {
        match self.date_range.selected {
            1 => JournalDateRange::Past7Days,
            2 => JournalDateRange::Past30Days,
            _ => JournalDateRange::AnyTime,
        }
    }

    pub(super) fn clear(&mut self) {
        self.tag_filter = TextInput::default();
        self.date_range.selected = 0;
        self.page_index = 0;
    }

    pub(super) fn newer(&mut self, page_index: usize) {
        self.page_index = page_index.saturating_sub(1);
    }

    pub(super) fn older(&mut self, page_index: usize, page_count: usize) {
        self.page_index = (page_index + 1).min(page_count.saturating_sub(1));
    }
}
