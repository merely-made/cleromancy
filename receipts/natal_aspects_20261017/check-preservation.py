from pathlib import Path
import json,hashlib
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy')
prior=json.loads((b/'prior-receipts.json').read_text())
for name,digest in prior.items(): assert hashlib.sha256((r/name).read_bytes()).hexdigest()==digest, name
result={'ok':True,'prior_receipt_paths':len(prior),'baseline':'8472bc8dca53c2646baea0f864db9d1d0578fd61'}
(b/'preservation.json').write_text(json.dumps(result,indent=2)+'\n');print(result)
