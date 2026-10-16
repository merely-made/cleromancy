// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod dom;
use cleromancy::situation::{ReadingPlace, ReadingSituation, SituationPreferences};
use cleromancy::{CleromancyHost, Consultation, ConsultationAction, ContextDraft};

#[test]
fn consent_is_explicit_and_saved_situation_does_not_enable_future_place_or_weather() {
    let mut c = Consultation::new(CleromancyHost::empty(muniment::MemoryBackend::new()));
    let field = pollster::block_on(c.install_typed_tarot_at(1)).unwrap()[0].clone();
    let context = pollster::block_on(c.save_context_at(
        ContextDraft::new("C6 reading", "What deserves attention?", ""),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(c.read(&context, &field, cleromancy::SelectionMode::Cast)).unwrap();
    let snapshot =
        ReadingSituation::capture(&detail.session, &SituationPreferences::default(), None).unwrap();
    let detail = c.record_situation(&snapshot).unwrap();
    let catalog = c.catalog().unwrap();
    let saved = detail.clone();
    let mut h = dom::harness(catalog.clone());
    h.update(move |ui| ui.present_reading(catalog, detail));
    let text = dom::text_at_key(&h, "recorded-situation");
    assert!(
        text.contains("Sunrise and sunset unavailable") && text.contains("Place was not supplied")
    );
    assert!(text.contains("Tropical sign positions") && text.contains("Moon:"));
    let ConsultationAction::Read { situation, .. } = dom::click_key(&mut h, "read").unwrap() else {
        panic!("read action")
    };
    assert_eq!(situation, SituationPreferences::default());
    let completed_catalog = c.catalog().unwrap();
    let completed_detail = saved.clone();
    h.update(move |ui| ui.present_reading(completed_catalog, completed_detail));
    // Merely choosing weather must not silently enable or infer a place.
    dom::choose(&mut h, "Open-Meteo current weather");
    assert!(dom::click_key(&mut h, "read").is_none());
    dom::choose(&mut h, "Use a place");
    dom::type_into(&mut h, "Latitude (degrees north)", "51.4769");
    dom::type_into(&mut h, "Longitude (degrees east)", "0");
    let ConsultationAction::Read { situation, .. } = dom::click_key(&mut h, "read").unwrap() else {
        panic!("read action")
    };
    assert!(situation.weather);
    assert_eq!(
        situation.place,
        Some(ReadingPlace::new(51_476_900, 0).unwrap())
    );
    let mut viewing = dom::harness(c.catalog().unwrap());
    let catalog = c.catalog().unwrap();
    viewing.update(move |ui| ui.present_reading(catalog, saved));
    let ConsultationAction::Read { situation, .. } = dom::click_key(&mut viewing, "read").unwrap()
    else {
        panic!("read action")
    };
    assert_eq!(situation, SituationPreferences::default());
    viewing.layout_at(420., 20000.);
    for key in ["recorded-situation", "situation-controls"] {
        let (x, _, w, _) = dom::rect_at_key(&viewing, key);
        assert!(x >= 0. && x + w <= 420.5, "{key}: {x} + {w}");
    }
}
