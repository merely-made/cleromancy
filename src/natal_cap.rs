// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional sealed numerical edition. Original natal and house receipts retain
//! their original calculation engines and wire representation.
use crate::natal::{NatalOutcome, NatalReading};
use crate::natal_houses::{HouseOutcome, HousePlacement};
use serde::{Deserialize, Serialize};
use turquet_cap::{apparent::APPARENT_BODIES, foundation::*, houses, motion};

pub const METHOD: &str = "cleromancy.natal-supplement/v1";
pub const SUPPLIER: &str = "Turquet 0.17.0@8e3755bbd0d40343eb4750b072801c941ff32802";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseChoice {
    WholeSign,
    Equal,
    Placidus,
}
impl HouseChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::WholeSign => "Whole Sign",
            Self::Equal => "Equal House",
            Self::Placidus => "Placidus",
        }
    }
    fn engine(self) -> houses::HouseSystem {
        match self {
            Self::WholeSign => houses::HouseSystem::WholeSign,
            Self::Equal => houses::HouseSystem::Equal,
            Self::Placidus => houses::HouseSystem::Placidus,
        }
    }
}
/// Explicit source text supplied by the person. No calculation fetches data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrientationSnapshot {
    pub finals2000a: String,
}
impl OrientationSnapshot {
    pub fn digest(&self) -> String {
        blake3::hash(self.finals2000a.as_bytes())
            .to_hex()
            .to_string()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionClass {
    Direct,
    Retrograde,
    NearStation,
}
impl MotionClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Retrograde => "retrograde",
            Self::NearStation => "near station (small sampled rate)",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetMotion {
    pub body: String,
    pub microdegrees_per_tt_day: i64,
    pub classification: MotionClass,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MotionOutcome {
    Unavailable {
        reason: String,
    },
    Calculated {
        sample_half_interval_seconds: u32,
        near_station_microdegrees_per_day: u32,
        planets: Vec<PlanetMotion>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedValues {
    pub dut1_microseconds: i64,
    pub xp_microarcseconds: i64,
    pub yp_microarcseconds: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NatalSupplement {
    pub schema: String,
    pub supplier: String,
    pub motion: MotionOutcome,
    pub house_system: Option<HouseChoice>,
    pub orientation: Option<OrientationSnapshot>,
    pub orientation_digest: Option<String>,
    pub earth_rotation: String,
    pub observed_values: Option<ObservedValues>,
    pub houses: Option<HouseOutcome>,
}
pub fn calculate(
    reading: &NatalReading,
    house_system: Option<HouseChoice>,
    orientation: Option<OrientationSnapshot>,
) -> Result<NatalSupplement, String> {
    if orientation
        .as_ref()
        .is_some_and(|s| s.finals2000a.len() > 1_000_000)
    {
        return Err("Retain a bounded IERS excerpt of at most 1 MB".into());
    }
    if orientation.is_some() && house_system.is_none() {
        return Err(
            "Observed Earth orientation requires an explicitly selected house system".into(),
        );
    }
    let earth_rotation = if let Some(s) = &orientation {
        format!(
            "IERS finals2000A retained observed Earth orientation; snapshot BLAKE3 {}. UT1 and polar motion interpolated within observed coverage; no celestial-pole offsets or refraction.",
            s.digest()
        )
    } else {
        crate::natal_houses::EARTH_ROTATION.into()
    };
    if matches!(reading.outcome, NatalOutcome::Unavailable { .. }) && orientation.is_some() {
        return Err("Observed Earth orientation needs an exact resolved birth time".into());
    }
    let mut observed_values = None;
    let (planet_motion, house_outcome) = match &reading.outcome {
        NatalOutcome::Unavailable { .. } => (
            MotionOutcome::Unavailable {
                reason:
                    "An exact resolved birth time is required for apparent longitudinal motion."
                        .into(),
            },
            house_system.map(|_| HouseOutcome::Unavailable {
                reason: "An exact resolved birth time is required for houses and ascendant.".into(),
            }),
        ),
        NatalOutcome::Calculated {
            resolution, chart, ..
        } => {
            let utc =
                chrono::DateTime::parse_from_rfc3339(&resolution.utc).map_err(|e| e.to_string())?;
            let epoch = ScaleAwareEpoch::from_unix_milliseconds(utc.timestamp_millis() as f64);
            // Preserve UTC midnight and the small civil/scale offsets separately.
            // Collapsing to one large JD can move a midnight just before an
            // exact retained EOP knot, especially at a leap boundary.
            let ms = utc.timestamp_millis();
            let days = ms.div_euclid(86_400_000);
            let seconds = ms.rem_euclid(86_400_000) as f64 / 1000.0;
            let leap = epoch.leap_seconds_iers() as f64;
            let tt = JulianDate::<TerrestrialTime>::from_parts(
                2_440_587.5 + days as f64,
                (seconds + leap + 32.184) / 86_400.0,
            )
            .map_err(|e| format!("{e:?}"))?;
            let planets = APPARENT_BODIES
                .into_iter()
                .map(|body| {
                    let m =
                        motion::analytical_longitude_motion(body, tt).map_err(|e| e.to_string())?;
                    Ok(retained_motion(body.name(), m.rate().degrees_per_day()))
                })
                .collect::<Result<Vec<_>, String>>()?;
            // Resolve retained observations before checking place so invalid or
            // out-of-coverage input is never silently ignored.
            let eop = orientation
                .as_ref()
                .map(|s| observed_at(s, tt))
                .transpose()?;
            observed_values = eop.as_ref().map(|(_, v)| v.clone());
            let h=house_system.map(|system| -> Result<HouseOutcome,String> {
                let Some((lat,lon))=reading.birth.facts.place.as_ref().and_then(|p|p.latitude_microdegrees.zip(p.longitude_microdegrees)) else {return Ok(HouseOutcome::Unavailable{reason:"No recorded birthplace coordinates. Houses and ascendant require explicit latitude and longitude; no place was inferred.".into()})};
                let lat=Latitude::from_degrees(lat as f64/1e6).map_err(|e|format!("{e:?}"))?;
                let lon=EastLongitude::from_degrees(lon as f64/1e6).map_err(|e|format!("{e:?}"))?;
                let g=match &eop {Some((e,_))=>houses::calculate_observed(tt,e,lat,lon,system.engine()),None=>houses::calculate(tt,JulianDate::<UniversalTime1>::from_parts(2_440_587.5+days as f64,seconds/86_400.0).unwrap(),lat,lon,system.engine())};
                Ok(match g {Err(e)=>HouseOutcome::Unavailable{reason:format!("{} unavailable: {e}. No alternate house system was substituted.",system.label())},Ok(g)=> {
                    let angle=|v:Longitude|((v.degrees()*1e6).round() as u32)%360_000_000;
                    HouseOutcome::Calculated{ascendant_microdegrees:angle(g.ascendant()),midheaven_microdegrees:angle(g.midheaven()),cusps_microdegrees:g.cusps().map(angle),placements:chart.positions.iter().map(|p|HousePlacement{body:p.body.clone(),house:g.house_of(Longitude::from_degrees(p.longitude_millidegrees as f64/1000.0).unwrap())}).collect()}
                }})
            }).transpose()?;
            (
                MotionOutcome::Calculated {
                    sample_half_interval_seconds: 864,
                    near_station_microdegrees_per_day: 1000,
                    planets,
                },
                h,
            )
        },
    };
    Ok(NatalSupplement {
        schema: METHOD.into(),
        supplier: SUPPLIER.into(),
        motion: planet_motion,
        house_system,
        orientation_digest: orientation.as_ref().map(|s| s.digest()),
        orientation,
        earth_rotation,
        observed_values,
        houses: house_outcome,
    })
}
fn observed_at(
    s: &OrientationSnapshot,
    tt: JulianDate<TerrestrialTime>,
) -> Result<(turquet_cap::observer::EarthOrientation, ObservedValues), String> {
    // Bound to the exact retained text, rather than the latest remote file.
    let series = turquet_cap::eop::EopSeries::parse_finals2000a(
        &s.finals2000a,
        "IERS finals2000A",
        &s.digest(),
    )
    .map_err(|e| e.to_string())?;
    let v = series.evaluate(tt).map_err(|e| e.to_string())?;
    Ok((
        v.earth_orientation,
        ObservedValues {
            dut1_microseconds: (v.dut1_seconds * 1e6).round() as i64,
            xp_microarcseconds: (v.xp_arcseconds * 1e6).round() as i64,
            yp_microarcseconds: (v.yp_arcseconds * 1e6).round() as i64,
        },
    ))
}

fn retained_motion(body: &str, degrees_per_day: f64) -> PlanetMotion {
    let rate = (degrees_per_day * 1e6).round() as i64;
    PlanetMotion {
        body: body.into(),
        microdegrees_per_tt_day: rate,
        classification: if rate.abs() <= 1000 {
            MotionClass::NearStation
        } else if rate < 0 {
            MotionClass::Retrograde
        } else {
            MotionClass::Direct
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classification_uses_the_retained_integer_rate() {
        for sign in [-1.0, 1.0] {
            assert_eq!(
                retained_motion("fixture", sign * 0.0010001).classification,
                MotionClass::NearStation
            );
            assert_eq!(
                retained_motion("fixture", sign * 0.0010006).classification,
                if sign < 0.0 {
                    MotionClass::Retrograde
                } else {
                    MotionClass::Direct
                }
            );
        }
    }
}
