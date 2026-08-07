# Cleromancy A23: resident sync consent

**Date:** 2026-08-07
**Scope:** make the selected personal-sync boundary a durable, local product
setting and apply it to the resident authority at runtime.

## Contract

`CleromancySyncSettings` stores one `CleromancySyncSelection` in a
product-owned `sync-settings.json` below an already-chosen Cleromancy data
root. A missing file means `Off`. The file carries a versioned schema, rejects
unknown fields and unknown schemas, and writes through a complete temporary
sibling before replacement.

The setting is application-scoped, local-only, live, and private. It records
this device's consent to offer or materialize Cleromancy data. It is not H7
graph truth and never travels to another device: synchronizing the selection
would let one device broaden another device's data-sharing policy.

`CleromancySessionAuthority` starts with `Off`. A containing host explicitly
loads settings, applies them, and may then call
`export_selected_sync_batch` or `import_selected_sync_projection`. The older
methods accepting an explicit selection remain available for a host that owns
its own configuration surface. Applying invalid settings fails before it
changes the live selection; saving remains an explicit host policy.

## Acceptance

```powershell
cargo test --features "graphshell-admission personal-sync" --test a23_resident_sync_consent --offline
```

The receipt proves that an absent file selects nothing, a reloaded
`ContextsAndReadings` choice enables one resident authority's ordinary reading
batch, a bad schema leaves that live choice intact, and a saved `Off` choice
returns it to an empty batch.

## Ownership

Cleromancy owns the selection, its local storage, and how it bounds domain
events. Graphshell owns graph identity, pairing, writer admission, operation
history, sync transport, and physical-device configuration. A headed host owns
the eventual settings UI and when it persists an edited selection.

## Stop rule

This does not add a headed settings pane, generic settings provider, H7 graph
or pairing configuration, automatic publishing, background sync, deletion
propagation, or device-to-device proof. The generic settings contract is ready
when Cleromancy has a real headed consumer, but this product setting does not
invent one.
