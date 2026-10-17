#!/usr/bin/env python3
"""Read-only verification of frozen natal-house evidence; no current-head renewal."""
from pathlib import Path
import json,hashlib,re,subprocess
r=Path(__file__).resolve().parents[1];l=json.loads((r/'receipts/natal-houses-qualification_20261017.json').read_text());p=r/'receipts'/l['pack'];sha=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
for name,digest in json.loads((p/'artifact-hashes.json').read_text()).items():assert sha(p/name)==digest,name
assert len(l['software'])==10 and all(c['exit']==0 and c['source']==l['source']for c in l['software'].values())
for name,expected in l['test_counts'].items():
 rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',(p/'software'/(name+'.log')).read_text());assert rows and all(x[0]=='ok'and x[2]=='0'for x in rows)
 assert expected=={'passed':sum(int(x[1])for x in rows),'failed':0,'ignored':sum(int(x[3])for x in rows)}
b=json.loads((p/'native-build.json').read_text());assert b['source']==l['source'] and b['turquet_houses']==l['turquet_houses']
for name,digest in b['files'].items():assert hashlib.sha256(subprocess.check_output(['git','show',l['source']+':'+name],cwd=r)).hexdigest()==digest,name
reviews=json.loads((p/'capture-reviews.json').read_text());assert len(reviews)==13
for item in reviews:assert sha(p/item['path'])==item['sha256'] and item['review']=='pass within stated scope'
a=[]
for name in l['native']['runs']:
 v=json.loads((p/name/'receipt.json').read_text());q=json.loads((p/name/'process.json').read_text());assert v['ok'] and not v['error'] and q['exit']==0 and not q['timeout'] and q['prompts']==1 and q['offline_ip_deny'] and q['binary_sha256']==b['binary_sha256'];assert not any('FAIL' in x for x in v['log']);a.append(v)
setup,created,reopened,catalog=a
assert created['readings']==reopened['readings']==catalog['readings'] and created['reflections']==reopened['reflections']==catalog['reflections']
assert reopened['shared']==catalog['shared'] and created['shared']['births']==reopened['shared']['births'] and created['shared']['subjects']==reopened['shared']['subjects']
assert setup['readings'][0] in created['readings'] and setup['reflections'][0] in created['reflections']
assert len(created['readings'])==3 and len(created['reflections'])==2 and len(reopened['shared']['associations'])==6
by={x['houses']['system']:x for x in created['readings'] if x['houses']['outcome']['kind']=='calculated'};assert set(by)=={'whole_sign','equal'}
w=by['whole_sign']['houses']['outcome'];e=by['equal']['houses']['outcome'];assert w['ascendant_microdegrees']==e['ascendant_microdegrees'] and w['midheaven_microdegrees']==e['midheaven_microdegrees'];assert e['cusps_microdegrees'][0]==e['ascendant_microdegrees'] and w['cusps_microdegrees'][0]%30_000_000==0 and w['cusps_microdegrees']!=e['cusps_microdegrees']
for x in by.values():assert x['birth'] in reopened['shared']['births'] and x['houses']['ut1_minus_utc_milliseconds']==0 and x['houses']['supplier'].endswith(l['turquet_houses'])
for name,record in json.loads((p/'prior-guard.json').read_text()).items():
 root=r if name=='cleromancy' else Path(record['root'])
 for n,digest in record['files'].items():assert sha(root/n)==digest,(name,n)
subprocess.run(['python3',str(r/'receipts/verify_c8_natal_hardened.py')],check=True)
print('Natal houses evidence verified: immutable engine/source, software gates, thirteen reviewed originals, explicit systems, exact retained natal/reflection values and fresh shared catalog, prior receipts intact.')
