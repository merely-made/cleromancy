# Custody and projection receiver: receipts

Receipts for [the plan](../2026-11-01/2026-10-29_custody_projection_receiver.md),
moved here on 2026-11-01 under DOC_POLICY §8. All evidence paths below are
retained on T-PC only (user `markik`); none is in this repository or on the
Windows laptop.

## 2026-10-29, receiving run

- Evidence root: `/home/markik/Code/testing/custody-contract-20261029`
- `root-split-receivers-check.log`: SHA-256
  `a682a07b8c8ecda91ba0c2be2d46ec41c84955dec038eeed694fe7f598d32088`
- `root-split-cleromancy-check.json`: lock wait, empty log.
- Resolved-edge audit: `testing/custody-projection-split-20261029/cleromancy-lock-normalized.json`

## 2026-10-29, completed focused qualification

- Run: `root-split-cleromancy-final`; log SHA-256
  `15457b1be832bb8fdec7bb57a05597a0074b54fb64cda2c2fd1f1e1e15683ae0`.
- Fixture lock audit report:
  `testing/custody-projection-split-20261029/cleromancy-final-fixture-lock.json`
- Limits retained: 3 GiB memory, zero swap, 150 percent CPU, one build job.
