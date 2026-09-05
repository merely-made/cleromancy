// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stored chart receipts and manual chart import. Calculation remains an
//! optional adapter action, while this reader only projects saved values.

use cambium::{
    GridColumn, GridSpec, GridView, SelectState, TextInput, button, data_grid, el, map_action,
    map_state, select,
};

use super::{ConsultationView, labelled_control, labelled_text, never_select_action};
use crate::ui::state::ConsultationUi;
use crate::{AstrologyChart, AstrologyFacts, AstrologyPlacement, AstrologyPosition};

pub(super) fn chart_region(ui: &ConsultationUi) -> ConsultationView {
    let labels = ui
        .catalog
        .astrology_charts
        .iter()
        .map(|stored| {
            format!(
                "{} · {} · {} placements · {}",
                stored.chart.moment.instant_utc,
                stored.facts.chart_digest,
                stored.facts.placements.len(),
                observer_label(&stored.chart),
            )
        })
        .collect::<Vec<_>>();
    let refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let picker = map_action(select(&ui.chart.selected_chart, &refs), never_select_action);
    let picker = map_state(picker, chart_select_state);
    let mut children = vec![
        Box::new(el::<_, ConsultationUi, ()>("h2", "Chart")) as ConsultationView,
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                "Saved UTC moments and source-qualified positions. Placement labels and aspects are derived from this stored chart receipt.",
            )
            .attr("data-key", "chart-introduction"),
        ) as ConsultationView,
        labelled_control("Stored chart", "cleromancy-chart-select", Box::new(picker)),
    ];
    if let Some(stored) = ui
        .catalog
        .astrology_charts
        .get(ui.chart.selected_chart.selected)
    {
        children.extend(stored_chart(&stored.chart, &stored.facts));
    } else {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                "No stored charts yet. Import one below, or calculate one when the optional ephemeris adapter is available.",
            )
            .attr("class", "empty-chart")
            .attr("data-key", "chart-empty"),
        ));
    }
    children.extend(import_controls(ui));
    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Chart")
            .attr("data-key", "region:chart"),
    )
}

fn stored_chart(chart: &AstrologyChart, facts: &AstrologyFacts) -> Vec<ConsultationView> {
    let observer = observer_label(chart);
    let mut views = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Stored chart receipt")) as ConsultationView,
        labelled_value(
            "UTC moment",
            chart.moment.instant_utc.as_str(),
            "chart-moment",
        ),
        labelled_value("Observer", &observer, "chart-observer"),
        labelled_value("Chart digest", &facts.chart_digest, "chart-digest"),
        labelled_value("Facts digest", &facts.digest(), "chart-facts-digest"),
        labelled_value(
            "Aspect orb",
            &format!("{} millidegrees", facts.orb_millidegrees),
            "chart-orb",
        ),
        Box::new(el::<_, ConsultationUi, ()>("h3", "Stored positions")) as ConsultationView,
        positions_grid(chart, facts),
    ];
    views.extend([
        Box::new(el::<_, ConsultationUi, ()>("h3", "Aspects")) as ConsultationView,
        aspects_grid(&facts.aspects),
        Box::new(el::<_, ConsultationUi, ()>("h3", "Source")) as ConsultationView,
        Box::new(
            el::<_, ConsultationUi, ()>(
                "dl",
                vec![
                    definition("Chart schema", &chart.schema),
                    definition("Calculation algorithm", &chart.algorithm),
                    definition("Calculation engine", &chart.engine),
                    definition("Ephemeris", &chart.ephemeris),
                    definition("Chart digest", &facts.chart_digest),
                    definition("Facts algorithm", &facts.algorithm),
                    definition("Facts schema", &facts.schema),
                ],
            )
            .attr("data-key", "chart-source"),
        ) as ConsultationView,
    ]);
    views
}

fn observer_label(chart: &AstrologyChart) -> String {
    match (
        chart.moment.latitude_microdegrees,
        chart.moment.longitude_microdegrees,
    ) {
        (Some(latitude), Some(longitude)) => format!("{latitude}, {longitude} microdegrees"),
        _ => "Global observer".to_string(),
    }
}

fn positions_grid(chart: &AstrologyChart, facts: &AstrologyFacts) -> ConsultationView {
    let rows = chart
        .positions
        .iter()
        .map(|position| {
            let placement = facts
                .placements
                .iter()
                .find(|placement| placement.body == position.body)
                .cloned();
            (position.clone(), placement)
        })
        .collect::<Vec<_>>();
    let spec = GridSpec {
        columns: vec![
            GridColumn::new("Body", 100.0),
            GridColumn::new("Longitude (millidegrees)", 180.0),
            GridColumn::new("Latitude (millidegrees)", 170.0),
            GridColumn::new("Sign", 100.0),
            GridColumn::new("Sign degree (millidegrees)", 190.0),
            GridColumn::new("Motion", 90.0),
        ],
        row_height: 26.0,
        header_height: 28.0,
        overscan: 1,
    };
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            data_grid(
                &spec,
                rows.len(),
                160.0,
                0.0,
                move |row, column| position_cell(&rows[row].0, rows[row].1.as_ref(), column),
                |_ui: &mut ConsultationUi, _column| {},
                |_| None,
            ),
        )
        .attr("role", "region")
        .attr("aria-label", "Stored positions")
        .attr("data-key", "chart-positions"),
    )
}

fn position_cell(
    position: &AstrologyPosition,
    placement: Option<&AstrologyPlacement>,
    column: usize,
) -> GridView<ConsultationUi, ()> {
    let value = match column {
        0 => position.body.clone(),
        1 => position.longitude_millidegrees.to_string(),
        2 => position.latitude_millidegrees.to_string(),
        3 => placement.map_or_else(
            || "unavailable".to_string(),
            |value| format!("{:?}", value.sign),
        ),
        4 => placement.map_or_else(
            || "unavailable".to_string(),
            |value| value.degree_millidegrees.to_string(),
        ),
        _ => match position.retrograde {
            Some(true) => "retrograde".to_string(),
            Some(false) => "direct".to_string(),
            None => "unknown".to_string(),
        },
    };
    Box::new(el::<_, ConsultationUi, ()>("span", value))
}

fn aspects_grid(aspects: &[crate::AstrologyAspect]) -> ConsultationView {
    let rows = aspects.to_vec();
    let spec = GridSpec {
        columns: vec![
            GridColumn::new("Bodies", 160.0),
            GridColumn::new("Aspect", 100.0),
            GridColumn::new("Exact (millidegrees)", 155.0),
            GridColumn::new("Separation (millidegrees)", 180.0),
            GridColumn::new("Orb (millidegrees)", 145.0),
        ],
        row_height: 26.0,
        header_height: 28.0,
        overscan: 1,
    };
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            data_grid(
                &spec,
                rows.len(),
                130.0,
                0.0,
                move |row, column| aspect_cell(&rows[row], column),
                |_ui: &mut ConsultationUi, _column| {},
                |_| None,
            ),
        )
        .attr("role", "region")
        .attr("aria-label", "Stored aspects")
        .attr("data-key", "chart-aspects"),
    )
}

fn aspect_cell(aspect: &crate::AstrologyAspect, column: usize) -> GridView<ConsultationUi, ()> {
    let value = match column {
        0 => format!("{} / {}", aspect.first, aspect.second),
        1 => format!("{:?}", aspect.kind),
        2 => aspect.exact_millidegrees.to_string(),
        3 => aspect.separation_millidegrees.to_string(),
        _ => aspect.orb_millidegrees.to_string(),
    };
    Box::new(el::<_, ConsultationUi, ()>("span", value))
}

fn labelled_value(label: &str, value: &str, key: &str) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ()>("p", format!("{label}: {value}"))
            .attr("data-key", key.to_string()),
    )
}

fn definition(label: &str, value: &str) -> ConsultationView {
    Box::new(el::<_, ConsultationUi, ()>(
        "div",
        (el("dt", label.to_string()), el("dd", value.to_string())),
    ))
}

fn import_controls(ui: &ConsultationUi) -> Vec<ConsultationView> {
    let controls = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Import chart")) as ConsultationView,
        labelled_text("Calculation algorithm", "cleromancy-astrology-algorithm", false, &ui.astrology_algorithm, astrology_algorithm_state),
        labelled_text("Calculation engine", "cleromancy-astrology-engine", false, &ui.astrology_engine, astrology_engine_state),
        labelled_text("Ephemeris source", "cleromancy-astrology-ephemeris", false, &ui.astrology_ephemeris, astrology_ephemeris_state),
        labelled_text("UTC instant", "cleromancy-astrology-instant", false, &ui.astrology_instant_utc, astrology_instant_state),
        labelled_text("Latitude microdegrees (optional)", "cleromancy-astrology-latitude", false, &ui.astrology_latitude, astrology_latitude_state),
        labelled_text("Longitude microdegrees (optional)", "cleromancy-astrology-longitude", false, &ui.astrology_longitude, astrology_longitude_state),
        labelled_text("Aspect orb millidegrees", "cleromancy-astrology-orb", false, &ui.astrology_orb, astrology_orb_state),
        labelled_text("Chart positions", "cleromancy-astrology-positions", true, &ui.astrology_positions, astrology_positions_state),
        Box::new(el::<_, ConsultationUi, ()>("p", "For a manual import, identify the algorithm, engine, and ephemeris, then copy positions as body | longitude millidegrees | latitude millidegrees | retrograde. Local calculation ignores those manual source and position fields.").attr("class", "context-explanation")) as ConsultationView,
        Box::new(button("Save chart facts", |ui: &mut ConsultationUi, _| {
            let action = ui.request_astrology_chart();
            ui.record_action(action);
        }).attr("data-key", "save-chart-facts").attr("aria-label", "Save imported chart facts")) as ConsultationView,
    ];
    #[cfg(feature = "analytic-ephemeris")]
    {
        controls.into_iter().chain(ephemeris_controls()).collect()
    }
    #[cfg(not(feature = "analytic-ephemeris"))]
    {
        controls
    }
}

#[cfg(feature = "analytic-ephemeris")]
fn ephemeris_controls() -> Vec<ConsultationView> {
    vec![
        Box::new(el::<_, ConsultationUi, ()>("p", "Charts are calculated locally from the built-in engine. Nothing is downloaded, and every chart records the engine revision that produced it.").attr("class", "context-explanation").attr("data-key", "ephemeris-status")) as ConsultationView,
        Box::new(button("Calculate and save chart", |ui: &mut ConsultationUi, _| {
            let action = ui.request_calculated_astrology_chart();
            ui.record_action(action);
        }).attr("data-key", "calculate-chart").attr("aria-label", "Calculate and save astrology chart")) as ConsultationView,
    ]
}

fn chart_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.chart.selected_chart
}
fn astrology_algorithm_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_algorithm
}
fn astrology_engine_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_engine
}
fn astrology_ephemeris_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_ephemeris
}
fn astrology_instant_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_instant_utc
}
fn astrology_latitude_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_latitude
}
fn astrology_longitude_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_longitude
}
fn astrology_orb_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_orb
}
fn astrology_positions_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_positions
}
