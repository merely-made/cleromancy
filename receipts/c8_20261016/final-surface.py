from pathlib import Path
import json,shutil
b=Path('/Users/markik/Code/testing/cleromancy/c8-20261016')
checks=json.loads((b/'product-checks-final.json').read_text());assert len(checks)==6 and all(v==0 for v in checks.values()),checks
p=b/'software-4adc154';p.mkdir()
for f in list(b.glob('product-*-final.log'))+[b/'product-checks-final.json']:shutil.move(str(f),p/f.name)
shutil.copy2(b/'source-inputs.json',b/'source-inputs-4adc154.json')
p=Path('src/ui/state.rs');s=p.read_text();a='''        self.replace_catalog(catalog);
        self.clear_selection();
        self.card_preview = None;
        self.field_select.selected = self
            .catalog
            .fields
            .iter()
            .position(|f| f.digest() == digest)
            .expect("installed die in catalog");''';assert a in s;s=s.replace(a,'''        self.replace_catalog(catalog);
        // Another admitted writer may withdraw an unused field after the ACK.
        let Some(index) = self.catalog.fields.iter().position(|f| f.digest() == digest) else {
            self.present_error("The die is no longer available in this session. Add it again before rolling.");
            return;
        };
        self.clear_selection();
        self.card_preview = None;
        self.error = None;
        self.field_select.selected = index;''',1)
a='''        let die = pollster::block_on(c.install_six_sided_die()).unwrap();
        let catalog = c.catalog().unwrap();''';assert a in s;s=s.replace(a,'''        let previous_catalog = c.catalog().unwrap();
        let die = pollster::block_on(c.install_six_sided_die()).unwrap();
        let catalog = c.catalog().unwrap();''',1)
a='''        ui.card_preview = Some((field, "0".into()));
        ui.present_die(catalog, &die);''';assert a in s;s=s.replace(a,'''        ui.card_preview = Some((field, "0".into()));
        ui.present_die(previous_catalog, &die);
        assert_eq!(ui.detail.as_ref(), Some(&old));
        assert!(ui.error.as_ref().unwrap().contains("no longer available"));
        ui.present_die(catalog, &die);
        assert!(ui.error.is_none());''',1);p.write_text(s)
p=Path('src/ui/view/reading.rs');s=p.read_text();s=s.replace('children.push(narrative_view(&narrative, &focused));','children.push(narrative_view(&narrative, &focused, if detail.field.rules == crate::UNIFORM_DIE_RULE { "Face" } else { "Card" }));',1);s=s.replace('fn narrative_view(narrative: &ReadingNarrative, focused: &str)', 'fn narrative_view(narrative: &ReadingNarrative, focused: &str, noun: &str)',1);s=s.replace('"Card {} of {} · {} · {}",','"{noun} {} of {} · {} · {}",',1);p.write_text(s)
p=Path('src/ui/view/shared.rs');s=p.read_text();s=s.replace('fn session_row(summary: &SessionSummary, now_ms: u64)', 'fn session_row(summary: &SessionSummary, now_ms: u64, fields: &[crate::Field])',1)
a='''    let placement_text = format!(
        "{placement_count} placement/card{}",
        plural(placement_count)
    );''';assert a in s;s=s.replace(a,'''    let die = fields.iter().any(|f| f.rules == crate::UNIFORM_DIE_RULE && f.digest() == summary.field_digest);
    let placement_text = if die {
        format!("{placement_count} outcome{}", plural(placement_count))
    } else {
        format!("{placement_count} placement/card{}", plural(placement_count))
    };''',1)
a='''        "Open session {}. Context: {}. Date: {}. {}. Cards: {}. Modes: {}.",''';assert a in s;s=s.replace(a,'''        "Open session {}. Context: {}. Date: {}. {}. {}: {}. Modes: {}.",''',1);s=s.replace('''        placement_text,
        card_text,''','''        placement_text,
        if die { "Outcomes" } else { "Cards" },
        card_text,''',1);p.write_text(s)
for name in ['trail','journal']:
 p=Path(f'src/ui/view/{name}.rs');s=p.read_text();a='session_row(summary, ui.journal.now_ms)';assert a in s;s=s.replace(a,'session_row(summary, ui.journal.now_ms, &ui.catalog.fields)');p.write_text(s)
p=Path('src/ui/view/consultation.rs');s=p.read_text();a='    let context_example = ui.detail.as_ref()';assert a in s;s=s.replace(a,'''    let context_example = if die {
        "A question to riff on: What am I exploring with this roll? Add the setting or question you want to remember alongside its outcome.".to_string()
    } else { ui.detail.as_ref()''',1)
a='''Add a concrete detail, such as a decision at work or a habit you are reconsidering.".into());''';assert a in s;s=s.replace(a,a+' };',1)
a='''                "Calculated follows the highest disclosed qualified weight.''';assert a in s;s=s.replace(a,'''                if die { "The saved question and situation stay with this roll. You can add a reflection after seeing its outcome. Chosen chart facts are saved alongside the roll." } else { "Calculated follows the highest disclosed qualified weight.''',1)
a='''Chosen chart facts are saved alongside the reading.",''';assert a in s;s=s.replace(a,'''Chosen chart facts are saved alongside the reading." },''',1);p.write_text(s)
p=Path('tests/dice.rs');s=p.read_text();a='''        assert!(!text.contains("upright"));''';assert a in s;s=s.replace(a,a+'''
        let narration = support::text_at_key(&h, "reading-narrative");
        assert!(narration.contains("Face 1 of 1"));
        assert!(!narration.contains("Card 1"));
        assert!(support::text_at_key(&h, "region:trail").contains("1 outcome"));''',1);p.write_text(s)
