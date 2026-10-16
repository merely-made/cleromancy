// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod support;
use cleromancy::{
    CleromancyHost, Consultation, ContextDraft, DerivedSelection, ReadingEngine, SelectionMode,
};

#[test]
fn uniform_die_uses_scalar_replay_and_saved_outcome_meaning() {
    pollster::block_on(async {
        let mut c = Consultation::new(CleromancyHost::empty(muniment::MemoryBackend::new()));
        let digest = c.install_six_sided_die().await.unwrap();
        assert_eq!(digest, c.install_six_sided_die().await.unwrap());
        assert_eq!(c.catalog().unwrap().fields.len(), 1);
        let context = c
            .save_context(ContextDraft::new(
                "Dice occasion",
                "Which result appears?",
                "",
            ))
            .await
            .unwrap();
        let selection = DerivedSelection::new("C8 public seed", "die-example").unwrap();
        let a = c
            .read_derived(&context, &digest, selection.clone())
            .await
            .unwrap();
        let b = c.read_derived(&context, &digest, selection).await.unwrap();
        assert_eq!(a.readings, b.readings);
        assert_ne!(a.session.id, b.session.id);
        let os = c
            .read(&context, &digest, SelectionMode::Cast)
            .await
            .unwrap();
        for detail in [&a, &os] {
            let reading = &detail.readings[0];
            assert!((1..=6).contains(&reading.candidate_id.parse::<u32>().unwrap()));
            assert!(reading.receipt.shuffle.is_none());
            ReadingEngine::replay(&detail.context, &detail.field, &reading.receipt).unwrap();
            let narrative = cleromancy::narrative::narrate(detail);
            assert!(narrative.overview.contains("6-sided die"));
            assert!(
                narrative.cards[0].text.contains("six")
                    || narrative.cards[0].text.contains("6 faces")
            );
            assert!(!narrative.cards[0].text.contains("upright"));
            assert!(!narrative.cards[0].text.contains("card"));
            assert_eq!(narrative.cards[0].position.label, "Outcome");
            assert!(
                narrative.cards[0]
                    .position
                    .meaning
                    .contains("No symbolic meaning")
            );
        }
        let mut changed = a.readings[0].clone();
        changed.receipt.selected_index = 99;
        assert!(ReadingEngine::replay(&a.context, &a.field, &changed.receipt).is_err());
        let mut h = support::harness(c.catalog().unwrap());
        h.update(move |ui| {
            ui.present_reading(c.catalog().unwrap(), a);
        });
        let text = support::text_at_key(&h, "focused-reading");
        assert!(text.contains("Outcome"));
        assert!(!text.contains("upright"));
        let narration = support::text_at_key(&h, "reading-narrative");
        assert!(narration.contains("Face 1 of 1"));
        assert!(!narration.contains("Card 1"));
        assert!(support::text_at_key(&h, "region:trail").contains("1 outcome"));
    });
}

#[test]
fn die_form_emits_explicit_single_roll_modes_and_refuses_stale_calculated_mode() {
    pollster::block_on(async {
        let mut c = Consultation::new(CleromancyHost::empty(muniment::MemoryBackend::new()));
        c.install_six_sided_die().await.unwrap();
        c.install_builtin_tarot_at(1).await.unwrap();
        let mut h = support::harness(c.catalog().unwrap());
        match support::one(support::click_key(&mut h, "install-six-sided-die")) {
            cleromancy::ConsultationAction::InstallSixSidedDie => {},
            other => panic!("unexpected action {other:?}"),
        }
        h.update(|ui| ui.present_error("test completes setup"));
        support::type_into(&mut h, "Context label", "C8 DOM");
        support::type_into(&mut h, "Question", "What result appears?");
        support::select(&mut h, "Stored field", "Rider-Waite-Smith Major Arcana");
        support::choose(&mut h, "Calculated");
        support::select(&mut h, "Stored field", "6-sided die");
        assert!(support::click_key(&mut h, "read").is_none());
        assert!(support::click_key(&mut h, "die-os").is_none());
        match support::one(support::click_key(&mut h, "read")) {
            cleromancy::ConsultationAction::Read { mode, layout, .. } => {
                assert_eq!(mode, SelectionMode::Cast);
                assert_eq!(layout, cleromancy::ConsultationLayout::Single);
            },
            other => panic!("unexpected {other:?}"),
        }
        h.update(|ui| ui.present_error("test completes roll"));
        support::click_key(&mut h, "die-seed");
        support::type_into(&mut h, "Derived seed", "C8 disclosed seed");
        support::type_into(&mut h, "Derived domain", "dice");
        match support::one(support::click_key(&mut h, "read")) {
            cleromancy::ConsultationAction::Read {
                mode,
                derivation,
                layout,
                ..
            } => {
                assert_eq!(mode, SelectionMode::Derived);
                assert_eq!(derivation.unwrap().seed, "C8 disclosed seed");
                assert_eq!(layout, cleromancy::ConsultationLayout::Single);
            },
            other => panic!("unexpected {other:?}"),
        }
    });
}
