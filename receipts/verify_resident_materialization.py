#!/usr/bin/env python3
"""Verify retained shared-reader evidence without opening a resident store."""
import hashlib
import json
from pathlib import Path
import subprocess

repo = Path(__file__).resolve().parents[1]
evidence = repo / 'receipts/resident_materialization_20261018'


def read(name):
    return json.loads((evidence / name).read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_bytes(rev, path):
    return subprocess.check_output(['git', 'show', rev + ':' + path], cwd=repo)


prior = read('prior-receipt-preservation.json')
for path, expected in prior['files'].items():
    assert sha(repo / path) == expected, path
    assert (repo / path).read_bytes() == git_bytes(prior['baseline'], path), path
inputs = read('candidate-inputs.json')
for path, expected in inputs['files'].items():
    assert sha(repo / path) == expected, path
    assert (repo / path).read_bytes() == git_bytes(inputs['source'], path), path
manifest = read('artifact-hashes.json')
for path, expected in manifest['files'].items():
    assert sha(evidence / path) == expected, path
runs = [read(run + '/process.json') for run in ('first-01', 'reopen-01')]
receipts = [read(run + '/receipt.json') for run in ('first-01', 'reopen-01')]
for process, receipt in zip(runs, receipts):
    assert process['exit'] == 0 and not process['timeout'] and process['ok']
    assert receipt['ok'] and receipt['catalog_ready'] and receipt['ids']
    assert process['binary_sha256'] == manifest['native_binary_sha256']
    assert process['offline_ip_deny'] and process['prompts'] == 1
assert receipts[0]['ids'] == receipts[1]['ids']
assert (evidence / 'first-01/cards.json').read_bytes() == (evidence / 'reopen-01/cards.json').read_bytes()
count = 0
for name in ('capture-reviews-first.json', 'capture-reviews-reopen.json'):
    for review in read(name):
        assert review['review'] and sha(evidence / review['file']) == review['sha256']
        count += 1
assert count == 11
failed = read('diagnostic-first-01/process.json')
assert failed['exit'] == 0 and not failed['ok']
assert not read('diagnostic-first-01/receipt.json')['ok']
software = read('software.json')
assert software['source'] == inputs['source']
assert all(gate['exit'] == 0 for gate in software['accepted_gates'])
assert {gate['name'] for gate in software['accepted_gates']} >= {'atomic-regression', 'resident', 'desktop-build', 'personal-sync-check', 'core-default'}
print(f'Shared resident materialization verified: {len(prior["files"])} prior receipts preserved, {len(inputs["files"])} source inputs, {count} reviewed captures, exact IDs/cards after reopening; failed diagnostic retained')
