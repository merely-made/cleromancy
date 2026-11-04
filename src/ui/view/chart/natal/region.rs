// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub(in super::super) fn region(ui: &ConsultationUi) -> Vec<ConsultationView> {
    let mut views: Vec<ConsultationView> = vec![
        Box::new(el("h2", "Personal natal readings")),
        Box::new(el(
            "p",
            "Choose a shared subject and saved birth revision. Birth facts are reusable across applications; a natal reading retains the exact revision it used. Place and timezone are supplied explicitly. Choose placements only, Whole Sign, Equal House, or Placidus. New occasions retain direct/retrograde motion facts. Houses and ascendant require an exact birth time and explicit birthplace coordinates.",
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
                Some(crate::natal_cap::HouseChoice::WholeSign),
                "Whole Sign",
                "natal-houses-whole",
            ),
            (
                Some(crate::natal_cap::HouseChoice::Equal),
                "Equal House",
                "natal-houses-equal",
            ),
            (
                Some(crate::natal_cap::HouseChoice::Placidus),
                "Placidus",
                "natal-houses-placidus",
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
        views.push(labelled_text(
            "Retained IERS finals2000A records (optional; final observed Bulletin B values only)",
            "cleromancy-natal-eop",
            true,
            &ui.natal_eop,
            eop,
        ));
        views.push(Box::new(el("p", "Paste an observed IERS excerpt covering the birth instant to use measured Earth orientation. Blank uses the disclosed UT1 = UTC approximation. Nothing is fetched automatically. Placidus is unavailable in polar regions; no alternate system is substituted.")));
        views.push(Box::new(el("h3", "Aspects for this new occasion")));
        let aspect_choices: Vec<ConsultationView> = vec![
            Box::new(
                button("Explicit aspect rules", |ui: &mut ConsultationUi, _| {
                    ui.natal_aspects_enabled = true
                })
                .attr("data-key", "natal-aspects-on")
                .attr("class", "natal-aspect-choice")
                .attr("aria-pressed", ui.natal_aspects_enabled.to_string()),
            ),
            Box::new(
                button(
                    "Original 1° aspect edition",
                    |ui: &mut ConsultationUi, _| ui.natal_aspects_enabled = false,
                )
                .attr("data-key", "natal-aspects-off")
                .attr("class", "natal-aspect-choice")
                .attr("aria-pressed", (!ui.natal_aspects_enabled).to_string()),
            ),
        ];
        views.push(Box::new(
            el("div", aspect_choices)
                .attr("role", "group")
                .attr("aria-label", "Natal aspect edition"),
        ));
        if ui.natal_aspects_enabled {
            views.push(Box::new(el("p", "Orbs are inclusive maximum differences from the exact angle. The editable product preset is conjunction/opposition 8°, trine/square 6°, sextile 4°. These are chosen geometric rules, with no luminary bonus or applying/separating claim. Zero admits exact matches only. All matching kinds are retained when wide orbs overlap.")));
            for (index, (label, id, get)) in [
                (
                    "Conjunction orb degrees",
                    "natal-orb-conjunction",
                    conjunction_orb as fn(&mut ConsultationUi) -> &mut TextInput,
                ),
                (
                    "Opposition orb degrees",
                    "natal-orb-opposition",
                    opposition_orb,
                ),
                ("Trine orb degrees", "natal-orb-trine", trine_orb),
                ("Square orb degrees", "natal-orb-square", square_orb),
                ("Sextile orb degrees", "natal-orb-sextile", sextile_orb),
            ]
            .into_iter()
            .enumerate()
            {
                views.push(labelled_text(
                    label,
                    id,
                    false,
                    &ui.natal_aspect_orbs[index],
                    get,
                ));
            }
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
                    let aspects = if ui.natal_aspects_enabled {
                        match aspect_rules(ui) {
                            Ok(rules) => Some(rules),
                            Err(error) => {
                                ui.error = Some(error);
                                return;
                            },
                        }
                    } else {
                        None
                    };
                    let action = ConsultationAction::CreateNatal {
                        resource: birth.resource.clone(),
                        question: ui.natal_question.text().to_string(),
                        fold: choice,
                        houses: ui.natal_house_system,
                        aspects,
                        orientation: (!ui.natal_eop.text().trim().is_empty()).then(|| {
                            crate::natal_cap::OrientationSnapshot {
                                finals2000a: ui.natal_eop.text().to_string(),
                            }
                        }),
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
                views.push(Box::new(el("p",format!("I resolved the saved civil birth time in {} to {}. Turquet calculates {} geocentric tropical placements; the original calculation edition retains a {} degree aspect orb. Explicit aspect rules, when chosen, appear below. {} {}",resolution.timezone,resolution.utc.replace('T'," ").replace("+00:00"," UTC"),facts.placements.len(),reading.orb_millidegrees as f64/1000.,crate::content::natal_pack().intro,house_summary(reading))).attr("data-key","natal-narration").attr("class","natal-narration")));
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
                        if reading.supplement.is_some() {
                            "Signed apparent longitude motion and optional house results are shown below with their retained method."
                        } else if reading.houses.is_some() {
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
        if let Some(receipt) = &reading.aspects {
            let mut aspect_views: Vec<ConsultationView> =
                vec![Box::new(el("h3", "Natal aspects · explicit orb rules"))];
            let rules = receipt.rules;
            aspect_views.push(Box::new(el("p", format!("Inclusive orbs: conjunction {:.3}°, opposition {:.3}°, trine {:.3}°, square {:.3}°, sextile {:.3}°. Shortest geocentric tropical longitude separation, from the saved millidegree positions. No latitude, house or motion adjustment.", rules.conjunction_millidegrees as f64/1000., rules.opposition_millidegrees as f64/1000., rules.trine_millidegrees as f64/1000., rules.square_millidegrees as f64/1000., rules.sextile_millidegrees as f64/1000.))));
            match &receipt.outcome {
                crate::natal_aspects::AspectOutcome::Unavailable { reason } => {
                    aspect_views.push(Box::new(el("p", reason.clone())))
                },
                crate::natal_aspects::AspectOutcome::Calculated { hits, .. } => {
                    if hits.is_empty() {
                        aspect_views
                            .push(Box::new(el("p", "No pairs meet these saved orb rules.")));
                    }
                    for hit in hits {
                        aspect_views.push(Box::new(el("p", format!("{} · {:?} · {}: separation {:.3}°; deviation {:.3}° from {:.0}°; allowed orb {:.3}°", hit.first, hit.kind, hit.second, hit.separation_millidegrees as f64/1000., hit.orb_millidegrees as f64/1000., hit.exact_millidegrees as f64/1000., rules.orb_millidegrees(hit.kind) as f64/1000.))));
                    }
                },
            }
            if ui.natal_details {
                aspect_views.push(Box::new(el("p", format!("Aspect method {}; schema {}. Original chart and its earlier calculation edition remain inspectable in the sealed receipt.", receipt.method, receipt.schema))));
            }
            views.push(Box::new(
                el("section", aspect_views)
                    .attr("class", "natal-aspects")
                    .attr("data-key", "natal-aspects"),
            ));
        }
        if let Some(s) = &reading.supplement {
            let mut motion_views: Vec<ConsultationView> =
                vec![Box::new(el("h3", "Apparent planetary motion"))];
            match &s.motion {
                crate::natal_cap::MotionOutcome::Unavailable { reason } => {
                    motion_views.push(Box::new(el("p", reason.clone())))
                },
                crate::natal_cap::MotionOutcome::Calculated {
                    sample_half_interval_seconds,
                    near_station_microdegrees_per_day,
                    planets,
                } => {
                    motion_views.push(Box::new(el("p",format!("Geocentric tropical longitude, sampled ±{} seconds. Rates round to a microdegree per day before classification. Near station means an absolute retained rate at most {}° per TT day; it does not identify an exact stationary instant.",sample_half_interval_seconds,*near_station_microdegrees_per_day as f64/1e6))));
                    for p in planets {
                        motion_views.push(Box::new(el(
                            "p",
                            format!(
                                "{}: {}, {:+.6}° per TT day",
                                p.body,
                                p.classification.label(),
                                p.microdegrees_per_tt_day as f64 / 1e6
                            ),
                        )));
                    }
                },
            }
            if let Some(o) = &s.observed_values {
                motion_views.push(Box::new(el("p",format!("Observed Earth orientation: UT1−UTC {:+.6} s; polar motion x {:+.6} arcsec, y {:+.6} arcsec. Retained IERS final observations.",o.dut1_microseconds as f64/1e6,o.xp_microarcseconds as f64/1e6,o.yp_microarcseconds as f64/1e6))));
            }
            if ui.natal_details {
                motion_views.push(Box::new(el("p",format!("Supplement {}; supplier {}. Original planetary positions retain their own earlier numerical edition.",s.schema,s.supplier))));
            }
            views.push(Box::new(
                el("section", motion_views)
                    .attr("class", "natal-motion")
                    .attr("data-key", "natal-motion"),
            ));
        }
        let mut house_views: Vec<ConsultationView> = Vec::new();
        let house_display = reading
            .supplement
            .as_ref()
            .and_then(|s| {
                s.house_system
                    .zip(s.houses.as_ref())
                    .map(|(system, outcome)| {
                        (
                            system.label(),
                            outcome,
                            s.earth_rotation.as_str(),
                            s.schema.as_str(),
                            s.supplier.as_str(),
                        )
                    })
            })
            .or_else(|| {
                reading.houses.as_ref().map(|h| {
                    (
                        h.system.label(),
                        &h.outcome,
                        h.earth_rotation.as_str(),
                        h.method.as_str(),
                        h.supplier.as_str(),
                    )
                })
            });
        if let Some((system, outcome, earth_rotation, method, supplier)) = house_display {
            use crate::natal_houses::HouseOutcome;
            house_views.push(Box::new(el(
                "h3",
                format!("{} houses and ascendant", system),
            )));
            match outcome {
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
                    house_views.push(Box::new(el("p",format!("Ascendant (rising sign): {}. Midheaven: {}. Birthplace: {}. {}. Airless tropical geometry; planetary houses use longitude only.",angle(*ascendant_microdegrees),angle(*midheaven_microdegrees),reading.birth.facts.place.as_ref().map(|p|p.label.as_str()).unwrap_or("unspecified"),earth_rotation))));
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
                house_views.push(Box::new(el("p",format!("House method {}; supplier {}; selected system {:?}; {}. Longitude angles retained in microdegrees.",method,supplier,system,earth_rotation))));
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
