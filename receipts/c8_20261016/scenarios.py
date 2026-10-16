from pathlib import Path
p=Path('src/ui/native/scenario_driver.rs');s=p.read_text()
s=s.replace('            let key = if key == "inference-keep-model" {','''            let key = if key == "c8-saved-os" || key == "c8-saved-seed" {
                let label = if key == "c8-saved-os" { "C8 operating-system roll" } else { "C8 public-seed roll" };
                let summary = self.ctx.runner.state().consultation.catalog.session_summaries.iter().find(|s| s.context_label == label).ok_or("no saved C8 occasion")?;
                format!("session:{}", summary.session_id)
            } else if key == "inference-keep-model" {''',1)
s=s.replace('            "c6-prepare-global" | "c6-prepare-weather" => {','''            "c8-prepare-os" | "c8-prepare-seed" => {
                let seeded = line == "c8-prepare-seed";
                self.ctx.runner.update(|state| {
                    let ui = &mut state.consultation;
                    ui.field_select.selected = ui.catalog.fields.iter().position(|f| f.system == "cleromancy.die/d6").expect("install die before preparing roll");
                    ui.context_select.selected = 0;
                    ui.context_label = cambium::TextInput::new(if seeded { "C8 public-seed roll" } else { "C8 operating-system roll" });
                    ui.question = cambium::TextInput::new("Which face appears on this roll?");
                    ui.tags = cambium::TextInput::default();
                    ui.additional_facts = cambium::TextInput::default();
                    ui.layout.selected = 0;
                    ui.mode.selected = if seeded {2} else {1};
                    ui.derived_seed = cambium::TextInput::new("C8 public die");
                    ui.derived_domain = cambium::TextInput::new("cleromancy.c8/qualification");
                    ui.astrology_facts_select.selected = 0;
                    ui.set_place_for_draw(None);
                    ui.set_weather_for_draw(false);
                });
                self.state.scenario_stage = 1;
                Ok(())
            },
            "c6-prepare-global" | "c6-prepare-weather" => {''',1)
s=s.replace('            if !detail.inferences.is_empty() {','''            if detail.field.rules == crate::UNIFORM_DIE_RULE {
                std::fs::write(self.dir.join("die-reading.json"), serde_json::to_vec_pretty(detail).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            }
            if !detail.inferences.is_empty() {''',1)
s=s.replace('                .with_field("inference-count", detail.inferences.len().to_string())','''                .with_field("die", (detail.field.rules == crate::UNIFORM_DIE_RULE).to_string())
                .with_field("die-seeded", detail.readings.first().is_some_and(|r| r.receipt.mode == crate::SelectionMode::Derived).to_string())
                .with_field("inference-count", detail.inferences.len().to_string())''',1)
p.write_text(s)
for mode in ['os','seed']:
 body=f'''wait 1800
assert snap catalog-ready == true
show-mere
wait 1800
settle 8
c5-control-reveal install-six-sided-die
settle 4
c5-control-click install-six-sided-die
wait 1800
assert snap product-busy == false
c8-prepare-{mode}
c5-control-reveal die-{'os' if mode=='os' else 'seed'}
settle 4
c5-control-click die-{'os' if mode=='os' else 'seed'}
reveal key die-method
settle 4
capture die-method
c5-control-reveal read
settle 4
c5-control-click read
wait 1800
assert snap product-busy == false
assert snap die == true
assert snap die-seeded == {'false' if mode=='os' else 'true'}
assert snap readings == 1
assert text Outcome
assert text 6 faces
reveal class reading-narrative
settle 4
capture die-narrative
resize 420 760
wait
reveal class reading-narrative
settle 4
capture compact-die-narrative
'''
 Path(f'receipts/c8-die-{mode}.scn').write_text(body)
 body=f'''wait 1800
assert snap catalog-ready == true
show-mere
wait 1800
settle 8
c5-control-reveal c8-saved-{mode}
settle 4
c5-control-click c8-saved-{mode}
wait 1800
assert snap product-busy == false
assert snap die == true
assert snap die-seeded == {'false' if mode=='os' else 'true'}
assert snap readings == 1
assert text Outcome
reveal class reading-narrative
settle 4
capture reopened-die-narrative
resize 420 760
wait
reveal class reading-narrative
settle 4
capture compact-reopened-die-narrative
'''
 Path(f'receipts/c8-die-{mode}-reopen.scn').write_text(body)
