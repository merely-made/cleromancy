from pathlib import Path
p=Path('src/ui/resident.rs');s=p.read_text();a='            #[cfg(feature = "analytic-ephemeris")]\n            WorkerUpdate::AstrologyChart { facts_digest, .. } => {';assert a in s;s=s.replace(a,'''            WorkerUpdate::Die { field_digest, .. } => Some(WorkerUpdate::Die {
                catalog: catalog.clone(),
                field_digest,
            }),
'''+a,1)
a='    fn envelope() -> session_item::SessionGraphV1 {';s=s.replace(a,'''    #[test]
    fn installed_die_survives_owner_refresh_with_authoritative_catalog() {
        let mut c = Consultation::new(CleromancyHost::empty(MemoryBackend::new()));
        let digest = pollster::block_on(c.install_six_sided_die()).unwrap();
        let stale = Consultation::new(CleromancyHost::empty(MemoryBackend::new())).catalog().unwrap();
        match owner_update(&c, WorkerUpdate::Die { catalog: stale, field_digest: digest.clone() }).unwrap() {
            WorkerUpdate::Die { catalog, field_digest } => {
                assert_eq!(field_digest, digest);
                assert_eq!(catalog.fields.len(), 1);
                assert_eq!(catalog.fields[0].digest(), digest);
            }, other => panic!("installed field selection was lost: {other:?}"),
        }
    }

'''+a,1);p.write_text(s)
p=Path('src/ui/native/scenario_driver.rs');s=p.read_text();a='            "show-mere" => {\n                if self.ctx.runner.state().mere.open {';assert a in s;s=s.replace(a,'''            "show-consultation" => {
                if self.ctx.runner.state().mere.open {
                    self.mere_step("show-mere")?;
                }
                self.ctx.runner.update(|state| crate::ui::screen::select_surface(&mut state.consultation.surface_tabs, crate::ui::screen::ConsultationScreen::Today));
                Ok(())
            },
'''+a,1)
a='                let seeded = line == "c8-prepare-seed";\n                self.ctx.runner.update';s=s.replace(a,'''                let seeded = line == "c8-prepare-seed";
                let field_index = self.ctx.runner.state().consultation.catalog.fields.iter().position(|f| f.system == "cleromancy.die/d6").ok_or("install die through consultation form before preparing roll")?;
                self.ctx.runner.update''',1)
a='''                    ui.field_select.selected = ui
                        .catalog
                        .fields
                        .iter()
                        .position(|f| f.system == "cleromancy.die/d6")
                        .expect("install die before preparing roll");''';assert a in s;s=s.replace(a,'                    ui.field_select.selected = field_index;',1)
a='            "c8-prepare-os" | "c8-prepare-seed" => {';s=s.replace(a,'''            "c8-prepare-reflection" => {
                self.ctx.runner.update(|state| {
                    state.consultation.reflection = cambium::TextInput::new("Public dice fixture: I recorded this outcome without assigning it a symbolic meaning.");
                });
                Ok(())
            },
'''+a,1);p.write_text(s)
for mode in ['os','seed']:
 p=Path(f'receipts/c8-die-{mode}.scn');s=p.read_text().replace('show-mere','show-consultation',1);s=s.replace('assert text Outcome\n', '''assert text Outcome
c8-prepare-reflection
c5-control-reveal save-reflection
settle 4
c5-control-click save-reflection
wait 1800
assert snap product-busy == false
assert snap reflections == 1
show-mere
settle 8
''',1);p.write_text(s)
