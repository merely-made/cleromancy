// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Durable graph records for factual sky timelines and their interpretations.
//!
//! Facts and interpretations are deliberately separate records. An
//! interpretation points back to its facts by digest and carries its own rule
//! pack provenance; the host does not invent a rule-pack graph node.

use mere::kernel::graph::apply::assert_relation;
use mere::kernel::graph::{EdgeAssertion, ProvenanceSubKind};

use super::*;
use crate::sky::{SkyDayFacts, SkyInterpretation};

impl<B: Backend> CleromancyHost<B> {
    /// Store one validated, immutable daily sky fact record.
    pub fn insert_sky_day_facts(&mut self, facts: &SkyDayFacts) -> Result<NodeKey, HostError> {
        facts
            .validate()
            .map_err(|error| HostError::InvalidStoredFacet {
                facet: SKY_DAY_FACTS_FACET,
                reason: error.to_string(),
            })?;
        let digest = facts.digest();
        let key = self.upsert_node(
            &format!("cleromancy://sky/facts/{digest}"),
            "Sky day facts",
            ["sky", "facts"],
        );
        self.set_facet(
            key,
            SKY_DAY_FACTS_FACET,
            serde_json::to_value(facts).unwrap(),
        )?;
        self.changed();
        Ok(key)
    }

    /// Store an interpretation after checking its declared facts dependency.
    /// The rule-pack identity remains in the interpretation payload itself.
    pub fn insert_sky_interpretation(
        &mut self,
        interpretation: &SkyInterpretation,
    ) -> Result<NodeKey, HostError> {
        let facts = self.sky_day_facts_for_digest(&interpretation.sky_facts_digest)?;
        interpretation
            .verify_binding(&facts)
            .map_err(|error| HostError::InvalidStoredFacet {
                facet: SKY_INTERPRETATION_FACET,
                reason: error.to_string(),
            })?;
        let digest = interpretation.digest();
        let key = self.upsert_node(
            &format!("cleromancy://sky/interpretation/{digest}"),
            "Sky interpretation",
            ["sky", "interpretation"],
        );
        self.set_facet(
            key,
            SKY_INTERPRETATION_FACET,
            serde_json::to_value(interpretation).unwrap(),
        )?;
        let facts_key = self
            .graph
            .get_node_by_url(&format!(
                "cleromancy://sky/facts/{}",
                interpretation.sky_facts_digest
            ))
            .expect("dependency was resolved above")
            .0;
        assert_relation(
            &mut self.graph,
            key,
            facts_key,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::GeneratedFrom,
            },
        );
        self.changed();
        Ok(key)
    }

    pub fn sky_day_facts_for_digest(&self, digest: &str) -> Result<SkyDayFacts, HostError> {
        let facts: SkyDayFacts = self.stored_sky_facet(
            &format!("cleromancy://sky/facts/{digest}"),
            SKY_DAY_FACTS_FACET,
            "sky day facts",
            digest,
        )?;
        if facts.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: SKY_DAY_FACTS_FACET,
                reason: "sky facts digest does not match its canonical address".to_string(),
            });
        }
        facts
            .validate()
            .map_err(|error| HostError::InvalidStoredFacet {
                facet: SKY_DAY_FACTS_FACET,
                reason: error.to_string(),
            })?;
        Ok(facts)
    }

    pub fn sky_interpretation_for_digest(
        &self,
        digest: &str,
    ) -> Result<SkyInterpretation, HostError> {
        let interpretation: SkyInterpretation = self.stored_sky_facet(
            &format!("cleromancy://sky/interpretation/{digest}"),
            SKY_INTERPRETATION_FACET,
            "sky interpretation",
            digest,
        )?;
        if interpretation.digest() != digest {
            return Err(HostError::InvalidStoredFacet {
                facet: SKY_INTERPRETATION_FACET,
                reason: "sky interpretation digest does not match its canonical address"
                    .to_string(),
            });
        }
        self.replay_sky_interpretation(&interpretation)?;
        Ok(interpretation)
    }

    pub fn replay_sky_interpretation(
        &self,
        interpretation: &SkyInterpretation,
    ) -> Result<SkyInterpretation, HostError> {
        let facts = self.sky_day_facts_for_digest(&interpretation.sky_facts_digest)?;
        interpretation
            .verify_binding(&facts)
            .map_err(|error| HostError::InvalidStoredFacet {
                facet: SKY_INTERPRETATION_FACET,
                reason: error.to_string(),
            })?;
        let stored = self.sky_interpretation_for_digest_unchecked(&interpretation.digest())?;
        if stored != *interpretation {
            return Err(HostError::InvalidStoredFacet {
                facet: SKY_INTERPRETATION_FACET,
                reason: "stored sky interpretation differs from supplied value".to_string(),
            });
        }
        Ok(stored)
    }

    fn stored_sky_facet<T: serde::de::DeserializeOwned>(
        &self,
        address: &str,
        facet: &'static str,
        kind: &'static str,
        digest: &str,
    ) -> Result<T, HostError> {
        let (key, _) =
            self.graph
                .get_node_by_url(address)
                .ok_or_else(|| HostError::MissingSkyDependency {
                    kind,
                    digest: digest.to_string(),
                })?;
        let value =
            self.facet_value(key, facet)
                .ok_or_else(|| HostError::MissingSkyDependency {
                    kind,
                    digest: digest.to_string(),
                })?;
        serde_json::from_value(value.clone()).map_err(|error| HostError::InvalidStoredFacet {
            facet,
            reason: error.to_string(),
        })
    }

    fn sky_interpretation_for_digest_unchecked(
        &self,
        digest: &str,
    ) -> Result<SkyInterpretation, HostError> {
        self.stored_sky_facet(
            &format!("cleromancy://sky/interpretation/{digest}"),
            SKY_INTERPRETATION_FACET,
            "sky interpretation",
            digest,
        )
    }
}
