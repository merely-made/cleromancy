"""Collect source-bound C7 evidence; never copy model weights, apps or worktrees."""
import pathlib, json, hashlib, shutil, subprocess, re
base=pathlib.Path(__file__).resolve().parent
repo=pathlib.Path('/Users/markik/Code/repos/cleromancy')
out=repo/'receipts/c7_20261016'
assert not out.exists(), 'receipt collection is append-only'
checks=json.loads((base/'product-checks-final.json').read_text())
assert len(checks)==6 and all(v==0 for v in checks.values()), checks
required=['native-reopen-01','native-final-reopen-01','native-final-regenerate-01','native-qualified-create-01','native-qualified-reopen-01']
for name in required:
 d=json.loads((base/name/'receipt.json').read_text());p=json.loads((base/name/'process.json').read_text())
 assert d['ok'] and not p['timeout'] and p['exit']==0 and p['offline_ip_deny'], name
assert not json.loads((base/'native-generate-01/receipt.json').read_text())['ok']
assert '8 passed; 0 failed; 1 ignored' in (base/'receiving-immutable-01.log').read_text()
out.mkdir()
allowed={'.json','.log','.md','.py','.scn','.lock','.txt','.png','.done'}
for p in sorted(base.iterdir()):
 if p.is_file() and p.suffix in allowed:
  shutil.copy2(p,out/p.name)
 elif p.is_dir() and (p.name.startswith('native-') or p.name.startswith('domain-')):
  target=out/p.name;target.mkdir()
  for child in sorted(p.iterdir()):
   if child.is_file() and child.suffix in allowed:shutil.copy2(child,target/child.name)
shutil.copy2(base/'model/cleromancy-model.json',out/'model-manifest.json')
checksums={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
(out/'artifact-hashes.json').write_text(json.dumps(checksums,indent=2)+'\n')
print('Collected',len(checksums),'artifacts')
