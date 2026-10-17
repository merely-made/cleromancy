// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Sealed natal occasions retain the exact shared birth revision they used.
use crate::{AstrologyChart, AstrologyFacts, AstrologyMoment, AstrologyPosition};
use chrono::{Datelike, LocalResult, Offset, TimeZone};
use dramatis::birth::BirthTime;
use personal_facts::BirthResource;
use serde::{Deserialize, Serialize};
use turquet_timeline::{
    apparent::{APPARENT_BODIES, ApparentSky},
    foundation::{JulianDate, ScaleAwareEpoch, TerrestrialTime},
};
pub const NATAL_FACET: &str = "cleromancy.natal-reading/v1";
pub const METHOD: &str = "cleromancy.natal/turquet-0.17-30875f30/v1";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoldChoice {
    #[default]
    Refuse,
    Earlier,
    Later,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CivilResolution {
    pub timezone: String,
    pub tzdb: String,
    pub utc: String,
    pub offset_seconds: i32,
    pub fold: FoldChoice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NatalPrompt {
    pub key: String,
    pub title: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NatalOutcome {
    Unavailable {
        reason: String,
    },
    Calculated {
        resolution: CivilResolution,
        chart: AstrologyChart,
        facts: AstrologyFacts,
        edition: String,
        edition_digest: String,
        prompts: Vec<NatalPrompt>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NatalReading {
    pub schema: String,
    pub id: String,
    pub created_at_ms: u64,
    pub nonce: String,
    pub persona: String,
    pub subject_label: String,
    pub birth: BirthResource,
    pub question: String,
    pub fold: FoldChoice,
    pub orb_millidegrees: u32,
    pub outcome: NatalOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub houses: Option<crate::natal_houses::HouseReceipt>,
}
impl NatalReading {
    pub fn new(
        persona: String,
        subject_label: String,
        birth: BirthResource,
        question: String,
        fold: FoldChoice,
        created_at_ms: u64,
        nonce: String,
    ) -> Result<Self, String> {
        let outcome = calculate(&birth, fold, 1000)?;
        let mut value = Self {
            schema: NATAL_FACET.into(),
            id: String::new(),
            created_at_ms,
            nonce,
            persona,
            subject_label,
            birth,
            question,
            fold,
            orb_millidegrees: 1000,
            outcome,
            houses: None,
        };
        value.id = value.identity();
        value.validate()?;
        Ok(value)
    }
    /// Add a versioned house receipt without rewriting any existing natal v1.
    pub fn with_houses(mut self, system: crate::natal_houses::HouseSystem) -> Result<Self, String> {
        self.houses = Some(crate::natal_houses::calculate(&self, system)?);
        self.id = self.identity();
        self.validate()?;
        Ok(self)
    }
    fn identity(&self) -> String {
        let mut v = serde_json::to_value(self).expect("serializable natal");
        v.as_object_mut().unwrap().remove("id");
        crate::context::canonical_digest(&v)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != NATAL_FACET
            || self.persona.trim().is_empty()
            || self.subject_label.trim().is_empty()
            || self.question.trim().is_empty()
            || self.nonce.len() != 32
            || !self.nonce.bytes().all(|c| c.is_ascii_hexdigit())
            || self.id != self.identity()
        {
            return Err("invalid natal reading identity".into());
        }
        if let dramatis::subject::SubjectRef::Contact { scope, .. } = &self.birth.facts.subject {
            if scope.as_str() != self.persona {
                return Err("Birth contact belongs to a different source persona".into());
            }
        }
        if self.outcome != calculate(&self.birth, self.fold, self.orb_millidegrees)? {
            return Err("natal computation differs from its receipt".into());
        }
        if let Some(receipt) = &self.houses {
            if *receipt != crate::natal_houses::calculate(self, receipt.system)? {
                return Err("Natal house computation differs from its receipt".into());
            }
        }
        Ok(())
    }
}
pub fn calculate(
    birth: &BirthResource,
    fold: FoldChoice,
    orb: u32,
) -> Result<NatalOutcome, String> {
    if personal_facts::birth_resource(&birth.facts)? != *birth {
        return Err("birth snapshot does not match the shared resource revision".into());
    }
    let f = &birth.facts;
    let (hour,minute,second)=match f.time {BirthTime::Known{hour,minute,second}=>(hour,minute,second),BirthTime::Unknown=>return Ok(NatalOutcome::Unavailable{reason:"Birth time is unknown. No exact chart, houses or rising sign were invented.".into()}),BirthTime::Approximate{..}=>return Ok(NatalOutcome::Unavailable{reason:"Birth time is approximate. This first natal edition requires exact time; no exact chart was invented.".into()})};
    let Some(zone) = &f.timezone else {
        return Ok(NatalOutcome::Unavailable {
            reason: "An explicit IANA timezone is needed to resolve the civil birth time.".into(),
        });
    };
    let tz: chrono_tz::Tz = zone
        .parse()
        .map_err(|_| "Unknown IANA timezone".to_string())?;
    let local =
        chrono::NaiveDate::from_ymd_opt(f.date.year, f.date.month.into(), f.date.day.into())
            .and_then(|d| d.and_hms_opt(hour.into(), minute.into(), second.into()))
            .ok_or("Invalid civil birth time")?;
    let resolved=match tz.from_local_datetime(&local){LocalResult::Single(value)=>value,LocalResult::None=>return Err("This civil time falls in a timezone gap; correct the source time rather than shifting it automatically.".into()),LocalResult::Ambiguous(a,b)=>match fold{FoldChoice::Earlier=>a,FoldChoice::Later=>b,FoldChoice::Refuse=>return Err("This civil time occurs twice. Explicitly choose the earlier or later occurrence.".into())}};
    let utc = resolved.with_timezone(&chrono::Utc);
    if !(1972..=2098).contains(&utc.year()) {
        return Err("Birth facts are retained, but this pinned numerical edition is qualified only for UTC years 1972–2098. No natal reading was saved.".into());
    }
    let resolution = CivilResolution {
        timezone: zone.clone(),
        tzdb: chrono_tz::IANA_TZDB_VERSION.into(),
        utc: utc.to_rfc3339(),
        offset_seconds: resolved.offset().fix().local_minus_utc(),
        fold,
    };
    let tt = JulianDate::<TerrestrialTime>::from_epoch(ScaleAwareEpoch::from_unix_milliseconds(
        utc.timestamp_millis() as f64,
    ));
    let provider = ApparentSky::at(tt);
    let mut positions = Vec::new();
    for body in APPARENT_BODIES {
        let p = provider
            .position(body)
            .map_err(|e| e.to_string())?
            .into_value();
        positions.push(AstrologyPosition::new(
            body.name(),
            ((p.direction().longitude().degrees().rem_euclid(360.0) * 1000.0).round() as u32)
                % 360000,
            (p.direction().latitude().degrees() * 1000.0).round() as i32,
        ));
    }
    let chart = AstrologyChart::new(
        METHOD,
        "Turquet 0.17.0@30875f30",
        "analytical apparent geocentric tropical; no houses, ascendant or retrograde calculation",
        AstrologyMoment::global(resolution.utc.clone()),
        positions,
    )
    .map_err(|e| e.to_string())?;
    let facts = chart.facts(orb).map_err(|e| e.to_string())?;
    let pack = crate::content::natal_pack();
    let prompts = pack
        .prompts(&facts)
        .into_iter()
        .map(|p| NatalPrompt {
            key: p.key,
            title: p.title,
            text: p.text,
        })
        .collect();
    Ok(NatalOutcome::Calculated {
        resolution,
        chart,
        facts,
        edition: pack.id.clone(),
        edition_digest: pack.digest(),
        prompts,
    })
}
