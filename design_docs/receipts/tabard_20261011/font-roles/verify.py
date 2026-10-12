# Copyright 2026 Mark AB (markik)
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify the committed font-role acceptance evidence; performs no mutations."""
from pathlib import Path
import hashlib, json, re
root = Path(__file__).resolve().parent
ledger = json.loads((root / "qualification.json").read_text())
receipts = []
frames = 0
captures = 0
for name in ("native-qualified", "native-reopen"):
    process = json.loads((root / name / "process.json").read_text())
    receipt = json.loads((root / name / "receipt.json").read_text())
    assert process["success"] and not process["timeout"] and receipt["ok"]
    assert process["binary_sha256"] == ledger["binary_sha256"]
    assert (root / name / "scenario.done").read_text().startswith("RESULT ok")
    lane = (root / name / "lane.done").read_text()
    assert "0 blank" in lane and "FAIL:" not in lane
    frames += int(re.search(r"presentation: (\d+) frames", lane).group(1))
    captures += process["captures"]
    receipts.append(receipt)
assert frames == ledger["native"]["presented_frames"] == 123
assert captures == ledger["native"]["reviewed_captures"] == 9
assert receipts[0]["ids"] == receipts[1]["ids"] == ledger["native"]["durable_ids"]
for key in ("sessions", "readings", "reflections"):
    assert receipts[0][key] == receipts[1][key] == ledger["native"]["counts"][key]
assert (root / "native-qualified/cards.json").read_bytes() == (root / "native-reopen/cards.json").read_bytes()
pre = json.loads((root / "pre-reopen-files.json").read_text())
for name, value in pre.items():
    assert hashlib.sha256((root / "qualified-profile" / name).read_bytes()).hexdigest() == value["sha256"]
for name, digest in ledger["native"]["capture_sha256"].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == digest
assert pre["appearance.json"]["contents"]["fonts"] == ledger["native"]["saved_fonts"]
print("Verified: 2 accepted processes, 9 original captures, preserved requests/files/cards and durable IDs")
