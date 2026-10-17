from pathlib import Path
import hashlib,json,struct
b=Path(__file__).resolve().parent
old=json.loads(Path('/Users/markik/Code/testing/cleromancy/natal-cap-20261017/native-reopen-final-01/receipt.json').read_text())
create=json.loads((b/'native-create-01/receipt.json').read_text())
phases=['native-create-01','native-reopen-01','installed-reopen-01']
original={r['id']:r for r in old['readings']}; frozen={r['id']:r for r in create['readings']}
assert len(original)==4 and len(frozen)==6
reviews={
 'aspect-choice-and-orbs.png':'All five editable degree inputs and explicit pressed choice readable; preset disclosed. Ordinary window; dimensions recorded from the original PNG.',
 'aspects-preset-wide.png':'All ten retained aspect pairs, separation/deviation/orb disclosure readable; lower motion section continues by scrolling.',
 'aspects-preset-compact.png':'All ten aspect pairs wrap at compact 420 logical width without horizontal clipping; motion section continues below viewport.',
 'aspects-exact-zero.png':'All five zero orbs displayed, honest no-match result, earlier motion facts and reflection controls readable.',
 'reopened-aspects-wide.png':'Saved aspect receipt presented after fresh encrypted offline startup; text remains legible.',
 'reopened-aspects-compact.png':'Saved aspect receipt wraps at compact width; subsequent content remains scrollable.',
 'reopened-shared-subject.png':'Shared subject/birth controls and saved revision catalog reopened; only visible subset claimed.'
}
summary=[];captures=[]
for phase in phases:
 run=b/phase; p=json.loads((run/'process.json').read_text());r=json.loads((run/'receipt.json').read_text())
 assert p['exit']==0 and not p['timeout'] and p['offline_ip_deny'] and p['prompts']==1
 assert r['ok'] and r['error'] is None
 readings={v['id']:v for v in r['readings']}; assert readings==frozen
 assert all(readings[k]==v for k,v in original.items())
 assert r['reflections']==old['reflections']
 if phase!='native-create-01':
  assert r['shared'] is not None
  assert r['shared']['births']==old['shared']['births']
  assert r['shared']['subjects']==old['shared']['subjects']
  previous=old['shared']['associations']; assert all(a in r['shared']['associations'] for a in previous)
 for f in sorted(run.glob('*.png')):
  data=f.read_bytes();w,h=struct.unpack('>II',data[16:24]);assert f.name in reviews
  captures.append({'path':str(f.relative_to(b)),'sha256':hashlib.sha256(data).hexdigest(),'width':w,'height':h,'review':reviews[f.name]})
 summary.append({'run':phase,'process':p,'readings':len(readings),'reflections':len(r['reflections'])})
assert len(captures)==10
result={'ok':True,'source':'3fa5a6db87cfa34df9fc1de0a0877181c9b4526f','runs':summary,'captures':captures,'exact_retained_readings':6,'preserved_original_readings':4,'preserved_reflections':2,'unlock':'production terminal fallback; graphical lane remains separate','execution':'packaged arm64 payload; final run relocated from extracted archive; shell launcher remains separately unqualified'}
(b/'native-qualification.json').write_text(json.dumps(result,indent=2)+'\n');print('Native aspect receipts verified:',len(summary),'fresh processes,',len(captures),'original captures; original values and shared resources intact.')
