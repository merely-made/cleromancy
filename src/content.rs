// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Authored presentation content, versioned independently of numerical facts.
use crate::{AspectKind, AstrologyFacts, ZodiacSign};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionDefinition {
    pub name: String,
    pub label: String,
    pub meaning: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionPack {
    pub schema: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub date: String,
    pub positions: Vec<PositionDefinition>,
}

impl PositionPack {
    pub fn digest(&self) -> String {
        crate::context::canonical_digest(self)
    }
    pub fn validate(&self) -> bool {
        let mut names = std::collections::BTreeSet::new();
        self.schema == "cleromancy.position-meanings/v1"
            && [&self.id, &self.version, &self.author, &self.date]
                .iter()
                .all(|s| !s.trim().is_empty())
            && !self.positions.is_empty()
            && self.positions.iter().all(|p| {
                !p.name.trim().is_empty()
                    && !p.label.trim().is_empty()
                    && !p.meaning.trim().is_empty()
                    && names.insert(&p.name)
            })
    }
}

pub fn position_pack() -> &'static PositionPack {
    static PACK: OnceLock<PositionPack> = OnceLock::new();
    PACK.get_or_init(|| {
        serde_json::from_str(include_str!("content/position-meanings-v1.json"))
            .expect("bundled position meanings")
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyTheme {
    pub body: String,
    pub theme: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignTheme {
    pub sign: ZodiacSign,
    pub approach: String,
    pub question: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AspectTheme {
    pub kind: AspectKind,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstrologyPromptPack {
    pub schema: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub date: String,
    pub intro: String,
    pub body_themes: Vec<BodyTheme>,
    pub sign_themes: Vec<SignTheme>,
    pub aspect_themes: Vec<AspectTheme>,
    pub placement_template: String,
    pub aspect_template: String,
    pub retrograde: String,
    pub rationale: String,
}

pub fn astrology_pack() -> &'static AstrologyPromptPack {
    static PACK: OnceLock<AstrologyPromptPack> = OnceLock::new();
    PACK.get_or_init(|| {
        serde_json::from_str(include_str!("content/astrology-prompts-v1.json"))
            .expect("bundled astrology prompts")
    })
}

pub struct AstrologyPrompt {
    pub key: String,
    pub title: String,
    pub text: String,
}
impl AstrologyPromptPack {
    pub fn digest(&self) -> String {
        crate::context::canonical_digest(self)
    }
    fn body(&self, name: &str) -> Option<&str> {
        self.body_themes
            .iter()
            .find(|x| x.body.eq_ignore_ascii_case(name))
            .map(|x| x.theme.as_str())
    }
    pub fn prompts(&self, facts: &AstrologyFacts) -> Vec<AstrologyPrompt> {
        let mut result = Vec::new();
        let mut placements = facts.placements.iter().collect::<Vec<_>>();
        placements.sort_by_key(|p| {
            self.body_themes
                .iter()
                .position(|x| x.body.eq_ignore_ascii_case(&p.body))
                .unwrap_or(usize::MAX)
        });
        for p in placements {
            let (Some(topic), Some(sign)) = (
                self.body(&p.body),
                self.sign_themes.iter().find(|x| x.sign == p.sign),
            ) else {
                continue;
            };
            let mut text = self
                .placement_template
                .replace("{topic}", topic)
                .replace("{approach}", &sign.approach)
                .replace("{question}", &sign.question);
            if p.retrograde == Some(true) {
                text.push_str(&self.retrograde);
            }
            result.push(AstrologyPrompt {
                key: format!("astrology-prompt:{}", p.body),
                title: format!(
                    "{} in {:?}",
                    crate::reading_scene::position_label(&p.body),
                    p.sign
                ),
                text,
            });
        }
        for (i, a) in facts.aspects.iter().enumerate() {
            let (Some(first), Some(second), Some(lens)) = (
                self.body(&a.first),
                self.body(&a.second),
                self.aspect_themes.iter().find(|x| x.kind == a.kind),
            ) else {
                continue;
            };
            result.push(AstrologyPrompt {
                key: format!("astrology-aspect:{i}"),
                title: format!("{} · {:?} · {}", a.first, a.kind, a.second),
                text: self
                    .aspect_template
                    .replace("{first}", first)
                    .replace("{second}", second)
                    .replace("{lens}", &lens.text),
            });
        }
        result
    }
    pub fn rationale(&self, facts: &AstrologyFacts) -> String {
        self.rationale.replace(
            "{orb}",
            &format!("{:.1}", facts.orb_millidegrees as f64 / 1000.0),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn content_editions_are_typed_and_complete() {
        let positions = super::position_pack();
        assert!(positions.validate());
        assert_eq!(positions.positions.len(), 4);
        let pack = super::astrology_pack();
        assert_eq!(pack.schema, "cleromancy.astrology-prompts/v1");
        assert_eq!(pack.version, "1");
        assert_eq!(pack.body_themes.len(), 10);
        assert_eq!(pack.sign_themes.len(), 12);
        assert_eq!(pack.aspect_themes.len(), 5);
        assert_eq!(pack.digest(), super::astrology_pack().digest());
    }
}

/// Authored natal interpretation, distinct from the preserved moment-reading edition.
pub fn natal_pack() -> &'static AstrologyPromptPack {
    static PACK: OnceLock<AstrologyPromptPack> = OnceLock::new();
    PACK.get_or_init(|| {
        serde_json::from_str(include_str!("content/astrology-natal-v1.json"))
            .expect("bundled natal prompts")
    })
}
