from pathlib import Path
import subprocess,json,sys
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy');profile=Path('/Users/markik/Code/testing/cleromancy/c4-20261016/profile-native')
assert (profile/'.c1-public-synthetic').read_bytes()==b'cleromancy-c1 public synthetic fixture/v1\n'
checks=json.loads((b/'product-checks-final.json').read_text());assert len(checks)==6 and all(v==0 for v in checks.values())
assert (b/'native-build-tuned.json').exists()
cases=[('native-os-create-04','c8-die-os.scn'),('native-os-reopen-01','c8-die-os-reopen.scn'),('native-seed-create-01','c8-die-seed.scn'),('native-seed-reopen-01','c8-die-seed-reopen.scn')]
for name,scene in cases:
 print('START',name,flush=True)
 subprocess.run([sys.executable,str(b/'run-tuned-native.py'),'reopen',name,str(profile),scene,'offline','without-model'],cwd=r,check=True)
 print('PASS',name,flush=True)
subprocess.run([sys.executable,str(b/'run-inspect.py'),'inspect','domain-after',str(profile)],cwd=r,check=True)
s=(b/'domain-after/terminal.log').read_text();d,_=json.JSONDecoder().raw_decode(s[s.index('{\n  "details"'):]);(b/'domain-after.json').write_text(json.dumps(d,indent=2)+'\n')
print('PASS final same-disk owner inspection',flush=True)
