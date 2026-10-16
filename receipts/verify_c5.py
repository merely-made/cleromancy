#!/usr/bin/env python3
"""Read-only C5 evidence check. No resident or application store is opened."""
import hashlib
import json
from pathlib import Path
import subprocess

repo = Path(__file__).resolve().parents[1]
evidence = repo / "receipts/c5_20261016"


def read(name):
    return json.loads((evidence / name).read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(rev, path):
    return subprocess.check_output(["git", "show", rev + ":" + path], cwd=repo)


prior = read("prior-receipt-preservation.json")
assert len(prior["files"]) == 333
for item in prior["files"]:
    assert sha(repo / item["path"]) == item["sha256"], item["path"]
    assert (repo / item["path"]).read_bytes() == git(prior["baseline"], item["path"])

native = read("qualified-source-inputs.json")
software = read("software-inputs.json")
assert software["same_production_inputs"]
for item in native["files"]:
    assert hashlib.sha256(git(native["source"], item["path"])).hexdigest() == item["sha256"]
    if not item["path"].startswith("tests/"):
        assert sha(repo / item["path"]) == item["sha256"], item["path"]
for item in software["files"]:
    assert sha(repo / item["path"]) == item["sha256"], item["path"]
    assert (repo / item["path"]).read_bytes() == git(software["source"], item["path"])

manifest = read("artifact-hashes.json")
for item in manifest["files"]:
    assert sha(evidence / item["path"]) == item["sha256"], item["path"]

reviews = read("capture-reviews.json")
count = 0
for run in reviews["runs"]:
    process, receipt = run["process"], run["receipt"]
    assert process["exit"] == 0 and not process["timeout"] and receipt["ok"]
    assert process["binary_sha256"] == native["native_sha256"]
    assert sha(repo / "receipts" / run["scenario"]) == process["scenario_sha256"]
    assert not any("still busy" in line for line in receipt["log"])
    for capture in run["captures"]:
        assert capture["review"]
        assert sha(repo / "receipts" / run["directory"] / capture["file"]) == capture["sha256"]
        count += 1
assert count == 10

before = read("before-qualified-native-domain.json")
saved = read("after-qualified-save-domain.json")
reopened = read("after-qualified-reopen-domain.json")
assert len(before["details"]) == 4 and len(saved["details"]) == 5
assert all(detail in saved["details"] for detail in before["details"])
assert before["views"] == saved["views"] and before["memory"] == saved["memory"]
assert saved == reopened
first_path = evidence / "native-first-02/shuffled-reading.json"
fresh_path = evidence / "native-reopen-01/shuffled-reading.json"
assert first_path.read_bytes() == fresh_path.read_bytes()
proof = json.loads(first_path.read_text())
readings = proof["readings"]
assert len(readings) == len({r["candidate_id"] for r in readings}) == 3
draws = [r["receipt"]["shuffle"] for r in readings]
assert all(d["shuffle"] == draws[0]["shuffle"] and d["count"] == 3 for d in draws)
assert [d["index"] for d in draws] == [0, 1, 2]
assert all(d["orientation"] in ["upright", "reversed"] for d in draws)
assert any(d["orientation"] == "reversed" for d in draws)
assert len(proof["reflections"]) == 1
os_saved = read("after-os-save-domain.json")
os_reopened = read("after-os-reopen-domain.json")
assert len(os_saved["details"]) == 6
assert all(detail in os_saved["details"] for detail in saved["details"])
assert os_saved["views"] == saved["views"] and os_saved["memory"] == saved["memory"]
assert os_saved == os_reopened
os_first = evidence / "native-os-first-01/shuffled-reading.json"
os_fresh = evidence / "native-os-reopen-01/shuffled-reading.json"
assert os_first.read_bytes() == os_fresh.read_bytes()
os_proof = json.loads(os_first.read_text())
os_readings = os_proof["readings"]
assert len(os_readings) == len({r["candidate_id"] for r in os_readings}) == 3
os_draws = [r["receipt"]["shuffle"] for r in os_readings]
assert [d["index"] for d in os_draws] == [0, 1, 2]
assert all(d["count"] == 3 and d["shuffle"] == os_draws[0]["shuffle"] for d in os_draws)
assert all(d["orientation"] in ["upright", "reversed"] for d in os_draws)
os_source = os_draws[0]["shuffle"]["source"]
assert os_source["kind"] == "operating_system" and len(os_source["seed_hex"]) == 64
assert len(os_proof["reflections"]) == 1
assert (repo / "tests/fixtures/pre-c5-selections.json").read_bytes() == (evidence / "pre-c5-selections.json").read_bytes()
ledger = json.loads((repo / "receipts/c5-shuffled-qualification_20261016.json").read_text())
assert ledger["native_source"] == native["source"]
assert ledger["software_source"] == software["source"]
assert all(g["exit"] == 0 and g["failed"] == 0 for g in ledger["gates"])
assert ledger["formatting_debt"]["changed_since_c4"] == []
assert read("isocosm-current-consumer.json")["manifest_search_exit"] == 1
print(f"C5 verified: {len(software['files'])} source inputs, 333 preserved prior paths, ten reviewed originals, byte-equal public-seed and OS-random reading reopens, exact prior occasions/view/memory")
