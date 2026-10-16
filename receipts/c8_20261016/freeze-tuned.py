from pathlib import Path
import shutil,subprocess,plistlib,hashlib,json
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy');checks=json.loads((b/'product-checks-final.json').read_text());assert len(checks)==6 and all(v==0 for v in checks.values())
app=b/'Cleromancy C8 tuned.app';assert not app.exists();(app/'Contents/MacOS').mkdir(parents=True)
p=r/'target/debug/cleromancy';unsigned=hashlib.sha256(p.read_bytes()).hexdigest();shutil.copy2(p,app/'Contents/MacOS/Cleromancy')
(app/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleExecutable':'Cleromancy','CFBundleIdentifier':'dev.merely.cleromancy.c8-tuned','CFBundleName':'Cleromancy C8 tuned','CFBundlePackageType':'APPL'}))
subprocess.run(['codesign','--force','--deep','--sign','-',str(app)],check=True)
(b/'native-build-tuned.json').write_text(json.dumps({'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'unsigned_binary_sha256':unsigned,'signed_binary_sha256':hashlib.sha256((app/'Contents/MacOS/Cleromancy').read_bytes()).hexdigest(),'command':'cargo build --locked --offline --bin cleromancy','compiler':'Cargo 1.99; rustc/rustdoc 1.97.1','bundle':str(app),'debug_cpu_overrides':{'cleromancy':1,'cleromancy-core-acceptance':1,'serde_json':2},'locks':'unchanged from qualified C7'},indent=2)+'\n')
s=(b/'run-native.py').read_text().replace('Cleromancy C8.app','Cleromancy C8 tuned.app');(b/'run-tuned-native.py').write_text(s)
