// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pure archive policy over cheap session summaries.

use crate::SessionSummary;

/// The archive's fixed DOM and interaction bound. It is product policy, not a
/// layout accident: one page is rendered at a time even for a large journal.
pub const JOURNAL_PAGE_SIZE: usize = 20;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

/// The journal's selectable UTC-duration date ranges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JournalDateRange {
    #[default]
    AnyTime,
    Past7Days,
    Past30Days,
}

impl JournalDateRange {
    pub fn label(self) -> &'static str {
        match self {
            Self::AnyTime => "Any time",
            Self::Past7Days => "Past 7 days",
            Self::Past30Days => "Past 30 days",
        }
    }

    fn includes(self, created_at_ms: u64, now_ms: u64) -> bool {
        let days = match self {
            Self::AnyTime => return true,
            Self::Past7Days => 7,
            Self::Past30Days => 30,
        };
        created_at_ms >= now_ms.saturating_sub(days * DAY_MS) && created_at_ms <= now_ms
    }
}

/// Format a created instant with elapsed UTC-duration buckets. This takes its
/// clock explicitly so rendering and tests never depend on local time zones.
pub fn format_relative_date(created_at_ms: u64, now_ms: u64) -> String {
    if created_at_ms > now_ms {
        return "in the future".to_string();
    }
    let elapsed_ms = now_ms.saturating_sub(created_at_ms);
    if elapsed_ms < DAY_MS {
        "today".to_string()
    } else if elapsed_ms < 2 * DAY_MS {
        "yesterday".to_string()
    } else {
        format!("{} days ago", elapsed_ms / DAY_MS)
    }
}

/// Filter summaries without graph access. A non-empty tag filter matches one
/// stored tag case-insensitively; date cutoffs include their exact boundary.
pub fn filter_journal_summaries<'a>(
    summaries: &'a [SessionSummary],
    tag_filter: &str,
    date_range: JournalDateRange,
    now_ms: u64,
) -> Vec<&'a SessionSummary> {
    let tag_filter = tag_filter.trim();
    summaries
        .iter()
        .filter(|summary| {
            (tag_filter.is_empty()
                || summary
                    .tags
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(tag_filter)))
                && date_range.includes(summary.created_at_ms, now_ms)
        })
        .collect()
}

/// One bounded archive page plus enough metadata to state the bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPage<'a> {
    pub summaries: Vec<&'a SessionSummary>,
    pub page_index: usize,
    pub page_count: usize,
    pub total: usize,
}

impl JournalPage<'_> {
    pub fn start(&self) -> usize {
        if self.total == 0 {
            0
        } else {
            self.page_index * JOURNAL_PAGE_SIZE + 1
        }
    }

    pub fn end(&self) -> usize {
        self.start() + self.summaries.len().saturating_sub(1)
    }
}

/// Apply filtering then take one archive page. Out-of-range page requests
/// clamp to the final page, which keeps a narrowed filter legible.
pub fn journal_page<'a>(
    summaries: &'a [SessionSummary],
    tag_filter: &str,
    date_range: JournalDateRange,
    now_ms: u64,
    requested_page: usize,
) -> JournalPage<'a> {
    let filtered = filter_journal_summaries(summaries, tag_filter, date_range, now_ms);
    let total = filtered.len();
    let page_count = total.div_ceil(JOURNAL_PAGE_SIZE).max(1);
    let page_index = requested_page.min(page_count - 1);
    let start = page_index * JOURNAL_PAGE_SIZE;
    let end = (start + JOURNAL_PAGE_SIZE).min(total);
    JournalPage {
        summaries: filtered[start..end].to_vec(),
        page_index,
        page_count,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_dates_use_elapsed_duration_buckets() {
        assert_eq!(format_relative_date(1_000, 1_000 + DAY_MS - 1), "today");
        assert_eq!(format_relative_date(1_000, 1_000 + DAY_MS), "yesterday");
        assert_eq!(
            format_relative_date(1_000, 1_000 + 2 * DAY_MS),
            "2 days ago"
        );
    }

    #[test]
    fn date_cutoff_is_inclusive() {
        let now_ms = 14 * DAY_MS;
        assert!(JournalDateRange::Past7Days.includes(7 * DAY_MS, now_ms));
        assert!(!JournalDateRange::Past7Days.includes(7 * DAY_MS - 1, now_ms));
        assert!(!JournalDateRange::Past7Days.includes(now_ms + 1, now_ms));
        assert_eq!(format_relative_date(now_ms + 1, now_ms), "in the future");
    }
}
