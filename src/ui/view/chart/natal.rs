// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::natal::{FoldChoice, NatalOutcome};
use crate::ui::view::{ConsultationView, labelled_text};
use crate::ui::{action::ConsultationAction, state::ConsultationUi};
use cambium::{TextInput, button, el};
fn draft(ui: &ConsultationUi) -> Result<dramatis::birth::BirthFacts, String> {
    let subject = ui
        .birth_catalog
        .as_ref()
        .and_then(|c| c.subjects.get(ui.birth_subject))
        .ok_or("Choose a shared subject")?
        .subject
        .clone();
    let d = ui
        .natal_date
        .text()
        .split('-')
        .map(str::parse::<i32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Use YYYY-MM-DD")?;
    if d.len() != 3 {
        return Err("Use YYYY-MM-DD".into());
    }
    let date = dramatis::birth::CivilDate {
        year: d[0],
        month: u8::try_from(d[1]).map_err(|_| "Invalid month")?,
        day: u8::try_from(d[2]).map_err(|_| "Invalid day")?,
    };
    let t = ui.natal_time.text().trim();
    let time = if t == "unknown" {
        dramatis::birth::BirthTime::Unknown
    } else {
        let approx = t.starts_with('~');
        let raw = t.trim_start_matches('~');
        let parts: Vec<_> = raw.split('±').map(str::trim).collect();
        let h = parts[0]
            .split(':')
            .map(str::parse::<u8>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Use HH:MM:SS or unknown")?;
        if h.len() != 3 || parts.len() > 2 {
            return Err("Use HH:MM:SS or ~HH:MM:SS ±seconds".into());
        }
        if approx {
            dramatis::birth::BirthTime::Approximate {
                hour: h[0],
                minute: h[1],
                second: h[2],
                uncertainty_seconds: parts
                    .get(1)
                    .ok_or("Supply approximate uncertainty in seconds")?
                    .parse()
                    .map_err(|_| "Invalid uncertainty")?,
            }
        } else {
            if parts.len() != 1 {
                return Err("Approximate time starts with ~".into());
            }
            dramatis::birth::BirthTime::Known {
                hour: h[0],
                minute: h[1],
                second: h[2],
            }
        }
    };
    let coordinate = |v: &str| -> Result<Option<i32>, String> {
        if v.trim().is_empty() {
            return Ok(None);
        }
        let n: f64 = v.trim().parse().map_err(|_| "Invalid coordinate")?;
        if !n.is_finite() || n.abs() > 180. {
            return Err("Coordinate out of range".into());
        }
        Ok(Some((n * 1e6).round() as i32))
    };
    let lat = coordinate(ui.natal_latitude.text())?;
    let lon = coordinate(ui.natal_longitude.text())?;
    let place = if ui.natal_place.text().trim().is_empty() {
        if lat.is_some() || lon.is_some() {
            return Err("Supply a place label with coordinates".into());
        }
        None
    } else {
        Some(dramatis::birth::BirthPlace {
            label: ui.natal_place.text().to_string(),
            latitude_microdegrees: lat,
            longitude_microdegrees: lon,
            precision: "User supplied; coordinate rounding to microdegrees".into(),
        })
    };
    let facts = dramatis::birth::BirthFacts {
        schema: dramatis::birth::BIRTH_FACTS_SCHEMA.into(),
        subject,
        date,
        time,
        place,
        timezone: (!ui.natal_zone.text().trim().is_empty())
            .then(|| ui.natal_zone.text().trim().to_string()),
        source: dramatis::birth::FactSource {
            label: ui.natal_source.text().to_string(),
            reference: None,
        },
    };
    facts.validate().map_err(|e| e.to_string())?;
    Ok(facts)
}
macro_rules! getter{($($name:ident=>$field:ident),*)=>{$(fn $name(ui:&mut ConsultationUi)->&mut TextInput{&mut ui.$field})*};}
getter!(eop=>natal_eop,reflection=>natal_reflection,name=>natal_name,date=>natal_date,time=>natal_time,zone=>natal_zone,place=>natal_place,latitude=>natal_latitude,longitude=>natal_longitude,source=>natal_source,question=>natal_question,fold=>natal_fold);

fn time_label(time: &dramatis::birth::BirthTime) -> String {
    match time {
        dramatis::birth::BirthTime::Unknown => "unknown".into(),
        dramatis::birth::BirthTime::Known {
            hour,
            minute,
            second,
        } => format!("{hour:02}:{minute:02}:{second:02}"),
        dramatis::birth::BirthTime::Approximate {
            hour,
            minute,
            second,
            uncertainty_seconds,
        } => format!("approximately {hour:02}:{minute:02}:{second:02} ±{uncertainty_seconds}s"),
    }
}

fn house_summary(reading: &crate::natal::NatalReading) -> String {
    use crate::natal_houses::HouseOutcome;
    if let Some(s) = &reading.supplement {
        if let Some(system) = s.house_system {
            return match s.houses.as_ref() {
                Some(HouseOutcome::Calculated {
                    ascendant_microdegrees: a,
                    ..
                }) => format!(
                    "{}: ascendant {:?} {:.3}°. Apparent motion retained. {}",
                    system.label(),
                    crate::astrology::ZodiacSign::from_longitude(*a / 1000),
                    (*a % 30_000_000) as f64 / 1e6,
                    s.earth_rotation
                ),
                Some(HouseOutcome::Unavailable { reason }) => format!(
                    "{}: {}. Apparent motion shown below.",
                    system.label(),
                    reason
                ),
                None => String::new(),
            };
        }
        return "Apparent direct/retrograde motion is retained below; no houses were selected."
            .into();
    }
    match reading.houses.as_ref() {
        None => String::new(),
        Some(h) => match &h.outcome {
            HouseOutcome::Unavailable { reason } => format!("{}: {}", h.system.label(), reason),
            HouseOutcome::Calculated {
                ascendant_microdegrees: a,
                ..
            } => format!(
                "{}: ascendant (rising sign) {:?} {:.3}°. {}.",
                h.system.label(),
                crate::astrology::ZodiacSign::from_longitude(*a / 1000),
                (*a % 30_000_000) as f64 / 1e6,
                h.earth_rotation
            ),
        },
    }
}

fn conjunction_orb(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.natal_aspect_orbs[0]
}
fn opposition_orb(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.natal_aspect_orbs[1]
}
fn trine_orb(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.natal_aspect_orbs[2]
}
fn square_orb(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.natal_aspect_orbs[3]
}
fn sextile_orb(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.natal_aspect_orbs[4]
}
fn aspect_rules(ui: &ConsultationUi) -> Result<crate::natal_aspects::AspectRules, String> {
    let mut values = [0; 5];
    for (index, input) in ui.natal_aspect_orbs.iter().enumerate() {
        let label = ["Conjunction", "Opposition", "Trine", "Square", "Sextile"][index];
        if input.text().trim().is_empty() {
            return Err(format!(
                "{label} orb: enter degrees, including 0 for exact matches."
            ));
        }
        values[index] = crate::ui::chart_input::degrees(input.text(), 3)
            .map_err(|error| format!("{label} orb: {error}"))?
            .parse::<u32>()
            .map_err(|_| format!("{label} orb cannot be negative."))?;
        if values[index] > 180_000 {
            return Err(format!("{label} orb must be at most 180 degrees."));
        }
    }
    let rules = crate::natal_aspects::AspectRules {
        conjunction_millidegrees: values[0],
        opposition_millidegrees: values[1],
        trine_millidegrees: values[2],
        square_millidegrees: values[3],
        sextile_millidegrees: values[4],
    };
    rules.validate()?;
    Ok(rules)
}

#[cfg(test)]
mod aspect_input_tests;

mod region;

pub(super) use region::region;
