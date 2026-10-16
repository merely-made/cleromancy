// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use cleromancy::situation::*;
use cleromancy::{
    CleromancyHost, ContextSnapshot, ReadingEngine, ReadingSession, TarotQualification,
};
const AT: u64 = 1_792_149_302_000;
fn fixture(at: u64) -> (CleromancyHost<muniment::MemoryBackend>, ReadingSession) {
    let context = ContextSnapshot::new("C6 public synthetic question", "cleromancy.c6-fixture/v1");
    let field = cleromancy::cards::builtin_decks()[0]
        .field(TarotQualification::Uniform)
        .unwrap();
    let reading =
        ReadingEngine::shuffle_derived(&context, &field, 1, "C6 public seed", "situation")
            .unwrap()
            .remove(0);
    let session = ReadingSession::single(
        at,
        "00000000000000000000000000000006",
        context.digest(),
        field.digest(),
        reading.id.clone(),
        None,
    )
    .unwrap();
    let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
    host.insert_session(&context, &field, &[reading], &session)
        .unwrap();
    (host, session)
}
fn observation() -> WeatherObservation {
    let place = ReadingPlace::new(51_476_900, 0).unwrap();
    WeatherObservation::from_response(
        &place,
        AT - 1000,
        weather_url(&place).unwrap(),
        include_str!("fixtures/c6-open-meteo-current.json").into(),
    )
    .unwrap()
}
fn bytes(host: &CleromancyHost<muniment::MemoryBackend>) -> Vec<u8> {
    serde_json::to_vec(&(host.graph().to_snapshot_at(0), host.graph().facets())).unwrap()
}
#[test]
fn place_off_records_global_facts_and_explicit_missing_local_facts_without_changing_the_draw() {
    let (mut host, session) = fixture(AT);
    let old_readings = host.replay_session(&session).unwrap();
    let situation =
        ReadingSituation::capture(&session, &SituationPreferences::default(), None).unwrap();
    assert_eq!(situation.sky.positions.len(), 10);
    assert!(situation.place.is_none() && situation.local.is_none() && situation.weather.is_none());
    for name in [
        "Astronomical season",
        "Sunrise and sunset",
        "Planetary hours",
        "Weather",
    ] {
        assert!(situation.unavailable.iter().any(|v| v.fact == name));
    }
    host.insert_reading_situation(&situation).unwrap();
    cleromancy::host::validate_divination_graph(host.graph()).unwrap();
    assert_eq!(old_readings, host.replay_session(&session).unwrap());
    assert_eq!(
        host.situation_for_session(&session.id).unwrap().unwrap(),
        situation
    );
    let before = bytes(&host);
    host.insert_reading_situation(&situation).unwrap();
    assert_eq!(before, bytes(&host));
}
#[test]
fn opted_in_weather_replays_from_exact_saved_response_and_tampering_is_refused_before_write() {
    let (mut host, session) = fixture(AT);
    let weather = observation();
    let place = weather.request_place.clone();
    let preferences = SituationPreferences {
        place: Some(place),
        weather: true,
    };
    let situation = ReadingSituation::capture(&session, &preferences, Some(weather)).unwrap();
    let local = situation.local.as_ref().unwrap();
    assert_eq!(local.astronomical_season.as_deref(), Some("Autumn"));
    assert!(
        local
            .solar_events
            .iter()
            .any(|v| v.kind == SolarEventKind::Rise)
    );
    assert!(
        local
            .solar_events
            .iter()
            .any(|v| v.kind == SolarEventKind::Set)
    );
    let hour = local.planetary_hour.as_ref().unwrap();
    assert!(hour.starts_ms <= AT && AT < hour.ends_ms);
    assert_eq!(
        situation
            .weather
            .as_ref()
            .unwrap()
            .values
            .temperature_millicelsius,
        12_800
    );
    let original = bytes(&host);
    let mut variants = Vec::new();
    let mut bad = situation.clone();
    bad.weather
        .as_mut()
        .unwrap()
        .values
        .temperature_millicelsius += 1;
    variants.push(bad);
    let mut bad = situation.clone();
    bad.weather.as_mut().unwrap().response_body = bad
        .weather
        .as_ref()
        .unwrap()
        .response_body
        .replace("12.8", "28.1");
    variants.push(bad);
    let mut bad = situation.clone();
    bad.place.as_mut().unwrap().longitude_microdegrees += 1;
    variants.push(bad);
    let mut bad = situation.clone();
    bad.instant_ms += 1;
    variants.push(bad);
    let mut bad = situation.clone();
    bad.sky.positions[0].degree_millidegrees += 1;
    variants.push(bad);
    let mut bad = situation.clone();
    let hour = bad.local.as_mut().unwrap().planetary_hour.as_mut().unwrap();
    hour.ruler = if hour.ruler == "Jupiter" {
        "Sun"
    } else {
        "Jupiter"
    }
    .into();
    variants.push(bad);
    for bad in variants {
        assert!(host.insert_reading_situation(&bad).is_err());
        assert_eq!(original, bytes(&host));
    }
    host.insert_reading_situation(&situation).unwrap();
    // Core replay has no transport dependency. The exact evidence can cross
    // serialization and a new graph reader without a weather provider.
    let encoded = serde_json::to_vec(&situation).unwrap();
    let decoded: ReadingSituation = serde_json::from_slice(&encoded).unwrap();
    decoded.verify(&session).unwrap();
    let reopened =
        CleromancyHost::<muniment::MemoryBackend>::candidate(host.graph().clone()).unwrap();
    assert_eq!(
        reopened
            .situation_for_session(&session.id)
            .unwrap()
            .unwrap(),
        decoded
    );
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), encoded);
}
#[test]
fn polar_and_equatorial_boundaries_are_disclosed_and_weather_never_enables_place() {
    let (_, session) = fixture(1_782_043_200_000); // June 2026.
    assert!(
        SituationPreferences {
            place: None,
            weather: true
        }
        .validate()
        .is_err()
    );
    let polar = SituationPreferences {
        place: Some(ReadingPlace::new(89_000_000, 0).unwrap()),
        weather: false,
    };
    let s = ReadingSituation::capture(&session, &polar, None).unwrap();
    assert!(s.local.as_ref().unwrap().planetary_hour.is_none());
    assert!(s.unavailable.iter().any(|v| v.fact == "Planetary hours"));
    let equator = SituationPreferences {
        place: Some(ReadingPlace::new(0, 0).unwrap()),
        weather: false,
    };
    assert!(
        ReadingSituation::capture(&session, &equator, None)
            .unwrap()
            .local
            .unwrap()
            .astronomical_season
            .is_none()
    );
    assert!(ReadingPlace::from_degrees("NaN", "0").is_err());
    assert!(ReadingPlace::from_degrees("91", "0").is_err());
    let mut w = observation();
    w.retrieved_at_ms += 86_400_000;
    assert!(w.verify(&w.request_place).is_err());
}
