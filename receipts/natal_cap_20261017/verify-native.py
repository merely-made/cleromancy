from pathlib import Path
import json,hashlib,struct
b=Path(__file__).resolve().parent
names=['native-setup-01','native-create-01','native-reopen-01','native-polar-01','native-reopen-final-01'];runs=[];reviews=[]
build=json.loads((b/'native-build.json').read_text())
for name in names:
 r=json.loads((b/name/'receipt.json').read_text());p=json.loads((b/name/'process.json').read_text())
 assert r['ok'] and not r['error'] and p['exit']==0 and not p['timeout'] and p['prompts']==1 and p['offline_ip_deny'] and p['binary_sha256']==build['binary_sha256']
 assert not any('FAIL' in x for x in r['log']);runs.append(r)
 for path in sorted((b/name).glob('*.png')):
  dims=struct.unpack('>II',path.read_bytes()[16:24]);assert dims in [(2320,1520),(840,1520)]
  note='Reviewed original: readable selected facts; lower content continues by scrolling.'
  if 'polar-unavailable' in path.name:note='Placidus unavailable explained with no alternative system substituted; wide/compact text and reflection controls readable.'
  elif 'motion' in path.name:note='Signed rates and direct/retrograde labels readable, including Saturn retrograde; near-station sampling/rounding policy readable. Observed captures retain explicit DUT1 and polar-motion values.'
  elif 'placidus' in path.name:note='Placidus ascendant/MC and unequal cusp rows readable. Observed captures disclose retained source hash and coverage. Compact text and hashes wrap; lower rows scroll.'
  elif 'provenance' in path.name:note='Supplement method and immutable supplier identity readable with original edition distinction; source text and hash retained in the receipt.'
  elif 'shared' in path.name:note='Fresh shared facts authoring controls visible; exact saved resource values verified separately. This crop does not show every selector at once.'
  elif 'missing-place' in path.name:note='Missing coordinates explained without invented houses; motion remains available. Motion capture includes Saturn retrograde.'
  reviews.append(dict(path=str(path.relative_to(b)),sha256=hashlib.sha256(path.read_bytes()).hexdigest(),physical_pixels=dims,review='pass within stated scope',notes=note))
a,c,r,p,f=runs
assert len(reviews)==22
assert c['readings']==r['readings'] and c['reflections']==r['reflections']
assert all(v in c['readings'] for v in a['readings']) and all(v in c['reflections'] for v in a['reflections'])
assert all(v in p['readings'] for v in c['readings']) and p['reflections']==c['reflections']
assert p['readings']==f['readings'] and p['reflections']==f['reflections']
assert len(f['readings'])==4 and len(f['reflections'])==2
assert len(f['shared']['births'])==3 and len(f['shared']['subjects'])==1
assert len(f['shared']['associations'])==8
for reading in f['readings']:
 assert any(a['subject']==reading['birth']['facts']['subject'] and a['resource']=={'domain':'divination','id':'cleromancy://natal/'+reading['id'],'revision':reading['id']} for a in f['shared']['associations'])
assert c['shared']['births']==r['shared']['births']==p['shared']['births']==f['shared']['births']
assert c['shared']['subjects']==r['shared']['subjects']==p['shared']['subjects']==f['shared']['subjects']
observed=[v for v in f['readings'] if v['supplement']['observed_values'] is not None];assert len(observed)==1
s=observed[0]['supplement'];assert s['orientation']['finals2000a']==Path('/Users/markik/Code/repos/cleromancy/tests/fixtures/natal-observed-finals2000a.txt').read_text()
assert s['observed_values']=={'dut1_microseconds':355066,'xp_microarcseconds':43335,'yp_microarcseconds':377605}
assert sum(v['supplement']['houses']['kind']=='calculated' for v in f['readings'])==2
polar=[v for v in f['readings'] if (v['birth']['facts'].get('place') or {}).get('latitude_microdegrees')==70000000];assert len(polar)==1 and polar[0]['supplement']['houses']['kind']=='unavailable'
for v in f['readings']:
 s=v['supplement'];assert s['supplier'].endswith(build['turquet_cap']) and s['motion']['kind']=='calculated' and len(s['motion']['planets'])==10
 assert v['birth'] in f['shared']['births']
 saturn=next(m for m in s['motion']['planets'] if m['body']=='saturn');assert saturn['classification']=='retrograde' and saturn['microdegrees_per_tt_day']<0
for field in ['readings','reflections','shared']:(b/('exact-'+field+'.json')).write_text(json.dumps(f[field],sort_keys=True,separators=(',',':'))+'\n')
(b/'capture-reviews.json').write_text(json.dumps(reviews,indent=2)+'\n')
record=dict(source=build['source'],binary_sha256=build['binary_sha256'],runs=names,accepted_originals_reviewed=22,exact_readings_and_reflections_after_two_staged_fresh_reopenings=True,shared_births_and_subjects_exact=True,all_four_natal_subject_resource_associations_present=True,bounds=['Public synthetic fixture; real encrypted terminal unlock','Outbound IP blocked in all five processes; no model configured','Creation UI shared-association cache is not a full fresh shared catalog; only saved birth/subject values and staged reading/reflection exactness are claimed','No full-database byte equality, performance, graphical unlock, browser wall or release renewal claim','Some captures begin at section/viewport edge and show lower rows by scrolling'])
(b/'native-qualification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record,indent=2))
