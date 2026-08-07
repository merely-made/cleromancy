# Cleromancy A21: resident authority persistence

**Date:** 2026-08-07
**Scope:** make the durable local-reading store reachable from the shared
resident authority after admitted writes.

## Contract

`CleromancySessionAuthority::persist(saved_at_secs)` flushes the same
`CleromancyHost` that all of its admitted endpoints mutate. It holds the
authority lock for the store operation, so the persisted graph is one coherent
point in endpoint mutation order. It stores only durable graph and facet truth;
projection sessions, disclosed resources, active instances, bound subjects,
and notices remain process-local.

The call is asynchronous and explicit. A product host chooses when to await it:
after an accepted write, on an idle boundary, at orderly shutdown, or under a
user-configured save policy. The carrier request itself does not hide storage
latency or silently claim a write is durable before the selected policy runs.

## Acceptance

```powershell
cargo test --features graphshell-admission --test a21_resident_persistence --offline
```

The receipt creates an astrology fact set and saved reading session, admits a
writer through the resident catalog, and accepts the bounded concurrence
action. It flushes the authority, drops all authority and endpoint handles,
then reopens a fresh Redb-backed host. The reopened graph replays the saved
Pattern occasion and its two stored members.

## Ownership

Cleromancy owns its persistence schedule and graph store. Graphshell still
owns carrier admission, session lifetime, and continued authority checks.
Servitor authorization remains live-only and is not treated as persisted
reading truth.

## Stop rule

This does not select an automatic save policy, queue background store jobs,
replicate records to peers, or make Graphshell's resident device host depend on
Cleromancy. Those are separate application-host composition decisions.
