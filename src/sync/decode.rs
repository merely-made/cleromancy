// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Decoding of the selected facets from an H7 materialized projection.

use chartulary::FacetId;
use graphshell::personal_sync::SyncProjection;

use super::shared::{invalid, validate_identity};
use super::{CleromancySyncError, CleromancySyncSelection};
use crate::host::{
    ASTROLOGY_CHART_FACET, ASTROLOGY_FACTS_FACET, CONCURRENCE_FACET, CONTEXT_FACET, FIELD_FACET,
    READING_FACET, REFLECTION_FACET, SESSION_FACET, SPREAD_FACET, SPREAD_TEMPLATE_FACET,
    THREE_CARD_SPREAD_FACET,
};
use crate::{
    AstrologyChart, AstrologyFacts, Concurrence, ContextSnapshot, Field, Reading, ReadingSession,
    Reflection, Spread, SpreadTemplate, ThreeCardSpread,
};

/// Every selected projection value, decoded, identity-checked, and sorted by
/// canonical identity.
pub(super) struct DecodedTruth {
    pub(super) contexts: Vec<ContextSnapshot>,
    pub(super) fields: Vec<Field>,
    pub(super) readings: Vec<Reading>,
    pub(super) sessions: Vec<ReadingSession>,
    pub(super) spreads: Vec<ThreeCardSpread>,
    pub(super) spread_templates: Vec<SpreadTemplate>,
    pub(super) authored_spreads: Vec<Spread>,
    pub(super) charts: Vec<AstrologyChart>,
    pub(super) facts: Vec<AstrologyFacts>,
    pub(super) concurrences: Vec<Concurrence>,
    pub(super) reflections: Vec<Reflection>,
}

pub(super) fn decode_projection(
    projection: &SyncProjection,
    selection: CleromancySyncSelection,
) -> Result<DecodedTruth, CleromancySyncError> {
    let context_facet = FacetId::new(CONTEXT_FACET);
    let field_facet = FacetId::new(FIELD_FACET);
    let reading_facet = FacetId::new(READING_FACET);
    let session_facet = FacetId::new(SESSION_FACET);
    let spread_facet = FacetId::new(THREE_CARD_SPREAD_FACET);
    let spread_template_facet = FacetId::new(SPREAD_TEMPLATE_FACET);
    let authored_spread_facet = FacetId::new(SPREAD_FACET);
    let chart_facet = FacetId::new(ASTROLOGY_CHART_FACET);
    let facts_facet = FacetId::new(ASTROLOGY_FACTS_FACET);
    let concurrence_facet = FacetId::new(CONCURRENCE_FACET);
    let reflection_facet = FacetId::new(REFLECTION_FACET);
    let mut contexts = Vec::<ContextSnapshot>::new();
    let mut fields = Vec::<Field>::new();
    let mut readings = Vec::<Reading>::new();
    let mut sessions = Vec::<ReadingSession>::new();
    let mut spreads = Vec::<ThreeCardSpread>::new();
    let mut spread_templates = Vec::<SpreadTemplate>::new();
    let mut authored_spreads = Vec::<Spread>::new();
    let mut charts = Vec::<AstrologyChart>::new();
    let mut facts = Vec::<AstrologyFacts>::new();
    let mut concurrences = Vec::<Concurrence>::new();
    let mut reflections = Vec::<Reflection>::new();
    for (_, node) in projection.graph.nodes() {
        let context = projection.graph.facets().get(&node.id, &context_facet);
        let field = projection.graph.facets().get(&node.id, &field_facet);
        let reading = projection.graph.facets().get(&node.id, &reading_facet);
        let session = projection.graph.facets().get(&node.id, &session_facet);
        let spread = projection.graph.facets().get(&node.id, &spread_facet);
        let spread_template = projection
            .graph
            .facets()
            .get(&node.id, &spread_template_facet);
        let authored_spread = projection
            .graph
            .facets()
            .get(&node.id, &authored_spread_facet);
        let chart = projection.graph.facets().get(&node.id, &chart_facet);
        let facts_value = projection.graph.facets().get(&node.id, &facts_facet);
        let concurrence = projection.graph.facets().get(&node.id, &concurrence_facet);
        let reflection = projection.graph.facets().get(&node.id, &reflection_facet);
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
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://context/{}", context.digest()),
            )?;
            contexts.push(context);
        } else if selection.includes_readings()
            && let Some(value) = field
        {
            let field: Field = serde_json::from_value(value.clone())
                .map_err(|e| invalid(node.id, format!("field facet does not decode: {e}")))?;
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://field/{}", field.digest()),
            )?;
            fields.push(field);
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
            readings.push(reading);
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
            sessions.push(session);
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
            spreads.push(spread);
        } else if selection.includes_sessions()
            && let Some(value) = spread_template
        {
            let template: SpreadTemplate = serde_json::from_value(value.clone()).map_err(|e| {
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
            spread_templates.push(template);
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
            authored_spreads.push(spread);
        } else if selection.includes_astrology()
            && let Some(value) = chart
        {
            let chart: AstrologyChart = serde_json::from_value(value.clone())
                .map_err(|e| invalid(node.id, format!("astrology chart does not decode: {e}")))?;
            chart
                .validate()
                .map_err(|e| invalid(node.id, e.to_string()))?;
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://astrology/chart/{}", chart.digest()),
            )?;
            charts.push(chart);
        } else if selection.includes_astrology()
            && let Some(value) = facts_value
        {
            let astrology_facts: AstrologyFacts = serde_json::from_value(value.clone())
                .map_err(|e| invalid(node.id, format!("astrology facts do not decode: {e}")))?;
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://astrology/facts/{}", astrology_facts.digest()),
            )?;
            facts.push(astrology_facts);
        } else if selection.includes_concurrences()
            && let Some(value) = concurrence
        {
            let concurrence: Concurrence = serde_json::from_value(value.clone())
                .map_err(|e| invalid(node.id, format!("concurrence facet does not decode: {e}")))?;
            concurrence
                .validate()
                .map_err(|e| invalid(node.id, e.to_string()))?;
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://concurrence/{}", concurrence.id),
            )?;
            concurrences.push(concurrence);
        } else if selection.includes_reflections()
            && let Some(value) = reflection
        {
            let reflection: Reflection = serde_json::from_value(value.clone())
                .map_err(|e| invalid(node.id, format!("reflection facet does not decode: {e}")))?;
            reflection
                .validate()
                .map_err(|e| invalid(node.id, e.to_string()))?;
            validate_identity(
                node.id,
                node.url(),
                &format!("cleromancy://reflection/{}", reflection.id),
            )?;
            reflections.push(reflection);
        }
    }
    contexts.sort_by_key(|context| context.digest());
    fields.sort_by_key(Field::digest);
    readings.sort_by(|left, right| left.id.cmp(&right.id));
    sessions.sort_by(|left, right| left.id.cmp(&right.id));
    spreads.sort_by(|left, right| left.id.cmp(&right.id));
    spread_templates.sort_by(|left, right| left.id.cmp(&right.id));
    authored_spreads.sort_by(|left, right| left.id.cmp(&right.id));
    charts.sort_by_key(AstrologyChart::digest);
    facts.sort_by_key(AstrologyFacts::digest);
    concurrences.sort_by(|left, right| left.id.cmp(&right.id));
    reflections.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(DecodedTruth {
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
