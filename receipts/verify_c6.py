#!/usr/bin/env python3
"""Read-only C6 source, capture, replay and preservation receipt verifier."""
import hashlib
import json
from pathlib import Path
import subprocess

repo = Path(__file__).resolve().parents[1]
evidence = repo / "receipts/c6_20261016"


def read(name):
    return json.loads((evidence / name).read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(revision, path):
    return subprocess.check_output(["git", "show", revision + ":" + path], cwd=repo)


prior = read("prior-receipt-preservation.json")
assert len(prior["files"]) == 424
for item in prior["files"]:
    path = repo / item["path"]
    assert sha(path) == item["sha256"], item["path"]
    assert path.read_bytes() == git(prior["baseline"], item["path"])

inputs = read("qualified-source-inputs.json")
assert len(inputs["files"]) == 198
for item in inputs["files"]:
    assert sha(repo / item["path"]) == item["sha256"], item["path"]
    assert (repo / item["path"]).read_bytes() == git(inputs["source"], item["path"])

for item in read("artifact-hashes.json")["files"]:
    assert sha(evidence / item["path"]) == item["sha256"], item["path"]

for file, counts in [("qualified-root-gates.json", [0, 120, 0, 160, 0, 4]),
                     ("qualified-core-gates.json", [36, 37, 38])]:
    gates = read(file)
    assert [g["passed"] for g in gates] == counts
    for gate in gates:
        assert gate["source"] == inputs["source"]
        assert gate["exit"] == gate["failed"] == gate["ignored"] == 0
        assert sha(evidence / gate["log"]) == gate["sha256"]

reviews = read("capture-reviews.json")
count = 0
for run in reviews["runs"]:
    process, receipt = run["process"], run["receipt"]
    assert process["exit"] == 0 and not process["timeout"] and receipt["ok"]
    native_inputs = read(run["input_manifest"])
    assert process["binary_sha256"] == native_inputs["native_sha256"]
    assert sha(repo / "receipts" / run["scenario"]) == process["scenario_sha256"]
    assert not any("still busy" in line for line in receipt["log"])
    if process["phase"] == "reopen":
        assert process["offline_ip_deny"]
        assert process["sandbox_policy"] == read("offline-control.json")["profile"]
    for capture in run["captures"]:
        assert capture["review"]
        assert sha(evidence / run["directory"] / capture["file"]) == capture["sha256"]
        count += 1
assert count == 10
assert read("offline-control.json")["exit"] != 0

baseline = read("before-native-domain.json")
staged = read("before-qualified-native-domain.json")
global_saved = read("after-qualified-global-save-domain.json")
global_fresh = read("after-qualified-global-reopen-domain.json")
weather_saved = read("after-qualified-weather-save-domain.json")
weather_fresh = read("after-qualified-weather-reopen-domain.json")
assert [len(s["details"]) for s in [baseline, staged, global_saved, weather_saved]] == [6, 8, 7, 8]
for previous, current in [(baseline, global_saved), (global_saved, weather_saved)]:
    assert all(detail in current["details"] for detail in previous["details"])
    assert previous["views"] == current["views"]
    assert previous["memory"] == current["memory"]
assert staged == weather_saved == global_fresh == weather_fresh
for kind in ["global", "weather"]:
    first = evidence / ("native-" + kind + "-first-01") / "situated-reading.json"
    fresh_name = "native-qualified-weather-reopen-04" if kind == "weather" else "native-qualified-global-reopen-01"
    fresh = evidence / fresh_name / "situated-reading.json"
    assert first.read_bytes() == fresh.read_bytes()
    detail = json.loads(first.read_text())
    situation = detail["situation"]
    assert situation["instant_ms"] == detail["session"]["created_at_ms"]
    assert situation["session_id"] == detail["session"]["id"]
    assert len(situation["sky"]["positions"]) == 10
    assert len(detail["readings"]) == 3 and len(detail["reflections"]) == 1
    if kind == "global":
        assert situation["place"] is situation["local"] is situation["weather"] is None
        assert {v["fact"] for v in situation["unavailable"]} >= {"Astronomical season", "Sunrise and sunset", "Planetary hours", "Weather"}
    else:
        weather = situation["weather"]
        assert situation["place"] == weather["request_place"]
        assert weather["request_url"] == weather["final_url"]
        raw = json.loads(weather["response_body"])
        assert round(raw["current"]["temperature_2m"] * 1000) == weather["values"]["temperature_millicelsius"]
        assert situation["local"]["astronomical_season"] == "Autumn"
        assert situation["local"]["planetary_hour"] is not None
        assert read("offline-control.json")["exit"] == 7

receiving = read("mere-receiving-qualification.json")
assert receiving["core"] == inputs["source"]
assert receiving["passed"] == 7 and receiving["failed"] == 0 and receiving["ignored"] == 1
assert read("format-debt.json")["new_reported_files"] == []
print(f"C6 verified: 198 source inputs, 424 preserved prior paths, {count} reviewed originals, exact offline situation reopens, unchanged earlier occasions/view/memory, seven receiving owner tests")
