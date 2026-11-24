// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The selected vocabulary interprets the saved reading without changing its receipt.
use std::{collections::BTreeMap, path::Path};

use mere_ontology::TermKind;
use serde_json::{Value, json};

use crate::{ConsultationDetail, cards::Arcana};

const LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const LENS: &str = include_str!("../../assets/ontology/lens.toml");
const TAROT: &str = include_str!("../../assets/ontology/tarot.toml");
const ROLES: &str = "https://mere.computer/ns/cleromancy/role#";

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ReadingOntology {
    terms: BTreeMap<String, Vec<String>>,
}

impl ReadingOntology {
    pub(super) fn load(directory: Option<&Path>) -> Result<Self, String> {
        let profile = mere_ontology::load(LENS, &[TAROT], directory)?;
        let mut terms: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (_, term) in profile.terms.iter() {
            let Some(role) = term
                .family
                .as_deref()
                .and_then(|iri| iri.strip_prefix(ROLES))
            else {
                continue;
            };
            let kind = match role {
                "reading" | "deck" | "card" | "major-card" | "minor-card" => TermKind::Class,
                "drawn-card" | "in-deck" => TermKind::Predicate,
                _ => continue,
            };
            if term.kind != kind {
                return Err(format!(
                    "{} needs kind {kind:?} for reading role {role}",
                    term.iri
                ));
            }
            terms
                .entry(role.to_owned())
                .or_default()
                .push(term.iri.clone());
        }
        Ok(Self { terms })
    }

    fn iris(&self, role: &str) -> &[String] {
        self.terms.get(role).map_or(&[], Vec::as_slice)
    }

    fn node(&self, id: &str, roles: &[&str], label: Option<&str>) -> Value {
        let mut node = json!({ "@id": id });
        if let Some(name) = label.filter(|name| !name.trim().is_empty()) {
            node[LABEL] = json!(name);
        }
        let classes: Vec<_> = roles.iter().flat_map(|role| self.iris(role)).collect();
        if !classes.is_empty() {
            node["@type"] = json!(classes);
        }
        node
    }

    fn relate(&self, node: &mut Value, role: &str, object: &str) {
        for predicate in self.iris(role) {
            node[predicate.as_str()] = json!({ "@id": object });
        }
    }

    /// The deck and card are fragments of a particular saved field snapshot.
    /// Its candidate order is immutable and the receipt records the drawn index.
    pub(super) fn graph(&self, detail: &ConsultationDetail) -> Option<Value> {
        if !detail.field.is_tarot() || self.terms.is_empty() {
            return None;
        }
        let field = format!("cleromancy://field/{}", detail.session.field_digest);
        let deck = format!("{field}#deck");
        let mut nodes = BTreeMap::new();
        let deck_title = detail.field.tarot.as_ref().map(|d| d.title.as_str());
        nodes.insert(deck.clone(), self.node(&deck, &["deck"], deck_title));
        for reading in &detail.readings {
            let card = format!("{field}#card-{}", reading.receipt.selected_index);
            let id = format!("cleromancy://reading/{}", reading.id);
            let title = (!reading.title.trim().is_empty()).then(|| reading.display_title());
            let mut result = self.node(&id, &["reading"], title.as_deref());
            self.relate(&mut result, "drawn-card", &card);
            nodes.insert(id, result);
            let typed = detail
                .field
                .tarot
                .as_ref()
                .and_then(|d| d.card(&reading.candidate_id));
            let roles: &[&str] = match typed.map(|c| c.key.arcana()) {
                Some(Arcana::Major) => &["card", "major-card"],
                Some(Arcana::Minor) => &["card", "minor-card"],
                None => &["card"],
            };
            let mut drawn = self.node(&card, roles, Some(&reading.title));
            self.relate(&mut drawn, "in-deck", &deck);
            nodes.insert(card, drawn);
        }
        Some(json!({ "@graph": nodes.into_values().collect::<Vec<_>>() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn local_pack_changes_the_projection_roles() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lens.toml"), LENS).unwrap();
        let path = dir.path().join("custom.toml");
        fs::write(
            &path,
            TAROT.replace("reading:drawnCard", "reading:selectedCard"),
        )
        .unwrap();
        let selected = ReadingOntology::load(Some(dir.path())).unwrap();
        assert_eq!(
            selected.iris("drawn-card"),
            ["https://mere.computer/ns/cleromancy#selectedCard"]
        );
        assert_ne!(selected, ReadingOntology::load(None).unwrap());
        fs::write(
            path,
            TAROT.replace("family = \"role:card\"", "family = \"role:in-deck\""),
        )
        .unwrap();
        assert!(
            ReadingOntology::load(Some(dir.path()))
                .unwrap_err()
                .contains("needs kind")
        );
    }
}
