#!/usr/bin/env python3
"""C1 first/reopen PTY runner, adapted from aspects release; default is check-only."""
import argparse, errno, json, os, pathlib, pty, select, signal, time
from fixture import BASE, PROFILE, MARKER, require, digest, inventory, preserve_old, validate_identity_only

REPO = pathlib.Path('/Users/markik/Code/repos/cleromancy')
ENDPOINT = '/tmp/srm-20261018-qualified-reopen.sock'
SCENARIOS = {'first': 'c1-resident-seed.scn', 'reopen': 'c1-resident-reopen.scn'}
PUBLIC_PASSPHRASE = b'C1 public synthetic identity only\n'
OFFLINE_POLICY = '(version 1)(allow default)(deny network-outbound (remote ip "*:*"))'

def accepted(exit_code, timed_out, receipt, phase):
    return (exit_code == 0 and not timed_out and receipt.get('ok') is True
            and receipt.get('phase') == phase and receipt.get('schema') == 'cleromancy.headed-scenario/v1'
            and receipt.get('catalog_ready') is True and bool(receipt.get('ids')))

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('phase', choices=SCENARIOS)
    ap.add_argument('run_name')
    ap.add_argument('--binary', type=pathlib.Path, required=True, help='New cohort binary supplied by parent; no historical default')
    ap.add_argument('--binary-sha256', required=True)
    ap.add_argument('--first-run', type=pathlib.Path, help='Successful first run, mandatory for reopen')
    ap.add_argument('--execute', action='store_true', help='Explicitly launch UI; omitted means preflight only')
    ap.add_argument('--timeout', type=int, default=360)
    ap.add_argument('--offline-ip-deny', action=argparse.BooleanOptionalAction, default=True,
                    help='Deny outbound IP with sandbox-exec; enabled by default, keeps local UNIX door available')
    a = ap.parse_args()
    require(a.timeout > 0, 'Timeout must be positive')
    require(a.run_name not in ('.', '..') and pathlib.Path(a.run_name).name == a.run_name, 'Run name must be one new directory name')
    binary = a.binary.resolve(strict=True)
    require(binary.is_file() and os.access(binary, os.X_OK), 'Binary is not executable')
    require(digest(binary) == a.binary_sha256, 'Binary differs from parent-supplied digest')
    argv = ['/usr/bin/sandbox-exec', '-p', OFFLINE_POLICY, str(binary)] if a.offline_ip_deny else [str(binary)]
    if a.offline_ip_deny:
        require(os.access(argv[0], os.X_OK), 'sandbox-exec is required for outbound IP denial')
    scenario = REPO/'receipts'/SCENARIOS[a.phase]
    require(scenario.is_file(), 'Scenario missing')
    out = BASE/a.run_name
    require(not out.exists(), 'Run output already exists; preserve it and choose a new name')
    require((PROFILE/'.c1-public-synthetic').read_bytes() == MARKER, 'Wrong fixture marker')
    preserve_old()
    first_receipt = None
    if a.phase == 'first':
        validate_identity_only()
    else:
        require(a.first_run is not None, 'Reopen requires --first-run')
        first = a.first_run.resolve(strict=True)
        require(first.parent == BASE and first != out, 'First-run receipt must belong to this preparation')
        process = json.loads((first/'process.json').read_text())
        first_receipt = json.loads((first/'receipt.json').read_text())
        require(accepted(process['exit'], process['timeout'], first_receipt, 'first') and process.get('ok') is True, 'First run did not pass')
        require(process['binary_sha256'] == a.binary_sha256, 'Reopen must use the same exact binary')
        require(process['profile'] == str(PROFILE), 'Reopen must use the same profile')
        require(bool(list((PROFILE/'shared').rglob('reservoir.redb'))), 'No established reservoir to reopen')
    require(not pathlib.Path(ENDPOINT).exists(), 'Fixture endpoint already exists; do not steal or remove it')
    plan = {'phase': a.phase, 'binary': str(binary), 'binary_sha256': a.binary_sha256,
            'scenario': str(scenario), 'scenario_sha256': digest(scenario), 'profile': str(PROFILE),
            'endpoint': ENDPOINT, 'output': str(out), 'check_only': not a.execute,
            'argv': argv, 'offline_ip_deny': a.offline_ip_deny,
            'sandbox_policy': OFFLINE_POLICY if a.offline_ip_deny else None}
    if not a.execute:
        print(json.dumps(plan, indent=2)); return
    out.mkdir()
    (out/'plan.json').write_text(json.dumps(plan, indent=2)+'\n')
    (out/'profile-before.json').write_text(json.dumps(inventory(PROFILE), indent=2)+'\n')
    env = dict(os.environ)
    for key in list(env):
        if key.startswith(('CLEROMANCY_', 'GRAPHSHELL_', 'MERE_', 'CAMBIUM_', 'TAPROOT_', 'MESQUITE_')):
            del env[key]
    env.update(PATH='/bin', TERM='dumb', CAMBIUM_HOST_PERF_TRACE='1', GRAPHSHELL_PROFILE='c1-public-synthetic',
               MERE_ROOT=str(PROFILE/'shared'), CLEROMANCY_VAULT_DIR=str(PROFILE/'ordinary-vault'),
               CLEROMANCY_ROOT=str(PROFILE/'cleromancy'), GRAPHSHELL_APP_ENDPOINT=ENDPOINT,
               CLEROMANCY_MODEL_DIR=str(BASE/'no-model-configured'),
               CLEROMANCY_SCENARIO=str(scenario), CLEROMANCY_SCENARIO_PHASE=a.phase,
               CLEROMANCY_CAPTURE_DIR=str(out), CLEROMANCY_PRESENTATION_TIMEOUT_SECS='60')
    require(not pathlib.Path(env['CLEROMANCY_MODEL_DIR']).exists(), 'Unexpected model directory')
    start = time.monotonic(); prompts = 0; buffer = b''; status = None; timed_out = False; eof = False
    pid, fd = pty.fork()
    if pid == 0:
        try: os.execve(argv[0], argv, env)
        except BaseException: os._exit(127)
    with (out/'terminal.log').open('xb') as log:
        while status is None:
            if time.monotonic()-start > a.timeout:
                timed_out = True
                os.kill(pid, signal.SIGTERM)
                deadline = time.monotonic()+5
                while time.monotonic() < deadline:
                    ended, candidate = os.waitpid(pid, os.WNOHANG)
                    if ended: status = candidate; break
                    time.sleep(0.05)
                if status is None:
                    os.kill(pid, signal.SIGKILL)
                    _, status = os.waitpid(pid, 0)
                break
            if not eof and select.select([fd], [], [], 0.2)[0]:
                try: data = os.read(fd, 65536)
                except OSError as error:
                    if error.errno != errno.EIO: raise
                    data = b''
                if not data: eof = True
                else:
                    log.write(data); log.flush(); buffer = (buffer+data)[-16384:]
                    # Existing vault unlock only. Never answer a new-vault creation prompt.
                    if b'Enter the vault passphrase to start.' in buffer and prompts == 0:
                        os.write(fd, PUBLIC_PASSPHRASE); prompts += 1; buffer = b''
            elif eof: time.sleep(0.05)
            ended, candidate = os.waitpid(pid, os.WNOHANG)
            if ended: status = candidate
    os.close(fd)
    exit_code = os.waitstatus_to_exitcode(status)
    receipt = {}; receipt_error = None
    try: receipt = json.loads((out/'receipt.json').read_text())
    except (OSError, ValueError) as error: receipt_error = str(error)
    ok = accepted(exit_code, timed_out, receipt, a.phase)
    if a.phase == 'reopen': ok = ok and receipt.get('ids') == first_receipt.get('ids')
    preservation_error = None
    try: preserve_old()
    except RuntimeError as error: preservation_error = str(error); ok = False
    result = dict(plan, ok=ok, exit=exit_code, timeout=timed_out, prompts=prompts,
                  elapsed_seconds=round(time.monotonic()-start, 2), receipt_error=receipt_error,
                  old_profile_preservation_error=preservation_error, unlock='production terminal fallback in owned PTY',
                  native_acceptance='requires separate capture review and domain verification')
    (out/'profile-after.json').write_text(json.dumps(inventory(PROFILE), indent=2)+'\n')
    (out/'process.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if ok else 1)
if __name__ == '__main__': main()
