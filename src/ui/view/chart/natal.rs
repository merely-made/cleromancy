// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::natal::{FoldChoice, NatalOutcome};
use crate::ui::view::{ConsultationView, labelled_text};
use crate::ui::{action::ConsultationAction, state::ConsultationUi};
use cambium::{TextInput, button, el};
pub(super) fn region(ui: &ConsultationUi) -> Vec<ConsultationView> {
    let mut views: Vec<ConsultationView> = vec![
        Box::new(el("h2", "Personal natal readings")),
        Box::new(el(
            "p",
            "Choose a shared subject and saved birth revision. Birth facts are reusable across applications; a natal reading retains the exact revision it used. Place and timezone are supplied explicitly. Choose placements only, Whole Sign, or Equal House. Houses and ascendant require an exact birth time and explicit birthplace coordinates.",
        )),
        Box::new(
            button("Open shared birth facts", |ui: &mut ConsultationUi, _| {
                ui.record_action(Some(ConsultationAction::RefreshBirthFacts))
            })
            .attr("data-key", "natal-open"),
        ),
    ];
    if !ui.birth_notice.is_empty() {
        views.push(Box::new(el("p", ui.birth_notice.clone())));
    }
    views.push(labelled_text(
        "New subject name",
        "cleromancy-natal-name",
        false,
        &ui.natal_name,
        name,
    ));
    views.push(Box::new(
        button("Create shared contact", |ui: &mut ConsultationUi, _| {
            let label = ui.natal_name.text().to_string();
            ui.record_action(Some(ConsultationAction::CreateSubject { label }));
        })
        .attr("data-key", "natal-create-subject"),
    ));
    if let Some(catalog) = &ui.birth_catalog {
        for (index, subject) in catalog.subjects.iter().enumerate() {
            views.push(Box::new(
                button(
                    format!(
                        "{}{}",
                        if index == ui.birth_subject {
                            "Selected: "
                        } else {
                            "Subject: "
                        },
                        subject.label
                    ),
                    move |ui: &mut ConsultationUi, _| ui.birth_subject = index,
                )
                .attr("data-key", format!("natal-subject-{index}")),
            ));
        }
        macro_rules! input {
            ($label:expr,$field:ident,$get:ident,$id:expr) => {
                views.push(labelled_text(
                    $label,
                    concat!("cleromancy-natal-", $id),
                    false,
                    &ui.$field,
                    $get,
                ));
            };
        }
        input!("Birth date (YYYY-MM-DD)", natal_date, date, "date");
        input!(
            "Birth time (HH:MM:SS, unknown, or ~HH:MM:SS ±seconds)",
            natal_time,
            time,
            "time"
        );
        input!(
            "IANA timezone (for example America/New_York)",
            natal_zone,
            zone,
            "zone"
        );
        input!("Birthplace label (optional)", natal_place, place, "place");
        input!(
            "Latitude degrees (optional)",
            natal_latitude,
            latitude,
            "latitude"
        );
        input!(
            "Longitude degrees (optional)",
            natal_longitude,
            longitude,
            "longitude"
        );
        input!("Birth facts source", natal_source, source, "source");
        views.push(Box::new(
            button(
                "Save shared birth revision",
                |ui: &mut ConsultationUi, _| match draft(ui) {
                    Ok(facts) => {
                        ui.record_action(Some(ConsultationAction::SaveBirthFacts { facts }))
                    },
                    Err(e) => ui.error = Some(e),
                },
            )
            .attr("data-key", "natal-save-birth"),
        ));
        for (index, birth) in catalog.births.iter().enumerate() {
            let d = &birth.facts.date;
            views.push(Box::new(
                button(
                    format!(
                        "{}Birth revision {}-{:02}-{:02} · {} · {} · {}",
                        if index == ui.birth_revision {
                            "Selected: "
                        } else {
                            ""
                        },
                        d.year,
                        d.month,
                        d.day,
                        time_label(&birth.facts.time),
                        catalog
                            .subjects
                            .iter()
                            .find(|s| s.subject == birth.facts.subject)
                            .map(|s| s.label.as_str())
                            .unwrap_or("Unavailable subject"),
                        &birth.resource.revision[..12]
                    ),
                    move |ui: &mut ConsultationUi, _| ui.birth_revision = index,
                )
                .attr("data-key", format!("natal-birth-{index}")),
            ));
        }
        input!(
            "Question for this natal occasion",
            natal_question,
            question,
            "question"
        );
        input!(
            "Repeated local time choice (refuse, earlier, later)",
            natal_fold,
            fold,
            "fold"
        );
        views.push(Box::new(el("p", "House system for this new occasion")));
        for (choice, label, key) in [
            (None, "Placements only", "natal-houses-none"),
            (
                Some(crate::natal_houses::HouseSystem::WholeSign),
                "Whole Sign",
                "natal-houses-whole",
            ),
            (
                Some(crate::natal_houses::HouseSystem::Equal),
                "Equal House",
                "natal-houses-equal",
            ),
        ] {
            views.push(Box::new(
                button(
                    format!(
                        "{}{}",
                        if ui.natal_house_system == choice {
                            "Selected: "
                        } else {
                            ""
                        },
                        label
                    ),
                    move |ui: &mut ConsultationUi, _| ui.natal_house_system = choice,
                )
                .attr("data-key", key),
            ));
        }
        views.push(Box::new(
            button("Create natal reading", |ui: &mut ConsultationUi, _| {
                let choice = match ui.natal_fold.text().trim() {
                    "" | "refuse" => FoldChoice::Refuse,
                    "earlier" => FoldChoice::Earlier,
                    "later" => FoldChoice::Later,
                    _ => {
                        ui.error = Some("Choose refuse, earlier or later.".into());
                        return;
                    },
                };
                if let Some(birth) = ui
                    .birth_catalog
                    .as_ref()
                    .and_then(|c| c.births.get(ui.birth_revision))
                {
                    let action = ConsultationAction::CreateNatal {
                        resource: birth.resource.clone(),
                        question: ui.natal_question.text().to_string(),
                        fold: choice,
                        houses: ui.natal_house_system,
                    };
                    ui.record_action(Some(action));
                }
            })
            .attr("data-key", "natal-read"),
        ));
    }
    let form = std::mem::take(&mut views);
    for reading in &ui.catalog.natal_readings {
        let id = reading.id.clone();
        views.push(Box::new(
            button(
                format!(
                    "Open natal occasion for {} · {}",
                    reading.subject_label,
                    crate::situation::utc_label(reading.created_at_ms)
                ),
                move |ui: &mut ConsultationUi, _| ui.natal_selected = Some(id.clone()),
            )
            .attr("data-key", format!("natal-select-{}", reading.id)),
        ));
    }
    let history = std::mem::take(&mut views);
    let selected = ui
        .natal_selected
        .as_ref()
        .and_then(|id| ui.catalog.natal_readings.iter().find(|r| &r.id == id))
        .or_else(|| ui.catalog.natal_readings.first());
    if let Some(reading) = selected {
        views.push(Box::new(el(
            "h3",
            format!("Natal reading for {}", reading.subject_label),
        )));
        views.push(Box::new(el("p", reading.question.clone())));
        let f = &reading.birth.facts;
        views.push(Box::new(el(
            "p",
            format!(
                "Birth date {}-{:02}-{:02}; civil time {}; timezone {}; source {}.",
                f.date.year,
                f.date.month,
                f.date.day,
                time_label(&f.time),
                f.timezone.as_deref().unwrap_or("unspecified"),
                f.source.label
            ),
        )));
        match &reading.outcome {
            NatalOutcome::Unavailable { reason } => views.push(Box::new(el("p", reason.clone()))),
            NatalOutcome::Calculated {
                resolution,
                facts,
                prompts,
                ..
            } => {
                views.push(Box::new(el("p",format!("I resolved the saved civil birth time in {} to {}. Turquet calculates {} geocentric tropical placements; aspects use a {} degree orb. {} {}",resolution.timezone,resolution.utc.replace('T'," ").replace("+00:00"," UTC"),facts.placements.len(),reading.orb_millidegrees as f64/1000.,crate::content::natal_pack().intro,house_summary(reading))).attr("data-key","natal-narration").attr("class","natal-narration")));
                views.push(Box::new(
                    button(
                        if ui.natal_details {
                            "Hide calculation details"
                        } else {
                            "Show calculation details"
                        },
                        |ui: &mut ConsultationUi, _| ui.natal_details = !ui.natal_details,
                    )
                    .attr("data-key", "natal-details"),
                ));
                // Keep one receipt slot mounted so toggling cannot shift the
                // retained prompt nodes into a different semantic role.
                let receipt = if ui.natal_details {
                    format!(
                        "Birth resource {} at revision {}; timezone database {}; selected UTC offset {} seconds; repeated-time choice {:?}; numerical method {}. {}",
                        reading.birth.resource.id,
                        reading.birth.resource.revision,
                        resolution.tzdb,
                        resolution.offset_seconds,
                        resolution.fold,
                        crate::natal::METHOD,
                        if reading.houses.is_some() {
                            "House results and their method are shown below. No retrograde calculation."
                        } else {
                            "No houses, rising sign or retrograde calculation."
                        }
                    )
                } else {
                    String::new()
                };
                views.push(Box::new(
                    el("p", receipt).attr("data-key", "natal-calculation-receipt"),
                ));

                for prompt in prompts {
                    views.push(Box::new(el("h4", prompt.title.clone())));
                    views.push(Box::new(el("p", prompt.text.clone())));
                }
            },
        }
        let mut house_views: Vec<ConsultationView> = Vec::new();
        if let Some(receipt) = &reading.houses {
            use crate::natal_houses::HouseOutcome;
            house_views.push(Box::new(el(
                "h3",
                format!("{} houses and ascendant", receipt.system.label()),
            )));
            match &receipt.outcome {
                HouseOutcome::Unavailable { reason } => {
                    house_views.push(Box::new(el("p", reason.clone())))
                },
                HouseOutcome::Calculated {
                    ascendant_microdegrees,
                    midheaven_microdegrees,
                    cusps_microdegrees,
                    placements,
                } => {
                    let angle = |value: u32| {
                        format!(
                            "{:?} {:.3}°",
                            crate::astrology::ZodiacSign::from_longitude(value / 1000),
                            (value % 30_000_000) as f64 / 1e6
                        )
                    };
                    house_views.push(Box::new(el("p",format!("Ascendant (rising sign): {}. Midheaven: {}. Birthplace: {}. {}. Airless tropical geometry; planetary houses use longitude only.",angle(*ascendant_microdegrees),angle(*midheaven_microdegrees),reading.birth.facts.place.as_ref().map(|p|p.label.as_str()).unwrap_or("unspecified"),receipt.earth_rotation))));
                    for (i, cusp) in cusps_microdegrees.iter().enumerate() {
                        house_views.push(Box::new(el(
                            "p",
                            format!("House {} begins at {}", i + 1, angle(*cusp)),
                        )));
                    }
                    for p in placements {
                        house_views.push(Box::new(el(
                            "p",
                            format!("{} in house {}", p.body, p.house),
                        )));
                    }
                },
            }
            if ui.natal_details {
                house_views.push(Box::new(el("p",format!("House method {}; supplier {}; selected system {:?}; {}. Longitude angles retained in microdegrees.",receipt.method,receipt.supplier,receipt.system,receipt.earth_rotation))));
            }
        }
        views.push(Box::new(
            el("section", house_views)
                .attr("data-key", "natal-houses")
                .attr("class", "natal-houses"),
        ));
        for reflection in ui
            .catalog
            .natal_reflections
            .iter()
            .filter(|r| r.session_id == reading.id)
        {
            views.push(Box::new(el(
                "p",
                format!("Reflection: {}", reflection.body),
            )));
        }
        let example = format!(
            "Thinking about {}: what in my experience speaks to this reading?",
            reading.question
        );
        views.push(Box::new(
            button(
                "Use a reflection example",
                move |ui: &mut ConsultationUi, _| {
                    ui.natal_reflection = TextInput::new(example.clone());
                },
            )
            .attr("data-key", format!("natal-example-{}", reading.id)),
        ));
        views.push(labelled_text(
            "Reflection on this natal occasion",
            "cleromancy-natal-reflection",
            true,
            &ui.natal_reflection,
            reflection,
        ));
        let id = reading.id.clone();
        views.push(Box::new(
            button(
                "Save natal reflection",
                move |ui: &mut ConsultationUi, _| {
                    let action = ConsultationAction::ReflectNatal {
                        id: id.clone(),
                        body: ui.natal_reflection.text().to_string(),
                    };
                    ui.record_action(Some(action));
                },
            )
            .attr("data-key", format!("natal-reflect-{}", reading.id)),
        ));
    }
    vec![
        Box::new(el("section", history).attr("data-key", "natal-history")),
        Box::new(el("section", views).attr("data-key", "natal-focused-occasion")),
        Box::new(el("section", form).attr("data-key", "natal-birth-authoring")),
    ]
}
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
getter!(reflection=>natal_reflection,name=>natal_name,date=>natal_date,time=>natal_time,zone=>natal_zone,place=>natal_place,latitude=>natal_latitude,longitude=>natal_longitude,source=>natal_source,question=>natal_question,fold=>natal_fold);

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
