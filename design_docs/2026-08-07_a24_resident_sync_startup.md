# Cleromancy A24: resident sync startup

**Date:** 2026-08-07
**Scope:** make a consent-aware resident-authority opening path, without
starting a Graphshell device host or broadening the sync boundary.

## Contract

`CleromancySessionAuthority::open_with_local_sync_settings` takes the product
data root and an already-selected Cleromancy backend. It loads the local
`sync-settings.json` before opening the durable graph, constructs the resident
authority, and applies that selection before returning it. An absent settings
file yields the existing `Off` default. An unreadable or incompatible settings
file returns `CleromancyResidentOpenError::SyncSettings`; no authority is
returned.

The constructor deliberately accepts a backend rather than deriving one. The
containing product chooses Redb path, process lifetime, and store policy;
Graphshell still owns personal graph identity, pairing, writer admission, and
transport. The data root is only the location of Cleromancy's consent file.

## Acceptance

```powershell
cargo test --features "graphshell-admission personal-sync" --test a24_resident_sync_startup --offline
```

The receipt persists one ordinary reading, then opens fresh authorities through
the startup constructor. With no file, the selected batch is empty. With a
saved `ContextsAndReadings` file, the authority immediately exposes the saved
context, field, and reading. Replacing the file with an incompatible schema
prevents another authority from opening.

## Ownership

Cleromancy owns this product startup helper, its graph store, and its local
consent. A containing host owns its backend choice and save schedule.
Graphshell owns all personal-graph and transport configuration.

## Stop rule

This does not add an executable resident/device host, a Graphshell carrier
loop, a settings UI, automatic persistence, automatic publishing, background
sync, or peer configuration. It makes a future concrete host start correctly
without requiring it to duplicate the product's consent loading sequence.
