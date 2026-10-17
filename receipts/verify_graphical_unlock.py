#!/usr/bin/env python3
"""Read-only verification of the final archive's graphical unlock witness."""
from pathlib import Path
import hashlib
import json
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPTS = ROOT / 'receipts'
PACK = RECEIPTS / 'graphical_unlock_20261017'


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    ledger = read(RECEIPTS / 'graphical-unlock-qualification_20261017.json')
    assert ledger['accepted']
    for name, expected in ledger['artifacts'].items():
        assert sha(RECEIPTS / name) == expected, name
    process = read(PACK / 'process.json')
    receipt = read(PACK / 'receipt.json')
    baseline = read(RECEIPTS / 'natal_aspects_20261017/installed-reopen-01/receipt.json')
    assert process['exit'] == 0 and not process['timeout'] and process['failure'] is None
    assert process['source'] == ledger['source']
    assert process['terminal_answers'] == 0 and process['offline_ip_deny'] and process['public_fixture']
    dialogs = process['graphical_provider_processes']
    assert dialogs and all('Identity vault' in item['command'] for item in dialogs)
    assert any('incorrect passphrase' in item['command'] for item in dialogs)
    assert receipt['ok'] and receipt['error'] is None
    assert receipt['readings'] == baseline['readings'] and len(receipt['readings']) == 6
    assert receipt['reflections'] == baseline['reflections'] and len(receipt['reflections']) == 2
    assert receipt['shared'] is None  # This scenario does not export shared resources.
    previous = read(RECEIPTS / 'natal-aspects-qualification_20261017.json')
    assert previous['archive']['sha256'] == ledger['archive_sha256']
    assert process['payload_sha256'] == 'f9c9c5f5eb89a0f8658d7c66d528370e5ea5ccba5afda7f6fa7a78fcd7fcd01a'
    assert process['entry_sha256'] == 'fa0802759623b56ed4056ceb48a2d69e8776c2c84195fb866faabfbbbe3d5618'
    assert process['scenario_sha256'] == sha(RECEIPTS / 'graphical-unlock-reopen.scn')
    review = read(PACK / 'capture-review.json')
    capture = PACK / review['capture']
    assert review['accepted'] and review['sha256'] == sha(capture)
    assert struct.unpack('>II', capture.read_bytes()[16:24]) == (review['width'], review['height'])
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', ledger['baseline'], '--', 'receipts'], cwd=ROOT, text=True).splitlines()
    assert len(paths) == ledger['prior_tracked_receipt_paths']
    for name in paths:
        original = subprocess.check_output(['git', 'show', ledger['baseline'] + ':' + name], cwd=ROOT)
        assert (ROOT / name).read_bytes() == original, name
    print('Graphical unlock verified: production dialog/retry, zero terminal answers, final archive launcher, six exact readings, two exact reflections, one reviewed original; all prior tracked receipts unchanged.')


if __name__ == '__main__':
    main()
