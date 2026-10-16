import os,subprocess,pathlib
base=pathlib.Path(__file__).resolve().parent
repo=pathlib.Path('/Users/markik/Code/repos/cleromancy')
env=dict(os.environ,RUSTC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc',RUSTDOC='/Users/markik/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustdoc')
for name,features in [('default',None),('legacy','legacy-receipts'),('sky','legacy-receipts,sky-timeline')]:
 args=['cargo','test','--manifest-path','core/Cargo.toml','--locked','--offline']
 if features:args+=['--features',features]
 with (base/('core-'+name+'-01.log')).open('wb') as log:
  result=subprocess.run(args,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
 print(name,result.returncode,flush=True)
 if result.returncode:raise SystemExit(result.returncode)
