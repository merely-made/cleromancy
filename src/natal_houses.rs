// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned optional extension: absent on all original natal v1 receipts.
use crate::natal::{NatalOutcome, NatalReading};
use serde::{Deserialize, Serialize};
use turquet_houses::{
    foundation::{
        EastLongitude, JulianDate, Latitude, Longitude, ScaleAwareEpoch, TerrestrialTime,
        TimeOffset, UniversalTime1,
    },
    houses,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseSystem {
    WholeSign,
    Equal,
}
impl HouseSystem {
    pub fn label(self) -> &'static str {
        match self {
            Self::WholeSign => "Whole Sign",
            Self::Equal => "Equal House",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousePlacement {
    pub body: String,
    pub house: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HouseOutcome {
    Unavailable {
        reason: String,
    },
    Calculated {
        ascendant_microdegrees: u32,
        midheaven_microdegrees: u32,
        cusps_microdegrees: [u32; 12],
        placements: Vec<HousePlacement>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HouseReceipt {
    pub schema: String,
    pub system: HouseSystem,
    pub method: String,
    pub supplier: String,
    pub earth_rotation: String,
    pub ut1_minus_utc_milliseconds: i32,
    pub outcome: HouseOutcome,
}
pub const METHOD: &str = "cleromancy.natal-houses/turquet-tropical-iau2006-2000a/v1";
pub const EARTH_ROTATION: &str = "UT1 approximated as UTC (UT1−UTC = 0); no polar-motion correction or downloaded Earth-orientation data";
pub fn calculate(reading: &NatalReading, system: HouseSystem) -> Result<HouseReceipt, String> {
    let outcome=match &reading.outcome {
        NatalOutcome::Unavailable{..}=>HouseOutcome::Unavailable{reason:"An exact resolved birth time is required for houses and ascendant; none were invented.".into()},
        NatalOutcome::Calculated{resolution,chart,..}=>{
            match reading.birth.facts.place.as_ref().and_then(|p|p.latitude_microdegrees.zip(p.longitude_microdegrees)) {
                None=>HouseOutcome::Unavailable{reason:"No recorded birthplace coordinates. Houses and ascendant require explicit latitude and longitude; no place was inferred.".into()},
                Some((lat,lon))=>{
                    let utc=chrono::DateTime::parse_from_rfc3339(&resolution.utc).map_err(|e|format!("{e:?}"))?;
                    let epoch=ScaleAwareEpoch::from_unix_milliseconds(utc.timestamp_millis() as f64);
                    let geometry=houses::calculate(JulianDate::<TerrestrialTime>::from_epoch(epoch),JulianDate::<UniversalTime1>::from_utc_epoch(epoch,TimeOffset::from_seconds(0.0).unwrap()),Latitude::from_degrees(lat as f64/1e6).map_err(|e|format!("{e:?}"))?,EastLongitude::from_degrees(lon as f64/1e6).map_err(|e|format!("{e:?}"))?,match system{HouseSystem::WholeSign=>houses::HouseSystem::WholeSign,HouseSystem::Equal=>houses::HouseSystem::Equal});
                    match geometry {
                        Err(e)=>HouseOutcome::Unavailable{reason:format!("Houses and ascendant unavailable: {e}. No alternate system was substituted.")},
                        Ok(g)=>{
                            let angle=|v:Longitude|((v.degrees()*1e6).round() as u32)%360_000_000;
                            let cusps=g.cusps().map(angle);
                            // Turquet assigns the retained planetary longitude to its full-precision
                            // computed intervals. Planetary latitude is explicitly excluded.
                            let placements=chart.positions.iter().map(|p|HousePlacement{body:p.body.clone(),house:g.house_of(Longitude::from_degrees(p.longitude_millidegrees as f64/1000.0).unwrap())}).collect();
                            HouseOutcome::Calculated{ascendant_microdegrees:angle(g.ascendant()),midheaven_microdegrees:angle(g.midheaven()),cusps_microdegrees:cusps,placements}
                        }
                    }
                }
            }
        }
    };
    Ok(HouseReceipt {
        schema: "cleromancy.natal-houses/v1".into(),
        system,
        method: METHOD.into(),
        supplier: "Turquet 0.17.0@e2f65c021a6bf744ab5fe77121e0441a46b47306".into(),
        earth_rotation: EARTH_ROTATION.into(),
        ut1_minus_utc_milliseconds: 0,
        outcome,
    })
}
