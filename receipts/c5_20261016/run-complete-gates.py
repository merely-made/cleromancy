from pathlib import Path
import subprocess,json,time,re,hashlib,os
repo=Path('/Users/markik/Code/repos/cleromancy');out=Path(__file__).parent
jobs=[('check',['check','--all-targets']),('default',['test','--no-fail-fast']),('historical',['test','--no-fail-fast','--features','legacy-receipts,personal-sync,graphshell-admission']),('historical-check',['check','--all-targets','--features','legacy-receipts,personal-sync,graphshell-admission']),('sky',['test','--features','legacy-receipts,sky-timeline','--test','sky_timeline'])]
result=[]
env=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc')
for name,args in jobs:
 cmd=['cargo',args[0],'--locked','--offline',*args[1:]];log=out/('complete-'+name+'.log');start=time.time()
 with log.open('w') as f:p=subprocess.run(cmd,cwd=repo,env=env,stdout=f,stderr=subprocess.STDOUT)
 s=log.read_text();counts=[tuple(map(int,v)) for v in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;',s)]
 record=dict(name=name,command=cmd,exit=p.returncode,elapsed=round(time.time()-start,2),passed=sum(v[0] for v in counts),failed=sum(v[1] for v in counts),ignored=sum(v[2] for v in counts),log=str(log),sha256=hashlib.sha256(log.read_bytes()).hexdigest());result.append(record)
 (out/'complete-gate-results.json').write_text(json.dumps(result,indent=2)+'\n');print(name,p.returncode,flush=True)
 if p.returncode:break
