#!/usr/bin/env python3
"""Read-only verification of retained C4 evidence; never opens resident files."""
import hashlib
import json
from pathlib import Path
import subprocess

repo = Path(__file__).resolve().parents[1]
evidence = repo / "receipts/c4_20261016"


def read(name):
    return json.loads((evidence / name).read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_bytes(rev, path):
    return subprocess.check_output(["git", "show", rev + ":" + path], cwd=repo)


prior = read("prior-receipt-preservation.json")
for item in prior["files"]:
    path = repo / item["path"]
    assert sha(path) == item["sha256"], item["path"]
    assert path.read_bytes() == git_bytes(prior["baseline"], item["path"])

inputs = read("ordered-compiled-inputs.json")
for item in inputs["files"]:
    assert sha(repo / item["path"]) == item["sha256"], item["path"]
    assert (repo / item["path"]).read_bytes() == git_bytes(inputs["source"], item["path"])

manifest = read("artifact-hashes.json")
for item in manifest["files"]:
    assert sha(evidence / item["path"]) == item["sha256"], item["path"]

captures = read("capture-reviews.json")
count = 0
for run in captures["runs"]:
    directory = repo / "receipts" / run["directory"]
    process = run["process"]
    receipt = run["receipt"]
    assert process["exit"] == 0 and not process["timeout"]
    assert receipt["ok"] and not any("still busy" in line for line in receipt["log"])
    assert process["binary_sha256"] == manifest["native_binary_sha256"]
    assert sha(repo / "receipts" / run["scenario"]) == process["scenario_sha256"]
    for item in run["files"]:
        assert sha(directory / item["path"]) == item["sha256"]
    for capture in run["captures"]:
        assert capture["review"] and sha(directory / capture["file"]) == capture["sha256"]
        count += 1
assert count == 11

before = read("before-native-01-domain.json")
retained = read("before-ordered-01-domain.json")
saved = read("after-ordered-save-01-domain.json")
reopened = read("after-ordered-reopen-lenses-01-domain.json")
assert before["details"] == retained["details"] == saved["details"] == reopened["details"]
assert len(before["details"]) == 3
assert len(retained["memory"]) == 2 and len(saved["memory"]) == 3
assert all(memory in saved["memory"] for memory in retained["memory"])
assert saved == reopened
view = saved["views"][0]
assert view["snapshot"]["recipe"]["definition"]["arrangement"]["kind"] == "grid.ordered"
assert view["paused"] and view["reduced_motion"]
assert len(view["positions"]) == 46 and len(view["roles"]) == len(view["foreground"]) == 3
native = read("native-ordered-01/ambient.json")
fresh = read("native-ordered-reopen-01/ambient.json")
lenses = read("native-ordered-lenses-01/ambient.json")
assert native["view"] == fresh["view"] == view
assert native["memory"] == fresh["memory"] == lenses["memory"]
assert lenses["view"]["families"] == ["history"]
assert all(run["error"] is None for run in [native, fresh, lenses])

software = json.loads((repo / "receipts/c4-software-qualification_20261016.json").read_text())
assert software["source_commit"] == inputs["source"]
assert all(gate["exit"] == 0 and gate["failed"] == 0 for gate in software["gates"])
assert software["formatting_debt"]["changed_since_c3"] == []
print(f"C4 evidence verified: {len(inputs['files'])} exact inputs, {len(prior['files'])} preserved prior paths, {count} reviewed originals, exact sealed data/view/memory reopen")
