from pathlib import Path
import subprocess,json,os
b=Path(__file__).resolve().parent;b=b/'software';b.mkdir(exist_ok=False);r=Path('/Users/markik/Code/repos/cleromancy')
e=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc',CARGO_BUILD_JOBS='4',TMPDIR='/tmp')
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip();cases=[('desktop-build',r,['build','--bin','cleromancy']),('check-default',r,['check','--all-targets']),('check-no-default',r,['check','--no-default-features','--all-targets']),('check-personal',r,['check','--features','personal-sync','--all-targets']),('check-admission',r,['check','--features','graphshell-admission','--all-targets']),('test-default',r,['test']),('test-historical',r,['test','--features','legacy-receipts,analytic-ephemeris,sky-timeline,personal-sync,graphshell-admission']),('core-default',r/'core',['test']),('core-legacy',r/'core',['test','--features','legacy-receipts']),('core-sky',r/'core',['test','--features','legacy-receipts,sky-timeline'])];results={}
for name,cwd,args in cases:
 with (b/(name+'.log')).open('w') as f:code=subprocess.run(['cargo',*args,'--locked','--offline'],cwd=cwd,env=e,stdout=f,stderr=subprocess.STDOUT).returncode
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()==source
 results[name]={'exit':code,'source':source,'args':args};(b/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(name,code,flush=True)
 if code:raise SystemExit(code)
