// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Product-owned, source-backed relation engines. No store, network source,
//! solver or interpretation generator is opened here.
use crate::{ConsultationCatalog, ConsultationDetail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod engines;
mod retained;
pub use retained::*;

pub const ENGINE_VERSION: u32 = 1;
pub const MAX_NEIGHBORS: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationFamily {
    Correspondence,
    History,
    Situation,
    Spread,
}
impl RelationFamily {
    pub const ALL: [Self; 4] = [
        Self::Correspondence,
        Self::History,
        Self::Situation,
        Self::Spread,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Correspondence => "Correspondence",
            Self::History => "History and reflections",
            Self::Situation => "Recorded situation",
            Self::Spread => "Within this reading",
        }
    }
    pub fn engine(self) -> &'static str {
        match self {
            Self::Correspondence => "cleromancy.typed-correspondence",
            Self::History => "cleromancy.card-history",
            Self::Situation => "cleromancy.recorded-situation",
            Self::Spread => "cleromancy.authored-positions",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub session_id: Option<String>,
    pub position: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientRelation {
    pub id: String,
    pub item_id: String,
    pub family: RelationFamily,
    pub reason: String,
    pub method: String,
    pub engine: String,
    pub version: u32,
    pub evidence: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardNeighborhood {
    pub session_id: String,
    pub position: String,
    pub subject: AmbientItem,
    pub revision: String,
    pub items: Vec<AmbientItem>,
    pub relations: Vec<AmbientRelation>,
    pub disclosures: Vec<String>,
}
impl CardNeighborhood {
    pub fn item(&self, id: &str) -> Option<&AmbientItem> {
        if self.subject.id == id {
            Some(&self.subject)
        } else {
            self.items.iter().find(|v| v.id == id)
        }
    }
    pub fn relations_for<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a AmbientRelation> {
        self.relations.iter().filter(move |r| r.item_id == id)
    }
}

/// Registration is explicit and deterministic; each engine supplies its own
/// disclosed method and evidence. Missing data produces no fabricated neighbor.
pub struct AmbientEngines {
    registered: Vec<RelationFamily>,
}
impl Default for AmbientEngines {
    fn default() -> Self {
        Self {
            registered: RelationFamily::ALL.to_vec(),
        }
    }
}
impl AmbientEngines {
    pub fn registered(&self) -> &[RelationFamily] {
        &self.registered
    }
    pub fn generate(
        &self,
        detail: &ConsultationDetail,
        position: &str,
        catalog: &ConsultationCatalog,
        history: &[ConsultationDetail],
    ) -> Result<CardNeighborhood, String> {
        let placement = detail
            .session
            .placements
            .iter()
            .find(|p| p.position == position)
            .ok_or("unknown reading position")?;
        let reading = detail
            .readings
            .iter()
            .find(|r| r.id == placement.reading_id)
            .ok_or("missing sealed reading")?;
        let subject = AmbientItem {
            id: format!("reading:{}:{position}", detail.session.id),
            title: reading.title.clone(),
            body: reading.interpretation.clone(),
            source: format!("cleromancy://reading/{}", reading.id),
            session_id: Some(detail.session.id.clone()),
            position: Some(position.into()),
        };
        let mut out = Builder {
            items: BTreeMap::new(),
            relations: BTreeMap::new(),
        };
        for family in &self.registered {
            match family {
                RelationFamily::Correspondence => {
                    engines::correspondence(detail, reading, catalog, &mut out)
                },
                RelationFamily::History => engines::history(detail, reading, history, &mut out),
                RelationFamily::Situation => engines::situation(detail, catalog, &mut out),
                RelationFamily::Spread => engines::spread(detail, position, &mut out),
            }
        }
        let mut result = CardNeighborhood { session_id: detail.session.id.clone(), position: position.into(), subject,
            revision: String::new(), items: out.items.into_values().collect(), relations: out.relations.into_values().collect(),
            disclosures: vec!["Relations use saved divination records only; no neighboring mere is opened and no place or network source is enabled.".into(),
            "Distances describe this arrangement, not a measured distance between symbolic meanings.".into()] };
        result.revision = digest(&result);
        Ok(result)
    }
}
struct Builder {
    items: BTreeMap<String, AmbientItem>,
    relations: BTreeMap<String, AmbientRelation>,
}
impl Builder {
    fn add(
        &mut self,
        item: AmbientItem,
        family: RelationFamily,
        method: &str,
        reason: String,
        evidence: Vec<String>,
    ) {
        let id = digest(&(item.id.as_str(), family, method, &reason, &evidence));
        self.relations.entry(id.clone()).or_insert(AmbientRelation {
            id,
            item_id: item.id.clone(),
            family,
            reason,
            method: method.into(),
            engine: family.engine().into(),
            version: ENGINE_VERSION,
            evidence,
        });
        self.items.entry(item.id.clone()).or_insert(item);
    }
}
pub(crate) fn digest(value: &impl Serialize) -> String {
    blake3::hash(&serde_json::to_vec(value).expect("serializable domain data"))
        .to_hex()
        .to_string()
}

impl CardNeighborhood {
    /// Fair bounded disclosure: each nonempty family gets a turn before another
    /// family fills the available neighborhood. The full engines remain usable
    /// for validating a previously examined exact item.
    pub fn bounded(mut self) -> Self {
        let mut chosen = BTreeSet::new();
        for index in 0..MAX_NEIGHBORS {
            for family in RelationFamily::ALL {
                if chosen.len() >= MAX_NEIGHBORS {
                    break;
                }
                if let Some(r) = self
                    .relations
                    .iter()
                    .filter(|r| r.family == family)
                    .nth(index)
                {
                    chosen.insert(r.item_id.clone());
                }
            }
        }
        self.items.retain(|item| chosen.contains(&item.id));
        self.relations.retain(|r| chosen.contains(&r.item_id));
        self
    }
}
