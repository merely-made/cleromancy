from pathlib import Path
import json,shutil
b=Path('/Users/markik/Code/testing/cleromancy/c8-20261016');checks=json.loads((b/'product-checks-final.json').read_text());assert len(checks)==6 and all(v==0 for v in checks.values())
p=b/'software-a50d316';p.mkdir()
for f in list(b.glob('product-*-final.log'))+[b/'product-checks-final.json']:shutil.move(str(f),p/f.name)
shutil.copy2(b/'source-inputs.json',b/'source-inputs-a50d316.json')
p=Path('Cargo.toml');s=p.read_text();s+='''
# Replay verifies saved typed decks and inference sources in desktop debug builds.
# Optimize those existing CPU paths without changing validation or stored bytes.
[profile.dev.package.cleromancy]
opt-level = 1
[profile.dev.package.cleromancy-core-acceptance]
opt-level = 1
[profile.dev.package.serde_json]
opt-level = 2
''';p.write_text(s)
p=Path('src/ui/state.rs');s=p.read_text();a='''    pub(crate) fn present_die(&mut self, catalog: ConsultationCatalog, digest: &str) {
        self.replace_catalog(catalog);''';assert a in s;s=s.replace(a,a+'''
        self.clear_selection();
        self.card_preview = None;''',1)
a='''    #[test]
    fn adding_decks_preserves_selected_field_identity() {''';assert a in s;s=s.replace(a,'''    #[test]
    fn choosing_die_opens_a_new_form_without_changing_the_previous_reading() {
        let mut c = crate::Consultation::new(crate::CleromancyHost::empty(muniment::MemoryBackend::new()));
        let field = pollster::block_on(c.install_builtin_tarot_at(1)).unwrap();
        let context = pollster::block_on(c.save_context_at(crate::ContextDraft::new("Previous occasion", "What deserves attention?", ""), 2)).unwrap();
        let old = pollster::block_on(c.read(&context, &field, crate::SelectionMode::Calculated)).unwrap();
        let die = pollster::block_on(c.install_six_sided_die()).unwrap();
        let catalog = c.catalog().unwrap();
        let mut ui = ConsultationUi::new(catalog.clone());
        ui.present_reading(catalog.clone(), old.clone());
        ui.card_preview = Some((field, "0".into()));
        ui.present_die(catalog, &die);
        assert!(ui.detail.is_none());
        assert!(ui.card_preview.is_none());
        assert_eq!(ui.catalog.fields[ui.field_select.selected].digest(), die);
        assert_eq!(ui.mode.selected, 1);
        assert_eq!(ui.layout.selected, 0);
        assert_eq!(c.detail(&old.session.id).unwrap(), old);
    }

'''+a,1);p.write_text(s)
p=Path('src/ui/native/scenario_driver.rs');s=p.read_text();a='            .or_else(|| line.strip_prefix("c7-control-"))';assert a in s;s=s.replace(a,a+'\n            .or_else(|| line.strip_prefix("c8-control-"))',1)
a='''            let node = {
                let dom = self.ctx.runner.dom();''';assert a in s;s=s.replace(a,'''            if line.starts_with("c8-control-") && key == "save-reflection" {
                let ui = &self.ctx.runner.state().consultation;
                let detail = ui.detail().ok_or("C8 reflection requires a selected die occasion")?;
                if detail.field.rules != crate::UNIFORM_DIE_RULE || !matches!(detail.context.label.as_str(), "C8 operating-system roll" | "C8 public-seed roll") {
                    return Err("C8 reflection refuses to write to an older non-die occasion".into());
                }
            }
'''+a,1)
a='''            "c8-prepare-reflection" => {
                self.ctx.runner.update(|state| {''';assert a in s;s=s.replace(a,'''            "c8-prepare-reflection" => {
                let detail = self.ctx.runner.state().consultation.detail().ok_or("C8 reflection has no current die result")?;
                if detail.field.rules != crate::UNIFORM_DIE_RULE {
                    return Err("C8 reflection requires the newly saved die result".into());
                }
                self.ctx.runner.update(|state| {''',1);p.write_text(s)
for mode in ['os','seed']:
 p=Path(f'receipts/c8-die-{mode}.scn');s=p.read_text();s=s.replace('c5-control-click read\n','c5-control-click read\nshow-mere\nsettle 8\n',1).replace('c5-control-reveal save-reflection','c8-control-reveal save-reflection').replace('c5-control-click save-reflection','c8-control-click save-reflection');p.write_text(s)
