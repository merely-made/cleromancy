# Cleromancy portable-core acceptance

This private package compiles Cleromancy's real root library from
`../src/lib.rs` without the native window or application projection adapter.
The resident imports its complete-candidate divination validator: canonical
record identity, supported facets, sealed reading replay and stored
requirements are checked before an owner installs a transaction. The validator
opens no store and leaves its input unchanged.

The default feature set is `portable-core`. The former snapshot-slot host is
available only through the explicit `legacy-receipts` harness feature. It is
not the ordinary application's persistence owner. The product worker attaches
to the resident-held divination mere and embeds the same authority on genuine
endpoint absence.

The frozen C1 gate uses Cargo 1.99.0, with the compiler and rustdoc explicitly
selected as Rust 1.97.1. In a shell with those two executable paths assigned to
`RUSTC` and `RUSTDOC`, run these commands from this directory:

```sh
cargo +1.99.0 test --locked -j 2
cargo +1.99.0 test --locked -j 2 --features legacy-receipts
cargo +1.99.0 test --locked -j 2 --features legacy-receipts,sky-timeline
```

The recorded outcomes are 18, 19 and 20 passing tests respectively. Exact
commands, immutable dependency identities and hashes are in
[the portable-core receipt](../receipts/c1-portable-core_20261014.json).
The separate [native C1 ledger](../receipts/c1-resident-qualification_20261014.json)
records product acceptance and the remaining shared browser gates. Compiler
compatibility does not promise Cargo 1.97 lockfile stability, and these portable
gates do not qualify a native window or full C1 acceptance.

This package is an acceptance boundary, not a second implementation. Its
library and test paths point directly at the root library and checked-in
resident-candidate and sky contracts. Immutable portable validator C remains
distinct from the final native D publication.

C3 continuation (2026-10-16): the new
[software receipt](../receipts/c3-software-qualification_20261016.json) records
24/25/26 passing portable gates on Cargo/Rust 1.99.0, separately from the frozen
C1/MSRV receipt above. Complete typed deck and position-definition snapshots
are validated as part of each field; old fields omit the new optional members
and retain their exact encoding. A newly built owner consumes this codec; an
older resident correctly refuses unfamiliar metadata before write. The [C3 native receipt](../receipts/c3-native-qualification_20261016.json)
now records bounded create/reopen qualification on the matching product codec.
Standalone-daemon receiving intake remains separate.
