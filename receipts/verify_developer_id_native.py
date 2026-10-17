#!/usr/bin/env python3
"""Read-only checks for signed archive launch and fresh-process reopening."""
from pathlib import Path
import argparse
import hashlib
import json
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPTS = ROOT / 'receipts'
PACK = RECEIPTS / 'developer_id_native_20261017'


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    ledger = read(RECEIPTS / 'developer-id-native-qualification_20261017.json')
    assert ledger['accepted'] and ledger['native_processes'] == 2
    for name, expected in ledger['artifacts'].items():
        assert sha(RECEIPTS / name) == expected, name
    package = read(PACK / 'package-verification.json')
    manifest = read(PACK / 'package-manifest.json')
    assert package['sha256'] == ledger['archive_sha256']
    if args.archive:
        assert sha(args.archive) == ledger['archive_sha256']
    assert package['bundle_verified'] and package['archive_roundtrip_verified']
    assert package['signing'] == 'developer-id' and not package['notarized']
    assert package['source_revision'] == ledger['source'] and package['profile'] == 'debug'
    for signature in [package['bundle_signature'], package['developer_signatures']['cleromancy-bin']]:
        assert signature['team'] == 'HWVZ5S7ACJ'
        assert signature['hardened_runtime'] and signature['timestamp']
        assert signature['authorities'][0] == 'Developer ID Application: Mark Boykim (HWVZ5S7ACJ)'
        assert signature['authorities'][-1] == 'Apple Root CA'
    old = read(RECEIPTS / 'natal_aspects_20261017/installed-reopen-01/receipt.json')
    report = read(PACK / 'native-review.json')
    assert report['ok'] and report['baseline_receipt_sha256'] == sha(RECEIPTS / 'natal_aspects_20261017/installed-reopen-01/receipt.json')
    assert len(report['captures']) == ledger['reviewed_originals'] == 6
    for phase in ['launch-01', 'reopen-01']:
        process = read(PACK / phase / 'process.json')
        receipt = read(PACK / phase / 'receipt.json')
        assert process['exit'] == 0 and not process['timeout'] and process['failure'] is None
        assert process['source'] == ledger['source']
        assert process['terminal_answers'] == 0 and process['offline_ip_deny'] and process['public_fixture']
        assert process['graphical_provider_processes']
        assert process['scenario_sha256'] == sha(PACK / 'signed-reopen.scn')
        assert process['payload_sha256'] == manifest['binaries']['cleromancy-bin']['sha256']
        assert process['entry_sha256'] == manifest['files']['Contents/MacOS/Cleromancy']['sha256']
        assert receipt['ok'] and receipt['error'] is None
        assert receipt['readings'] == old['readings'] and len(receipt['readings']) == 6
        assert receipt['reflections'] == old['reflections'] and len(receipt['reflections']) == 2
        assert receipt['shared'] == old['shared']
    for review in report['captures']:
        capture = PACK / review['path']
        assert sha(capture) == review['sha256']
        assert struct.unpack('>II', capture.read_bytes()[16:24]) == (review['width'], review['height'])
    signatures = read(PACK / 'post-launch-signatures.json')
    assert len(signatures) == 6 and all(item['exit'] == 0 for item in signatures)
    prior = read(PACK / 'prior-receipts.json')
    baseline = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', ledger['baseline'], '--', 'receipts'], cwd=ROOT, text=True).splitlines()
    assert set(baseline) == set(prior) and len(prior) == ledger['prior_tracked_receipt_paths'] == 1249
    for name, expected in prior.items():
        assert sha(ROOT / name) == expected, name
    print('Verified signed archive: two graphical native processes, six reviewed originals, six exact readings/two reflections, exact shared exports and all prior tracked receipts preserved. Notarization and distribution remain separate.')


if __name__ == '__main__':
    main()
