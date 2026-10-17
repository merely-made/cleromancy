#!/usr/bin/env python3
"""Read-only verification of the bounded natal-aspects and packaging evidence."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPTS = ROOT / 'receipts'
PACK = RECEIPTS / 'natal_aspects_20261017'
SOURCE = '3fa5a6db87cfa34df9fc1de0a0877181c9b4526f'


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, help='Optional actual ZIP to compare with the recorded archive SHA-256')
    args = parser.parse_args()
    ledger = read(RECEIPTS / 'natal-aspects-qualification_20261017.json')
    assert ledger['source'] == SOURCE
    for name, expected in ledger['artifacts'].items():
        assert sha(RECEIPTS / name) == expected, name
    prior = read(PACK / 'prior-receipts.json')
    baseline_paths = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', '8472bc8', '--', 'receipts'], cwd=ROOT, text=True
    ).splitlines()
    assert set(baseline_paths).issubset(prior)
    assert len(baseline_paths) == ledger['preserved_tracked_receipt_paths'] == 1154
    assert len(prior) == ledger['preserved_prior_receipt_paths'] == 1550
    for name, expected in prior.items():
        if name in baseline_paths or (ROOT / name).exists():
            assert sha(ROOT / name) == expected, name
    build = read(PACK / 'native-build.json')
    assert build['source'] == SOURCE and build['features'] == 'default'
    for name, expected in build['files'].items():
        blob = subprocess.check_output(['git', 'show', SOURCE + ':' + name], cwd=ROOT)
        assert hashlib.sha256(blob).hexdigest() == expected, name
    compiled = read(PACK / 'compiled-inputs.json')
    assert compiled['source'] == SOURCE and compiled['count'] == len(compiled['files']) > 100
    assert all(build['files'][name] == value for name, value in compiled['files'].items())
    software = read(PACK / 'software/results.json')
    assert len(software) == ledger['software_gate_count'] == 11
    for name, result in software.items():
        assert result['exit'] == 0 and result['source'] == SOURCE, name
    for name, counts in ledger['software_counts'].items():
        text = (PACK / 'software' / (name + '.log')).read_text()
        matches = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', text)
        assert matches and all(int(m[1]) == 0 for m in matches), name
        assert sum(int(m[0]) for m in matches) == counts['passed'] > 0
        assert sum(int(m[2]) for m in matches) == counts['ignored']
    native = read(PACK / 'native-qualification.json')
    assert native['ok'] and native['source'] == SOURCE
    assert len(native['runs']) == ledger['native_processes'] == 3
    assert len(native['captures']) == ledger['reviewed_originals'] == 10
    manifest = read(PACK / 'package-04/Cleromancy Aspects Candidate.app/Contents/Resources/package-manifest.json')
    assert manifest['source_revision'] == SOURCE
    tooling = read(PACK / 'packaging-tool-source.json')
    assert tooling['source'] == ledger['packaging_tool_source']
    for name, expected in tooling['files'].items():
        blob = subprocess.check_output(['git', 'show', tooling['source'] + ':' + name], cwd=ROOT)
        assert hashlib.sha256(blob).hexdigest() == expected
    assert manifest['packager_sha256'] == tooling['files']['packaging/macos/package.py']
    assert manifest['files']['Contents/Resources/INSTALL.txt']['sha256'] == tooling['files']['packaging/macos/INSTALL.txt']
    assert manifest['input_binaries']['cleromancy-bin']['sha256'] == build['binary_sha256']
    payload_sha = manifest['binaries']['cleromancy-bin']['sha256']
    created = read(PACK / 'native-create-01/receipt.json')
    readings = {v['id']: v for v in created['readings']}
    assert len(readings) == 6 and sum(v.get('aspects') is not None for v in readings.values()) == 2
    original = read(RECEIPTS / 'natal_cap_20261017/native-reopen-final-01/receipt.json')
    assert len(original['readings']) == 4
    assert all(readings[value['id']] == value for value in original['readings'])
    assert created['reflections'] == original['reflections']
    for run in native['runs']:
        folder = PACK / run['run']
        process = read(folder / 'process.json')
        receipt = read(folder / 'receipt.json')
        assert process == run['process']
        assert process['exit'] == 0 and not process['timeout'] and process['offline_ip_deny']
        assert process['prompts'] == 1
        assert process['binary_sha256'] == payload_sha
        assert receipt['ok'] and receipt['error'] is None
        assert {v['id']: v for v in receipt['readings']} == readings
        assert receipt['reflections'] == created['reflections']
        if run['run'] != 'native-create-01':
            for field in ('births', 'subjects'):
                assert receipt['shared'][field] == original['shared'][field]
            assert all(value in receipt['shared']['associations'] for value in original['shared']['associations'])
    for capture in native['captures']:
        assert capture['review'] and sha(PACK / capture['path']) == capture['sha256']
    diagnostic = read(PACK / 'unlock-01/receipt.json')
    assert not diagnostic['ok'] and not ledger['graphical_unlock']['accepted']
    comparison = read(PACK / 'package-04-comparison.json')
    assert comparison['ok'] and comparison['qualified_payload_and_launcher_unchanged']
    assert comparison['payload_sha256'] == payload_sha
    assert comparison['final_archive'] == ledger['archive']['sha256']
    archive = ledger['archive']
    assert archive['bundle_verified'] and archive['archive_roundtrip_verified']
    assert archive['source_revision'] == SOURCE and archive['profile'] == 'debug'
    assert archive['signing'] == 'ad-hoc' and not archive['notarized']
    assert not ledger['public_release_ready']
    signing = read(PACK / 'signing-attempt.json')
    assert signing['developer_id_present'] and not signing['actual_signing_succeeded']
    assert signing['os_error'] == 'errSecInternalComponent'
    assert ledger['developer_id_attempt'] == signing
    if args.archive:
        assert sha(args.archive) == archive['sha256']
    print('Verified: frozen source, eleven software gates, ten reviewed captures, three native processes, exact retained readings, prior receipts, and bounded packaging evidence. Graphical unlock and public release remain unqualified.')


if __name__ == '__main__':
    main()
