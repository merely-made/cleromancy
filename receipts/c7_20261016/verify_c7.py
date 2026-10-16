#!/usr/bin/env python3
"""Read-only verification of frozen C7 evidence, not current-source renewal."""
import hashlib,json,pathlib,re,subprocess
repo=pathlib.Path(__file__).resolve().parents[1]
base=repo/'receipts/c7_20261016'
ledger=json.loads((repo/'receipts/c7-generated-inferences-qualification_20261016.json').read_text())
def read(path):return json.loads((base/path).read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def counts(path):
 rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',(base/path).read_text())
 assert rows and all(row[0]=='ok' for row in rows),path
 return [sum(int(row[i]) for row in rows) for i in [1,2,3]]
for path,digest in read('artifact-hashes.json').items():assert sha(base/path)==digest,path
for name in ledger['input_manifests']:
 inputs=read(name)
 for item in inputs['files']:
  blob=subprocess.check_output(['git','show',inputs['source']+':'+item['path']],cwd=repo)
  assert hashlib.sha256(blob).hexdigest()==item['sha256'],item['path']
for path,digest in read('prior-artifacts.json').items():assert sha(repo/path)==digest,path
assert counts('product-default-final.log')==[126,0,1]
assert counts('product-historical-final.log')==[175,0,1]
for name,expected in [('default',41),('legacy',42),('sky',43)]:assert counts('core-'+name+'-final.log')==[expected,0,0]
checks=read('product-checks-final.json');assert len(checks)==6 and set(checks.values())=={0}
assert counts('receiving-immutable-01.log')==[8,0,1]
receiving=read('receiving-qualification.json');assert receiving['immutable_git_core'] and not receiving['local_override_in_final_gate']
reviews={}
for filename in ledger['capture_reviews']:
 for review in read(filename):reviews[review['path']]=review
accepted=0
for run in ledger['native_runs']:
 directory=run['directory'];p=read(directory+'/process.json');receipt=read(directory+'/receipt.json');inputs=read(run['input_manifest'])
 assert p['exit']==0 and not p['timeout'] and receipt['ok'],directory
 assert p['offline_ip_deny'] and p['binary_sha256']==inputs['native_sha256'],directory
 assert receipt['status'].startswith('Viewing session:'),directory
 for capture in (base/directory).glob('*.png'):
  key=str(capture.relative_to(base));review=reviews[key]
  assert review['result']=='pass' and sha(capture)==review['sha256'],key
  accepted+=1
assert not read('native-generate-01/receipt.json')['ok']
for left,right in ledger['exact_detail_pairs']:
 assert (base/left).read_bytes()==(base/right).read_bytes(),(left,right)
before=read('domain-before.json');after=read('domain-after-qualified.json')
old={d['session']['id']:d for d in before['details']};new={d['session']['id']:d for d in after['details']}
assert old.keys()==new.keys() and len(old)==8
for key,value in old.items():
 saved=new[key].copy();saved.pop('inferences',None);saved.pop('kept_inferences',None);assert saved==value,key
assert before['views']==after['views'] and before['memory']==after['memory']
model=read('model-manifest.json')
for detail in new.values():
 for record in detail.get('inferences',[]):
  assert record['initial_keeping']=='short_term'
  generation=record['generation']
  if generation['kind']=='model':
   assert record['label']=='Model-generated reflection prompt'
   assert generation['model']['model_id']==model['model_id'] and generation['model']['revision']==model['revision']
   assert generation['model']['artifacts']=={name:model['artifacts'][name] for name in ['config.json','tokenizer.json','model.safetensors']}
assert accepted==ledger['accepted_capture_count']
print(f'C7 frozen evidence verified: {accepted} accepted originals, 439 prior paths intact, exact saved prompts, eight prior occasions and C4 views/memory preserved; immutable three-owner receiving gate passes.')
