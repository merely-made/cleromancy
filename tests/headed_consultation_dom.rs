// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Semantic acceptance of the complete consultation/read/reflection flow.
//!
//! This is deliberately driven through the extracted host `Harness`, so the
//! receipt exercises the same layout-backed pointer and keyboard routing as
//! the headed host. Domain actions remain typed `ConsultationAction`s, but
//! Cambium's retained view action is unit and records them in `ConsultationUi`.

#[path = "support/cleromancy_dom.rs"]
mod support;

use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    Consultation, ConsultationAction, ConsultationContext, ConsultationLayout, ReadingError,
};
use muniment::MemoryBackend;
use support::{
    accessible_label, attr_at_id, attr_at_key, choose, click_key, count_attr, harness, has_attr,
    has_key, has_text_at_key, has_text_in_id, one, select, text_at_key, type_into,
};

#[test]
fn retained_consultation_dispatches_a_complete_reading_and_reflection() {
    let host = cleromancy::CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let catalog = consultation.catalog().unwrap();
    let mut invalid = harness(catalog.clone());

    assert_eq!(count_attr(&invalid, "role", "region"), 3);
    for region in ["Consultation", "Reading", "Journal"] {
        let key = format!("region:{}", region.to_lowercase());
        assert_eq!(
            attr_at_key(&invalid, &key, "role").as_deref(),
            Some("region")
        );
        assert!(has_text_at_key(&invalid, &key, region));
    }

    assert!(click_key(&mut invalid, "read").is_none());
    assert!(
        invalid
            .state()
            .error()
            .is_some_and(|error| error.contains("context label and question"))
    );
    assert!(has_attr(&invalid, "role", "alert"));

    let mut h = harness(catalog);

    h.tab(true);
    assert_eq!(
        h.focus().and_then(|node| accessible_label(&h, node)),
        Some("Context".into())
    );
    h.tab(true);
    assert_eq!(
        h.focus().and_then(|node| accessible_label(&h, node)),
        Some("Context label".into())
    );
    h.tab(true);
    assert_eq!(
        h.focus().and_then(|node| accessible_label(&h, node)),
        Some("Question".into())
    );

    type_into(&mut h, "Context label", "A changing structure");
    type_into(&mut h, "Question", "What deserves attention now?");
    type_into(&mut h, "Tags", "change, reflection");
    type_into(&mut h, "Additional facts", "season: late summer");
    select(&mut h, "Stored field", "Rider-Waite-Smith Major Arcana");
    choose(&mut h, "Cast");
    choose(&mut h, "Three cards");

    let (context, field_digest, mode, layout) = match one(click_key(&mut h, "read")) {
        ConsultationAction::Read {
            context,
            field_digest,
            mode,
            layout,
            ..
        } => (context, field_digest, mode, layout),
        other => panic!("expected read action, found {other:?}"),
    };
    let context_digest = match context {
        ConsultationContext::New(draft) => {
            pollster::block_on(consultation.save_context_at(draft, 2)).unwrap()
        }
        ConsultationContext::Existing(_) => panic!("the first reading must author its context"),
    };
    let mut entropy = FixedEntropy::new(7_u64..64);
    assert_eq!(layout, ConsultationLayout::ThreeCard);
    let detail = pollster::block_on(consultation.read_three_card_at_with_entropy(
        &context_digest,
        &field_digest,
        1_000,
        3,
        &mut entropy,
    ))
    .unwrap();
    assert_eq!(detail.readings.len(), 3);
    let expected_title = detail.readings[0].title.clone();
    let expected_prompt = detail.readings[0].interpretation.clone();
    let expected_algorithm = detail.readings[0].receipt.algorithm.clone();
    let session_id = detail.session.id.clone();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_reading(catalog, detail));

    for position in ["foundation", "tension", "next_step"] {
        assert!(has_key(&h, &format!("reading-position:{position}")));
    }
    assert_eq!(text_at_key(&h, "result-title"), expected_title);
    assert_eq!(text_at_key(&h, "result-prompt"), expected_prompt);

    assert_eq!(
        attr_at_id(&h, "cleromancy-workings-trigger", "aria-expanded").as_deref(),
        Some("false")
    );
    assert!(h.click_on(
        &genet_probe::Selector::role("button").with_attr("id", "cleromancy-workings-trigger",)
    ));
    assert_eq!(
        attr_at_id(&h, "cleromancy-workings-trigger", "aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(attr_at_id(&h, "cleromancy-workings-panel", "hidden"), None);
    assert!(has_text_in_id(&h, "cleromancy-workings-panel", "Algorithm"));
    assert!(has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        &expected_algorithm
    ));
    assert!(has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "Context digest"
    ));
    assert!(has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "Bounded sample"
    ));

    assert!(click_key(&mut h, "save-reflection").is_none());
    assert!(
        h.state()
            .error()
            .is_some_and(|error| error.contains("Enter a reflection"))
    );
    type_into(
        &mut h,
        "Reflection",
        "The structure is useful when it remains revisable.",
    );
    let body = match one(click_key(&mut h, "save-reflection")) {
        ConsultationAction::SaveReflection {
            session_id: id,
            body,
        } => {
            assert_eq!(id, session_id);
            body
        }
        other => panic!("expected reflection action, found {other:?}"),
    };
    let reflected = pollster::block_on(consultation.reflect_at_with_entropy(
        &session_id,
        body,
        2_000,
        4,
        &mut entropy,
    ))
    .unwrap();
    let reflection_id = reflected.reflections[0].id.clone();
    let reflection_body = reflected.reflections[0].body.clone();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_reflection(catalog, reflected));
    assert_eq!(
        text_at_key(&h, &format!("reflection:{reflection_id}")),
        reflection_body
    );

    type_into(&mut h, "Reflection", "A later follow-up remains separate.");
    let second_body = match one(click_key(&mut h, "save-reflection")) {
        ConsultationAction::SaveReflection { body, .. } => body,
        other => panic!("expected second reflection action, found {other:?}"),
    };
    let reflected_twice = pollster::block_on(consultation.reflect_at_with_entropy(
        &session_id,
        second_body,
        2_100,
        4,
        &mut entropy,
    ))
    .unwrap();
    assert_eq!(reflected_twice.reflections.len(), 2);
    let second_reflection_id = reflected_twice.reflections[0].id.clone();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_reflection(catalog, reflected_twice));
    assert!(has_key(&h, &format!("reflection:{second_reflection_id}")));
    assert!(has_key(&h, &format!("reflection:{reflection_id}")));

    let mut comparison_entropy = FixedEntropy::new(0x70_u64..0xa0);
    let comparison_session = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        mode,
        2_200,
        5,
        &mut comparison_entropy,
    ))
    .unwrap();
    let comparison_id = comparison_session.session.id.clone();
    let current_detail = consultation.detail(&session_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_session(catalog, current_detail));
    select(
        &mut h,
        "Compare with",
        &format!("Session {}", &comparison_id[..12]),
    );
    let (left_session_id, right_session_id) = match one(click_key(&mut h, "compare-receipts")) {
        ConsultationAction::CompareSessions {
            left_session_id,
            right_session_id,
        } => (left_session_id, right_session_id),
        other => panic!("expected comparison action, found {other:?}"),
    };
    assert_eq!(left_session_id, session_id);
    assert_eq!(right_session_id, comparison_id);
    let comparison = consultation
        .compare_receipts(&left_session_id, &right_session_id)
        .unwrap();
    let detail = consultation.detail(&left_session_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_comparison(catalog, detail, comparison));
    assert!(has_key(&h, "receipt-comparison-summary"));

    let selected_id = match one(click_key(&mut h, &format!("session:{session_id}"))) {
        ConsultationAction::SelectSession { session_id } => session_id,
        other => panic!("expected session action, found {other:?}"),
    };
    let selected = consultation.detail(&selected_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_session(catalog, selected));
    assert_eq!(h.state().detail().unwrap().session.id, session_id);

    for (label, id) in [
        ("Context label", "cleromancy-context-label"),
        ("Question", "cleromancy-question"),
        ("Tags", "cleromancy-tags"),
        ("Additional facts", "cleromancy-additional-facts"),
        ("Reflection", "cleromancy-reflection"),
    ] {
        assert_eq!(attr_at_id(&h, id, "aria-label").as_deref(), Some(label));
        assert!(has_text_in_id(&h, id, label));
    }
}

struct FixedEntropy {
    words: std::collections::VecDeque<u64>,
}

impl FixedEntropy {
    fn new(words: impl IntoIterator<Item = u64>) -> Self {
        Self {
            words: words.into_iter().collect(),
        }
    }
}

impl EntropySource for FixedEntropy {
    fn next_u64(&mut self) -> Result<u64, ReadingError> {
        self.words
            .pop_front()
            .ok_or_else(|| ReadingError::Entropy("fixed source exhausted".to_string()))
    }
}
