#!/usr/bin/env python3
"""Owned PTY, public fixture only; production terminal unlock, no prompt stub."""
import os, sys, time, select, pty, json, pathlib, hashlib, signal
base = pathlib.Path(__file__).resolve().parent
repo = pathlib.Path('/Users/markik/Code/repos/cleromancy')
phase, run_name = sys.argv[1:3]
profile = base / (sys.argv[3] if len(sys.argv)>3 else 'profile')
assert phase in ('setup', 'inspect')
out = base / run_name
out.mkdir() # Never overwrite a failed or accepted attempt.
binary = pathlib.Path('/Users/markik/Code/testing/cleromancy/c7-20261016/c7-inspect-accepted')
env = dict(os.environ)
for key in list(env):
    if key.startswith(('CLEROMANCY_', 'GRAPHSHELL_', 'MERE_')): del env[key]
env.update(PATH='/bin', TERM='dumb', MERE_ROOT=str(profile/'shared'),
 CLEROMANCY_VAULT_DIR=str(profile/'ordinary-vault'),
 CLEROMANCY_ROOT=str(profile/'cleromancy'),
 GRAPHSHELL_APP_ENDPOINT='/tmp/c6-20261016.sock',
 CLEROMANCY_CAPTURE_DIR=str(out), CLEROMANCY_PRESENTATION_TIMEOUT_SECS='60')
binary_digest=hashlib.sha256(binary.read_bytes()).hexdigest()
start=time.time(); prompts=0; buf=b''; status=None; timed_out=False
pid, fd=pty.fork()
if pid==0: os.execve(str(binary), [str(binary),str(profile)]+(['inspect'] if phase=='inspect' else []), env)
with open(out/'terminal.log', 'wb') as log:
 while True:
    if time.time()-start>180:
        os.kill(pid, signal.SIGTERM); timed_out=True; break
    ready,_,_=select.select([fd],[],[],0.5)
    if ready:
        try: data=os.read(fd,65536)
        except OSError: data=b''
        if data:
            log.write(data); log.flush(); buf+=data
            if b'Enter the vault passphrase to start.' in buf or ((b'passphrase' in buf.lower() or b'password' in buf.lower()) and buf.rstrip().endswith((b':',b'?',b'>'))):
                os.write(fd,b'C1 public synthetic identity only\n'); prompts+=1; buf=b''
        else: break
    ended, status_candidate=os.waitpid(pid,os.WNOHANG)
    if ended: status=status_candidate; break
if status is None: _,status=os.waitpid(pid,0)
result=dict(phase=phase,exit=os.waitstatus_to_exitcode(status),timeout=timed_out,prompts=prompts,
 elapsed_seconds=round(time.time()-start,2),binary_sha256=binary_digest,
 unlock='production NativeOrTerminal in owned PTY, PATH=/bin')
(out/'process.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
sys.exit(result['exit'])
