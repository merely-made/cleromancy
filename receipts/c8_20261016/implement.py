from pathlib import Path
r=Path('/Users/markik/Code/repos/cleromancy')
def edit(p,a,b):
 p=r/p;s=p.read_text();assert a in s,(p,a);p.write_text(s.replace(a,b,1))
edit('src/ui/action.rs','    InstallTarotDecks,','    InstallTarotDecks,\n    InstallSixSidedDie,')
edit('src/consultation/mod.rs','    pub async fn save_context(&mut self, draft: ContextDraft)', '''    /// Add the first dice surface using the existing uniform roll field and replay rules.
    pub async fn install_six_sided_die(&mut self) -> Result<String, ConsultationError> {
        self.ensure_writable()?;
        let mut field = crate::RollIntentPayload::new(6).with_label("Six-sided die").field();
        field.position_meanings = Some(serde_json::from_str(include_str!("../content/die-position-v1.json"))
            .expect("bundled die position"));
        let digest = field.digest();
        self.host.insert_field(&field)?;
        self.persist(unix_time_secs()?).await?;
        Ok(digest)
    }

    pub async fn save_context(&mut self, draft: ContextDraft)''')
(r/'src/content/die-position-v1.json').write_text('''{
  "schema": "cleromancy.position-meanings/v1",
  "id": "cleromancy.die.single",
  "version": "1",
  "author": "Cleromancy",
  "date": "2026-10-16",
  "positions": [{"name": "focus", "label": "Outcome", "meaning": "The face selected by this single roll. Each face has equal weight; your question does not change its probability. No symbolic meaning is assigned to the number."}]
}
''')
edit('src/ui/worker.rs','    Catalog(ConsultationCatalog),','    Catalog(ConsultationCatalog),\n    Die { catalog: ConsultationCatalog, field_digest: String },')
edit('src/ui/worker.rs','        ConsultationAction::InstallTarotDecks => {','''        ConsultationAction::InstallSixSidedDie => {
            let field_digest = pollster::block_on(consultation.install_six_sided_die())?;
            Ok(WorkerUpdate::Die { catalog: consultation.catalog()?, field_digest })
        },
        ConsultationAction::InstallTarotDecks => {''')
edit('src/ui/native/mod.rs','        WorkerUpdate::Catalog(catalog) => {','''        WorkerUpdate::Die { catalog, field_digest } => {
            state.catalog_ready = true;
            runner.update(|state| state.consultation.present_die(catalog, &field_digest));
        },
        WorkerUpdate::Catalog(catalog) => {''')
edit('src/ui/state.rs','    pub(super) fn request_typed_decks(&mut self)', '''    pub(crate) fn present_die(&mut self, catalog: ConsultationCatalog, digest: &str) {
        self.replace_catalog(catalog);
        self.field_select.selected = self.catalog.fields.iter().position(|f| f.digest() == digest).expect("installed die in catalog");
        self.mode.selected = 1;
        self.layout.selected = 0;
        self.status = ConsultationStatus::Ready;
    }

    pub(super) fn request_die(&mut self) -> Option<ConsultationAction> {
        if self.status.is_busy() { return None; }
        self.error = None;
        self.status = ConsultationStatus::SavingSetup;
        Some(ConsultationAction::InstallSixSidedDie)
    }

    pub(super) fn request_typed_decks(&mut self)''')
edit('src/ui/state.rs','        let context = if self.context_select.selected == 0 {','''        if field.rules == crate::UNIFORM_DIE_RULE && (self.mode.selected == 0 || self.layout.selected != 0) {
            self.present_error("Choose operating-system randomness or a public seed, and a single outcome, for this die roll.");
            return None;
        }
        let context = if self.context_select.selected == 0 {''')
edit('src/narrative.rs','    } else if names == ["focus"] {','''    } else if names == ["focus"] && detail.field.rules == UNIFORM_DIE_RULE {
        "a single die roll".to_string()
    } else if names == ["focus"] {''')
edit('src/narrative.rs','        } else if field.system == RWS_MAJOR_ARCANA_ID {','''        } else if field.rules == UNIFORM_DIE_RULE {
            format!("a {}-sided die", field.candidates.len())
        } else if field.system == RWS_MAJOR_ARCANA_ID {''')
edit('src/ui/view/reading.rs','            children.push(super::inference::generated(ui, detail));\n            children.push(super::ambient::neighborhood(ui));','''            if detail.field.rules != crate::UNIFORM_DIE_RULE {
                children.push(super::inference::generated(ui, detail));
                children.push(super::ambient::neighborhood(ui));
            }''')
edit('src/ui/view/consultation.rs','    let mode = map_action(','    let die = ui.catalog.fields.get(ui.field_select.selected).is_some_and(|f| f.rules == crate::UNIFORM_DIE_RULE);\n    let mode = map_action(')
edit('src/ui/view/consultation.rs','        labelled_control("Selection mode", "cleromancy-mode", Box::new(mode)),\n        labelled_control("Reading shape", "cleromancy-layout", Box::new(layout)),','''        Box::new(button("Add a six-sided die", |ui: &mut ConsultationUi, _| {
            let action = ui.request_die(); ui.record_action(action);
        }).attr("data-key", "install-six-sided-die")),''')
edit('src/ui/view/consultation.rs','    let context_example =','''    if die {
        for (label, key, selected) in [("Use operating-system randomness", "die-os", 1), ("Use a public seed", "die-seed", 2)] {
            children.push(Box::new(button(label, move |ui: &mut ConsultationUi, _| {
                ui.mode.selected = selected;
                ui.layout.selected = 0;
            }).attr("data-key", key).attr("aria-pressed", if ui.mode.selected == selected {"true"} else {"false"})));
        }
        children.push(Box::new(el::<_, ConsultationUi, ()>("p", "One roll, one outcome. All six faces have equal weight. Separate rolls may repeat a face. A public seed and domain reproduce the same result with the same saved context. There is no orientation or assigned symbolic meaning.").attr("data-key", "die-method")));
    } else {
        children.push(labelled_control("Selection mode", "cleromancy-mode", Box::new(mode)));
        children.push(labelled_control("Reading shape", "cleromancy-layout", Box::new(layout)));
    }
    let context_example =''')
edit('src/ui/view/consultation.rs','    if ui.layout.selected == 2 {','    if !die && ui.layout.selected == 2 {')
edit('src/ui/view/consultation.rs','        button("Read", |ui:', '        button(if die { "Roll the die" } else { "Read" }, |ui:')
edit('src/ui/view/consultation.rs','    } else if field.system == RWS_MAJOR_ARCANA_ID {','    } else if field.rules == crate::UNIFORM_DIE_RULE {\n        format!("{}-sided die", field.candidates.len())\n    } else if field.system == RWS_MAJOR_ARCANA_ID {')
