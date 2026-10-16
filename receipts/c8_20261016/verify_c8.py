#!/usr/bin/env python3
"""Read-only verification of frozen first-die evidence, not future source recertification."""
from pathlib import Path
import json,hashlib,subprocess
r=Path(__file__).resolve().parent.parent
b=r/'receipts/c8_20261016'
def read(p): return json.loads(p.read_text())
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
for name,digest in read(b/'artifact-hashes.json').items(): assert sha(b/name)==digest,name
for name,digest in read(b/'prior-artifacts.json').items(): assert sha(r/name)==digest,name
manifest=read(b/'source-inputs.json')
for row in manifest['files']:
 data=subprocess.check_output(['git','show',manifest['source']+':'+row['path']],cwd=r)
 assert hashlib.sha256(data).hexdigest()==row['sha256'],row['path']
bridge=read(b/'source-bridge.json')
assert manifest['source']==bridge['native_source']
changed=subprocess.check_output(['git','diff','--name-only',bridge['software_source'],bridge['native_source']],cwd=r,text=True).splitlines()
assert changed==bridge['only_changes']
assert all(name.endswith('.scn') for name in changed)
build=read(b/'native-build-tuned.json');assert build['source']==manifest['source']
checks=read(b/'product-checks-final.json');assert len(checks)==6 and all(v==0 for v in checks.values())
for name in ['default','legacy','sky']:
 log=(b/('core-'+name+'-final.log')).read_text();assert 'test result: FAILED' not in log and 'test result: ok.' in log
reviews=read(b/'capture-reviews.json');reviewed={v['path'] for v in reviews}
captures=[]
runs={'os':{'create':'native-os-create-04','reopen':'native-os-reopen-01'},'seed':{'create':'native-seed-create-01','reopen':'native-seed-reopen-01'}}
for mode in ['os','seed']:
 for stage in ['create','reopen']:
  d=b/runs[mode][stage]
  p=read(d/'process.json');assert p['exit']==0 and not p['timeout'] and p['offline_ip_deny']
  assert read(d/'receipt.json')['ok']
  assert p['binary_sha256']==build['signed_binary_sha256']
  scene={'native-os-create-04':'c8-die-os.scn','native-os-reopen-01':'c8-die-os-reopen.scn','native-seed-create-01':'c8-die-seed.scn','native-seed-reopen-01':'c8-die-seed-reopen.scn'}[d.name]
  assert p['scenario_sha256']==sha(b/scene)
  for image in d.glob('*.png'):
   rel=str(image.relative_to(b));assert rel in reviewed,rel;captures.append(rel)
 a=read(b/runs[mode]['create']/'die-reading.json')
 c=read(b/runs[mode]['reopen']/'die-reading.json')
 assert (b/runs[mode]['create']/'die-reading.json').read_bytes()==(b/runs[mode]['reopen']/'die-reading.json').read_bytes()
 assert a==c
 assert len(a['readings'])==1 and a['field']['system']=='cleromancy.die/d6'
 reading=a['readings'][0];assert 1<=int(reading['candidate_id'])<=6
 assert not reading['receipt'].get('shuffle')
 assert reading['receipt']['mode']==('cast' if mode=='os' else 'derived')
 assert a['situation'] and not a['situation'].get('place')
 assert not a.get('inferences')
before=read(b/'domain-before.json');after=read(b/'domain-after.json')
old={d['session']['id']:d for d in before['details']};new={d['session']['id']:d for d in after['details']}
assert len(new)==len(old)+2
assert all(new[k]==d for k,d in old.items())
assert before['views']==after['views'] and before['memory']==after['memory']
assert len(captures)==14 and len(reviewed)==14
print('C8 first die verified: fourteen reviewed originals, two exact fresh-process reopenings, all prior occasions and C4 view/memory preserved; prior receipts intact.')
