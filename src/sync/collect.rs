// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Collection of the selected local graph truth for export.

use muniment::Backend;

use super::shared::{SelectedNode, invalid, selected_node, validate_identity};
use super::{CleromancySyncError, CleromancySyncSelection};
use crate::host::{
    ASTROLOGY_CHART_FACET, ASTROLOGY_FACTS_FACET, CONCURRENCE_FACET, CONTEXT_FACET, FIELD_FACET,
    READING_FACET, REFLECTION_FACET, SESSION_FACET, SPREAD_FACET, SPREAD_TEMPLATE_FACET,
    THREE_CARD_SPREAD_FACET,
};
use crate::{
    AstrologyChart, AstrologyFacts, CleromancyHost, Concurrence, ContextSnapshot, Field, Reading,
    ReadingSession, Reflection, Spread, SpreadTemplate, ThreeCardSpread,
};

/// Every selected node, decoded, identity-checked, and sorted by node id.
pub(super) struct SelectedTruth {
    pub(super) contexts: Vec<(String, SelectedNode)>,
    pub(super) fields: Vec<(String, SelectedNode)>,
    pub(super) readings: Vec<(Reading, SelectedNode)>,
    pub(super) sessions: Vec<(ReadingSession, SelectedNode)>,
    pub(super) spreads: Vec<(ThreeCardSpread, SelectedNode)>,
    pub(super) spread_templates: Vec<(SpreadTemplate, SelectedNode)>,
    pub(super) authored_spreads: Vec<(Spread, SelectedNode)>,
    pub(super) charts: Vec<(AstrologyChart, SelectedNode)>,
    pub(super) facts: Vec<(AstrologyFacts, SelectedNode)>,
    pub(super) concurrences: Vec<(Concurrence, SelectedNode)>,
    pub(super) reflections: Vec<(Reflection, SelectedNode)>,
}

pub(super) fn collect_selected<B: Backend>(
    host: &CleromancyHost<B>,
    selection: CleromancySyncSelection,
) -> Result<SelectedTruth, CleromancySyncError> {
    let mut contexts = Vec::<(String, SelectedNode)>::new();
    let mut fields = Vec::<(String, SelectedNode)>::new();
    let mut readings = Vec::<(Reading, SelectedNode)>::new();
    let mut sessions = Vec::<(ReadingSession, SelectedNode)>::new();
    let mut spreads = Vec::<(ThreeCardSpread, SelectedNode)>::new();
    let mut spread_templates = Vec::<(SpreadTemplate, SelectedNode)>::new();
    let mut authored_spreads = Vec::<(Spread, SelectedNode)>::new();
    let mut charts = Vec::<(AstrologyChart, SelectedNode)>::new();
    let mut facts = Vec::<(AstrologyFacts, SelectedNode)>::new();
    let mut concurrences = Vec::<(Concurrence, SelectedNode)>::new();
    let mut reflections = Vec::<(Reflection, SelectedNode)>::new();

    if selection.includes_contexts() {
        for (key, node) in host.graph().nodes() {
            let context = host.facet_value(key, CONTEXT_FACET);
            let field = host.facet_value(key, FIELD_FACET);
            let reading = host.facet_value(key, READING_FACET);
            let session = host.facet_value(key, SESSION_FACET);
            let spread = host.facet_value(key, THREE_CARD_SPREAD_FACET);
            let spread_template = host.facet_value(key, SPREAD_TEMPLATE_FACET);
            let authored_spread = host.facet_value(key, SPREAD_FACET);
            let chart = host.facet_value(key, ASTROLOGY_CHART_FACET);
            let facts_value = host.facet_value(key, ASTROLOGY_FACTS_FACET);
            let concurrence = host.facet_value(key, CONCURRENCE_FACET);
            let reflection = host.facet_value(key, REFLECTION_FACET);
            if [
                context.is_some(),
                field.is_some(),
                reading.is_some(),
                session.is_some(),
                spread.is_some(),
                spread_template.is_some(),
                authored_spread.is_some(),
                chart.is_some(),
                facts_value.is_some(),
                concurrence.is_some(),
                reflection.is_some(),
            ]
            .into_iter()
            .filter(|present| *present)
            .count()
                > 1
            {
                return Err(invalid(
                    node.id,
                    "carries more than one Cleromancy domain facet",
                ));
            }
            if let Some(value) = context {
                let context: ContextSnapshot = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("context facet does not decode: {e}")))?;
                let digest = context.digest();
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://context/{digest}"),
                )?;
                contexts.push((digest, selected_node(node, CONTEXT_FACET, value.clone())));
            } else if selection.includes_readings()
                && let Some(value) = field
            {
                let field: Field = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("field facet does not decode: {e}")))?;
                let digest = field.digest();
                validate_identity(node.id, node.url(), &format!("cleromancy://field/{digest}"))?;
                fields.push((digest, selected_node(node, FIELD_FACET, value.clone())));
            } else if selection.includes_readings()
                && let Some(value) = reading
            {
                let reading: Reading = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("reading facet does not decode: {e}")))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://reading/{}", reading.id),
                )?;
                readings.push((reading, selected_node(node, READING_FACET, value.clone())));
            } else if selection.includes_sessions()
                && let Some(value) = session
            {
                let session: ReadingSession = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("session facet does not decode: {e}")))?;
                session
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://session/{}", session.id),
                )?;
                sessions.push((session, selected_node(node, SESSION_FACET, value.clone())));
            } else if selection.includes_sessions()
                && let Some(value) = spread
            {
                let spread: ThreeCardSpread = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("spread facet does not decode: {e}")))?;
                spread
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://spread/three-card/{}", spread.id),
                )?;
                spreads.push((
                    spread,
                    selected_node(node, THREE_CARD_SPREAD_FACET, value.clone()),
                ));
            } else if selection.includes_sessions()
                && let Some(value) = spread_template
            {
                let template: SpreadTemplate =
                    serde_json::from_value(value.clone()).map_err(|e| {
                        invalid(
                            node.id,
                            format!("spread template facet does not decode: {e}"),
                        )
                    })?;
                template
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://spread-template/{}", template.id),
                )?;
                spread_templates.push((
                    template,
                    selected_node(node, SPREAD_TEMPLATE_FACET, value.clone()),
                ));
            } else if selection.includes_sessions()
                && let Some(value) = authored_spread
            {
                let spread: Spread = serde_json::from_value(value.clone()).map_err(|e| {
                    invalid(
                        node.id,
                        format!("authored spread facet does not decode: {e}"),
                    )
                })?;
                spread
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://spread/{}", spread.id),
                )?;
                authored_spreads.push((spread, selected_node(node, SPREAD_FACET, value.clone())));
            } else if selection.includes_astrology()
                && let Some(value) = chart
            {
                let chart: AstrologyChart = serde_json::from_value(value.clone()).map_err(|e| {
                    invalid(node.id, format!("astrology chart does not decode: {e}"))
                })?;
                chart
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                let digest = chart.digest();
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://astrology/chart/{digest}"),
                )?;
                charts.push((
                    chart,
                    selected_node(node, ASTROLOGY_CHART_FACET, value.clone()),
                ));
            } else if selection.includes_astrology()
                && let Some(value) = facts_value
            {
                let astrology_facts: AstrologyFacts = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(node.id, format!("astrology facts do not decode: {e}")))?;
                let digest = astrology_facts.digest();
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://astrology/facts/{digest}"),
                )?;
                facts.push((
                    astrology_facts,
                    selected_node(node, ASTROLOGY_FACTS_FACET, value.clone()),
                ));
            } else if selection.includes_concurrences()
                && let Some(value) = concurrence
            {
                let concurrence: Concurrence =
                    serde_json::from_value(value.clone()).map_err(|e| {
                        invalid(node.id, format!("concurrence facet does not decode: {e}"))
                    })?;
                concurrence
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://concurrence/{}", concurrence.id),
                )?;
                concurrences.push((
                    concurrence,
                    selected_node(node, CONCURRENCE_FACET, value.clone()),
                ));
            } else if selection.includes_reflections()
                && let Some(value) = reflection
            {
                let reflection: Reflection =
                    serde_json::from_value(value.clone()).map_err(|e| {
                        invalid(node.id, format!("reflection facet does not decode: {e}"))
                    })?;
                reflection
                    .validate()
                    .map_err(|e| invalid(node.id, e.to_string()))?;
                validate_identity(
                    node.id,
                    node.url(),
                    &format!("cleromancy://reflection/{}", reflection.id),
                )?;
                reflections.push((
                    reflection,
                    selected_node(node, REFLECTION_FACET, value.clone()),
                ));
            }
        }
    }

    contexts.sort_by_key(|(_, node)| node.id);
    fields.sort_by_key(|(_, node)| node.id);
    readings.sort_by_key(|(_, node)| node.id);
    sessions.sort_by_key(|(_, node)| node.id);
    spreads.sort_by_key(|(_, node)| node.id);
    spread_templates.sort_by_key(|(_, node)| node.id);
    authored_spreads.sort_by_key(|(_, node)| node.id);
    charts.sort_by_key(|(_, node)| node.id);
    facts.sort_by_key(|(_, node)| node.id);
    concurrences.sort_by_key(|(_, node)| node.id);
    reflections.sort_by_key(|(_, node)| node.id);

    Ok(SelectedTruth {
        contexts,
        fields,
        readings,
        sessions,
        spreads,
        spread_templates,
        authored_spreads,
        charts,
        facts,
        concurrences,
        reflections,
    })
}
