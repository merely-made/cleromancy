# Cleromancy portable-core acceptance

This private package compiles Cleromancy's real root library and checked-in
sky timeline test without the retired native Genet layout seam.

`portable-core` excludes the application projection adapter and native window.
It retains the complete `CleromancyHost` graph, persistence, reading, and sky
record implementation. The host's canvas projection module is excluded because
it belongs to the separate current-Cambium window migration.

Run the measured T5b gate from a neutral directory so Cleromancy's ignored
local Cargo patches do not override the pinned dependencies:

```powershell
$env:CARGO_HOME = 'C:\t\cleromancy-core-cargo-home'
$env:CARGO_TARGET_DIR = 'C:\t\cleromancy-t5b-core-target'
cargo test --manifest-path C:\Users\mark_\Code\repos\cleromancy\core\Cargo.toml --features sky-timeline -j 1 --locked
```

This package is an acceptance boundary, not a second Cleromancy library. Its
library and test paths point directly at `../src/lib.rs` and the three
checked-in `../tests/sky*.rs` contracts.
