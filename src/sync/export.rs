// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Projection of collected local truth into the H7 event vocabulary.

use std::collections::{BTreeMap, BTreeSet};

use graphshell::personal_sync::PersonalGraphEvent;
use mere::kernel::graph::EdgeAssertion;
use muniment::Backend;

use super::collect::{SelectedTruth, collect_selected};
use super::shared::{append_node_events, batch_digest, generic_semantic_kind, invalid};
use super::{
    CleromancySyncBatch, CleromancySyncError, CleromancySyncSelection, SYNC_BATCH_SCHEMA,
};
use crate::{CleromancyHost, HostError, ThreeCardRelationKind};

/// Project the current local graph into the generic H7 event vocabulary.
/// Nothing is authored here: the caller still owns identity, roster, durable
/// store, transport, and the moment at which this batch is published.
pub fn export_sync_batch<B: Backend>(
    host: &CleromancyHost<B>,
    selection: CleromancySyncSelection,
) -> Result<CleromancySyncBatch, CleromancySyncError> {
    let SelectedTruth {
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
    } = collect_selected(host, selection)?;

    let context_ids = contexts
        .iter()
        .map(|(digest, node)| (digest.clone(), node.id))
        .collect::<BTreeMap<_, _>>();
    let field_ids = fields
        .iter()
        .map(|(digest, node)| (digest.clone(), node.id))
        .collect::<BTreeMap<_, _>>();
    let reading_ids = readings
        .iter()
        .map(|(reading, node)| (reading.id.clone(), node.id))
        .collect::<BTreeMap<_, _>>();
    let session_ids = sessions
        .iter()
        .map(|(session, node)| (session.id.clone(), node.id))
        .collect::<BTreeMap<_, _>>();
    let spread_template_ids = spread_templates
        .iter()
        .map(|(template, node)| (template.id.clone(), node.id))
        .collect::<BTreeMap<_, _>>();
    let spread_template_values = spread_templates
        .iter()
        .map(|(template, _)| (template.id.clone(), template))
        .collect::<BTreeMap<_, _>>();
    let chart_ids = charts
        .iter()
        .map(|(chart, node)| (chart.digest(), node.id))
        .collect::<BTreeMap<_, _>>();
    let chart_values = charts
        .iter()
        .map(|(chart, _)| (chart.digest(), chart))
        .collect::<BTreeMap<_, _>>();
    let address_ids = contexts
        .iter()
        .map(|(_, node)| (&node.address, node.id))
        .chain(fields.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(readings.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(sessions.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(spreads.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(
            spread_templates
                .iter()
                .map(|(_, node)| (&node.address, node.id)),
        )
        .chain(
            authored_spreads
                .iter()
                .map(|(_, node)| (&node.address, node.id)),
        )
        .chain(charts.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(facts.iter().map(|(_, node)| (&node.address, node.id)))
        .chain(
            concurrences
                .iter()
                .map(|(_, node)| (&node.address, node.id)),
        )
        .chain(reflections.iter().map(|(_, node)| (&node.address, node.id)))
        .map(|(address, id)| (address.clone(), id))
        .collect::<BTreeMap<_, _>>();
    let fact_chart_digests = facts
        .iter()
        .map(|(facts, _)| facts.chart_digest.clone())
        .collect::<BTreeSet<_>>();
    for (facts, _) in &facts {
        let Some(chart) = chart_values.get(&facts.chart_digest) else {
            return Err(CleromancySyncError::MissingAstrologyChart {
                facts: facts.digest(),
                chart_digest: facts.chart_digest.clone(),
            });
        };
        facts.verify(chart).map_err(HostError::from)?;
    }
    for chart in chart_values.keys() {
        if !fact_chart_digests.contains(chart) {
            return Err(CleromancySyncError::MissingAstrologyFacts {
                chart_digest: chart.clone(),
            });
        }
    }
    for (concurrence, _) in &concurrences {
        for member in &concurrence.members {
            if !address_ids.contains_key(&member.address) {
                return Err(CleromancySyncError::MissingConcurrenceMember {
                    concurrence: concurrence.id.clone(),
                    address: member.address.clone(),
                });
            }
        }
    }
    let mut events = Vec::new();
    for (_, node) in &contexts {
        append_node_events(&mut events, node);
    }
    for (_, node) in &fields {
        append_node_events(&mut events, node);
    }
    for (_, node) in &readings {
        append_node_events(&mut events, node);
    }
    for (_, node) in &sessions {
        append_node_events(&mut events, node);
    }
    for (_, node) in &spreads {
        append_node_events(&mut events, node);
    }
    for (_, node) in &spread_templates {
        append_node_events(&mut events, node);
    }
    for (_, node) in &authored_spreads {
        append_node_events(&mut events, node);
    }
    for (_, node) in &charts {
        append_node_events(&mut events, node);
    }
    for (_, node) in &facts {
        append_node_events(&mut events, node);
    }
    for (_, node) in &concurrences {
        append_node_events(&mut events, node);
    }
    for (_, node) in &reflections {
        append_node_events(&mut events, node);
    }
    for (reading, node) in &readings {
        let Some(&context) = context_ids.get(&reading.receipt.context_digest) else {
            return Err(CleromancySyncError::MissingContext {
                reading: reading.id.clone(),
                context_digest: reading.receipt.context_digest.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: context,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        let Some(&field) = field_ids.get(&reading.receipt.field_digest) else {
            return Err(CleromancySyncError::MissingField {
                reading: reading.id.clone(),
                field_digest: reading.receipt.field_digest.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: field,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
    }
    for (session, node) in &sessions {
        let Some(&context) = context_ids.get(&session.context_digest) else {
            return Err(CleromancySyncError::MissingSessionContext {
                session: session.id.clone(),
                context_digest: session.context_digest.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: context,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        let Some(&field) = field_ids.get(&session.field_digest) else {
            return Err(CleromancySyncError::MissingSessionField {
                session: session.id.clone(),
                field_digest: session.field_digest.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: field,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        for placement in &session.placements {
            let Some(&reading) = reading_ids.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSessionReading {
                    session: session.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: node.id,
                to: reading,
                assertion: EdgeAssertion::Containment {
                    sub_kind: mere::kernel::graph::ContainmentSubKind::CollectionMember,
                },
            });
        }
    }
    for (spread, node) in &spreads {
        let Some(&session) = session_ids.get(&spread.session_id) else {
            return Err(CleromancySyncError::MissingSpreadSession {
                spread: spread.id.clone(),
                session: spread.session_id.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: session,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        let placement_ids = spread
            .placements
            .iter()
            .map(|placement| (placement.position, placement.reading_id.clone()))
            .collect::<BTreeMap<_, _>>();
        for placement in &spread.placements {
            let Some(&reading) = reading_ids.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSpreadReading {
                    spread: spread.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: node.id,
                to: reading,
                assertion: EdgeAssertion::Containment {
                    sub_kind: mere::kernel::graph::ContainmentSubKind::CollectionMember,
                },
            });
        }
        for relation in &spread.relations {
            let Some(from_id) = placement_ids.get(&relation.from) else {
                return Err(invalid(node.id, "spread relation source position"));
            };
            let Some(to_id) = placement_ids.get(&relation.to) else {
                return Err(invalid(node.id, "spread relation target position"));
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: reading_ids[from_id],
                to: reading_ids[to_id],
                assertion: EdgeAssertion::Semantic {
                    sub_kind: match relation.kind {
                        ThreeCardRelationKind::Questions => {
                            mere::kernel::graph::SemanticSubKind::Questions
                        }
                        ThreeCardRelationKind::NextStep => {
                            mere::kernel::graph::SemanticSubKind::NextStep
                        }
                    },
                    label: Some(relation.label.clone()),
                    decay_progress: None,
                },
            });
        }
    }
    for (spread, node) in &authored_spreads {
        let Some(&session) = session_ids.get(&spread.session_id) else {
            return Err(CleromancySyncError::MissingSpreadSession {
                spread: spread.id.clone(),
                session: spread.session_id.clone(),
            });
        };
        let Some(&template) = spread_template_ids.get(&spread.template_id) else {
            return Err(CleromancySyncError::MissingSpreadTemplate {
                spread: spread.id.clone(),
                template: spread.template_id.clone(),
            });
        };
        let template_value = spread_template_values[&spread.template_id];
        let session_value = sessions
            .iter()
            .find(|(value, _)| value.id == spread.session_id)
            .map(|(value, _)| value)
            .expect("session id map and values share entries");
        if !template_value
            .positions
            .iter()
            .map(|position| position.name.as_str())
            .eq(session_value
                .placements
                .iter()
                .map(|placement| placement.position.as_str()))
        {
            return Err(invalid(
                node.id,
                "spread template does not match session positions",
            ));
        }
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: session,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: template,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
        let placement_ids = session_value
            .placements
            .iter()
            .map(|placement| (placement.position.as_str(), placement.reading_id.as_str()))
            .collect::<BTreeMap<_, _>>();
        for placement in &session_value.placements {
            let Some(&reading) = reading_ids.get(&placement.reading_id) else {
                return Err(CleromancySyncError::MissingSpreadReading {
                    spread: spread.id.clone(),
                    reading: placement.reading_id.clone(),
                });
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: node.id,
                to: reading,
                assertion: EdgeAssertion::Containment {
                    sub_kind: mere::kernel::graph::ContainmentSubKind::CollectionMember,
                },
            });
        }
        for relation in &template_value.relations {
            let Some(from_id) = placement_ids.get(relation.from.as_str()) else {
                return Err(invalid(node.id, "spread relation source position"));
            };
            let Some(to_id) = placement_ids.get(relation.to.as_str()) else {
                return Err(invalid(node.id, "spread relation target position"));
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: reading_ids[*from_id],
                to: reading_ids[*to_id],
                assertion: EdgeAssertion::Semantic {
                    sub_kind: generic_semantic_kind(relation.kind),
                    label: Some(relation.label.clone()),
                    decay_progress: None,
                },
            });
        }
    }
    for (facts, node) in &facts {
        let Some(&chart) = chart_ids.get(&facts.chart_digest) else {
            return Err(CleromancySyncError::MissingAstrologyChart {
                facts: facts.digest(),
                chart_digest: facts.chart_digest.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: chart,
            assertion: EdgeAssertion::Provenance {
                sub_kind: mere::kernel::graph::ProvenanceSubKind::GeneratedFrom,
            },
        });
    }
    for (concurrence, node) in &concurrences {
        for member in &concurrence.members {
            let Some(&member_id) = address_ids.get(&member.address) else {
                return Err(CleromancySyncError::MissingConcurrenceMember {
                    concurrence: concurrence.id.clone(),
                    address: member.address.clone(),
                });
            };
            events.push(PersonalGraphEvent::AssertRelation {
                from: node.id,
                to: member_id,
                assertion: EdgeAssertion::Containment {
                    sub_kind: mere::kernel::graph::ContainmentSubKind::CollectionMember,
                },
            });
        }
    }
    for (reflection, node) in &reflections {
        let Some(&session) = session_ids.get(&reflection.session_id) else {
            return Err(CleromancySyncError::MissingReflectionSession {
                reflection: reflection.id.clone(),
                session: reflection.session_id.clone(),
            });
        };
        events.push(PersonalGraphEvent::AssertRelation {
            from: node.id,
            to: session,
            assertion: EdgeAssertion::Semantic {
                sub_kind: mere::kernel::graph::SemanticSubKind::Elaborates,
                label: Some("reflects on".to_string()),
                decay_progress: None,
            },
        });
    }

    let digest = batch_digest(selection, &events);
    Ok(CleromancySyncBatch {
        schema: SYNC_BATCH_SCHEMA,
        selection,
        events,
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
        digest,
    })
}
