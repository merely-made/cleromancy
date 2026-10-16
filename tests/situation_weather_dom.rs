// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/cleromancy_dom.rs"]
mod dom;
use cleromancy::situation::*;
use cleromancy::{
    CleromancyHost, Consultation, ContextSnapshot, ReadingEngine, ReadingSession,
    TarotQualification,
};
#[test]
fn reopening_a_weather_reading_shows_saved_evidence_with_both_future_consents_off() {
    let context = ContextSnapshot::new(
        "C6 Greenwich public fixture",
        "cleromancy.context/manual/v1",
    );
    let field = cleromancy::cards::builtin_decks()[0]
        .field(TarotQualification::Uniform)
        .unwrap();
    let reading = ReadingEngine::shuffle_derived(&context, &field, 1, "C6 public", "weather")
        .unwrap()
        .remove(0);
    let session = ReadingSession::single(
        1_792_149_302_000,
        "00000000000000000000000000000006",
        context.digest(),
        field.digest(),
        reading.id.clone(),
        None,
    )
    .unwrap();
    let place = ReadingPlace::new(51_476_900, 0).unwrap();
    let observation = WeatherObservation::from_response(
        &place,
        session.created_at_ms - 1000,
        weather_url(&place).unwrap(),
        include_str!("fixtures/c6-open-meteo-current.json").into(),
    )
    .unwrap();
    let snapshot = ReadingSituation::capture(
        &session,
        &SituationPreferences {
            place: Some(place),
            weather: true,
        },
        Some(observation),
    )
    .unwrap();
    let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
    host.insert_session(&context, &field, &[reading], &session)
        .unwrap();
    host.insert_reading_situation(&snapshot).unwrap();
    let c = Consultation::new(host);
    let detail = c.detail(&session.id).unwrap();
    let catalog = c.catalog().unwrap();
    let mut h = dom::harness(catalog.clone());
    h.update(move |ui| ui.present_reading(catalog, detail));
    assert!(dom::text_at_key(&h, "situation-weather").contains("12.8 °C"));
    assert!(dom::text_at_key(&h, "situation-weather").contains("model estimate"));
    assert!(dom::text_at_key(&h, "situation-hour").contains("Traditional planetary hour"));
    assert!(dom::text_at_key(&h, "situation-circumstances").contains("UT1-minus-UTC=0"));
    let cleromancy::ConsultationAction::Read { situation, .. } =
        dom::click_key(&mut h, "read").unwrap()
    else {
        panic!("read")
    };
    assert_eq!(situation, SituationPreferences::default());
    h.layout_at(420., 20000.);
    for key in [
        "situation-weather",
        "situation-hour",
        "situation-circumstances",
    ] {
        let (x, _, w, _) = dom::rect_at_key(&h, key);
        assert!(x >= 0. && x + w <= 420.5, "{key}: {x}+{w}");
    }
}
