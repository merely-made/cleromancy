// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validation and local materialization of a decoded H7 projection.

use std::collections::{BTreeMap, BTreeSet};

use graphshell::personal_sync::SyncProjection;
use mere::kernel::graph::Graph;
use muniment::Backend;

use super::decode::{DecodedTruth, decode_projection};
use super::shared::invalid;
use super::{CleromancySyncError, CleromancySyncImport, CleromancySyncSelection};
use crate::host::{
    ASTROLOGY_CHART_FACET, ASTROLOGY_FACTS_FACET, CONCURRENCE_FACET, CONTEXT_FACET, FIELD_FACET,
    READING_FACET, REFLECTION_FACET, SESSION_FACET, SPREAD_FACET, SPREAD_TEMPLATE_FACET,
    THREE_CARD_SPREAD_FACET,
};
use crate::{CleromancyHost, HostError, ReadingEngine, ReadingError, Spread};

/// Merge the selected Cleromancy facets from an H7 materialization into the
/// local graph. The complete projection is validated before local mutation.
/// Deletions are deliberately not imported in A4.
pub fn import_sync_projection<B: Backend>(
    host: &mut CleromancyHost<B>,
    projection: &SyncProjection,
    selection: CleromancySyncSelection,
) -> Result<CleromancySyncImport, CleromancySyncError> {
    if matches!(selection, CleromancySyncSelection::Off) {
        return Ok(CleromancySyncImport::default());
    }
    if !projection.pending.is_empty() {
        return Err(CleromancySyncError::PendingHistory(
            projection.pending.len(),
        ));
    }
    for conflict in &projection.conflicts {
        let context = format!("/facet/{CONTEXT_FACET}");
        let field = format!("/facet/{FIELD_FACET}");
        let reading = format!("/facet/{READING_FACET}");
        let session = format!("/facet/{SESSION_FACET}");
        let spread = format!("/facet/{THREE_CARD_SPREAD_FACET}");
        let spread_template = format!("/facet/{SPREAD_TEMPLATE_FACET}");
        let authored_spread = format!("/facet/{SPREAD_FACET}");
        let chart = format!("/facet/{ASTROLOGY_CHART_FACET}");
        let facts = format!("/facet/{ASTROLOGY_FACTS_FACET}");
        let concurrence = format!("/facet/{CONCURRENCE_FACET}");
        let reflection = format!("/facet/{REFLECTION_FACET}");
        if conflict.target.ends_with(&context)
            || (selection.includes_readings()
                && (conflict.target.ends_with(&field)
                    || conflict.target.ends_with(&reading)
                    || conflict.target.ends_with(&session)
                    || conflict.target.ends_with(&spread)
                    || conflict.target.ends_with(&spread_template)
                    || conflict.target.ends_with(&authored_spread)
                    || conflict.target.ends_with(&chart)
                    || conflict.target.ends_with(&facts)
                    || conflict.target.ends_with(&concurrence)))
            || (selection.includes_reflections() && conflict.target.ends_with(&reflection))
        {
            return Err(CleromancySyncError::Conflict(conflict.target.clone()));
        }
    }

    let DecodedTruth {
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
    } = decode_projection(projection, selection)?;

    let contexts_by_digest = contexts
        .iter()
        .map(|context| (context.digest(), context))
        .collect::<BTreeMap<_, _>>();
    let fields_by_digest = fields
        .iter()
        .map(|field| (field.digest(), field))
        .collect::<BTreeMap<_, _>>();
    let readings_by_id = readings
        .iter()
        .map(|reading| (reading.id.clone(), reading))
        .collect::<BTreeMap<_, _>>();
    let sessions_by_id = sessions
        .iter()
        .map(|session| (session.id.clone(), session))
        .collect::<BTreeMap<_, _>>();
    let spread_templates_by_id = spread_templates
        .iter()
        .map(|template| (template.id.clone(), template))
        .collect::<BTreeMap<_, _>>();
    let charts_by_digest = charts
        .iter()
        .map(|chart| (chart.digest(), chart))
        .collect::<BTreeMap<_, _>>();
    let fact_chart_digests = facts
        .iter()
        .map(|facts| facts.chart_digest.clone())
        .collect::<BTreeSet<_>>();
    let selected_addresses = contexts
        .iter()
        .map(|context| format!("cleromancy://context/{}", context.digest()))
        .chain(
            fields
                .iter()
                .map(|field| format!("cleromancy://field/{}", field.digest())),
        )
        .chain(
            readings
                .iter()
                .map(|reading| format!("cleromancy://reading/{}", reading.id)),
        )
        .chain(
            sessions
                .iter()
                .map(|session| format!("cleromancy://session/{}", session.id)),
        )
        .chain(
            spreads
                .iter()
                .map(|spread| format!("cleromancy://spread/three-card/{}", spread.id)),
        )
        .chain(
            spread_templates
                .iter()
                .map(|template| format!("cleromancy://spread-template/{}", template.id)),
        )
        .chain(
            authored_spreads
                .iter()
                .map(|spread| format!("cleromancy://spread/{}", spread.id)),
        )
        .chain(
            charts
                .iter()
                .map(|chart| format!("cleromancy://astrology/chart/{}", chart.digest())),
        )
        .chain(
            facts
                .iter()
                .map(|facts| format!("cleromancy://astrology/facts/{}", facts.digest())),
        )
        .chain(
            concurrences
                .iter()
                .map(|concurrence| format!("cleromancy://concurrence/{}", concurrence.id)),
        )
        .chain(
            reflections
                .iter()
                .map(|reflection| format!("cleromancy://reflection/{}", reflection.id)),
        )
        .collect::<BTreeSet<_>>();
    for facts in &facts {
        let Some(chart) = charts_by_digest.get(&facts.chart_digest) else {
            return Err(CleromancySyncError::MissingAstrologyChart {
                facts: facts.digest(),
                chart_digest: facts.chart_digest.clone(),
            });
        };
        facts.verify(chart).map_err(HostError::from)?;
    }
    for chart in &charts {
        if !fact_chart_digests.contains(&chart.digest()) {
            return Err(CleromancySyncError::MissingAstrologyFacts {
                chart_digest: chart.digest(),
            });
        }
    }
    for concurrence in &concurrences {
        for member in &concurrence.members {
            if !selected_addresses.contains(&member.address) {
                return Err(CleromancySyncError::MissingConcurrenceMember {
                    concurrence: concurrence.id.clone(),
                    address: member.address.clone(),
                });
            }
        }
    }
    for reading in &readings {
        let Some(context) = contexts_by_digest.get(&reading.receipt.context_digest) else {
            return Err(CleromancySyncError::MissingContext {
                reading: reading.id.clone(),
                context_digest: reading.receipt.context_digest.clone(),
            });
        };
        let Some(field) = fields_by_digest.get(&reading.receipt.field_digest) else {
            return Err(CleromancySyncError::MissingField {
                reading: reading.id.clone(),
                field_digest: reading.receipt.field_digest.clone(),
            });
        };
        let replayed =
            ReadingEngine::replay(context, field, &reading.receipt).map_err(HostError::from)?;
        if replayed != *reading {
            return Err(HostError::Reading(ReadingError::ReceiptMismatch(
                "sealed reading".to_string(),
            ))
            .into());
        }
    }
    for session in &sessions {
        let Some(context) = contexts_by_digest.get(&session.context_digest) else {
            return Err(CleromancySyncError::MissingSessionContext {
                session: session.id.clone(),
                context_digest: session.context_digest.clone(),
            });
        };
        let Some(field) = fields_by_digest.get(&session.field_digest) else {
            return Err(CleromancySyncError::MissingSessionField {
                session: session.id.clone(),
                field_digest: session.field_digest.clone(),
            });
        };
        for placement in &session.placements {
            let Some(reading) = readings_by_id.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSessionReading {
                    session: session.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            if reading.receipt.context_digest != session.context_digest
                || reading.receipt.field_digest != session.field_digest
            {
                return Err(invalid(
                    Graph::node_namespace_id(&format!("cleromancy://session/{}", session.id)),
                    "session placement does not share the session context and field",
                ));
            }
            let replayed =
                ReadingEngine::replay(context, field, &reading.receipt).map_err(HostError::from)?;
            if replayed != **reading {
                return Err(HostError::Reading(ReadingError::ReceiptMismatch(
                    "sealed reading".to_string(),
                ))
                .into());
            }
        }
    }
    for spread in &spreads {
        let Some(session) = sessions_by_id.get(&spread.session_id) else {
            return Err(CleromancySyncError::MissingSpreadSession {
                spread: spread.id.clone(),
                session: spread.session_id.clone(),
            });
        };
        for placement in &spread.placements {
            let Some(reading) = readings_by_id.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSpreadReading {
                    spread: spread.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            let Some(session_placement) = session
                .placements
                .iter()
                .find(|candidate| candidate.position == placement.position.as_str())
            else {
                return Err(invalid(
                    Graph::node_namespace_id(&format!(
                        "cleromancy://spread/three-card/{}",
                        spread.id
                    )),
                    "spread position is not in its session",
                ));
            };
            if session_placement.reading_id != placement.reading_id
                || reading.receipt.context_digest != session.context_digest
                || reading.receipt.field_digest != session.field_digest
            {
                return Err(invalid(
                    Graph::node_namespace_id(&format!(
                        "cleromancy://spread/three-card/{}",
                        spread.id
                    )),
                    "spread placement does not match its session",
                ));
            }
        }
    }
    for spread in &authored_spreads {
        let Some(template) = spread_templates_by_id.get(&spread.template_id) else {
            return Err(CleromancySyncError::MissingSpreadTemplate {
                spread: spread.id.clone(),
                template: spread.template_id.clone(),
            });
        };
        let Some(session) = sessions_by_id.get(&spread.session_id) else {
            return Err(CleromancySyncError::MissingSpreadSession {
                spread: spread.id.clone(),
                session: spread.session_id.clone(),
            });
        };
        let expected = Spread::new(template, session).map_err(HostError::from)?;
        if expected != *spread {
            return Err(invalid(
                Graph::node_namespace_id(&format!("cleromancy://spread/{}", spread.id)),
                "spread does not match its template and session",
            ));
        }
        for placement in &session.placements {
            let Some(reading) = readings_by_id.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSpreadReading {
                    spread: spread.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            if reading.receipt.context_digest != session.context_digest
                || reading.receipt.field_digest != session.field_digest
            {
                return Err(invalid(
                    Graph::node_namespace_id(&format!("cleromancy://spread/{}", spread.id)),
                    "spread session reading does not match its session",
                ));
            }
        }
    }
    for reflection in &reflections {
        if !sessions_by_id.contains_key(&reflection.session_id) {
            return Err(CleromancySyncError::MissingReflectionSession {
                reflection: reflection.id.clone(),
                session: reflection.session_id.clone(),
            });
        }
    }

    for context in &contexts {
        host.insert_context(context)?;
    }
    for field in &fields {
        host.insert_field(field)?;
    }
    for reading in &readings {
        host.insert_reading(
            contexts_by_digest[&reading.receipt.context_digest],
            fields_by_digest[&reading.receipt.field_digest],
            reading,
        )?;
    }
    for session in &sessions {
        let session_readings = session
            .placements
            .iter()
            .map(|placement| readings_by_id[&placement.reading_id].clone())
            .collect::<Vec<_>>();
        host.insert_session(
            contexts_by_digest[&session.context_digest],
            fields_by_digest[&session.field_digest],
            &session_readings,
            session,
        )?;
    }
    for spread in &spreads {
        let session = sessions_by_id[&spread.session_id];
        let spread_readings = spread
            .placements
            .iter()
            .map(|placement| readings_by_id[&placement.reading_id].clone())
            .collect::<Vec<_>>();
        host.insert_three_card_spread(session, &spread_readings, spread)?;
    }
    for template in &spread_templates {
        host.insert_spread_template(template)?;
    }
    for spread in &authored_spreads {
        let session = sessions_by_id[&spread.session_id];
        let template = spread_templates_by_id[&spread.template_id];
        let spread_readings = session
            .placements
            .iter()
            .map(|placement| readings_by_id[&placement.reading_id].clone())
            .collect::<Vec<_>>();
        host.insert_spread(session, &spread_readings, template, spread)?;
    }
    for facts in &facts {
        let chart = charts_by_digest[&facts.chart_digest];
        host.insert_astrology_chart(chart, facts.orb_millidegrees)?;
    }
    for concurrence in &concurrences {
        host.insert_concurrence(concurrence)?;
    }
    for reflection in &reflections {
        host.insert_reflection(sessions_by_id[&reflection.session_id], reflection)?;
    }
    Ok(CleromancySyncImport {
        contexts: contexts.len(),
        fields: fields.len(),
        readings: readings.len(),
        sessions: sessions.len(),
        spreads: spreads.len() + authored_spreads.len(),
        spread_templates: spread_templates.len(),
        charts: charts.len(),
        facts: facts.len(),
        concurrences: concurrences.len(),
        reflections: reflections.len(),
    })
}
