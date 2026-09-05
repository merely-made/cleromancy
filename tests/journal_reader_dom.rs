// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! R2's archive-reader receipt: list semantics stay summary-only, while a
//! selected occasion crosses the explicit replay boundary for its detail.

#[path = "support/cleromancy_dom.rs"]
mod support;

use std::time::{SystemTime, UNIX_EPOCH};

use cleromancy::{
    CleromancyHost, Consultation, ConsultationAction, ConsultationCatalog, ContextDraft,
    SelectionMode, SessionSummary, SummaryPlacement,
};
use muniment::MemoryBackend;
use support::{
    attr_at_key, click_key, count_attr, harness, has_key, has_text_at_key, one, select,
    switch_surface, take_action, text_at_key, type_into,
};

#[test]
fn journal_rows_filter_compose_clear_and_page_without_product_actions() {
    let now_ms = unix_ms();
    let catalog = ConsultationCatalog {
        contexts: Vec::new(),
        fields: Vec::new(),
        spread_templates: Vec::new(),
        astrology_facts: Vec::new(),
        session_summaries: (0..25)
            .map(|index| summary(index, now_ms - u64::from(index) * 24 * 60 * 60 * 1_000))
            .collect(),
    };
    let mut h = harness(catalog);
    switch_surface(&mut h, "Journal");

    let first_key = "session:session-000";
    assert!(has_key(&h, first_key));
    assert!(!has_key(&h, "session:session-020"));
    let row_label = attr_at_key(&h, first_key, "aria-label").expect("row label");
    assert!(row_label.contains("session-000"));
    assert!(row_label.contains("Context: Context 0"));
    assert!(row_label.contains("Date: today"));
    assert!(row_label.contains("3 placement/cards"));
    assert!(
        row_label.contains("Cards: focus: The Hermit, obstacle: The Tower, direction: The Star")
    );
    assert!(row_label.contains("Modes: Calculated and Cast"));
    assert_eq!(text_at_key(&h, "session-pips:session-000"), "▫▫▫");
    assert_eq!(
        attr_at_key(&h, "session-pips:session-000", "aria-hidden").as_deref(),
        Some("true")
    );
    assert_eq!(count_attr(&h, "class", "session-pips"), 20);
    assert!(text_at_key(&h, "journal-bound").contains("Showing 1–20 of 25"));

    assert!(click_key(&mut h, "journal-older").is_none());
    assert!(has_key(&h, "session:session-020"));
    assert!(!has_key(&h, first_key));
    assert!(text_at_key(&h, "journal-bound").contains("Showing 21–25 of 25"));

    type_into(&mut h, "Tag filter", "archive");
    assert!(take_action(&mut h).is_none());
    select(&mut h, "Date range", "Past 7 days");
    assert!(take_action(&mut h).is_none());
    assert!(has_key(&h, first_key));
    assert!(!has_key(&h, "session:session-008"));
    assert!(text_at_key(&h, "journal-bound").contains("of 8 matching sessions"));

    assert!(click_key(&mut h, "clear-journal-filters").is_none());
    assert!(has_key(&h, first_key));
    assert!(!has_key(&h, "session:session-020"));
    assert!(text_at_key(&h, "journal-bound").contains("Showing 1–20 of 25"));

    type_into(&mut h, "Tag filter", "missing");
    assert!(text_at_key(&h, "journal-bound").contains(
        "Showing 0 matching sessions. Page 1 of 1. Each page shows at most 20 sessions."
    ));
    assert!(click_key(&mut h, "clear-journal-filters").is_none());
}

#[test]
fn journal_selected_detail_and_comparison_keep_replay_explicit() {
    let host = CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    let field = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(
        consultation.save_context_at(
            ContextDraft::new(
                "Journal context",
                "What should remain visible?",
                "archive, proof",
            )
            .with_additional_facts("season: autumn\nsource: local"),
            2,
        ),
    )
    .unwrap();
    let first = pollster::block_on(consultation.read_at_with_entropy(
        &context,
        &field,
        SelectionMode::Calculated,
        unix_ms(),
        3,
        &mut FixedEntropy,
    ))
    .unwrap();
    let second = pollster::block_on(consultation.read_at_with_entropy(
        &context,
        &field,
        SelectionMode::Cast,
        unix_ms(),
        4,
        &mut FixedEntropy,
    ))
    .unwrap();
    let first_id = first.session.id.clone();
    let second_id = second.session.id.clone();
    let mut h = harness(consultation.catalog().unwrap());
    switch_surface(&mut h, "Journal");

    let selected = match one(click_key(&mut h, &format!("session:{first_id}"))) {
        ConsultationAction::SelectSession { session_id } => session_id,
        other => panic!("expected selected session, found {other:?}"),
    };
    let detail = consultation.detail(&selected).unwrap();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_session(catalog, detail));
    assert!(has_text_at_key(
        &h,
        "journal-context-label",
        "Label: Journal context"
    ));
    assert!(has_text_at_key(
        &h,
        "journal-context-question",
        "Question: What should remain visible?"
    ));
    assert!(has_text_at_key(
        &h,
        "journal-context-tags",
        "Tags: archive, proof"
    ));
    assert!(has_text_at_key(
        &h,
        "journal-context-fact:season",
        "season: autumn"
    ));
    assert!(has_key(&h, "journal-reading:focus"));

    select(
        &mut h,
        "Compare with",
        &format!("Journal context ({})", &second_id[..12]),
    );
    let (left_session_id, right_session_id) = match one(click_key(&mut h, "compare-receipts")) {
        ConsultationAction::CompareSessions {
            left_session_id,
            right_session_id,
        } => (left_session_id, right_session_id),
        other => panic!("expected comparison, found {other:?}"),
    };
    assert_eq!(left_session_id, first_id);
    assert_eq!(right_session_id, second_id);
    let comparison = consultation
        .compare_receipts(&left_session_id, &right_session_id)
        .unwrap();
    let detail = consultation.detail(&left_session_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_comparison(catalog, detail, comparison));
    assert!(has_key(&h, "receipt-comparison-summary"));
}

fn summary(index: u8, created_at_ms: u64) -> SessionSummary {
    let placements = if index == 0 {
        vec![
            placement("focus", "The Hermit", "major-09", SelectionMode::Calculated),
            placement("obstacle", "The Tower", "major-16", SelectionMode::Cast),
            placement(
                "direction",
                "The Star",
                "major-17",
                SelectionMode::Calculated,
            ),
        ]
    } else {
        vec![placement(
            "focus",
            "The Hermit",
            "major-09",
            SelectionMode::Calculated,
        )]
    };
    SessionSummary {
        session_id: format!("session-{index:03}"),
        created_at_ms,
        context_label: format!("Context {index}"),
        question: Some("What belongs in the archive?".to_string()),
        tags: if index < 22 {
            vec!["archive".to_string()]
        } else {
            vec!["other".to_string()]
        },
        context_digest: format!("context-{index}"),
        field_digest: "tarot".to_string(),
        placements,
    }
}

fn placement(
    position: &str,
    title: &str,
    candidate_id: &str,
    mode: SelectionMode,
) -> SummaryPlacement {
    SummaryPlacement {
        position: position.to_string(),
        title: title.to_string(),
        candidate_id: candidate_id.to_string(),
        mode,
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}

struct FixedEntropy;

impl cleromancy::moirai::clotho::EntropySource for FixedEntropy {
    fn next_u64(&mut self) -> Result<u64, cleromancy::ReadingError> {
        Ok(7)
    }
}
