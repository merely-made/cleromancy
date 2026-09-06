// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stored chart receipts and manual chart import. Calculation remains an
//! optional adapter action, while this reader only projects saved values.

use cambium::{
    AngleStripMark, GridColumn, GridSpec, GridView, Key, NamedKey, SelectState, TextInput, button,
    custom_leaf, data_grid, el, map_action, map_state, on_click, on_key, select,
};

use super::{ConsultationView, labelled_control, labelled_text, never_select_action};
use crate::ui::chart_state::ChartState;
use crate::ui::state::ConsultationUi;
use crate::{AstrologyChart, AstrologyFacts, AstrologyPlacement, AstrologyPosition};

pub(in crate::ui) const CHART_ANGLE_STRIP_KEY: u64 = 0x434c_4552_414e_474c;
const CHART_ANGLE_STRIP_WIDTH: u32 = 720;
const CHART_ANGLE_STRIP_HEIGHT: u32 = 36;

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
    let picker = map_state(picker, selected_chart_state);
    // Aspect choice is scoped to the selected chart. Reset it on every chart
    // picker navigation or pointer turn so an index from a larger aspect set
    // cannot reappear against another chart.
    let picker = on_click(
        el::<_, ChartState, ()>("div", picker),
        |chart: &mut ChartState, _| {
            chart.selected_aspect.selected = 0;
        },
    );
    let picker = on_key(picker, |chart: &mut ChartState, event| {
        if matches!(
            event.key,
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowUp | NamedKey::Home | NamedKey::End)
        ) {
            chart.selected_aspect.selected = 0;
        }
    });
    let picker = map_state(picker, chart_state);
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
        children.extend(stored_chart(ui, &stored.chart, &stored.facts));
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

fn stored_chart(
    ui: &ConsultationUi,
    chart: &AstrologyChart,
    facts: &AstrologyFacts,
) -> Vec<ConsultationView> {
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
        ecliptic_strip(chart, facts),
        positions_grid(chart, facts),
    ];
    views.push(Box::new(el::<_, ConsultationUi, ()>("h3", "Aspects")) as ConsultationView);
    // Keep the complete receipt grid beside the selected, view-local dimension
    // projection.
    views.extend(super::chart_aspect::selected_aspect_views(ui, chart, facts));
    views.extend([
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

fn ecliptic_strip(chart: &AstrologyChart, facts: &AstrologyFacts) -> ConsultationView {
    let labels = chart
        .positions
        .iter()
        .map(|position| {
            let placement = facts
                .placements
                .iter()
                .find(|placement| placement.body == position.body);
            let sign = placement.map_or_else(
                || "sign unavailable".to_string(),
                |placement| {
                    format!(
                        "{:?}, sign degree {} millidegrees",
                        placement.sign, placement.degree_millidegrees
                    )
                },
            );
            el::<_, ConsultationUi, ()>(
                "li",
                format!(
                    "{}: longitude {} millidegrees; latitude {} millidegrees; {}; {}",
                    position.body,
                    position.longitude_millidegrees,
                    position.latitude_millidegrees,
                    sign,
                    motion_label(position),
                ),
            )
            .attr("data-angle-body", position.body.clone())
        })
        .collect::<Vec<_>>();
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            vec![
                Box::new(
                    el::<_, ConsultationUi, ()>(
                        "p",
                        "Cyclic overview from 0 through 360 degrees. The complete stored values remain in the grid below.",
                    )
                    .attr("class", "chart-ecliptic-explanation"),
                ) as ConsultationView,
                Box::new(
                    custom_leaf::<ConsultationUi, ()>(
                        CHART_ANGLE_STRIP_KEY,
                        CHART_ANGLE_STRIP_WIDTH,
                        CHART_ANGLE_STRIP_HEIGHT,
                    )
                    .attr("aria-hidden", "true")
                    .attr("data-key", "chart-ecliptic-leaf"),
                ) as ConsultationView,
                Box::new(
                    el::<_, ConsultationUi, ()>("ul", labels)
                        .attr("class", "chart-ecliptic-labels")
                        .attr("data-key", "chart-ecliptic-labels"),
                ) as ConsultationView,
            ],
        )
        .attr("role", "group")
        .attr("aria-label", "Ecliptic positions")
        .attr("data-key", "chart-ecliptic-strip"),
    )
}

pub(in crate::ui) fn selected_angle_strip_marks(
    ui: &ConsultationUi,
) -> Option<Vec<AngleStripMark>> {
    if ui.screen().key() != "chart" {
        return None;
    }
    let stored = ui
        .catalog
        .astrology_charts
        .get(ui.chart.selected_chart.selected)?;
    Some(
        stored
            .chart
            .positions
            .iter()
            .map(|position| {
                let (red, green, blue) = marker_color(&position.body);
                AngleStripMark::rgb(
                    position.longitude_millidegrees as f32 / 360_000.0,
                    red,
                    green,
                    blue,
                )
            })
            .collect(),
    )
}

pub(in crate::ui) fn selected_aspect_dimension(
    ui: &ConsultationUi,
) -> Option<super::chart_aspect::AspectDimension> {
    if ui.screen().key() != "chart" {
        return None;
    }
    let stored = ui
        .catalog
        .astrology_charts
        .get(ui.chart.selected_chart.selected)?;
    super::chart_aspect::selected_dimension(ui, &stored.chart, &stored.facts)
}

fn marker_color(body: &str) -> (f32, f32, f32) {
    match body {
        "Sun" => (0.95, 0.72, 0.25),
        "Moon" => (0.65, 0.75, 0.92),
        "Mercury" => (0.72, 0.62, 0.48),
        "Venus" => (0.82, 0.48, 0.68),
        "Mars" => (0.84, 0.34, 0.25),
        "Jupiter" => (0.72, 0.48, 0.24),
        "Saturn" => (0.62, 0.58, 0.42),
        "Uranus" => (0.30, 0.72, 0.72),
        "Neptune" => (0.32, 0.48, 0.86),
        "Pluto" => (0.58, 0.38, 0.64),
        _ => (0.78, 0.78, 0.76),
    }
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
        _ => motion_label(position).to_string(),
    };
    Box::new(el::<_, ConsultationUi, ()>("span", value))
}

fn motion_label(position: &AstrologyPosition) -> &'static str {
    match position.retrograde {
        Some(true) => "retrograde",
        Some(false) => "direct",
        None => "unknown",
    }
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

fn chart_state(ui: &mut ConsultationUi) -> &mut ChartState {
    &mut ui.chart
}
fn selected_chart_state(chart: &mut ChartState) -> &mut SelectState {
    &mut chart.selected_chart
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
