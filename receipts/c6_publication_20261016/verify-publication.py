from pathlib import Path
import subprocess,json,hashlib,os
b=Path(__file__).parent;d=b/'publication-fetch';d.mkdir(exist_ok=True)
repos={'cleromancy':Path('/Users/markik/Code/repos/cleromancy'),'mere':Path('/Users/markik/Code/repos/mere')}
targets=[('mere-supplier','mere','refs/heads/codex/c6-cold-replay-20261016','25cbe8106df3fd6431d83acd756ab6bcabbaba40'),('cleromancy','cleromancy','refs/heads/main','3d45fc5a196a308ff611f590ce84b4965260d0f1'),('mere','mere','refs/heads/main','623e21bf1db3c068323b3c31e3e9647d02de1cad')]
results=[]
for label,name,ref,expected in targets:
 bare=d/(label+'.git');assert not bare.exists();subprocess.run(['git','init','--bare','--quiet',str(bare)],check=True)
 url='https://github.com/merely-made/'+name+'.git'
 cmd=['git','-C',str(bare),'fetch','--no-tags','--depth=1','--filter=blob:none',url,ref]
 with (d/(label+'.log')).open('w') as f:subprocess.run(cmd,env=dict(os.environ,GIT_TERMINAL_PROMPT='0'),stdout=f,stderr=subprocess.STDOUT,check=True)
 actual=subprocess.check_output(['git','-C',str(bare),'rev-parse','FETCH_HEAD'],text=True).strip();assert actual==expected
 tree=subprocess.check_output(['git','-C',str(bare),'ls-tree','-rz','FETCH_HEAD'])
 objects={entry.split(b'\t',1)[1].decode():entry.split(b'\t',1)[0].split()[2].decode() for entry in tree.split(b'\0') if entry}
 paths=([v['path'] for v in json.loads((b/'qualified-source-inputs.json').read_text())['files']] if name=='cleromancy' else ['ports/djinn/src/embedded_reservoir.rs'] if label=='mere-supplier' else ['Cargo.lock','ports/djinn/src/embedded_reservoir.rs','ports/djinn/tests/embedded_reservoir_validation.rs','testing/receipts/cleromancy-c6-intake-20261016/qualification.json'])
 checked=[]
 for path in paths:
  raw=(repos[name]/path).read_bytes();oid=subprocess.check_output(['git','hash-object','--stdin'],input=raw,text=False).decode().strip();assert objects[path]==oid,path
  checked.append(dict(path=path,git_blob=oid,sha256=hashlib.sha256(raw).hexdigest()))
 results.append(dict(repository=url,ref=ref,revision=actual,verification='Fresh bare GitHub fetch; independently retrieved commit and source tree; local input bytes match every selected remote blob identity',checked=checked))
 print(label,actual,len(checked),'matching source blobs',flush=True)
(b/'publication-refs.json').write_text(json.dumps(dict(schema='cleromancy.c6-publication/v1',date='2026-10-16',status='Published and independently verified',checkpoints=results,product_source='adc434dba77a6287d0132b0e6030f32ea36615f3',source_receipt='3d45fc5a196a308ff611f590ce84b4965260d0f1',receiving_source='ea7bc55630167aeafe6dffbba914bba8747450da',receiving_publication='623e21bf1db3c068323b3c31e3e9647d02de1cad',supplier='25cbe8106df3fd6431d83acd756ab6bcabbaba40',prior_receipt_paths_preserved=424),indent=2)+'\n')
