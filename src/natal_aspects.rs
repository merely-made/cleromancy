// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional, replayable longitude aspects over the original retained natal chart.
//! This edition supplies geometric facts only; no interpretation is generated.
use crate::{
    AspectKind, AstrologyAspect, AstrologyChart,
    natal::{NatalOutcome, NatalReading},
};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "cleromancy.natal-aspects/v1";
pub const METHOD: &str = "cleromancy.natal-aspects/shortest-longitude-inclusive-orbs/v1";

/// Inclusive maximum deviation from each exact angle, in integer millidegrees.
/// Zero means exact matches only. Each rule must be at most 180000.
/// Wide custom rules may match multiple kinds for a pair; all are retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AspectRules {
    pub conjunction_millidegrees: u32,
    pub opposition_millidegrees: u32,
    pub trine_millidegrees: u32,
    pub square_millidegrees: u32,
    pub sextile_millidegrees: u32,
}

/// Host-facing name for the explicitly selected and retained orb rules.
pub type OrbPolicy = AspectRules;

impl Default for AspectRules {
    /// Explicit product preset: conjunction/opposition 8°, trine/square 6°,
    /// sextile 4°. No luminary bonus, latitude, house, or motion adjustment.
    fn default() -> Self {
        Self {
            conjunction_millidegrees: 8000,
            opposition_millidegrees: 8000,
            trine_millidegrees: 6000,
            square_millidegrees: 6000,
            sextile_millidegrees: 4000,
        }
    }
}

impl AspectRules {
    pub fn orb_millidegrees(self, kind: AspectKind) -> u32 {
        match kind {
            AspectKind::Conjunction => self.conjunction_millidegrees,
            AspectKind::Opposition => self.opposition_millidegrees,
            AspectKind::Trine => self.trine_millidegrees,
            AspectKind::Square => self.square_millidegrees,
            AspectKind::Sextile => self.sextile_millidegrees,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if AspectKind::ordered()
            .into_iter()
            .any(|kind| self.orb_millidegrees(kind) > 180_000)
        {
            return Err("Natal aspect orbs must be at most 180000 millidegrees".into());
        }
        Ok(())
    }
}

// Reuse the existing typed aspect vocabulary and hit representation. The
// original astrology derivation has one shared orb and chooses one nearest
// kind; this edition independently applies each retained rule instead.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AspectOutcome {
    Unavailable {
        reason: String,
    },
    Calculated {
        chart_digest: String,
        /// Lexical body pair order, then increasing exact aspect angle.
        hits: Vec<AstrologyAspect>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AspectReceipt {
    pub schema: String,
    pub method: String,
    pub rules: AspectRules,
    pub outcome: AspectOutcome,
}

/// Derive from validated integer longitudes, independent of chart input order.
pub fn derive(chart: &AstrologyChart, rules: AspectRules) -> Result<Vec<AstrologyAspect>, String> {
    rules.validate()?;
    chart.validate().map_err(|e| e.to_string())?;
    let mut positions: Vec<_> = chart.positions.iter().collect();
    positions.sort_by(|a, b| a.body.cmp(&b.body));
    let mut hits = Vec::new();
    for (index, first) in positions.iter().enumerate() {
        for second in &positions[index + 1..] {
            let separation = first.separation_millidegrees(second);
            for kind in AspectKind::ordered() {
                let exact = kind.exact_millidegrees();
                let orb = separation.abs_diff(exact);
                if orb <= rules.orb_millidegrees(kind) {
                    hits.push(AstrologyAspect {
                        first: first.body.clone(),
                        second: second.body.clone(),
                        kind,
                        exact_millidegrees: exact,
                        separation_millidegrees: separation,
                        orb_millidegrees: orb,
                    });
                }
            }
        }
    }
    Ok(hits)
}

pub fn calculate(reading: &NatalReading, rules: AspectRules) -> Result<AspectReceipt, String> {
    rules.validate()?;
    let outcome = match &reading.outcome {
        NatalOutcome::Unavailable { .. } => AspectOutcome::Unavailable {
            reason: "An exact resolved natal chart is required for aspects; none were invented."
                .into(),
        },
        NatalOutcome::Calculated { chart, .. } => AspectOutcome::Calculated {
            chart_digest: chart.digest(),
            hits: derive(chart, rules)?,
        },
    };
    Ok(AspectReceipt {
        schema: SCHEMA.into(),
        method: METHOD.into(),
        rules,
        outcome,
    })
}
