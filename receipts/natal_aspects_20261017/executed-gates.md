# Natal aspects focused gates — 2026-10-17

Base HEAD: `8472bc8dca53c2646baea0f864db9d1d0578fd61`, with uncommitted domain and parent UI changes.

All Cargo commands used explicit rustc/rustdoc 1.97.1 and Cargo 1.99.0 executable paths, `TMPDIR=/tmp`, and `CARGO_BUILD_JOBS=4`.

- PASS: `cargo test --manifest-path core/Cargo.toml --locked --offline --test natal`: 17 passed, 0 failed, exit 0. Latest observed run completed in 13.70s. Output was returned in the agent tool transcript; no separate raw log was captured for that completed core run.
- PASS: `git diff --check`, exit 0, before the root run.
- PASS: scoped rustfmt check of `src/natal_aspects.rs` and `tests/natal.rs`, exit 0, before the parent integration follow-up.
- BLOCKED: root `cargo test --locked --offline --test natal`, exit 101. Compilation failed with E0308 at `src/ui/native/scenario_driver.rs:794`: `Ok(true)` supplied where `Ok(())` is expected. No root natal tests executed. Full stdout/stderr and actual exit code: `root-natal.log`.
- NOT RUN here: UI parser unit test, broad matrix, desktop/native scenarios, fresh-process reopen, publication. Parent explicitly owns these gates and the scenario driver fix.

The core tests exercise inclusive boundaries for all five aspects; 0/360 and opposition wrap; exact and per-kind custom rules; stable pair/angle ordering; latitude independence; invalid rules/longitude rejection; JSON and graph replay; original outcome preservation; original natal and house receipts; and resealed schema/method/rules/chart-digest/hit tamper rejection through validation and insertion admission.

API: `NatalReading::with_aspects(AspectRules)`; `OrbPolicy` aliases `AspectRules`; `NatalReading.aspects: Option<AspectReceipt>`. Default conjunction/opposition/trine/square/sextile orbs: 8000/8000/6000/6000/4000 millidegrees. No domain API concerns identified in parent action/resident wiring.

Source hashes and tool versions: `focused-test-inputs.json`. UI source changed during the root compile; that snapshot must not be described as a final UI qualification. No commit or push performed by this domain task. Compile slot released after root failure.

## Compile-slot handback

Verified current parent source fixed both aspect scenario command arms to `Ok(())`. Started corrected-source root natal rerun, then terminated only this agent's Cargo/rustc processes at the parent's explicit request to take over compilation. Cargo returned signal termination (-15), not a test failure. No root tests executed and UI parser unit command was not started. See `root-natal-rerun.log`, `focused-rerun-inputs.json`, and `focused-rerun-results.json`. No further compiles will be run by this agent. Core natal 17/17 remains the completed domain gate.
