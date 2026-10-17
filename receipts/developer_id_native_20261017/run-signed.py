#!/usr/bin/env python3
"""Run only when Mark is ready to answer the real public-fixture vault dialog."""
from pathlib import Path
import os, sys, pty, select, time, signal, json, hashlib, subprocess
b = Path(__file__).resolve().parent
r = Path('/Users/markik/Code/repos/cleromancy')
profile = b / 'profile'
assert (profile / '.c1-public-synthetic').read_text() == 'cleromancy-c1 public synthetic fixture/v1\n'
output = b / sys.argv[1]
output.mkdir(exist_ok=False)
entry = b / 'installed/Cleromancy Aspects Developer ID Candidate.app/Contents/MacOS/Cleromancy'
payload = entry.with_name('cleromancy-bin')
scenario = b / 'signed-reopen.scn'
env = {k: v for k, v in os.environ.items() if not k.startswith(('CLEROMANCY_', 'MERE_', 'GRAPHSHELL_'))}
env.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', TERM='dumb', CAMBIUM_HOST_PERF_TRACE='1',
 MERE_ROOT=str(profile/'shared'), CLEROMANCY_VAULT_DIR=str(profile/'ordinary-vault'),
 CLEROMANCY_ROOT=str(profile/'cleromancy'), CLEROMANCY_MODEL_DIR=str(b/'not-configured'),
 GRAPHSHELL_APP_ENDPOINT='/tmp/clero-signed-20261017.sock',
 CLEROMANCY_SCENARIO=str(scenario), CLEROMANCY_SCENARIO_PHASE='natal-reopen',
 CLEROMANCY_CAPTURE_DIR=str(output), CLEROMANCY_PRESENTATION_TIMEOUT_SECS='60')
policy = '(version 1)(allow default)(deny network-outbound (remote ip "*:*"))'
start = time.time(); status = None; failure = None; timed_out = False; providers = []; last_inventory = 0
pid, fd = pty.fork()
if pid == 0:
 os.execve('/usr/bin/sandbox-exec', ['sandbox-exec', '-p', policy, str(entry)], env)
def terminate_owned():
 if os.getpgid(pid) == pid: os.killpg(pid, signal.SIGTERM)
 else: os.kill(pid, signal.SIGTERM)
with (output/'terminal.log').open('wb') as log:
 while True:
  if time.time()-start > 900:
   timed_out = True; failure = 'No completed graphical scenario within the input window'; terminate_owned(); break
  if time.time() - last_inventory > 1:
   last_inventory = time.time()
   rows = []
   for line in subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,args='], text=True).splitlines():
    parts = line.strip().split(None, 2)
    if len(parts)==3: rows.append((int(parts[0]), int(parts[1]), parts[2]))
   owned = {pid}
   for _ in range(6): owned.update(child for child,parent,_ in rows if parent in owned)
   for child,parent,args in rows:
    if child in owned and 'osascript' in args and 'display dialog' in args and 'Identity vault' in args:
     witness = {'pid':child,'parent':parent,'command':args}
     if witness not in providers: providers.append(witness)
  ready, _, _ = select.select([fd], [], [], .5)
  if ready:
   try: data = os.read(fd, 65536)
   except OSError: data = b''
   if not data: break
   log.write(data); log.flush()
   if b'Enter the vault passphrase to start.' in data:
    failure = 'Production prompt fell back to terminal; no answer supplied'; terminate_owned(); break
  ended, candidate = os.waitpid(pid, os.WNOHANG)
  if ended: status = candidate; break
if status is None: _, status = os.waitpid(pid, 0)
if not providers and failure is None: failure = 'No owned production graphical dialog provider was observed'
result = {'graphical_provider_processes':providers,'exit':os.waitstatus_to_exitcode(status),'timeout':timed_out,'failure':failure,
 'elapsed_seconds':round(time.time()-start,2),'source':'3fa5a6db87cfa34df9fc1de0a0877181c9b4526f',
 'entry':str(entry),'entry_sha256':hashlib.sha256(entry.read_bytes()).hexdigest(),
 'payload_sha256':hashlib.sha256(payload.read_bytes()).hexdigest(),
 'scenario_sha256':hashlib.sha256(scenario.read_bytes()).hexdigest(),
 'terminal_answers':0,'unlock':'production NativeOrTerminal graphical; manual public-fixture input',
 'offline_ip_deny':True,'public_fixture':True}
(output/'process.json').write_text(json.dumps(result,indent=2)+'\n')
receipt = output/'receipt.json'; ok = receipt.exists() and json.loads(receipt.read_text())['ok']
print(json.dumps(result), flush=True)
sys.exit(0 if ok and result['exit']==0 and failure is None else 1)
