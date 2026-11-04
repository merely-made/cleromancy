// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl ScenarioDriver<'_, '_> {
    pub(super) fn app_step(&mut self, line: &str) -> Result<(), String> {
        if let Some(command) = line
            .strip_prefix("c4-")
            .or_else(|| line.strip_prefix("c5-control-"))
            .or_else(|| line.strip_prefix("c7-control-"))
            .or_else(|| line.strip_prefix("c8-control-"))
        {
            return self.control_step(line, command);
        }
        match line {
            "natal-aspects-exact-draft" => {
                self.ctx.runner.update(|s| {
                    s.consultation.natal_aspect_orbs =
                        std::array::from_fn(|_| cambium::TextInput::new("0"))
                });
                Ok(())
            },
            "natal-select-aspects" => {
                self.ctx.runner.update(|s| {
                    s.consultation.natal_selected = s
                        .consultation
                        .catalog
                        .natal_readings
                        .iter()
                        .find(|r| r.aspects.is_some())
                        .map(|r| r.id.clone())
                });
                Ok(())
            },
            "natal-prepare-subject" => {
                self.ctx.runner.update(|s| {
                    s.consultation.natal_name =
                        cambium::TextInput::new("Public synthetic natal subject")
                });
                Ok(())
            },
            "natal-prepare-birth" => {
                self.ctx.runner.update(|s| {
                    let ui = &mut s.consultation;
                    ui.natal_date = cambium::TextInput::new("2000-01-01");
                    ui.natal_time = cambium::TextInput::new("12:00:00");
                    ui.natal_zone = cambium::TextInput::new("UTC");
                    ui.natal_source =
                        cambium::TextInput::new("Public synthetic fixture; no real birth data");
                    ui.natal_question = cambium::TextInput::new(
                        "Which themes can I reflect on in this synthetic natal occasion?",
                    );
                });
                Ok(())
            },
            "natal-prepare-place" => {
                self.ctx.runner.update(|s| {
                    let ui = &mut s.consultation;
                    ui.natal_place = cambium::TextInput::new("Public New York coordinate fixture");
                    ui.natal_latitude = cambium::TextInput::new("40.7128");
                    ui.natal_longitude = cambium::TextInput::new("-74.0060");
                });
                Ok(())
            },
            "natal-select-whole" | "natal-select-equal" | "natal-select-missing-place" => {
                let target = match line {
                    "natal-select-whole" => Some(crate::natal_houses::HouseSystem::WholeSign),
                    _ => Some(crate::natal_houses::HouseSystem::Equal),
                };
                let missing = line == "natal-select-missing-place";
                self.ctx.runner.update(|s| {
                    let ui = &mut s.consultation;
                    ui.natal_selected = ui
                        .catalog
                        .natal_readings
                        .iter()
                        .find(|r| {
                            r.houses.as_ref().is_some_and(|h| {
                                if missing {
                                    matches!(
                                        h.outcome,
                                        crate::natal_houses::HouseOutcome::Unavailable { .. }
                                    )
                                } else {
                                    Some(h.system) == target
                                        && matches!(
                                            h.outcome,
                                            crate::natal_houses::HouseOutcome::Calculated { .. }
                                        )
                                }
                            })
                        })
                        .map(|r| r.id.clone());
                });
                Ok(())
            },
            "natal-prepare-correction" => {
                self.ctx
                    .runner
                    .update(|s| s.consultation.natal_time = cambium::TextInput::new("13:00:00"));
                Ok(())
            },
            "natal-prepare-observed" => {
                self.ctx.runner.update(|s| {
                    s.consultation.natal_eop = cambium::TextInput::new(include_str!(
                        "../../../../tests/fixtures/natal-observed-finals2000a.txt"
                    ))
                });
                Ok(())
            },
            "natal-clear-observed" => {
                self.ctx
                    .runner
                    .update(|s| s.consultation.natal_eop = cambium::TextInput::default());
                Ok(())
            },
            "natal-prepare-polar" => {
                self.ctx.runner.update(|s| {
                    s.consultation.natal_latitude = cambium::TextInput::new("70.0");
                    s.consultation.natal_longitude = cambium::TextInput::new("0.0");
                    s.consultation.natal_place =
                        cambium::TextInput::new("Public synthetic polar fixture");
                });
                Ok(())
            },
            "natal-select-cap" | "natal-select-observed" | "natal-select-polar" => {
                let observed = line == "natal-select-observed";
                let polar = line == "natal-select-polar";
                self.ctx.runner.update(|s|s.consultation.natal_selected=s.consultation.catalog.natal_readings.iter().find(|r|r.supplement.as_ref().is_some_and(|c|c.house_system==Some(crate::natal_cap::HouseChoice::Placidus)&&if polar {matches!(c.houses,Some(crate::natal_houses::HouseOutcome::Unavailable{..}))}else{c.observed_values.is_some()==observed&&matches!(c.houses,Some(crate::natal_houses::HouseOutcome::Calculated{..}))})).map(|r|r.id.clone()));
                Ok(())
            },
            "natal-prepare-reflection" => {
                self.ctx.runner.update(|s|s.consultation.natal_reflection=cambium::TextInput::new("Public natal fixture: an authored prompt offers a question, not a prediction."));
                Ok(())
            },
            "c8-prepare-reflection" => {
                let detail = self
                    .ctx
                    .runner
                    .state()
                    .consultation
                    .detail()
                    .ok_or("C8 reflection has no current die result")?;
                if detail.field.rules != crate::UNIFORM_DIE_RULE {
                    return Err("C8 reflection requires the newly saved die result".into());
                }
                self.ctx.runner.update(|state| {
                    state.consultation.reflection = cambium::TextInput::new("Public dice fixture: I recorded this outcome without assigning it a symbolic meaning.");
                });
                Ok(())
            },
            "c8-prepare-os" | "c8-prepare-seed" => {
                let seeded = line == "c8-prepare-seed";
                let field_index = self
                    .ctx
                    .runner
                    .state()
                    .consultation
                    .catalog
                    .fields
                    .iter()
                    .position(|f| f.system == "cleromancy.die/d6")
                    .ok_or("install die through consultation form before preparing roll")?;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.field_select.selected = field_index;
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new(if seeded {
                        "C8 public-seed roll"
                    } else {
                        "C8 operating-system roll"
                    });
                    ui.question = cambium::TextInput::new("Which face appears on this roll?");
                    ui.tags = cambium::TextInput::default();
                    ui.additional_facts = cambium::TextInput::default();
                    ui.layout.selected = 0;
                    ui.mode.selected = if seeded { 2 } else { 1 };
                    ui.derived_seed = cambium::TextInput::new("C8 public die");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c8/qualification");
                    ui.astrology_facts_select.selected = 0;
                    ui.set_place_for_draw(None);
                    ui.set_weather_for_draw(false);
                });
                self.state.scenario_stage = 1;
                Ok(())
            },
            "c6-prepare-global" | "c6-prepare-weather" => {
                let weather = line == "c6-prepare-weather";
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new(if weather {
                        "C6 public Greenwich weather"
                    } else {
                        "C6 without place"
                    });
                    ui.question =
                        cambium::TextInput::new("What detail of this occasion deserves attention?");
                    ui.tags = cambium::TextInput::default();
                    ui.additional_facts = cambium::TextInput::default();
                    ui.layout.selected = 1;
                    ui.mode.selected = 2;
                    ui.derived_seed = cambium::TextInput::new("C6 public situation");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c6/qualification");
                    ui.astrology_facts_select.selected = 0;
                    ui.set_place_for_draw(if weather {
                        Some(
                            crate::situation::ReadingPlace::new(51_476_900, 0)
                                .expect("public Greenwich coordinates"),
                        )
                    } else {
                        None
                    });
                    ui.set_weather_for_draw(weather);
                });
                self.state.scenario_stage = 1;
                Ok(())
            },
            "c5-read" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(field) = ui.catalog.fields.iter().position(|f| {
                        f.tarot
                            .as_ref()
                            .is_some_and(|d| d.id == "cleromancy.tarot/rws-complete-reflective-v1")
                    }) else {
                        error = Some("missing C5 typed deck".into());
                        return;
                    };
                    let Some(template) = ui
                        .catalog
                        .spread_templates
                        .iter()
                        .position(|t| t.label == "C2 Letting go")
                    else {
                        error = Some("missing saved custom layout".into());
                        return;
                    };
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new("C5 one shuffled deck");
                    ui.question = cambium::TextInput::new("What can I release next?");
                    ui.tags = cambium::TextInput::new("");
                    ui.additional_facts = cambium::TextInput::new("");
                    ui.field_select.selected = field;
                    ui.mode.selected = 2;
                    ui.derived_seed = cambium::TextInput::new("C5 public deck");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c5/qualification");
                    ui.layout.selected = 2;
                    ui.template_select.selected = template + 1;
                    ui.astrology_facts_select.selected = 0;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c3-read" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(index) = ui.catalog.fields.iter().position(|f| {
                        f.tarot
                            .as_ref()
                            .is_some_and(|d| d.id == "cleromancy.tarot/rws-complete-reflective-v1")
                    }) else {
                        error = Some("C3 RWS typed deck is missing".to_string());
                        return;
                    };
                    ui.context_label = cambium::TextInput::new("C3 patient courage");
                    ui.question = cambium::TextInput::new("What can I meet patiently?");
                    ui.tags = cambium::TextInput::new("courage");
                    ui.field_select.selected = index;
                    ui.mode.selected = 0;
                    ui.layout.selected = 0;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c3-install" | "c3-crosswalk" | "c3-return" => {
                let key = if line == "c3-install" {
                    "install-typed-decks"
                } else if line == "c3-crosswalk" {
                    "crosswalk:cleromancy.tarot/marseille-complete-reflective-v1"
                } else {
                    "return-drawn-card"
                };
                let node = {
                    let projection = self
                        .ctx
                        .a11y_projection()
                        .ok_or("host accessibility projection unavailable for scenario control")?;
                    let dom = self.ctx.runner.dom();
                    let dom = dom.borrow();
                    taproot::matching_with_projection(
                        &dom,
                        &taproot::Selector::role("button").with_attr("data-key", key),
                        &projection,
                    )
                    .into_iter()
                    .next()
                    .ok_or("C3 inspection control missing")?
                };
                let (_, _, width, height) = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("C3 inspection control is not painted")?;
                self.ctx
                    .runner
                    .dispatch_click(node, cambium::PointerClick::at((width / 2.0, height / 2.0)));
                if line == "c3-install" {
                    submit_pending(self.ctx.runner, self.state.worker.as_ref());
                }
                Ok(())
            },
            "c2-author-layout" => {
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.template_label = cambium::TextInput::new("C2 Letting go");
                    ui.template_positions = cambium::TextInput::new("foundation | What I must release\ntension | What resists\nnext_step | What I can try");
                    ui.template_relations = cambium::TextInput::new("tension | questions | foundation | presses on what I hold");
                    let action = ui.request_spread_template();
                    ui.record_action(action);
                });
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            "c2-read-layout" => {
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    let Some(index) = ui
                        .catalog
                        .spread_templates
                        .iter()
                        .position(|t| t.label == "C2 Letting go")
                    else {
                        error = Some("C2 authored layout is missing".to_string());
                        return;
                    };
                    ui.context_label = cambium::TextInput::new("C2 change of habit");
                    ui.question = cambium::TextInput::new("What am I ready to set down?");
                    ui.tags = cambium::TextInput::new("change, reflection");
                    ui.additional_facts = cambium::TextInput::new("season: autumn");
                    ui.field_select.selected = 0;
                    ui.mode.selected = 1;
                    ui.layout.selected = 2;
                    ui.template_select.selected = index + 1;
                    let action = ui.request_read();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                self.state.scenario_stage = 1;
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            _ if line.starts_with("assert visible key ") => {
                let target = line.trim_start_matches("assert visible key ");
                let node = self.keyed(target)?;
                let painted = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("missing painted bounds")?;
                let visible = self
                    .ctx
                    .visible_rect(node)
                    .ok_or("missing visible bounds")?;
                if painted.2 <= 0.0
                    || painted.3 <= 0.0
                    || (painted.0 - visible.0).abs() > 1.0
                    || (painted.1 - visible.1).abs() > 1.0
                    || (painted.2 - visible.2).abs() > 1.0
                    || (painted.3 - visible.3).abs() > 1.0
                {
                    return Err(format!(
                        "{target} clipped: painted={painted:?}, visible={visible:?}"
                    ));
                }
                eprintln!("cleromancy: visible {target}: painted={painted:?}, visible={visible:?}");
                Ok(())
            },
            _ if line.starts_with("reveal class ") => {
                let class = line.trim_start_matches("reveal class ");
                let node = {
                    let dom = self.ctx.runner.dom();
                    let dom = dom.borrow();
                    taproot::matching_classes(&dom, &taproot::Selector::class(class))
                        .into_iter()
                        .next()
                        .ok_or_else(|| format!("no .{class}"))?
                };
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Start);
                Ok(())
            },
            _ if line.starts_with("reveal key ") => {
                let node = self.keyed(line.trim_start_matches("reveal key "))?;
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Start);
                Ok(())
            },
            // Acceptance-only: deliver the mounted view's own hover handler,
            // as the host does when its hit target enters or leaves.
            _ if line.starts_with("hover key ") || line.starts_with("unhover key ") => {
                let (phase, key) = match line.strip_prefix("hover key ") {
                    Some(key) => (cambium::HoverPhase::Enter, key),
                    None => (
                        cambium::HoverPhase::Leave,
                        line.trim_start_matches("unhover key "),
                    ),
                };
                let node = self.keyed(key)?;
                let (_, _, width, height) = self
                    .ctx
                    .painted_rect(node)
                    .ok_or_else(|| format!("{key} has no painted bounds"))?;
                self.ctx.runner.dispatch_hover(
                    node,
                    cambium::HoverEvent::new(phase, (width / 2.0, height / 2.0), (width, height)),
                );
                Ok(())
            },
            _ if line.starts_with("reveal mere-control ") => {
                let step = line.trim_start_matches("reveal mere-control ");
                let node = self.mere_control(step)?;
                self.ctx
                    .scroll_into_view(node, cambium_genet_winit_host::ScrollAlign::Nearest);
                Ok(())
            },
            _ if line.starts_with("assert visible mere-control ") => {
                let step = line.trim_start_matches("assert visible mere-control ");
                let node = self.mere_control(step)?;
                let painted = self
                    .ctx
                    .painted_rect(node)
                    .ok_or("control has no painted bounds")?;
                let visible = self
                    .ctx
                    .visible_rect(node)
                    .ok_or("control is clipped out of view")?;
                let (width, height) = self.ctx.logical_size;
                let tolerance = 1.0;
                if painted.2 <= 0.0
                    || painted.3 <= 0.0
                    || visible.2 + tolerance < painted.2
                    || visible.3 + tolerance < painted.3
                    || visible.0 < -tolerance
                    || visible.1 < -tolerance
                    || visible.0 + visible.2 > width + tolerance
                    || visible.1 + visible.3 > height + tolerance
                {
                    return Err(format!(
                        "mere-control {step} not fully reachable: painted={painted:?}, visible={visible:?}, viewport={width}x{height}"
                    ));
                }
                let event = format!(
                    "visible mere-control {step}: painted={painted:?}, visible={visible:?}, viewport={width}x{height}"
                );
                // Acceptance-only: retain the exact observed bounds in native.log
                // as well as the event stream, whose successful payloads Taproot
                // does not currently include in its saved Outcome.
                eprintln!("cleromancy: {event}");
                self.state.probe_events.push(event);
                Ok(())
            },
            "advance" => self.advance(),
            "show-consultation" => {
                if self.ctx.runner.state().mere.open {
                    self.mere_step("show-mere")?;
                }
                self.ctx.runner.update(|state| {
                    crate::ui::screen::select_surface(
                        &mut state.consultation.surface_tabs,
                        crate::ui::screen::ConsultationScreen::Today,
                    )
                });
                Ok(())
            },
            "show-mere" => {
                if self.ctx.runner.state().mere.open {
                    Ok(())
                } else {
                    self.mere_step(line)
                }
            },
            _ if line.starts_with("mere-") => self.mere_step(line),
            "show-chart" => {
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    crate::ui::screen::select_surface(
                        &mut ui.surface_tabs,
                        crate::ui::screen::ConsultationScreen::Chart,
                    );
                });
                Ok(())
            },
            #[cfg(feature = "analytic-ephemeris")]
            _ if line.starts_with("calculate-chart ") => {
                let instant = line.trim_start_matches("calculate-chart ");
                let mut error = None;
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.astrology_instant_utc = cambium::TextInput::new(instant);
                    let action = ui.request_calculated_astrology_chart();
                    ui.record_action(action);
                    error = ui.error().map(str::to_string);
                });
                if let Some(error) = error {
                    return Err(error);
                }
                submit_pending(self.ctx.runner, self.state.worker.as_ref());
                Ok(())
            },
            _ => Err(format!("unknown Cleromancy scenario verb: {line}")),
        }
    }
}
