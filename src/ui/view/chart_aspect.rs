// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View-local selection and projection of one stored chart aspect.
//!
//! The complete aspect grid remains the receipt surface. This module adds a
//! second, read-only projection for the selected relation and keeps its
//! geometry separate from the labels and provenance shown in the DOM.

use cambium::{
    DimensionLineTraversal, SelectState, custom_leaf, el, map_action, map_state, select,
};

use super::{ConsultationView, never_select_action};
use crate::ui::state::ConsultationUi;
use crate::{AstrologyAspect, AstrologyChart, AstrologyFacts, AstrologyPosition};

pub(in crate::ui) const CHART_ASPECT_DIMENSION_KEY: u64 = 0x434c_4552_41535043;
const CHART_ASPECT_DIMENSION_WIDTH: u32 = 720;
const CHART_ASPECT_DIMENSION_HEIGHT: u32 = 36;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui) struct AspectDimension {
    pub(in crate::ui) original_first: String,
    pub(in crate::ui) original_second: String,
    pub(in crate::ui) kind: crate::AspectKind,
    pub(in crate::ui) first_longitude_millidegrees: u32,
    pub(in crate::ui) second_longitude_millidegrees: u32,
    pub(in crate::ui) start_body: String,
    pub(in crate::ui) end_body: String,
    pub(in crate::ui) start_millidegrees: u32,
    pub(in crate::ui) end_millidegrees: u32,
    pub(in crate::ui) traversal: DimensionLineTraversal,
    pub(in crate::ui) measured_separation_millidegrees: u32,
    pub(in crate::ui) exact_target_millidegrees: u32,
    pub(in crate::ui) orb_millidegrees: u32,
    pub(in crate::ui) start: f32,
    pub(in crate::ui) end: f32,
    pub(in crate::ui) target: f32,
}

pub(super) fn selected_aspect_views(
    ui: &ConsultationUi,
    chart: &AstrologyChart,
    facts: &AstrologyFacts,
) -> Vec<ConsultationView> {
    let mut views = Vec::new();
    if !facts.aspects.is_empty() {
        views.push(aspect_selector(ui, facts));
    }
    if let Some(dimension) = selected_dimension(ui, chart, facts) {
        views.push(selected_dimension_view(&dimension, facts));
    }
    views
}

pub(in crate::ui) fn selected_dimension(
    ui: &ConsultationUi,
    chart: &AstrologyChart,
    facts: &AstrologyFacts,
) -> Option<AspectDimension> {
    let index = ui
        .chart
        .selected_aspect
        .selected
        .min(facts.aspects.len().saturating_sub(1));
    facts
        .aspects
        .get(index)
        .and_then(|aspect| project_aspect(aspect, &chart.positions))
}

fn aspect_selector(ui: &ConsultationUi, facts: &AstrologyFacts) -> ConsultationView {
    let labels = facts
        .aspects
        .iter()
        .map(|aspect| {
            format!(
                "{} / {} · {:?} · {} millidegrees",
                aspect.first, aspect.second, aspect.kind, aspect.separation_millidegrees
            )
        })
        .collect::<Vec<_>>();
    let refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let mut state = ui.chart.selected_aspect.clone();
    state.selected = state.selected.min(refs.len().saturating_sub(1));
    let picker = map_action(select(&state, &refs), never_select_action);
    let picker = map_state(picker, aspect_select_state);
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            (
                el("span", "Aspect relation").attr("class", "control-label"),
                picker,
            ),
        )
        .attr("class", "control")
        .attr("id", "cleromancy-aspect-select")
        .attr("data-control", "Aspect relation"),
    )
}

fn selected_dimension_view(
    dimension: &AspectDimension,
    facts: &AstrologyFacts,
) -> ConsultationView {
    let route = match dimension.traversal {
        DimensionLineTraversal::Direct => "direct",
        DimensionLineTraversal::Wrapped => "wrapped",
    };
    let labels = vec![
        value(
            "Original aspect bodies",
            format!(
                "{} / {}",
                dimension.original_first, dimension.original_second
            ),
            "chart-aspect-original-bodies",
        ),
        value(
            "Displayed endpoints",
            format!(
                "{} ({} millidegrees) -> {} ({} millidegrees)",
                dimension.start_body,
                dimension.start_millidegrees,
                dimension.end_body,
                dimension.end_millidegrees
            ),
            "chart-aspect-endpoints",
        ),
        value(
            "Raw longitudes",
            format!(
                "{}={} millidegrees; {}={} millidegrees",
                dimension.original_first,
                dimension.first_longitude_millidegrees,
                dimension.original_second,
                dimension.second_longitude_millidegrees
            ),
            "chart-aspect-raw-longitudes",
        ),
        value("Kind", format!("{:?}", dimension.kind), "chart-aspect-kind"),
        value(
            "Measured separation",
            format!(
                "{} millidegrees",
                dimension.measured_separation_millidegrees
            ),
            "chart-aspect-measured-separation",
        ),
        value(
            "Exact target",
            format!("{} millidegrees", dimension.exact_target_millidegrees),
            "chart-aspect-exact-target",
        ),
        value(
            "Orb",
            format!("{} millidegrees", dimension.orb_millidegrees),
            "chart-aspect-orb",
        ),
        value("Units", "millidegrees".to_string(), "chart-aspect-units"),
        value("Route", route.to_string(), "chart-aspect-route"),
        value(
            "Chart digest",
            facts.chart_digest.clone(),
            "chart-aspect-chart-digest",
        ),
        value("Facts digest", facts.digest(), "chart-aspect-facts-digest"),
    ];
    let leaf = custom_leaf::<ConsultationUi, ()>(
        CHART_ASPECT_DIMENSION_KEY,
        CHART_ASPECT_DIMENSION_WIDTH,
        CHART_ASPECT_DIMENSION_HEIGHT,
    )
    .attr("aria-hidden", "true")
    .attr("data-key", "chart-selected-aspect-leaf");
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            vec![
                Box::new(leaf) as ConsultationView,
                Box::new(
                    el::<_, ConsultationUi, ()>("div", labels)
                        .attr("data-key", "chart-selected-aspect-details"),
                ) as ConsultationView,
            ],
        )
        .attr("role", "group")
        .attr("aria-label", "Selected aspect dimension")
        .attr("data-key", "chart-selected-aspect"),
    )
}

fn value(label: &str, value: String, key: &str) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ()>("p", format!("{label}: {value}"))
            .attr("data-key", key.to_string()),
    )
}

fn project_aspect(
    aspect: &AstrologyAspect,
    positions: &[AstrologyPosition],
) -> Option<AspectDimension> {
    let first = positions
        .iter()
        .find(|position| position.body == aspect.first)?;
    let second = positions
        .iter()
        .find(|position| position.body == aspect.second)?;
    let raw_difference = first
        .longitude_millidegrees
        .abs_diff(second.longitude_millidegrees);
    let (start_body, start_millidegrees, end_body, end_millidegrees, traversal) =
        if raw_difference <= 180_000 {
            if first.longitude_millidegrees <= second.longitude_millidegrees {
                (
                    first.body.clone(),
                    first.longitude_millidegrees,
                    second.body.clone(),
                    second.longitude_millidegrees,
                    DimensionLineTraversal::Direct,
                )
            } else {
                (
                    second.body.clone(),
                    second.longitude_millidegrees,
                    first.body.clone(),
                    first.longitude_millidegrees,
                    DimensionLineTraversal::Direct,
                )
            }
        } else if first.longitude_millidegrees >= second.longitude_millidegrees {
            (
                first.body.clone(),
                first.longitude_millidegrees,
                second.body.clone(),
                second.longitude_millidegrees,
                DimensionLineTraversal::Wrapped,
            )
        } else {
            (
                second.body.clone(),
                second.longitude_millidegrees,
                first.body.clone(),
                first.longitude_millidegrees,
                DimensionLineTraversal::Wrapped,
            )
        };
    let exact_target_millidegrees = (start_millidegrees + aspect.exact_millidegrees) % 360_000;
    Some(AspectDimension {
        original_first: aspect.first.clone(),
        original_second: aspect.second.clone(),
        kind: aspect.kind,
        first_longitude_millidegrees: first.longitude_millidegrees,
        second_longitude_millidegrees: second.longitude_millidegrees,
        start_body,
        end_body,
        start: start_millidegrees as f32 / 360_000.0,
        end: end_millidegrees as f32 / 360_000.0,
        start_millidegrees,
        end_millidegrees,
        traversal,
        measured_separation_millidegrees: aspect.separation_millidegrees,
        exact_target_millidegrees,
        orb_millidegrees: aspect.orb_millidegrees,
        target: exact_target_millidegrees as f32 / 360_000.0,
    })
}

fn aspect_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.chart.selected_aspect
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AspectKind;

    fn aspect(first: &str, second: &str, exact: u32, separation: u32) -> AstrologyAspect {
        AstrologyAspect {
            first: first.to_string(),
            second: second.to_string(),
            kind: match exact {
                0 => AspectKind::Conjunction,
                60_000 => AspectKind::Sextile,
                90_000 => AspectKind::Square,
                120_000 => AspectKind::Trine,
                _ => AspectKind::Opposition,
            },
            exact_millidegrees: exact,
            separation_millidegrees: separation,
            orb_millidegrees: 1_000,
        }
    }

    #[test]
    fn ordinary_aspect_uses_increasing_direct_endpoints() {
        let positions = [
            AstrologyPosition::new("A", 10_000, 0),
            AstrologyPosition::new("B", 70_000, 0),
        ];
        let projected = project_aspect(&aspect("A", "B", 60_000, 60_000), &positions).unwrap();
        assert_eq!(projected.traversal, DimensionLineTraversal::Direct);
        assert_eq!(
            (projected.start_millidegrees, projected.end_millidegrees),
            (10_000, 70_000)
        );
        assert_eq!(projected.exact_target_millidegrees, 70_000);
    }

    #[test]
    fn wrapped_aspect_uses_descending_raw_order_and_wraps_target() {
        let positions = [
            AstrologyPosition::new("A", 350_000, 0),
            AstrologyPosition::new("B", 50_000, 0),
        ];
        let projected = project_aspect(&aspect("A", "B", 60_000, 60_000), &positions).unwrap();
        assert_eq!(projected.traversal, DimensionLineTraversal::Wrapped);
        assert_eq!(
            (projected.start_millidegrees, projected.end_millidegrees),
            (350_000, 50_000)
        );
        assert_eq!(projected.exact_target_millidegrees, 50_000);
    }

    #[test]
    fn coincident_aspect_stays_direct_at_one_tick() {
        let positions = [
            AstrologyPosition::new("A", 120_000, 0),
            AstrologyPosition::new("B", 120_000, 0),
        ];
        let projected = project_aspect(&aspect("A", "B", 0, 0), &positions).unwrap();
        assert_eq!(projected.traversal, DimensionLineTraversal::Direct);
        assert_eq!(projected.start_millidegrees, projected.end_millidegrees);
        assert_eq!(projected.target, projected.start);
    }
}
