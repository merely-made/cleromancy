from pathlib import Path
import subprocess,os,json,hashlib,re
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy')
env=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc',CARGO_BUILD_JOBS='4')
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()
cases=[('default',['cargo','test','--locked','--offline']),('expanded',['cargo','test','--locked','--offline','--features','legacy-receipts,analytic-ephemeris,sky-timeline,personal-sync,graphshell-admission']),('all-targets',['cargo','check','--locked','--offline','--all-targets']),('no-default',['cargo','check','--locked','--offline','--all-targets','--no-default-features']),('personal-sync',['cargo','check','--locked','--offline','--all-targets','--features','personal-sync']),('graphshell-admission',['cargo','check','--locked','--offline','--all-targets','--features','graphshell-admission']),('core-default',['cargo','test','--manifest-path','core/Cargo.toml','--locked','--offline']),('core-legacy',['cargo','test','--manifest-path','core/Cargo.toml','--locked','--offline','--features','legacy-receipts']),('core-sky',['cargo','test','--manifest-path','core/Cargo.toml','--locked','--offline','--features','legacy-receipts,sky-timeline']),('desktop-build',['cargo','build','--locked','--offline','--bin','cleromancy'])]
results={}
for name,args in cases:
 p=b/(name+'.log');assert not p.exists()
 print('START',name,flush=True)
 with p.open('wb') as log:
  log.write((json.dumps({'source':source,'command':args,'RUSTC':env['RUSTC'],'RUSTDOC':env['RUSTDOC'],'CARGO_BUILD_JOBS':'4'})+'\n').encode());log.flush()
  x=subprocess.run(args,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT)
 text=p.read_text();counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)]
 results[name]={'exit':x.returncode,'command':args,'passed':sum(v[0] for v in counts),'failed':sum(v[1] for v in counts),'ignored':sum(v[2] for v in counts)}
 (b/'checks.json').write_text(json.dumps({'source':source,'results':results},indent=2)+'\n');print(name,results[name],flush=True)
 if x.returncode:raise SystemExit(x.returncode)
print('PASS all scoped intake gates',flush=True)
