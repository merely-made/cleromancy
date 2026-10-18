#!/usr/bin/env python3
"""Fresh public identity-only staging; provisioning is an explicit separate action."""
import argparse, hashlib, json, pathlib, subprocess
BASE = pathlib.Path(__file__).resolve().parent
PROFILE = BASE / 'profile-first-01'
MARKER = b'cleromancy-c1 public synthetic fixture/v1\n'
PERSONA = '00000000-0000-0000-0000-00c120261013'
OLD = pathlib.Path('/Users/markik/Code/testing/cleromancy/aspects-release-20261017')

def require(condition, message):
    if not condition: raise RuntimeError(message)

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''): h.update(chunk)
    return h.hexdigest()

def inventory(root):
    require(not root.is_symlink(), f'Symlink root refused: {root}')
    result = {}
    for p in sorted(root.rglob('*')):
        require(not p.is_symlink(), f'Symlink refused: {p}')
        if p.is_file(): result[str(p.relative_to(root))] = digest(p)
    return result

def old_inventory():
    roots = sorted({p.parent for p in OLD.rglob('.c1-public-synthetic')})
    require(bool(roots), 'No old public profiles found')
    return {str(root): inventory(root) for root in roots}

def preserve_old():
    expected = json.loads((BASE/'old-profile-sha256-before.json').read_text())
    require(old_inventory() == expected, 'Old profile bytes changed')
    return sum(map(len, expected.values()))

def validate_identity_only():
    require((PROFILE/'.c1-public-synthetic').read_bytes() == MARKER, 'Wrong fixture marker')
    expected = {
        '.c1-public-synthetic', 'ordinary-vault/vault.json',
        'shared/identity/master.seed', 'shared/identity/wallet.json',
        'shared/identity/device-roster.json', f'shared/personas/{PERSONA}/wallet.json',
        f'shared/personas/{PERSONA}/private-epoch-bridge.json',
    }
    files = inventory(PROFILE)
    require(set(files) == expected, f'Identity-only file set differs: {sorted(set(files) ^ expected)}')
    require((PROFILE/'shared/identity/master.seed').read_bytes() == bytes([0xc1])*32, 'Wrong public identity seed')
    process = json.loads((BASE/'provisioning-process.json').read_text())
    require(process.get('exit') == 0, 'Provisioning helper did not exit 0')
    receipt = json.loads((BASE/'provisioning.json').read_text())
    require(receipt.get('schema') == 'cleromancy.c1-ordinary-provisioning/v1', 'Wrong provisioning schema')
    for key in ['public_fixture', 'standard_passphrase_vault', 'typed_wallet_bootstrap', 'owner_startup_unchanged']:
        require(receipt.get(key) is True, f'Missing provisioning fact: {key}')
    for key in ['graph_seeded', 'reservoir_opened']:
        require(receipt.get(key) is False, f'Unexpected domain provisioning: {key}')
    require(receipt.get('persona') == PERSONA and receipt.get('profile') == 'c1-public-synthetic', 'Wrong fixture identity')
    require(pathlib.Path(receipt['shared_root']).resolve() == PROFILE/'shared', 'Wrong shared root')
    require(pathlib.Path(receipt['vault_dir']).resolve() == PROFILE/'ordinary-vault', 'Wrong vault root')
    return files

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('mode', choices=['stage', 'provision', 'check'])
    ap.add_argument('--helper', type=pathlib.Path)
    a = ap.parse_args()
    if a.mode == 'stage':
        before = old_inventory()
        with (BASE/'old-profile-sha256-before.json').open('x') as f: json.dump(before, f, indent=2); f.write('\n')
        PROFILE.mkdir()
        (PROFILE/'.c1-public-synthetic').write_bytes(MARKER)
        require(inventory(PROFILE) == {'.c1-public-synthetic': hashlib.sha256(MARKER).hexdigest()}, 'Staging was not empty')
        result = {'ok': True, 'state': 'marked-empty-unprovisioned', 'identity_only_ready': False,
                  'profile': str(PROFILE), 'old_profile_files_preserved': preserve_old()}
        (BASE/'staging.json').write_text(json.dumps(result, indent=2)+'\n')
    elif a.mode == 'provision':
        require(a.helper is not None, '--helper is required')
        helper = a.helper.resolve(strict=True)
        require(set(inventory(PROFILE)) == {'.c1-public-synthetic'}, 'Provisioning requires a newly marked empty fixture')
        require(not (BASE/'provisioning-process.json').exists(), 'Provisioning already attempted; retain attempt and use a new fixture')
        require(not (BASE/'provisioning.json').exists(), 'Provisioning receipt already exists')
        preserve_old()
        command = [str(helper), '/tmp/srm-20261018-clero.sock', str(PROFILE), 'provision-ordinary']
        with (BASE/'provisioning.log').open('xb') as log:
            completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=log, timeout=120)
        result = {'command': command, 'exit': completed.returncode, 'helper_sha256': digest(helper)}
        (BASE/'provisioning-process.json').write_text(json.dumps(result, indent=2)+'\n')
        (BASE/'provisioning-stdout.txt').write_bytes(completed.stdout)
        require(completed.returncode == 0, 'Provisioning helper did not exit 0')
        receipt = json.loads(completed.stdout)
        (BASE/'provisioning.json').write_text(json.dumps(receipt, indent=2)+'\n')
        result.update(ok=True, state='identity-only-ready', files=validate_identity_only(), old_profile_files_preserved=preserve_old())
        (BASE/'identity-only-precondition.json').write_text(json.dumps(result, indent=2)+'\n')
    else:
        result = {'ok': True, 'state': 'identity-only-ready', 'files': validate_identity_only(), 'old_profile_files_preserved': preserve_old()}
    print(json.dumps(result, indent=2))
if __name__ == '__main__': main()
