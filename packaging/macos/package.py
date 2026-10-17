#!/usr/bin/env python3
"""Package an existing arm64 binary; never build, launch, or inspect user stores."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]
LAUNCHER = b'''#!/bin/sh
set -eu
bundle_bin=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PATH="$bundle_bin:/usr/bin:/bin:/usr/sbin:/sbin"
export PATH
exec "$bundle_bin/cleromancy-bin" "$@"
'''


def run(*args):
    try:
        return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, timeout=180).strip()
    except subprocess.TimeoutExpired as error:
        raise ValueError(f'{args[0]} timed out; any required keychain access remains unapproved') from error
    except subprocess.CalledProcessError as error:
        raise ValueError(f'{args[0]} failed ({error.returncode}): {error.output.strip()}') from error



def developer_signature(path):
    run('/usr/bin/codesign', '--verify', '--strict', '-R=anchor apple generic and certificate leaf[field.1.2.840.113635.100.6.1.13] exists', str(path))
    details = run('/usr/bin/codesign', '--display', '--verbose=4', str(path))
    authorities = [line[len('Authority='):] for line in details.splitlines() if line.startswith('Authority=')]
    team = next((line.split('=', 1)[1] for line in details.splitlines() if line.startswith('TeamIdentifier=')), None)
    timestamp = next((line.split('=', 1)[1] for line in details.splitlines() if line.startswith('Timestamp=')), None)
    runtime = any('flags=' in line and '(runtime)' in line for line in details.splitlines())
    if not authorities or not authorities[0].startswith('Developer ID Application:') or not team or team == 'not set' or not timestamp or not runtime:
        raise ValueError('Developer ID signing requires an Application certificate, trusted timestamp and hardened runtime')
    return {'authorities': authorities, 'team': team, 'timestamp': timestamp, 'hardened_runtime': runtime}


def sign(path, identity):
    flags = ['--timestamp=none'] if identity == '-' else ['--timestamp', '--options', 'runtime']
    run('/usr/bin/codesign', '--force', '--sign', identity, *flags, str(path))
    return None if identity == '-' else developer_signature(path)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def inspect_binary(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'Not a regular binary: {path}')
    if run('/usr/bin/lipo', '-archs', str(path)) != 'arm64':
        raise ValueError('Only thin macOS arm64 binaries are supported')
    deps = [line.strip().split(' (')[0] for line in
            run('/usr/bin/otool', '-L', str(path)).splitlines()[1:]]
    if any(not p.startswith(('/System/Library/', '/usr/lib/')) for p in deps):
        raise ValueError(f'Non-system dynamic dependency: {deps}')
    load = run('/usr/bin/otool', '-l', str(path))
    match = re.search(r'cmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform 1\s+minos ([\d.]+)', load)
    if not match:
        raise ValueError('Missing macOS LC_BUILD_VERSION')
    return {'sha256': digest(path), 'dependencies': deps, 'minimum_macos': match[1]}


def json_file(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def licenses(metadata, destination):
    """Copy only named legal files, never an arbitrary package directory."""
    destination.mkdir()
    for name in ('LICENSE-MIT', 'LICENSE-APACHE', 'THIRD_PARTY_LICENSES.md'):
        shutil.copyfile(ROOT / name, destination / name)
    for name in ('ibm-plex-mono', 'source-serif-4'):
        shutil.copyfile(ROOT / 'assets/fonts' / name / 'OFL.txt', destination / (name + '-OFL.txt'))
    rows = []
    for package in sorted(metadata['packages'], key=lambda p: p['id']):
        folder = Path(package['manifest_path']).parent
        candidates = set()
        # Git workspace members can inherit repository-root license files.
        for parent in (folder, *folder.parents):
            if parent == ROOT.parent or parent == Path.home() or parent == Path('/'):
                break
            for p in parent.iterdir():
                if p.is_file() and not p.is_symlink() and re.match(
                        r'^(LICENSE|LICENCE|COPYING|NOTICE|OFL|UNLICENSE|PROVENANCE)([._-].*)?$', p.name, re.I):
                    candidates.add(p)
            for dirname in ('LICENSES', 'licenses'):
                legal_dir = parent / dirname
                if legal_dir.is_dir() and not legal_dir.is_symlink():
                    candidates.update(p for p in legal_dir.iterdir() if p.is_file() and not p.is_symlink())
            if (parent / '.git').exists() or ((parent / '.cargo-ok').exists() and '/registry/' in str(parent)):
                break
            if parent.name == 'src' or parent.name == 'checkouts':
                break
        if package.get('license_file'):
            candidates.add(folder / package['license_file'])
        legal = []
        key = hashlib.sha256(package['id'].encode()).hexdigest()[:16]
        for p in sorted(candidates):
            if not p.is_file() or p.is_symlink():
                continue
            sha = digest(p)
            relative = f'{key}/{sha[:12]}-{p.name}'
            target = destination / relative
            target.parent.mkdir(exist_ok=True)
            shutil.copyfile(p, target)
            legal.append(relative)
        rows.append({'name': package['name'], 'version': package['version'],
                     'source': package.get('source'), 'repository': package.get('repository'),
                     'license': package.get('license'), 'texts': legal})
    json_file(destination / 'dependency-inventory.json', rows)
    return [f"{p['name']} {p['version']}" for p in rows if not p['texts']]


def verify(app):
    manifest = json.loads((app / 'Contents/Resources/package-manifest.json').read_text())
    expected = manifest['files']
    actual = {str(p.relative_to(app)) for p in app.rglob('*') if p.is_file()
              and '_CodeSignature' not in p.parts
              and p.name != 'package-manifest.json'}
    if actual != set(expected):
        raise ValueError(f'Bundle allowlist mismatch: {actual ^ set(expected)}')
    if any(p.is_symlink() for p in app.rglob('*')):
        raise ValueError('Bundle contains symlink')
    for name, record in expected.items():
        p = app / name
        if digest(p) != record['sha256'] or stat.S_IMODE(p.stat().st_mode) != record['mode']:
            raise ValueError(f'Bundle hash/mode mismatch: {name}')
    info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    if info['CFBundleExecutable'] != 'Cleromancy' or info['CFBundlePackageType'] != 'APPL':
        raise ValueError('Invalid bundle entry point')
    if (app / 'Contents/MacOS/Cleromancy').read_bytes() != LAUNCHER:
        raise ValueError('Unexpected launcher')
    for name in manifest['binaries']:
        binary = app / 'Contents/MacOS' / name
        if inspect_binary(binary) != manifest['binaries'][name]:
            raise ValueError(f'Binary requirements changed: {name}')
        run('/usr/bin/codesign', '--verify', '--strict', str(binary))
    if manifest.get('signing') == 'developer-id':
        for name, signature in manifest['developer_signatures'].items():
            if developer_signature(app / 'Contents/MacOS' / name) != signature:
                raise ValueError(f'Developer ID signature differs: {name}')
        bundle = developer_signature(app)
        if any(signature['team'] != bundle['team'] for signature in manifest['developer_signatures'].values()):
            raise ValueError('Bundle and payload signing teams differ')
    run('/usr/bin/plutil', '-lint', str(app / 'Contents/Info.plist'))
    run('/usr/bin/codesign', '--verify', '--deep', '--strict', str(app))
    return manifest


def package(args):
    if sys.platform != 'darwin':
        raise ValueError('Packaging requires macOS Apple command-line tools')
    if not re.fullmatch(r'[0-9a-f]{40}', args.source_revision):
        raise ValueError('Provide the full binary source revision; do not infer it from current HEAD')
    if not re.fullmatch(r'\d+\.\d+\.\d+', args.version):
        raise ValueError('Version must have three numeric components')
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9 ._-]{0,70}', args.app_name):
        raise ValueError('Unsafe app name')
    if not re.fullmatch(r'[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+', args.bundle_id):
        raise ValueError('Invalid bundle identifier')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    app = output / (args.app_name + '.app')
    macos = app / 'Contents/MacOS'
    resources = app / 'Contents/Resources'
    macos.mkdir(parents=True)
    resources.mkdir()
    inputs = {'cleromancy-bin': args.binary.resolve()}
    for item in args.helper:
        name, path = item.split('=', 1)
        if not re.fullmatch(r'[a-zA-Z0-9_-]+', name) or name.lower() in ('cleromancy', 'cleromancy-bin') or name.lower() in {n.lower() for n in inputs}:
            raise ValueError('Invalid or duplicate helper name')
        inputs[name] = Path(path).resolve()
    provenance = {}
    binaries = {}
    signatures = {}
    for name, source in inputs.items():
        provenance[name] = inspect_binary(source)
        shutil.copyfile(source, macos / name)
        (macos / name).chmod(0o755)
        if digest(macos / name) != provenance[name]['sha256']:
            raise ValueError('Binary changed during copy')
        signature = sign(macos / name, args.signing_identity)
        if signature: signatures[name] = signature
        binaries[name] = inspect_binary(macos / name)
    (macos / 'Cleromancy').write_bytes(LAUNCHER)
    (macos / 'Cleromancy').chmod(0o755)
    minimum = max((b['minimum_macos'] for b in binaries.values()), key=lambda v: tuple(map(int, v.split('.'))))
    info = {'CFBundleIdentifier': args.bundle_id, 'CFBundleName': args.app_name,
            'CFBundleDisplayName': args.app_name, 'CFBundleExecutable': 'Cleromancy',
            'CFBundlePackageType': 'APPL', 'CFBundleVersion': args.version,
            'CFBundleShortVersionString': args.version, 'LSMinimumSystemVersion': minimum,
            'NSHighResolutionCapable': True}
    (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(info, sort_keys=True))
    metadata = json.loads(args.metadata.read_text())
    missing = licenses(metadata, resources / 'Licenses')
    shutil.copyfile(Path(__file__).with_name('INSTALL.txt'), resources / 'INSTALL.txt')
    manifest = {'schema': 'cleromancy.macos-package/v1', 'source_revision': args.source_revision,
                'source_evidence': args.source_evidence, 'profile': args.profile,
                'input_binaries': provenance, 'binaries': binaries,
                'cargo_metadata_sha256': digest(args.metadata),
                'packager_sha256': digest(Path(__file__)),
                'signing': 'ad-hoc' if args.signing_identity == '-' else 'developer-id', 'notarized': False,
                'signing_identity': args.signing_identity, 'developer_signatures': signatures,
                'missing_dependency_license_texts': missing,
                'runtime_helpers': ['/bin/sh', '/usr/bin/osascript'],
                'qualification': 'Structural packaging only; no execution or graphical acceptance',
                'files': {}}
    for p in sorted(app.rglob('*')):
        if p.is_file():
            p.chmod(0o755 if p.parent == macos else 0o644)
            manifest['files'][str(p.relative_to(app))] = {'sha256': digest(p), 'mode': stat.S_IMODE(p.stat().st_mode)}
    json_file(resources / 'package-manifest.json', manifest)
    bundle_signature = sign(app, args.signing_identity)
    if bundle_signature and any(v['team'] != bundle_signature['team'] for v in signatures.values()):
        raise ValueError('Bundle and payload signing teams differ')
    verify(app)
    archive = output / f'Cleromancy-{args.version}-macos-arm64-{args.profile}.zip'
    # Sorted, normalized ZIP entries without host xattrs/resource forks or times.
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for p in sorted(app.rglob('*')):
            if not p.is_file():
                continue
            entry = zipfile.ZipInfo(str(p.relative_to(output)), (2020, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = (stat.S_IFREG | stat.S_IMODE(p.stat().st_mode)) << 16
            z.writestr(entry, p.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    with tempfile.TemporaryDirectory(prefix='cleromancy-archive-check-') as temporary:
        run('/usr/bin/ditto', '-x', '-k', str(archive), temporary)
        verify(Path(temporary) / app.name)
    receipt = {'archive': archive.name, 'sha256': digest(archive),
               'archive_roundtrip_verified': True, 'bundle_verified': True,
               'source_revision': args.source_revision, 'profile': args.profile,
               'input_binaries': provenance, 'missing_dependency_license_texts': missing,
               'notarized': False, 'signing': manifest['signing'], 'developer_signatures': signatures,
               'bundle_signature': bundle_signature, 'executed': False}
    json_file(output / 'verification.json', receipt)
    (output / 'SHA256SUMS').write_text(f"{receipt['sha256']}  {archive.name}\n")
    print(json.dumps({k: v for k, v in receipt.items() if k not in ('input_binaries', 'missing_dependency_license_texts')}, indent=2))
    print(f'License inventory: {len(metadata["packages"])} packages; {len(missing)} missing text sets (see verification.json)')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', type=Path, metavar='APP')
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--metadata', type=Path, help='Full cargo metadata --locked --offline --format-version 1 JSON')
    parser.add_argument('--source-revision')
    parser.add_argument('--source-evidence', help='Public, non-personal provenance description; included in bundle')
    parser.add_argument('--profile', choices=('debug', 'release', 'unknown'))
    parser.add_argument('--version', default='0.1.0')
    parser.add_argument('--signing-identity', default='-', help='Developer ID Application identity fingerprint, or - for ad-hoc. Developer ID requests a trusted Apple timestamp and hardened runtime.')
    parser.add_argument('--app-name', default='Cleromancy Packaging Candidate')
    parser.add_argument('--bundle-id', default='dev.merely.cleromancy.packaging.candidate20261017')
    parser.add_argument('--helper', action='append', default=[], metavar='NAME=ARM64_BINARY')
    parser.add_argument('--output', type=Path, help='Must be a new directory')
    args = parser.parse_args()
    if args.verify:
        verify(args.verify.resolve())
        print('Bundle verified (structural checks only; not launched)')
    else:
        for name in ('binary', 'metadata', 'source_revision', 'source_evidence', 'profile', 'output'):
            if getattr(args, name) is None:
                parser.error('--' + name.replace('_', '-') + ' is required')
        package(args)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
