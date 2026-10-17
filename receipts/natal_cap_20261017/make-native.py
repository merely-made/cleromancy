from pathlib import Path
import subprocess,shutil,json,hashlib
b=Path(__file__).resolve().parent;r=Path('/Users/markik/Code/repos/cleromancy');source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()
old=Path('/Users/markik/Code/testing/cleromancy/natal-houses-20261017/Cleromancy Houses v1.app');new=b/'Cleromancy Natal Cap v1.app';shutil.copytree(old,new);exe=new/'Contents/MacOS/Cleromancy';shutil.copy2(r/'target/debug/cleromancy',exe);subprocess.run(['codesign','--force','--deep','--sign','-',str(new)],check=True)
files=['Cargo.toml','Cargo.lock','core/Cargo.toml','core/Cargo.lock']+[str(p.relative_to(r)) for p in (r/'src').rglob('*') if p.is_file()]
record={'source':source,'turquet_cap':'8e3755bbd0d40343eb4750b072801c941ff32802','turquet_houses':'e2f65c021a6bf744ab5fe77121e0441a46b47306','turquet_planets':'30875f30e83909f3030bc01237a5046e14cde29b','binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'files':{n:hashlib.sha256((r/n).read_bytes()).hexdigest() for n in files},'command':'cargo build --bin cleromancy --locked --offline','compiler':'Cargo 1.99; rustc/rustdoc 1.97.1'}
(b/'native-build.json').write_text(json.dumps(record,indent=2)+'\n')
