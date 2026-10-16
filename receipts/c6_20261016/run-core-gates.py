from pathlib import Path
import subprocess,json,os,time,re,hashlib
r=Path('/Users/markik/Code/repos/cleromancy'); b=Path(__file__).parent
env=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc')
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()
results=[]
jobs=[('default',['test','--manifest-path','core/Cargo.toml']),('legacy',['test','--manifest-path','core/Cargo.toml','--features','legacy-receipts']),('sky',['test','--manifest-path','core/Cargo.toml','--features','legacy-receipts,sky-timeline'])]
for name,args in jobs:
 cmd=['cargo',args[0],'--locked','--offline',*args[1:]]
 start=time.time();log=b/('core-'+name+'.log')
 with log.open('w') as f: p=subprocess.run(cmd,cwd=r,env=env,stdout=f,stderr=subprocess.STDOUT)
 matches=[tuple(map(int,v)) for v in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;',log.read_text())]
 item=dict(name=name,source=source,command=cmd,exit=p.returncode,elapsed_seconds=round(time.time()-start,2),passed=sum(v[0] for v in matches),failed=sum(v[1] for v in matches),ignored=sum(v[2] for v in matches),log=log.name,sha256=hashlib.sha256(log.read_bytes()).hexdigest())
 results.append(item);(b/'core-gates.json').write_text(json.dumps(results,indent=2)+'\n'); print(name,p.returncode,flush=True)
 if p.returncode:break
