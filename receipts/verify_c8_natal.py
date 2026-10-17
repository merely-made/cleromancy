#!/usr/bin/env python3
"""Read-only verification of frozen shared natal evidence, not current-source renewal."""
from pathlib import Path
import hashlib,json,re,subprocess
repo=Path(__file__).resolve().parents[1]
ledger=json.loads((repo/'receipts/c8-natal-qualification_20261017.json').read_text());pack=repo/'receipts'/ledger['pack']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for path,digest in json.loads((pack/'artifact-hashes.json').read_text()).items():assert sha(pack/path)==digest,path
assert len(ledger['checks'])==9 and all(v['exit']==0 and v['source']==ledger['source'] for v in ledger['checks'].values())
for name,expected in ledger['software_counts'].items():
 rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',(pack/'software'/(name+'.log')).read_text());assert rows and all(r[0]=='ok' and r[2]=='0' for r in rows)
 assert {'passed':sum(int(r[1]) for r in rows),'failed':0,'ignored':sum(int(r[3]) for r in rows)}==expected
build=json.loads((pack/'native-build-04.json').read_text());assert build['source']==ledger['source']
for path,digest in build['files'].items():
 data=subprocess.check_output(['git','show',ledger['source']+':'+path],cwd=repo);assert hashlib.sha256(data).hexdigest()==digest,path
reviews=json.loads((pack/'capture-reviews.json').read_text());assert len(reviews)==12
for image in reviews:assert sha(pack/image['path'])==image['sha256'] and image['review']=='pass within stated scope'
native=[]
for name in ledger['native']['runs']:
 record=json.loads((pack/name/'receipt.json').read_text());process=json.loads((pack/name/'process.json').read_text())
 assert record['ok'] and process['exit']==0 and not process['timeout'] and process['offline_ip_deny'] and process['prompts']==1
 assert process['binary_sha256']==build['binary_sha256'] and not any('FAIL' in line for line in record['log'])
 native.append(record)
for other in native[1:]:assert other['readings']==native[0]['readings'] and other['reflections']==native[0]['reflections']
assert native[0]['shared']==native[1]['shared']
assert len(native[0]['shared']['births'])==2 and len(native[0]['shared']['subjects'])==1 and len(native[0]['shared']['associations'])==4
assert native[0]['readings'][0]['birth'] in native[0]['shared']['births']
prior=json.loads((pack/'prior-guard.json').read_text())
for name,record in prior.items():
 root=repo if name=='cleromancy' else Path(record['root'])
 for path,digest in record['files'].items():assert sha(root/path)==digest,(name,path)
print('Shared natal evidence verified: twelve reviewed originals, exact natal/reflection reopening, two shared birth revisions, and prior receipts/profile intact.')
