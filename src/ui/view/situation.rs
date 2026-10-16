// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::{ConsultationView, labelled_control, labelled_text};
use crate::{
    ConsultationDetail,
    situation::{SolarEventKind, utc_label},
    ui::state::ConsultationUi,
};
use cambium::{RadioGroup, TextInput, el, map_action, map_state, radio_group};

fn no_action(_: &mut RadioGroup, _: ()) {}
fn place_state(ui: &mut ConsultationUi) -> &mut RadioGroup {
    &mut ui.place_consent
}
fn weather_state(ui: &mut ConsultationUi) -> &mut RadioGroup {
    &mut ui.weather_consent
}
fn latitude(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.place_latitude
}
fn longitude(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.place_longitude
}

pub(super) fn controls(ui: &ConsultationUi) -> ConsultationView {
    let place = map_state(
        map_action(
            radio_group(&ui.place_consent, &["No place", "Use a place"]),
            no_action,
        ),
        place_state,
    );
    let weather = map_state(
        map_action(
            radio_group(
                &ui.weather_consent,
                &["Weather off", "Open-Meteo current weather"],
            ),
            no_action,
        ),
        weather_state,
    );
    let mut items: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Situation for this draw")),
        Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "Time, Moon phase and geocentric tropical sign positions are recorded locally with each new draw.",
        )),
        labelled_control("Place", "cleromancy-place-consent", Box::new(place)),
    ];
    if ui.place_consent.selected == 1 {
        items.push(labelled_text(
            "Latitude (degrees north)",
            "cleromancy-place-latitude",
            false,
            &ui.place_latitude,
            latitude,
        ));
        items.push(labelled_text(
            "Longitude (degrees east)",
            "cleromancy-place-longitude",
            false,
            &ui.place_longitude,
            longitude,
        ));
        items.push(Box::new(el::<_,ConsultationUi,()>("p","This place is saved with the draw for its astronomical season, sunrise, sunset and traditional planetary hour. No device location is requested.")));
    }
    items.push(labelled_control(
        "Weather",
        "cleromancy-weather-consent",
        Box::new(weather),
    ));
    items.push(Box::new(el::<_,ConsultationUi,()>("p","Weather is off by default. Choosing Open-Meteo sends the entered coordinates to its network service when you Read, and saves the returned model estimate. Later viewing and replay use the saved response offline.")));
    Box::new(
        el::<_, ConsultationUi, ()>("section", items)
            .attr("data-key", "situation-controls")
            .attr("aria-label", "Situation for this draw")
            .attr("class", "situation-controls"),
    )
}

pub(super) fn recorded(detail: &ConsultationDetail) -> ConsultationView {
    let mut rows: Vec<(String, String)> = Vec::new();
    let mut row = |key: &str, text: String| rows.push((key.into(), text));
    row(
        "situation-time",
        format!("Recorded time: {}", utc_label(detail.session.created_at_ms)),
    );
    if let Some(s) = &detail.situation {
        row(
            "situation-moon",
            format!(
                "Moon: {} · {:.1}% illuminated. Turquet geocentric calculation.",
                s.sky.phase_band,
                s.sky.illuminated_parts_per_million as f64 / 10_000.0
            ),
        );
        row(
            "situation-signs",
            format!(
                "Tropical sign positions: {}",
                s.sky
                    .positions
                    .iter()
                    .map(|p| format!(
                        "{} in {:?} {:.3}°",
                        p.body,
                        p.sign,
                        p.degree_millidegrees as f64 / 1000.
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        );
        if let Some(place) = &s.place {
            row(
                "situation-place",
                format!(
                    "Saved place: {:.6}° north, {:.6}° east.",
                    place.latitude_microdegrees as f64 / 1e6,
                    place.longitude_microdegrees as f64 / 1e6
                ),
            );
        }
        if let Some(local) = &s.local {
            if let Some(season) = &local.astronomical_season {
                row(
                    "situation-season",
                    format!("Astronomical season: {season} (hemisphere convention)."),
                );
            }
            let events = local
                .solar_events
                .iter()
                .filter(|e| e.instant_ms.abs_diff(s.instant_ms) <= 86_400_000)
                .map(|e| {
                    format!(
                        "{} {}",
                        if e.kind == SolarEventKind::Rise {
                            "Sunrise"
                        } else {
                            "Sunset"
                        },
                        utc_label(e.instant_ms)
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            if !events.is_empty() {
                row("situation-solar-events", events);
            }
            if let Some(h) = &local.planetary_hour {
                row(
                    "situation-hour",
                    format!(
                        "Traditional planetary hour {}: {} · {} to {}. {}. Source: {}.",
                        h.ordinal,
                        h.ruler,
                        utc_label(h.starts_ms),
                        utc_label(h.ends_ms),
                        h.convention,
                        h.tradition_source
                    ),
                );
            }
            row("situation-circumstances", local.circumstances.clone());
        }
        if let Some(w) = &s.weather {
            row(
                "situation-weather",
                format!(
                    "Saved weather: {:.1} °C; {:.1}% relative humidity; {:.2} mm precipitation; wind {:.2} m/s; WMO code {}. Valid {}, retrieved {}. {}. {}.",
                    w.values.temperature_millicelsius as f64 / 1000.,
                    w.values.humidity_permille as f64 / 10.,
                    w.values.precipitation_micrometers as f64 / 1000.,
                    w.values.wind_millimeters_per_second as f64 / 1000.,
                    w.values.wmo_code,
                    utc_label(w.values.valid_at_ms),
                    utc_label(w.retrieved_at_ms),
                    w.provider,
                    w.source
                ),
            );
        }
        for (i, f) in s.unavailable.iter().enumerate() {
            row(
                &format!("situation-unavailable-{i}"),
                format!("{} unavailable: {}", f.fact, f.reason),
            );
        }
        row(
            "situation-source",
            format!(
                "Sky source: {}. These are saved draw-time facts; scene navigation does not fetch current observations.",
                s.sky.provider
            ),
        );
    } else {
        row("situation-legacy","This earlier reading has no recorded C6 sky or weather snapshot. Opening it has not fetched or inferred one.".into());
    }
    let list: Vec<ConsultationView> = rows
        .into_iter()
        .map(|(key, text)| {
            Box::new(
                el::<_, ConsultationUi, ()>("li", text)
                    .attr("data-key", key.clone())
                    .attr("class", key),
            ) as ConsultationView
        })
        .collect();
    Box::new(
        el::<_, ConsultationUi, ()>(
            "section",
            vec![
                Box::new(el::<_, ConsultationUi, ()>("h3", "At this draw")) as ConsultationView,
                Box::new(el::<_, ConsultationUi, ()>("ul", list)),
            ],
        )
        .attr("data-key", "recorded-situation")
        .attr("class", "recorded-situation")
        .attr("aria-label", "Recorded situation"),
    )
}
