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
    /// Every stored daily sky record, sorted by UTC civil day then digest.
    ///
    /// This is an enumerator over persisted facts only. It verifies each
    /// canonical digest address and structural payload, but never calls a sky
    /// adapter or recalculates an event for a list surface.
    pub fn sky_day_facts(&self) -> Result<Vec<SkyDayFacts>, HostError> {
        let mut facts =
            self.canonical_facet_values(SKY_DAY_FACTS_FACET, SkyDayFacts::digest, |digest| {
                format!("cleromancy://sky/facts/{digest}")
            })?;
        for value in &facts {
            value
                .validate()
                .map_err(|error| HostError::InvalidStoredFacet {
                    facet: SKY_DAY_FACTS_FACET,
                    reason: error.to_string(),
                })?;
        }
        facts.sort_by_key(|value| {
            (
                value.civil_day.year,
                value.civil_day.month,
                value.civil_day.day,
                value.digest(),
            )
        });
        Ok(facts)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SkyEarthOrientationApproximation, SkyEarthOrientationPolicy, SkyFact, SkyFactKind,
        SkyNumericalPolicy, SkyProvenance, SkySearchControls, SkyTtInterval,
        SkyTwilightMeasurement, SkyTwilightPolicy, UtcCivilDay, Wgs84Observer,
    };
    use muniment::MemoryBackend;

    #[test]
    fn enumerator_rejects_a_structurally_invalid_canonical_record() {
        let mut host = CleromancyHost::empty(MemoryBackend::new());
        let mut malformed = fixture();
        malformed.schema = "cleromancy.sky-day-facts/unknown".to_string();
        let digest = malformed.digest();
        let key = host.upsert_node(
            &format!("cleromancy://sky/facts/{digest}"),
            "Malformed sky day facts",
            ["sky", "facts"],
        );
        host.set_facet(
            key,
            SKY_DAY_FACTS_FACET,
            serde_json::to_value(malformed).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            host.sky_day_facts(),
            Err(HostError::InvalidStoredFacet {
                facet: SKY_DAY_FACTS_FACET,
                ..
            })
        ));
    }

    #[test]
    fn enumerator_rejects_a_valid_record_at_the_wrong_digest_address() {
        let mut host = CleromancyHost::empty(MemoryBackend::new());
        let facts = fixture();
        let key = host.upsert_node(
            "cleromancy://sky/facts/not-the-record-digest",
            "Misaddressed sky day facts",
            ["sky", "facts"],
        );
        host.set_facet(
            key,
            SKY_DAY_FACTS_FACET,
            serde_json::to_value(facts).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            host.sky_day_facts(),
            Err(HostError::InvalidStoredFacet {
                facet: SKY_DAY_FACTS_FACET,
                ..
            })
        ));
    }

    fn fixture() -> SkyDayFacts {
        SkyDayFacts::new(
            UtcCivilDay::new(2024, 4, 8).unwrap(),
            Wgs84Observer::new(32_776_700, -96_797_000, 130_000).unwrap(),
            SkyNumericalPolicy {
                phase_search: SkySearchControls::new(3_600, 100).unwrap(),
                twilight: SkyTwilightPolicy {
                    measurement: SkyTwilightMeasurement::AirlessSolarCenter,
                    altitude_millidegrees: -6_000,
                    search: SkySearchControls::new(900, 100).unwrap(),
                },
                earth_orientation: SkyEarthOrientationPolicy {
                    authority: "fixture".to_string(),
                    snapshot: "fixture".to_string(),
                    approximation:
                        SkyEarthOrientationApproximation::ConstantUt1MinusUtcZeroPolarMotion {
                            ut1_minus_utc_milliseconds: 0,
                        },
                },
            },
            [SkyFact {
                kind: SkyFactKind::Dawn,
                interval: SkyTtInterval::new(2_460_408.9, 2_460_408.91).unwrap(),
                provenance: SkyProvenance {
                    model: "fixture".to_string(),
                    provider: "fixture".to_string(),
                    provider_snapshot: None,
                    transform: "fixture".to_string(),
                    earth_orientation: "fixture".to_string(),
                },
            }],
        )
        .unwrap()
    }
}
