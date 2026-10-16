from pathlib import Path
import json,shutil,hashlib
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy');out=r/'receipts/c8_20261016'
assert not out.exists();out.mkdir()
allowed={'.json','.log','.md','.py','.scn','.lock','.txt','.png','.done'}
for p in sorted(b.rglob('*')):
 if not p.is_file() or any(part.endswith('.app') for part in p.relative_to(b).parts) or p.suffix not in allowed:continue
 q=out/p.relative_to(b);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
for p in sorted((r/'receipts').glob('c8-die*.scn')):shutil.copy2(p,out/p.name)
shutil.copy2(b/'verify_c8.py',r/'receipts/verify_c8.py')
(out/'README.md').write_text('''# C8 first die qualification, 2026-10-16

This pack qualifies one six-sided die through the existing scalar Cast/Derived
receipt and resident validation contracts. Operating-system and public-seed
rolls each save one face, its context, situation and reflection. All faces have
equal weight; independent rolls may repeat. There is no assigned symbolic
meaning or orientation. This is the first bounded C8 slice; other systems,
multiple dice, generated die interpretations and physical/dynamics casting
remain outside this acceptance.

Software source is `e51964a9387d1b846171871f0bb1e565ee86a5f7`.
Frozen native source is `a4abd1bb6c00d5b174b077ede899aa7ee7b8c8e8`;
`source-bridge.json` binds the three scenario-only differences. Implementation,
assets and dependency bytes are otherwise identical. The signed native binary
is `afe5e4739820b51fb6bda3348018d3b4dd4b0a572de111f508d2b94e3b129699`.
`native-build-tuned.json` binds that exact build. Older build manifests and
historically named final artifacts retain their original, narrower identities.
No dependency repin or new Mere code was needed.

Default/expanded product gates pass 130/179 tests, each with one intentional
local-model probe ignored. All four all-target checks pass, including
personal-sync and graphshell-admission. Portable gates pass 41/42/43 without
failures or ignores on 4adc154; subsequent changes affect only UI/tests/scenarios.
Cargo 1.99 drives explicit Rust/Rustdoc 1.97.1. Named debug CPU optimization
changes no receipt, validator, replay check or timeout cap.

Four whole native runs pass on the retained public synthetic C4 profile:
OS creation, fresh OS reopening, keyboard public-seed creation and fresh seeded
reopening. The two results happen to be face 3. Fourteen original native
captures are individually reviewed in capture-reviews.json: wide and compact
narratives, targeted compact randomness controls and the Roll control. Reopening
changes the method form through keyboard actions without rolling again. Mode
captures show the targeted control; the other option may be above the viewport.
Method captures begin partway through the explanation after scrolling. Narrative
captures show the complete explanation within the focused reader. Dense generic
Graphshell labels in the companion pane remain visible presentation debt.

Every process uses the actual encrypted vault's production NativeOrTerminal
passphrase prompt in an owned PTY, with process-local PATH=/bin. Outbound IP
network access is denied; no local model is configured. This does not qualify
graphical unlock. These bounded runs establish no startup reliability, general
performance, release packaging, all-system C8 completion or shared-wall claim.

Raw die-reading.json bytes match between each creation and actual fresh process.
A final same-disk owner inspection finds ten occasions: all fields of the eight
prior occasions remain exact, including stored C7 prompts and explicit keeping;
the complete C4 domain view and memory values are unchanged. The two additions
are the saved die occasions. This is domain-value preservation, not every shared
disk byte: cold startup can append projection/view/epoch state. The inspector
is the retained C7 helper, whose exact binary digest is in its process records.
All 618 prior receipt/scenario paths retain their SHA256 values.

Preparation failures, their five diagnostic originals, and one passing bounded
installation original remain separately recorded in diagnostic-capture-reviews.json.
They are not among the fourteen whole-flow originals. The first attempts exposed
a missing form control, a lost install response, attachment timeout and slow
replay. One process was explicitly stopped, and the seeded diagnostic timed out.
Source corrections and named CPU optimization preceded the frozen candidate;
none of these earlier runs is relabeled as passing. Intermediate compiler/test
failures and an intentionally interrupted preparation matrix remain historical.
All external apps, profiles, binaries, prior receipts and model artifacts remain
intact; executables, vault data and model weights are not copied into this pack.

Run `python3 receipts/verify_c8.py` from the repository. It verifies these frozen
artifacts and Git inputs; it does not recertify later product source.
''')
hashes={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
(out/'artifact-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
runs=['native-os-create-04','native-os-reopen-01','native-seed-create-01','native-seed-reopen-01']
ledger={'schema':'cleromancy.c8-first-die-qualification/v1','date':'2026-10-16','status':'bounded first-die native qualification complete; publication recorded separately','software_source':'e51964a9387d1b846171871f0bb1e565ee86a5f7','native_source':'a4abd1bb6c00d5b174b077ede899aa7ee7b8c8e8','native_build':json.loads((b/'native-build-tuned.json').read_text()),'source_bridge':json.loads((b/'source-bridge.json').read_text()),'software_checks':json.loads((b/'product-checks-final.json').read_text()),'portable_core_counts':{'default':41,'legacy':42,'sky':43},'native_runs':{n:{'process':json.loads((b/n/'process.json').read_text()),'receipt':json.loads((b/n/'receipt.json').read_text())} for n in runs},'accepted_originals':14,'preserved_diagnostic_originals':5,'bounded_installation_originals':1,'preserved_prior_receipt_paths':618,'prior_occasions_exact':8,'new_die_occasions':2,'fresh_reopen_detail_bytes_exact':True,'c4_domain_views_and_memory_exact':True,'evidence':'c8_20261016/artifact-hashes.json','scope':'One D6, OS or public-seed selection, no interpretation or orientation; no graphical-unlock, performance, release, shared-wall or complete C8 claim.'}
(r/'receipts/c8-first-die-qualification_20261016.json').write_text(json.dumps(ledger,indent=2)+'\n')
print('Collected',len(hashes),'artifacts, bytes',sum(p.stat().st_size for p in out.rglob('*') if p.is_file()))
