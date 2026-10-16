// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod dom;
use cleromancy::{
    CleromancyHost, Consultation, ContextDraft, DerivedSelection, SpreadTemplateDraft,
};
#[test]
fn public_seed_multi_position_reading_exposes_orientation_and_saved_meaning_accessibly() {
    let mut c = Consultation::new(CleromancyHost::empty(muniment::MemoryBackend::new()));
    let field = pollster::block_on(c.install_typed_tarot_at(1)).unwrap()[0].clone();
    let context = pollster::block_on(c.save_context_at(
        ContextDraft::new("C5 one shuffled deck", "What can I release next?", ""),
        2,
    ))
    .unwrap();
    let template = pollster::block_on(c.save_spread_template_at(
        SpreadTemplateDraft::new(
            "C5 Three concerns",
            "foundation | What I must release\ntension | What resists\nnext_step | What I can try",
        ),
        3,
    ))
    .unwrap();
    let selection = DerivedSelection::new("C5 public deck", "cleromancy.c5/qualification").unwrap();
    let detail =
        pollster::block_on(c.read_seeded_spread(&context, &field, Some(&template), &selection))
            .unwrap();
    assert!(detail.readings.iter().all(|r| r.receipt.shuffle.is_some()));
    assert!(
        detail
            .readings
            .iter()
            .any(|r| r.orientation() == cleromancy::Orientation::Reversed)
    );
    let catalog = c.catalog().unwrap();
    let expected = detail.clone();
    let mut h = dom::harness(catalog.clone());
    h.update(move |ui| ui.present_reading(catalog, detail));
    for (i, p) in expected.session.placements.iter().enumerate() {
        let r = &expected.readings[i];
        let key = format!("scene-card:{}", p.position);
        assert_eq!(
            dom::attr_at_key(&h, &key, "data-orientation").as_deref(),
            Some(r.orientation().label())
        );
        assert!(
            dom::attr_at_key(&h, &key, "aria-label")
                .unwrap()
                .contains(r.orientation().label())
        );
        assert!(
            dom::text_at_key(&h, &format!("narration:{}", p.position)).contains("cannot repeat")
        );
    }
    assert_eq!(
        dom::text_at_key(&h, "reading-position:foundation"),
        "What I must release"
    );
    assert!(dom::text_at_key(&h, "reading-narrative").contains("C5 public deck"));
    // Rehydration restores the public seed controls as well as the sealed draw.
    let cleromancy::ConsultationAction::Read {
        mode,
        derivation,
        layout,
        ..
    } = dom::click_key(&mut h, "read").expect("rehydrated read action")
    else {
        panic!("wrong action")
    };
    assert_eq!(mode, cleromancy::SelectionMode::Derived);
    assert!(matches!(
        layout,
        cleromancy::ConsultationLayout::Authored(_)
    ));
    assert_eq!(derivation.unwrap().seed, "C5 public deck");
    h.layout_at(420.0, 16000.0);
    for p in &expected.session.placements {
        let (x, _, w, _) = dom::rect_at_key(&h, &format!("scene-card:{}", p.position));
        assert!(x >= 0.0 && x + w <= 420.5);
    }
}
