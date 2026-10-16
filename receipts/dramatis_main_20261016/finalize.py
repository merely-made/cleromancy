from pathlib import Path
import json,hashlib,shutil,subprocess
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy')
read=lambda p:json.loads(p.read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
checks=read(b/'checks.json');assert len(checks['results'])==10 and all(v['exit']==0 and v['failed']==0 for v in checks['results'].values())
prior=read(b/'prior-artifacts.json')
for name,digest in prior.items():assert sha(r/name)==digest,name
inputs=read(b/'source-inputs.json');assert inputs['source']==checks['source']
for name,digest in inputs['files'].items():assert sha(r/name)==digest,name
for graph in read(b/'resolved-sources.json').values():assert graph['dramatis_sources']==['git+https://github.com/merely-made/dramatis.git?branch=main#2f27f1a939aa2e046b78f7db18dcd8f81e0d7c59']
subprocess.run(['python3','receipts/verify_c8.py'],cwd=r,check=True,stdout=(b/'verify-c8.log').open('w'))
subprocess.run(['python3','receipts/verify_c7.py'],cwd=r,check=True,stdout=(b/'verify-c7.log').open('w'))
pack=r/'receipts/dramatis_main_20261016';assert not pack.exists();pack.mkdir()
for p in sorted(b.iterdir()):
 if p.is_file() and p.suffix in {'.json','.log','.py','.before'}:shutil.copy2(p,pack/p.name)
(pack/'README.md').write_text('''# Published Dramatis main intake, 2026-10-16

Cleromancy's app and portable-core manifests already track the canonical
Dramatis main URL. Cargo aligns both lockfiles to published main
2f27f1a939aa2e046b78f7db18dcd8f81e0d7c59. The app formerly locked 5c89001;
the portable core formerly locked 814ffa0. Both resolved graphs now contain
one Dramatis revision. No import or product source changes were necessary.

Cargo update changes no package versions, package set, other Git revisions
or source selectors. In the app lock it also re-resolves existing registry
edges (syn, constant_time_eq and windows-sys); these are retained as Cargo
generated them and covered by the scoped gates. The core lock changes only
the two Dramatis source revisions. Original lockfiles and update logs remain
in this pack. Metadata's initial offline unfiltered attempt lacked an already
locked antithesis_sdk cache entry; subsequent locked platform-filtered metadata
succeeds. That cache diagnostic is preserved; no dependency version changed.

Checks bind committed source in source-inputs.json: default and expanded
product suites, four all-target variants, three portable-core suites and the
desktop build. See checks.json for exact commands/counts. Cargo 1.99 runs
explicit rustc/rustdoc 1.97.1 with four build jobs. Metadata summaries bind the
full root-platform and core graphs. All prior receipt paths in prior-artifacts.json
remain byte-identical, and the existing C7/C8 frozen-evidence verifiers pass.

This is source/software/build compatibility, not renewed native qualification.
No public fixture was opened by a desktop run for this intake. C7 and C8 native
captures, fresh-process reopenings and exact binaries retain their earlier
sources. The separate local Dramatis subject-reference proposal is unpublished
and excluded from this supplier revision; no natal API, birth-fact storage or
natal reading is adopted by this dependency update.
''')
hashes={str(p.relative_to(pack)):sha(p) for p in sorted(pack.iterdir()) if p.is_file()};(pack/'artifact-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
receipt={'schema':'cleromancy.dramatis-main-intake/v1','date':'2026-10-16','status':'software and desktop build qualified; publication recorded separately','source':checks['source'],'dramatis_main':'2f27f1a939aa2e046b78f7db18dcd8f81e0d7c59','previous_root':'5c89001a6ff1aac3f6105ed50b6ae5f09c9f27e2','previous_core':'814ffa09f7826f55b4b9e977306b053e751b5e8e','resolved_graphs':read(b/'resolved-sources.json'),'checks':checks['results'],'source_inputs':inputs,'prior_receipt_paths_intact':len(prior),'old_c7_c8_verifiers':'pass','desktop_binary_sha256':sha(r/'target/debug/cleromancy'),'evidence':'dramatis_main_20261016/artifact-hashes.json','native_requalified':False,'natal_contract_included':False}
(r/'receipts/dramatis-main-intake_20261016.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'checks':checks['results'],'prior_paths':len(prior),'artifacts':len(hashes)},indent=2))
