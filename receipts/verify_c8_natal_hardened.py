#!/usr/bin/env python3
"""Read-only checks of immutable hardened intake evidence; no current-head renewal."""
from pathlib import Path
import json,hashlib,subprocess,re
r=Path(__file__).resolve().parents[1];l=json.loads((r/'receipts/c8-natal-hardened-intake_20261017.json').read_text());p=r/'receipts'/l['pack']
sha=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
for name,digest in json.loads((p/'artifact-hashes.json').read_text()).items():assert sha(p/name)==digest,name
assert len(l['checks'])==9 and all(c['exit']==0 and c['source']==l['source'] for c in l['checks'].values())
for name,count in l['software_counts'].items():
 rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',(p/'software'/(name+'.log')).read_text());assert rows and all(x[0]=='ok' and x[2]=='0' for x in rows)
 assert count=={'passed':sum(int(x[1]) for x in rows),'failed':0,'ignored':sum(int(x[3]) for x in rows)}
b=json.loads((p/'native-build-05.json').read_text());assert b['source']==l['source'] and b['mere']==l['mere']
for name,digest in b['files'].items():assert hashlib.sha256(subprocess.check_output(['git','show',l['source']+':'+name],cwd=r)).hexdigest()==digest,name
reviews=json.loads((p/'capture-reviews-05.json').read_text());assert len(reviews)==12
for image in reviews:assert sha(p/image['path'])==image['sha256'] and image['review']=='pass within stated scope'
runs=[]
for name in l['native']['runs']:
 a=json.loads((p/name/'receipt.json').read_text());z=json.loads((p/name/'process.json').read_text());assert a['ok'] and z['exit']==0 and not z['timeout'] and z['prompts']==1 and z['offline_ip_deny'] and z['binary_sha256']==b['binary_sha256'];assert not any('FAIL' in x for x in a['log']);runs.append(a)
for a in runs[1:]:assert a['readings']==runs[0]['readings'] and a['reflections']==runs[0]['reflections']
assert runs[0]['shared']==runs[1]['shared'];assert len(runs[0]['shared']['births'])==2 and len(runs[0]['shared']['associations'])==4
assert runs[0]['readings'][0]['birth'] in runs[0]['shared']['births']
assert runs[0]['readings'][0]['birth']['facts']['time']=={'Known':{'hour':12,'minute':0,'second':0}}
assert any(x['facts']['time']=={'Known':{'hour':13,'minute':0,'second':0}} for x in runs[0]['shared']['births'])
d=l['daemon_intake'];assert sha(p/d['receipt'])==d['sha256']
owner=json.loads((p/d['receipt']).read_text());assert owner['cleromancy_core']==l['source'] and d['cleromancy_core']==l['source']
subprocess.run(['python3',str(r/'receipts/verify_c8_natal.py')],check=True)
print('Hardened natal intake verified: exact source, nine software gates, twelve reviewed originals, birth correction and three-process offline reopening.')
