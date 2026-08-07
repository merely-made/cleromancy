# Cleromancy A22: resident Pattern sync bridge

**Date:** 2026-08-07
**Scope:** carry the complete saved Pattern occasion through the existing
selected personal-sync mapping, and expose that mapping through the resident
Cleromancy authority.

## Contract

`ContextsAndReadings` now carries the complete replayable constellation:
context, field, reading, saved reading session, authored spread, astrology
chart, derived astrology facts, and Pattern occasion. It remains an explicit
selection, with `Off` as the default. Reflections still require
`ContextsReadingsAndReflections`.

The batch advances to `cleromancy.sync-batch/v5`. Facts retain their generated
from relationship to the exact chart. A Pattern occasion retains its collection
members, including its saved facts and reading session. Export refuses a fact
without its selected chart, an invalid fact/chart pair, or an occasion with a
missing selected member.

Import decodes and validates all selected values before mutating the local
graph. It verifies every fact set against its chart, requires each selected
chart to retain at least one fact set, and checks each occasion member is
present. It then restores the ordinary reading dependencies, each chart/fact
pair, and finally the occasion. This makes an imported occasion replayable
without claiming that astrology caused or weighted the reading.

`CleromancySessionAuthority` only delegates to that existing product mapping.
It does not acquire a second replication store or transport responsibility.
The caller still chooses whether to author an H7 operation, which identity and
roster authorize it, and when any admitted operation travels.

## Acceptance

```powershell
cargo test --features personal-sync --test a4 --offline
cargo test --features "graphshell-admission personal-sync" --test a22_resident_pattern_sync --offline
```

The A22 receipt admits a writer, creates the bounded Pattern occasion through
the retained local carrier, exports its selected truth from the resident
authority, and authors it into an independent in-memory H7 replica. A second
resident authority materializes that signed projection, re-exports byte-for-
byte-equivalent events, and projects the saved Pattern occasion card.

## Ownership

Cleromancy owns selection semantics, replay validation, local graph
materialization, and the limited concurrence claim. Graphshell H7 owns identity,
roster admission, causal history, operation storage, conflict detection, and
transport. A containing host owns sync settings and publish timing.

## Stop rule

This does not add automatic publishing, pairing or key-management settings,
background synchronization, peer discovery, transport receipts, or physical
network proof. Those require a concrete host-level sync policy and carrier.
