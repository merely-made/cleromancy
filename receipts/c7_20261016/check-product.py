import subprocess,pathlib,os,json
base=pathlib.Path(__file__).resolve().parent
repo='/Users/markik/Code/repos/cleromancy'
env=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc')
cases=[('default',['cargo','test','--locked','--offline']),('historical',['cargo','test','--locked','--offline','--features','legacy-receipts,analytic-ephemeris,sky-timeline,personal-sync,graphshell-admission']),('no-default-check',['cargo','check','--locked','--offline','--all-targets','--no-default-features']),('default-check',['cargo','check','--locked','--offline','--all-targets']),('personal-check',['cargo','check','--locked','--offline','--all-targets','--features','personal-sync']),('admission-check',['cargo','check','--locked','--offline','--all-targets','--features','graphshell-admission'])]
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
results={}
for name,args in cases:
 with (base/('product-'+name+'-01.log')).open('wb') as log:
  log.write((json.dumps({'source':source,'command':args})+'\n').encode());log.flush()
  r=subprocess.run(args,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
 results[name]=r.returncode; (base/'product-checks.json').write_text(json.dumps(results,indent=2)+'\n'); print(name,r.returncode,flush=True)
 if r.returncode:raise SystemExit(r.returncode)
