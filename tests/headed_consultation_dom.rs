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
    SelectionMode,
};
use muniment::MemoryBackend;
use support::{
    accessible_label, attr_at_id, attr_at_key, choose, click_key, count_attr, harness, has_attr,
    has_key, has_text_at_key, has_text_in_id, next_surface, one, select, switch_surface,
    take_action, text_at_key, type_into,
};

#[test]
fn four_surface_tabs_are_local_accessible_and_bounded() {
    let host = cleromancy::CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    let field_digest = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context_digest = pollster::block_on(consultation.save_context_at(
        cleromancy::ContextDraft::new("Six occasions", "Which one is this?", "archive"),
        2,
    ))
    .unwrap();
    let mut entropy = FixedEntropy::new(0_u64..64);
    for offset in 0_u64..6 {
        pollster::block_on(consultation.read_at_with_entropy(
            &context_digest,
            &field_digest,
            SelectionMode::Calculated,
            1_000 + offset,
            3 + offset,
            &mut entropy,
        ))
        .unwrap();
    }
    let catalog = consultation.catalog().unwrap();
    assert_eq!(catalog.session_summaries.len(), 6);
    let session_ids = catalog
        .session_summaries
        .iter()
        .map(|summary| summary.session_id.clone())
        .collect::<Vec<_>>();
    let mut keyboard = harness(catalog.clone());
    keyboard.tab(true);
    assert_eq!(
        keyboard
            .focus()
            .and_then(|node| accessible_label(&keyboard, node)),
        Some("Surfaces".into())
    );
    next_surface(&mut keyboard);
    assert_surface(&keyboard, "journal", &["journal"]);
    next_surface(&mut keyboard);
    #[cfg(not(feature = "sky-timeline"))]
    assert_surface(&keyboard, "chart", &["chart"]);
    #[cfg(feature = "sky-timeline")]
    assert_surface(&keyboard, "sky", &["sky"]);

    let mut h = harness(catalog);

    assert_eq!(count_attr(&h, "role", "tablist"), 1);
    assert_eq!(count_attr(&h, "role", "tab"), 4);
    for (label, selected) in [
        ("Today", "true"),
        ("Journal", "false"),
        ("Sky", "false"),
        ("Chart", "false"),
    ] {
        let id = format!("cleromancy-surfaces-item-{}", label.to_ascii_lowercase());
        assert_eq!(
            attr_at_id(&h, &id, "aria-selected").as_deref(),
            Some(selected)
        );
    }
    let sky_id = "cleromancy-surfaces-item-sky";
    #[cfg(not(feature = "sky-timeline"))]
    {
        assert_eq!(
            attr_at_id(&h, sky_id, "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            attr_at_id(&h, sky_id, "aria-description").as_deref(),
            Some("The sky surface needs the sky-timeline feature.")
        );
        assert!(has_text(
            &h,
            "The sky surface needs the sky-timeline feature."
        ));
    }
    #[cfg(feature = "sky-timeline")]
    assert_eq!(
        attr_at_id(&h, sky_id, "aria-disabled").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr_at_id(&h, "cleromancy-surfaces-item-chart", "aria-disabled").as_deref(),
        Some("false")
    );
    assert!(take_action(&mut h).is_none());

    assert_surface(&h, "today", &["consultation", "reading", "trail"]);
    for session_id in session_ids.iter().take(5) {
        assert!(has_key(&h, &format!("session:{session_id}")));
    }
    assert!(
        !has_key(&h, &format!("session:{}", session_ids[5])),
        "Today must not grow past its five-row trail"
    );

    switch_surface(&mut h, "Journal");
    assert_surface(&h, "journal", &["journal"]);
    for session_id in &session_ids {
        assert!(has_key(&h, &format!("session:{session_id}")));
    }

    switch_surface(&mut h, "Chart");
    assert_surface(&h, "chart", &["chart"]);
}

fn assert_surface(harness: &support::App, screen: &str, visible_regions: &[&str]) {
    assert!(has_attr(harness, "data-screen", screen));
    assert_eq!(
        attr_at_id(
            harness,
            &format!("cleromancy-surface-{screen}"),
            "aria-labelledby"
        )
        .as_deref(),
        Some(format!("cleromancy-surfaces-item-{screen}").as_str())
    );
    for tab in ["today", "journal", "sky", "chart"] {
        assert_eq!(
            attr_at_id(
                harness,
                &format!("cleromancy-surfaces-item-{tab}"),
                "aria-selected"
            )
            .as_deref(),
            Some(if tab == screen { "true" } else { "false" }),
            "tab selection drifted from the rendered {screen} surface"
        );
    }
    for region in [
        "consultation",
        "reading",
        "trail",
        "journal",
        "sky",
        "chart",
    ] {
        assert_eq!(
            has_key(harness, &format!("region:{region}")),
            visible_regions.contains(&region),
            "unexpected visibility for {region} on {screen}"
        );
    }
}

#[test]
fn retained_consultation_dispatches_a_complete_reading_and_reflection() {
    let host = cleromancy::CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let catalog = consultation.catalog().unwrap();
    let mut invalid = harness(catalog.clone());

    // Today is the default surface: the consultation form, the current
    // reading, and the bounded recent trail. The full journal is one tab away.
    assert_eq!(count_attr(&invalid, "role", "region"), 3);
    for (key, heading) in [
        ("region:consultation", "Consultation"),
        ("region:reading", "Reading"),
        ("region:trail", "Recent"),
    ] {
        assert_eq!(
            attr_at_key(&invalid, key, "role").as_deref(),
            Some("region")
        );
        assert!(has_text_at_key(&invalid, key, heading));
    }
    switch_surface(&mut invalid, "Journal");
    assert_eq!(count_attr(&invalid, "role", "region"), 1);
    assert_eq!(
        attr_at_key(&invalid, "region:journal", "role").as_deref(),
        Some("region")
    );
    assert!(has_text_at_key(&invalid, "region:journal", "Journal"));
    switch_surface(&mut invalid, "Today");

    assert!(click_key(&mut invalid, "read").is_none());
    assert!(
        invalid
            .state()
            .error()
            .is_some_and(|error| error.contains("context label and question")),
        "read action error {:?}; button {:?}; form {:?}",
        invalid.state().error(),
        support::rect_at_key(&invalid, "read"),
        support::rect_at_key(&invalid, "region:consultation")
    );
    assert!(has_attr(&invalid, "role", "alert"));

    let mut h = harness(catalog);

    // The surface tab bar sits above the form, so it takes the first stop in
    // the tab order; only the active tab is focusable.
    h.tab(true);
    assert_eq!(
        h.focus().and_then(|node| accessible_label(&h, node)),
        Some("Surfaces".into())
    );
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
        },
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
    let expected_tension = detail.readings[1].interpretation.clone();
    let unchanged_session = detail.session.clone();
    // Repeated source results remain distinct scene occurrences.
    let mut repeated = detail.clone();
    repeated.session.placements[1].reading_id = repeated.session.placements[0].reading_id.clone();
    let scene = cleromancy::reading_scene::reading_scene(&repeated);
    assert_eq!(scene.scene.items[0].source, scene.scene.items[1].source);
    assert_ne!(scene.positions[0], scene.positions[1]);
    assert_ne!(
        scene.scene.items[0].transform,
        scene.scene.items[1].transform
    );
    let session_id = detail.session.id.clone();
    let catalog = consultation.catalog().unwrap();
    h.update(move |ui| ui.present_reading(catalog, detail));

    for position in ["foundation", "tension", "next_step"] {
        assert!(has_key(&h, &format!("reading-position:{position}")));
    }
    assert_eq!(text_at_key(&h, "result-title"), expected_title);
    assert_eq!(text_at_key(&h, "result-prompt"), expected_prompt);
    assert_eq!(
        text_at_key(&h, "reading-question"),
        "What deserves attention now?"
    );
    h.layout_at(1160.0, 760.0);
    let (form_x, _, form_width, _) = support::rect_at_key(&h, "region:consultation");
    let (reading_x, reading_y, reading_width, _) = support::rect_at_key(&h, "region:reading");
    assert!(
        reading_x >= form_x + form_width,
        "reading must sit beside the form"
    );
    assert!(
        reading_x + reading_width <= 1160.0,
        "reading must fit the window: {reading_x} + {reading_width}"
    );
    assert!(
        reading_y < 380.0,
        "reading must begin in the first viewport"
    );
    for key in [
        "result-title",
        "result-title:tension",
        "result-title:next_step",
    ] {
        let (x, y, width, height) = support::rect_at_key(&h, key);
        assert!(
            x >= 0.0 && x + width <= 1160.0 && y + height <= 760.0,
            "card title must be visible: {key}: {x}, {y}, {width}, {height}"
        );
    }
    h.move_to(1150.0, 400.0);
    let mut previous = h.element_scroll_total();
    for _ in 0..48 {
        h.wheel(0.0, 32.0);
        let current = h.element_scroll_total();
        assert!(
            current >= previous,
            "scroll reversed: {previous} -> {current}"
        );
        assert!(
            current - previous <= 32.1,
            "scroll jumped: {previous} -> {current}"
        );
        previous = current;
    }
    assert!(previous > 100.0, "the page must actually scroll");
    h.wheel(0.0, -100000.0);
    h.layout_at(1600.0, 4000.0);

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
    assert!(has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "The choice"
    ));
    assert!(has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "A fresh chance draw chose this result from the available possibilities, using their relative chances."
    ));
    assert!(!has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "Context digest"
    ));
    assert!(!has_text_in_id(
        &h,
        "cleromancy-workings-panel",
        "Bounded sample"
    ));
    assert!(click_key(&mut h, "scene-card:tension").is_none());
    assert_eq!(text_at_key(&h, "result-prompt"), expected_tension);
    assert_eq!(
        attr_at_key(&h, "scene-card:tension", "aria-pressed").as_deref(),
        Some("true")
    );
    assert_eq!(h.state().detail().unwrap().session, unchanged_session);
    switch_surface(&mut h, "Journal");
    switch_surface(&mut h, "Today");
    assert_eq!(text_at_key(&h, "result-prompt"), expected_tension);

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
        },
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
    // Receipt comparison lives on the Journal surface. Presenting a session did
    // not move the surface, so the switch is explicit.
    assert!(has_key(&h, "region:trail"));
    switch_surface(&mut h, "Journal");
    assert!(take_action(&mut h).is_none());
    select(
        &mut h,
        "Compare with",
        &format!("A changing structure ({})", &comparison_id[..12]),
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
    switch_surface(&mut h, "Today");

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
