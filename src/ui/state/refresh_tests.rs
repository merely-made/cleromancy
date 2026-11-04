// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
#[test]
fn natal_actions_wait_for_owner_acknowledgement_and_refuse_double_submission() {
    let c = crate::Consultation::new(crate::CleromancyHost::empty(muniment::MemoryBackend::new()));
    let mut ui = ConsultationUi::new(c.catalog().unwrap());
    ui.record_action(Some(ConsultationAction::RefreshBirthFacts));
    assert!(ui.status.is_busy());
    ui.record_action(Some(ConsultationAction::CreateSubject {
        label: "Must wait".into(),
    }));
    assert!(matches!(
        ui.take_pending_action(),
        Some(ConsultationAction::RefreshBirthFacts)
    ));
    ui.record_action(Some(ConsultationAction::CreateSubject {
        label: "Still waiting".into(),
    }));
    assert!(ui.take_pending_action().is_none());
    ui.status = ConsultationStatus::Ready;
    ui.record_action(Some(ConsultationAction::CreateSubject {
        label: "After acknowledgement".into(),
    }));
    assert!(matches!(
        ui.take_pending_action(),
        Some(ConsultationAction::CreateSubject { .. })
    ));
}

#[test]
fn choosing_die_opens_a_new_form_without_changing_the_previous_reading() {
    let mut c =
        crate::Consultation::new(crate::CleromancyHost::empty(muniment::MemoryBackend::new()));
    let field = pollster::block_on(c.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(c.save_context_at(
        crate::ContextDraft::new("Previous occasion", "What deserves attention?", ""),
        2,
    ))
    .unwrap();
    let old =
        pollster::block_on(c.read(&context, &field, crate::SelectionMode::Calculated)).unwrap();
    let previous_catalog = c.catalog().unwrap();
    let die = pollster::block_on(c.install_six_sided_die()).unwrap();
    let catalog = c.catalog().unwrap();
    let mut ui = ConsultationUi::new(catalog.clone());
    ui.present_reading(catalog.clone(), old.clone());
    ui.card_preview = Some((field, "0".into()));
    ui.present_die(previous_catalog, &die);
    assert_eq!(ui.detail.as_ref(), Some(&old));
    assert!(ui.error.as_ref().unwrap().contains("no longer available"));
    ui.present_die(catalog, &die);
    assert!(ui.error.is_none());
    assert!(ui.detail.is_none());
    assert!(ui.card_preview.is_none());
    assert_eq!(ui.catalog.fields[ui.field_select.selected].digest(), die);
    assert_eq!(ui.mode.selected, 1);
    assert_eq!(ui.layout.selected, 0);
    assert_eq!(c.detail(&old.session.id).unwrap(), old);
}

#[test]
fn adding_decks_preserves_selected_field_identity() {
    let host = crate::CleromancyHost::empty(muniment::MemoryBackend::new());
    let mut consultation = crate::Consultation::new(host);
    let selected = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let mut ui = ConsultationUi::new(consultation.catalog().unwrap());
    pollster::block_on(consultation.install_typed_tarot_at(2)).unwrap();
    ui.replace_catalog(consultation.catalog().unwrap());
    assert_eq!(
        ui.catalog.fields[ui.field_select.selected].digest(),
        selected
    );
    assert_eq!(ui.catalog.fields.len(), 3);
    assert!(ui.request_typed_decks().is_some());
    assert!(ui.status.is_busy());
    assert!(ui.request_typed_decks().is_none());
}

#[test]
fn owner_focus_refresh_cannot_complete_loading_or_a_neighborhood_request() {
    struct Entropy;
    impl crate::moirai::clotho::EntropySource for Entropy {
        fn next_u64(&mut self) -> Result<u64, crate::ReadingError> {
            Ok(0)
        }
    }
    let mut c =
        crate::Consultation::new(crate::CleromancyHost::empty(muniment::MemoryBackend::new()));
    let field = pollster::block_on(c.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(c.save_context_at(
        ContextDraft::new("Refresh", "What is still pending?", ""),
        1,
    ))
    .unwrap();
    let detail = pollster::block_on(c.read_at_with_entropy(
        &context,
        &field,
        SelectionMode::Calculated,
        2000,
        2,
        &mut Entropy,
    ))
    .unwrap();
    let mut ui = ConsultationUi::new(c.catalog().unwrap());
    ui.request_session(detail.session.id.clone());
    ui.present_external_session(c.catalog().unwrap(), detail.clone());
    assert_eq!(ui.status, ConsultationStatus::LoadingSession);
    ui.status = ConsultationStatus::SavingSetup;
    ui.present_external_session(c.catalog().unwrap(), detail.clone());
    assert_eq!(ui.status, ConsultationStatus::SavingSetup);
    let context = c.ambient_context(&detail.session.id, "focus").unwrap();
    ui.present_neighborhood(c.catalog().unwrap(), detail, context);
    assert!(!ui.status.is_busy());
    assert!(ui.neighborhood_view().is_some());
}

#[test]
fn catalog_refresh_does_not_complete_a_pending_read() {
    let host = crate::CleromancyHost::empty(muniment::MemoryBackend::new());
    let catalog = crate::Consultation::new(host).catalog().unwrap();
    let mut ui = ConsultationUi::new(catalog.clone());
    ui.status = ConsultationStatus::SavingReading;
    ui.replace_catalog(catalog);
    assert!(
        ui.status.is_busy(),
        "the reading's detail has not arrived yet"
    );
}
