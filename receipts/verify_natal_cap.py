#!/usr/bin/env python3
"""Read-only verification of frozen natal cap evidence. Does not run the app."""
from pathlib import Path
import hashlib,json,re,subprocess
repo=Path(__file__).resolve().parents[1]
pack=repo/'receipts/natal_cap_20261017'
ledger=json.loads((repo/'receipts/natal-cap-qualification_20261017.json').read_text())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for name,h in ledger['artifacts'].items():assert digest(pack/name)==h,name
build=json.loads((pack/'native-build.json').read_text());assert build['source']==ledger['consumer_source']
for name,h in build['recorded_local_depfile_inputs'].items():
 raw=subprocess.check_output(['git','show',build['source']+':'+name],cwd=repo)
 assert hashlib.sha256(raw).hexdigest()==h,name
for name,h in build['files'].items():
 raw=subprocess.check_output(['git','show',build['source']+':'+name],cwd=repo)
 assert hashlib.sha256(raw).hexdigest()==h,name
native=json.loads((pack/'native-qualification.json').read_text());assert native['accepted_originals_reviewed']==22
runs=[]
for name in native['runs']:
 r=json.loads((pack/name/'receipt.json').read_text());p=json.loads((pack/name/'process.json').read_text())
 assert r['ok'] and not r['error'] and p['exit']==0 and not p['timeout'] and p['prompts']==1 and p['offline_ip_deny']
 assert p['binary_sha256']==build['binary_sha256'];assert not any('FAIL' in x for x in r['log']);runs.append(r)
assert runs[1]['readings']==runs[2]['readings'] and runs[1]['reflections']==runs[2]['reflections']
assert runs[3]['readings']==runs[4]['readings'] and runs[3]['reflections']==runs[4]['reflections']
assert all(x in runs[3]['readings'] for x in runs[1]['readings'])
assert len(runs[4]['readings'])==4 and len(runs[4]['reflections'])==2
assert len(runs[4]['shared']['associations'])==8
for reading in runs[4]['readings']:
 assert any(a['subject']==reading['birth']['facts']['subject'] and a['resource']=={'domain':'divination','id':'cleromancy://natal/'+reading['id'],'revision':reading['id']} for a in runs[4]['shared']['associations'])
for field in ['readings','reflections','shared']:assert json.loads((pack/('exact-'+field+'.json')).read_text())==runs[4][field]
reviews=json.loads((pack/'capture-reviews.json').read_text());assert len(reviews)==22
for v in reviews:assert digest(pack/v['path'])==v['sha256']
g=json.loads((pack/'prior-guard.json').read_text())
assert len(g['cleromancy']['files'])==1046
for name,h in g['cleromancy']['files'].items():assert digest(repo/name)==h,name
supplier=repo.parent/'turquet'
if supplier.exists():
 for name,h in g['turquet']['files'].items():assert digest(supplier/name)==h,name
software=json.loads((pack/'software/results.json').read_text());assert len(software)==10
for name,v in software.items():
 assert v['exit']==0 and v['source']==ledger['consumer_source'],name
 if name.startswith(('test-','core-')):
  s=(pack/'software'/(name+'.log')).read_text();assert 'test result: FAILED' not in s and 'test result: ok.' in s,name
  count=sum(map(int,re.findall(r'test result: ok\. (\d+) passed',s)))
  assert count==ledger['software_counts'][name],(name,count)
owner=json.loads((pack/'mere-integrated-qualification.json').read_text());assert owner['exit']==0 and owner['passed']==225 and owner['consumer']==ledger['consumer_source']
subprocess.run(['python3',str(repo/'receipts/verify_natal_houses.py')],check=True)
print('Natal cap verified: immutable source, ten software gates, 22 reviewed originals, two staged exact fresh reopenings, observed/polar outcomes, shared-owner qualification and earlier evidence intact.')
