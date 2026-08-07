# Cleromancy A25: local sync-consent command

**Date:** 2026-08-07
**Scope:** make Cleromancy's existing local sync selection configurable without
starting a resident host, graph store, or Graphshell transport.

## Contract

When built with `personal-sync`, the `cleromancy` binary reserves one explicit
management command:

```powershell
cargo run --features personal-sync --bin cleromancy -- sync-consent show
cargo run --features personal-sync --bin cleromancy -- sync-consent set contexts-and-readings
```

The supported names are `off`, `contexts`, `contexts-and-readings`, and
`contexts-readings-and-reflections`. The command reads or writes only the
product-owned `sync-settings.json` under `CLEROMANCY_ROOT` (or Cleromancy's
normal local data root). It never opens `cleromancy.redb`, starts a resident
authority, authors graph events, or contacts a peer. An unknown selection
prints a clear error and leaves the prior file unchanged.

The ordinary one-argument receipt command remains intact. In a build without
`personal-sync`, invoking `sync-consent` fails with an instruction to enable
that feature rather than treating the command name as an output path.

## Acceptance

```powershell
cargo test --features personal-sync --test a25_sync_consent_command --offline
cargo test --test a0 --offline
```

The command receipt proves default `Off`, a saved
`contexts-and-readings` selection, no Redb store creation, and a rejected
unknown selection that preserves the prior value. The A0 receipt confirms the
ordinary no-feature reading path still works.

## Ownership

Cleromancy owns its consent vocabulary, local file, and command. A future
headed host may edit the same file through its own control. Graphshell owns
personal graph identity, pairing, writer admission, and transport settings.

## Stop rule

This is not a generic configuration CLI, a settings pane, an H7 management
surface, a pairing tool, an automatic publisher, or a resident sync host. It
only makes the existing per-product consent boundary usable now.
