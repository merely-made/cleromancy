// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Portable-card rendering for every projected Cleromancy node kind.

use chirograph::{CardValueV1, PortableCardV1};

use super::*;
use crate::{AstrologyChart, AstrologyFacts, Concurrence, Reflection, ThreeCardSpread};

impl<B: Backend> CleromancyHost<B> {
    pub(super) fn card_for(&self, key: NodeKey) -> PortableCardV1 {
        let node = self.graph.get_node(key).expect("card key remains in graph");
        let mut tags = node.tags.iter().cloned().collect::<Vec<_>>();
        tags.sort();
        let mut values = vec![CardValueV1 {
            label: "Address".to_string(),
            value: node.url().to_string(),
        }];
        if let Some(value) = self.facet_value(key, READING_FACET) {
            if let Ok(reading) = serde_json::from_value::<Reading>(value.clone()) {
                let enrichment_values = reading.receipt.enrichment.as_ref().map(|qualification| {
                    let evidence = &qualification.evidence;
                    vec![
                        CardValueV1 {
                            label: "External source".to_string(),
                            value: format!(
                                "{} / {}",
                                evidence.endpoint_label, evidence.projection_label
                            ),
                        },
                        CardValueV1 {
                            label: "Evidence digest".to_string(),
                            value: evidence.evidence_digest.clone(),
                        },
                        CardValueV1 {
                            label: "Evidence cards".to_string(),
                            value: evidence
                                .sources
                                .iter()
                                .map(|source| source.presentation.as_str())
                                .collect::<Vec<_>>()
                                .join("; "),
                        },
                        CardValueV1 {
                            label: "External matches".to_string(),
                            value: format!("{:?}", qualification.candidate_terms),
                        },
                        CardValueV1 {
                            label: "External additions".to_string(),
                            value: format!("{:?}", qualification.weight_additions),
                        },
                    ]
                });
                values.extend([
                    CardValueV1 {
                        label: "Mode".to_string(),
                        value: format!("{:?}", reading.receipt.mode).to_lowercase(),
                    },
                    CardValueV1 {
                        label: "System".to_string(),
                        value: reading.system,
                    },
                    CardValueV1 {
                        label: "Interpretation".to_string(),
                        value: reading.interpretation,
                    },
                    CardValueV1 {
                        label: "Weights".to_string(),
                        value: format!("{:?}", reading.receipt.qualified_weights),
                    },
                    CardValueV1 {
                        label: "Sample".to_string(),
                        value: reading
                            .receipt
                            .sample
                            .map_or_else(|| "not used".to_string(), |sample| sample.to_string()),
                    },
                ]);
                if let Some(enrichment_values) = enrichment_values {
                    values.extend(enrichment_values);
                }
            }
        } else if let Some(value) = self.facet_value(key, SESSION_FACET)
            && let Ok(session) = serde_json::from_value::<ReadingSession>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Recorded at".to_string(),
                    value: format!("{} ms since Unix epoch", session.created_at_ms),
                },
                CardValueV1 {
                    label: "Placements".to_string(),
                    value: session
                        .placements
                        .iter()
                        .map(|placement| {
                            format!("{}: {}", placement.position, placement.reading_id)
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                CardValueV1 {
                    label: "Client token".to_string(),
                    value: session
                        .client_token
                        .unwrap_or_else(|| "not supplied".to_string()),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, THREE_CARD_SPREAD_FACET)
            && let Ok(spread) = serde_json::from_value::<ThreeCardSpread>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Session".to_string(),
                    value: spread.session_id,
                },
                CardValueV1 {
                    label: "Placements".to_string(),
                    value: spread
                        .placements
                        .iter()
                        .map(|placement| {
                            format!("{}: {}", placement.position.as_str(), placement.reading_id)
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                CardValueV1 {
                    label: "Relationships".to_string(),
                    value: spread
                        .relations
                        .iter()
                        .map(|relation| {
                            format!(
                                "{} -> {} ({})",
                                relation.from.as_str(),
                                relation.to.as_str(),
                                relation.label
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, REFLECTION_FACET)
            && let Ok(reflection) = serde_json::from_value::<Reflection>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Recorded at".to_string(),
                    value: format!("{} ms since Unix epoch", reflection.created_at_ms),
                },
                CardValueV1 {
                    label: "Session".to_string(),
                    value: reflection.session_id,
                },
                CardValueV1 {
                    label: "Reflection".to_string(),
                    value: reflection.body,
                },
            ]);
        } else if let Some(value) = self.facet_value(key, CONCURRENCE_FACET)
            && let Ok(concurrence) = serde_json::from_value::<Concurrence>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Label".to_string(),
                    value: concurrence.label,
                },
                CardValueV1 {
                    label: "Recorded at".to_string(),
                    value: format!("{} ms since Unix epoch", concurrence.created_at_ms),
                },
                CardValueV1 {
                    label: "Members".to_string(),
                    value: concurrence
                        .members
                        .iter()
                        .map(|member| format!("{}: {}", member.role, member.address))
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                CardValueV1 {
                    label: "Claim".to_string(),
                    value: "Consulted together; no causal or interpretive relation asserted"
                        .to_string(),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, ASTROLOGY_CHART_FACET)
            && let Ok(chart) = serde_json::from_value::<AstrologyChart>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Digest".to_string(),
                    value: chart.digest(),
                },
                CardValueV1 {
                    label: "Algorithm".to_string(),
                    value: chart.algorithm,
                },
                CardValueV1 {
                    label: "Engine".to_string(),
                    value: chart.engine,
                },
                CardValueV1 {
                    label: "Ephemeris".to_string(),
                    value: chart.ephemeris,
                },
                CardValueV1 {
                    label: "Moment".to_string(),
                    value: chart.moment.instant_utc,
                },
                CardValueV1 {
                    label: "Positions".to_string(),
                    value: chart
                        .positions
                        .iter()
                        .map(|position| {
                            format!(
                                "{}: {} mdeg longitude, {} mdeg latitude{}",
                                position.body,
                                position.longitude_millidegrees,
                                position.latitude_millidegrees,
                                position
                                    .retrograde
                                    .map_or_else(String::new, |value| format!(
                                        ", retrograde={value}"
                                    )),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, ASTROLOGY_FACTS_FACET)
            && let Ok(facts) = serde_json::from_value::<AstrologyFacts>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Digest".to_string(),
                    value: facts.digest(),
                },
                CardValueV1 {
                    label: "Chart".to_string(),
                    value: facts.chart_digest,
                },
                CardValueV1 {
                    label: "Algorithm".to_string(),
                    value: facts.algorithm,
                },
                CardValueV1 {
                    label: "Orb".to_string(),
                    value: format!("{} millidegrees", facts.orb_millidegrees),
                },
                CardValueV1 {
                    label: "Placements".to_string(),
                    value: facts
                        .placements
                        .iter()
                        .map(|placement| {
                            format!(
                                "{}: {:?} +{} mdeg",
                                placement.body, placement.sign, placement.degree_millidegrees
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                CardValueV1 {
                    label: "Aspects".to_string(),
                    value: facts
                        .aspects
                        .iter()
                        .map(|aspect| {
                            format!(
                                "{} / {}: {:?} ({} mdeg, orb {})",
                                aspect.first,
                                aspect.second,
                                aspect.kind,
                                aspect.separation_millidegrees,
                                aspect.orb_millidegrees
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; "),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, FIELD_FACET)
            && let Ok(field) = serde_json::from_value::<Field>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Digest".to_string(),
                    value: field.digest(),
                },
                CardValueV1 {
                    label: "System".to_string(),
                    value: field.system,
                },
                CardValueV1 {
                    label: "Rules".to_string(),
                    value: field.rules,
                },
                CardValueV1 {
                    label: "Candidates".to_string(),
                    value: field
                        .candidates
                        .iter()
                        .map(|candidate| format!("{} ({})", candidate.title, candidate.base_weight))
                        .collect::<Vec<_>>()
                        .join("; "),
                },
            ]);
        } else if let Some(value) = self.facet_value(key, CONTEXT_FACET)
            && let Ok(context) = serde_json::from_value::<ContextSnapshot>(value.clone())
        {
            values.extend([
                CardValueV1 {
                    label: "Schema".to_string(),
                    value: context.schema,
                },
                CardValueV1 {
                    label: "Facts".to_string(),
                    value: context
                        .facts
                        .iter()
                        .map(|(name, value)| format!("{name}: {value}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                },
            ]);
        }
        PortableCardV1 {
            title: node.title.clone(),
            values,
            badges: tags,
            media: Vec::new(),
        }
    }
}
