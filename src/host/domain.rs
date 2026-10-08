// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Read-only domain validation for a resident's complete candidate graph.
//!
//! Validation uses the same record decoders and replay rules as Cleromancy's
//! reader. It does not open a store, calculate new sky facts or draw again.
//! The resident must call it before installing a candidate or writing its log.

use std::collections::BTreeSet;

use super::*;
use crate::{
    AstrologyChart, Concurrence, Reflection, SkyInterpretation, Spread, SpreadTemplate,
    ThreeCardSpread,
};

const DOMAIN_FACETS: [&str; 13] = [
    CONTEXT_FACET,
    FIELD_FACET,
    READING_FACET,
    SESSION_FACET,
    REFLECTION_FACET,
    THREE_CARD_SPREAD_FACET,
    SPREAD_TEMPLATE_FACET,
    SPREAD_FACET,
    ASTROLOGY_CHART_FACET,
    ASTROLOGY_FACTS_FACET,
    CONCURRENCE_FACET,
    SKY_DAY_FACTS_FACET,
    SKY_INTERPRETATION_FACET,
];

/// Validate every Cleromancy record and its dependencies in `graph`.
///
/// Empty graphs and unrelated facet namespaces are allowed. Unknown versions
/// in Cleromancy's namespace are refused instead of being accepted without
/// validation. This is the product-owned validator prepared for C1; the
/// resident's transaction boundary remains responsible for invoking it.
pub fn validate_divination_graph(graph: &Graph) -> Result<(), HostError> {
    let mut reader = CleromancyHost::empty(muniment::MemoryBackend::new());
    reader.graph = graph.clone();
    reader.validate_domain()
}

impl<B: Backend> CleromancyHost<B> {
    fn validate_domain(&self) -> Result<(), HostError> {
        let node_ids = self
            .graph
            .nodes()
            .map(|(_, node)| node.id)
            .collect::<BTreeSet<_>>();
        for (id, facets) in self.graph.facets().iter() {
            for (facet, _) in facets.iter() {
                if facet.as_str().starts_with("cleromancy.") {
                    if !DOMAIN_FACETS.contains(&facet.as_str()) {
                        return Err(invalid(format!(
                            "unsupported domain facet {}",
                            facet.as_str()
                        )));
                    }
                    if !node_ids.contains(id) {
                        return Err(invalid(format!(
                            "domain facet {} has no node",
                            facet.as_str()
                        )));
                    }
                }
            }
        }
        for (key, node) in self.graph.nodes() {
            let count = DOMAIN_FACETS
                .iter()
                .filter(|facet| self.facet_value(key, facet).is_some())
                .count();
            if count > 1 || (node.url().starts_with("cleromancy://") && count != 1) {
                return Err(invalid(format!(
                    "{} must carry exactly one domain record",
                    node.url()
                )));
            }
            if count == 1 && node.id != Graph::node_namespace_id(node.url()) {
                return Err(invalid(format!(
                    "{} has a noncanonical node id",
                    node.url()
                )));
            }
        }

        // Canonical enumeration decodes every value and verifies its address,
        // including records not referenced by a current reading session.
        self.contexts()?;
        self.fields()?;
        self.spread_templates()?;
        self.sessions()?;
        self.astrology_facts()?;
        self.sky_day_facts()?;

        let readings = self.canonical_facet_values(
            READING_FACET,
            |reading: &Reading| reading.id.clone(),
            |id| format!("cleromancy://reading/{id}"),
        )?;
        for reading in readings {
            if self.replay_reading(&reading)? != reading {
                return Err(ReadingError::ReceiptMismatch("sealed reading".to_string()).into());
            }
        }
        let charts =
            self.canonical_facet_values(ASTROLOGY_CHART_FACET, AstrologyChart::digest, |id| {
                format!("cleromancy://astrology/chart/{id}")
            })?;
        for chart in charts {
            chart.validate()?;
        }
        let templates = self.canonical_facet_values(
            SPREAD_TEMPLATE_FACET,
            |template: &SpreadTemplate| template.id.clone(),
            |id| format!("cleromancy://spread-template/{id}"),
        )?;
        for template in templates {
            template.validate()?;
        }
        let spreads = self.canonical_facet_values(
            THREE_CARD_SPREAD_FACET,
            |spread: &ThreeCardSpread| spread.id.clone(),
            |id| format!("cleromancy://spread/three-card/{id}"),
        )?;
        for spread in spreads {
            spread.validate()?;
            self.replay_three_card_spread(&spread)?;
            let session = self.reading_session_for_id(&spread.session_id)?;
            for placement in &spread.placements {
                if !session.placements.iter().any(|saved| {
                    saved.position == placement.position.as_str()
                        && saved.reading_id == placement.reading_id
                }) {
                    return Err(invalid(
                        "three-card placement differs from its reading session",
                    ));
                }
            }
        }
        let spreads = self.canonical_facet_values(
            SPREAD_FACET,
            |spread: &Spread| spread.id.clone(),
            |id| format!("cleromancy://spread/{id}"),
        )?;
        for spread in spreads {
            spread.validate()?;
            self.replay_spread(&spread)?;
        }
        let reflections = self.canonical_facet_values(
            REFLECTION_FACET,
            |reflection: &Reflection| reflection.id.clone(),
            |id| format!("cleromancy://reflection/{id}"),
        )?;
        for reflection in reflections {
            reflection.validate()?;
            self.reading_session_for_id(&reflection.session_id)?;
        }
        let concurrences = self.canonical_facet_values(
            CONCURRENCE_FACET,
            |concurrence: &Concurrence| concurrence.id.clone(),
            |id| format!("cleromancy://concurrence/{id}"),
        )?;
        for concurrence in concurrences {
            self.replay_concurrence(&concurrence)?;
        }
        let interpretations = self.canonical_facet_values(
            SKY_INTERPRETATION_FACET,
            SkyInterpretation::digest,
            |id| format!("cleromancy://sky/interpretation/{id}"),
        )?;
        for interpretation in interpretations {
            self.replay_sky_interpretation(&interpretation)?;
        }
        Ok(())
    }
}

fn invalid(reason: impl Into<String>) -> HostError {
    HostError::InvalidSnapshot(format!("divination domain: {}", reason.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartulary::{AcceptAll, FacetId};

    fn fixture() -> Graph {
        let (context, field) = crate::a0_fixture();
        let reading = ReadingEngine::calculate(&context, &field).unwrap();
        let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
        host.insert_reading(&context, &field, &reading).unwrap();
        host.graph.clone()
    }

    fn bytes(graph: &Graph) -> Vec<u8> {
        let mut snapshot = graph.to_snapshot();
        snapshot.timestamp_secs = 0;
        serde_json::to_vec(&(snapshot, graph.facets())).unwrap()
    }

    #[test]
    fn validates_complete_reading_without_mutating_candidate() {
        let graph = fixture();
        let before = bytes(&graph);
        validate_divination_graph(&graph).unwrap();
        assert_eq!(before, bytes(&graph));
        validate_divination_graph(&Graph::new()).unwrap();
    }

    #[test]
    fn rejects_tampered_unreferenced_sealed_reading() {
        let mut graph = fixture();
        let id = graph
            .nodes()
            .find(|(_, node)| node.url().starts_with("cleromancy://reading/"))
            .unwrap()
            .1
            .id;
        let facet = FacetId::new(READING_FACET);
        let mut value = graph.facets().get(&id, &facet).unwrap().clone();
        value["candidate_id"] = serde_json::json!("invented");
        graph
            .facets_mut()
            .set(id, facet, value, &AcceptAll)
            .unwrap();
        assert!(validate_divination_graph(&graph).is_err());
    }

    #[test]
    fn rejects_deleted_dependency_and_missing_domain_facet() {
        let mut graph = fixture();
        let id = graph
            .nodes()
            .find(|(_, node)| node.url().starts_with("cleromancy://context/"))
            .unwrap()
            .1
            .id;
        graph.facets_mut().remove_node(&id);
        assert!(validate_divination_graph(&graph).is_err());
    }

    #[test]
    fn rejects_a_reading_after_its_field_node_is_removed() {
        use mere::kernel::graph::apply::{GraphDelta, apply_graph_delta};
        let mut graph = fixture();
        let key = graph
            .nodes()
            .find(|(_, node)| node.url().starts_with("cleromancy://field/"))
            .unwrap()
            .0;
        apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key });
        assert!(validate_divination_graph(&graph).is_err());
    }

    #[test]
    fn rejects_orphan_domain_facets() {
        let mut graph = fixture();
        let id = Graph::node_namespace_id("cleromancy://reflection/orphan");
        graph
            .facets_mut()
            .set(
                id,
                FacetId::new(REFLECTION_FACET),
                serde_json::json!({}),
                &AcceptAll,
            )
            .unwrap();
        assert!(validate_divination_graph(&graph).is_err());
    }

    #[test]
    fn validates_a_session_and_reflection_and_refuses_a_missing_session() {
        use mere::kernel::graph::apply::{GraphDelta, apply_graph_delta};
        let (context, field) = crate::a0_fixture();
        let reading = ReadingEngine::calculate(&context, &field).unwrap();
        let session = ReadingSession::single(
            1,
            "00112233445566778899aabbccddeeff",
            context.digest(),
            field.digest(),
            reading.id.clone(),
            None,
        )
        .unwrap();
        let reflection = Reflection::new(
            session.id.clone(),
            2,
            "112233445566778899aabbccddeeff00",
            "A retained reflection",
        )
        .unwrap();
        let mut host = CleromancyHost::empty(muniment::MemoryBackend::new());
        host.insert_session(&context, &field, &[reading], &session)
            .unwrap();
        host.insert_reflection(&session, &reflection).unwrap();
        validate_divination_graph(&host.graph).unwrap();
        let key = host
            .graph
            .get_node_by_url(&format!("cleromancy://session/{}", session.id))
            .unwrap()
            .0;
        apply_graph_delta(&mut host.graph, GraphDelta::RemoveNode { key });
        assert!(validate_divination_graph(&host.graph).is_err());
    }

    #[test]
    fn rejects_unknown_domain_version_and_multiple_domain_records() {
        let mut graph = fixture();
        let id = graph.nodes().next().unwrap().1.id;
        graph
            .facets_mut()
            .set(
                id,
                FacetId::new("cleromancy.future/v9"),
                serde_json::json!({}),
                &AcceptAll,
            )
            .unwrap();
        assert!(validate_divination_graph(&graph).is_err());
        let mut graph = fixture();
        graph
            .facets_mut()
            .set(
                id,
                FacetId::new(REFLECTION_FACET),
                serde_json::json!({}),
                &AcceptAll,
            )
            .unwrap();
        assert!(validate_divination_graph(&graph).is_err());
    }

    #[test]
    fn preserves_unrelated_unknown_facets() {
        let mut graph = fixture();
        let id = graph.nodes().next().unwrap().1.id;
        graph
            .facets_mut()
            .set(
                id,
                FacetId::new("future.presentation/v9"),
                serde_json::json!({"opaque": true}),
                &AcceptAll,
            )
            .unwrap();
        validate_divination_graph(&graph).unwrap();
    }
}
