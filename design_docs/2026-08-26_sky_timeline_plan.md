# Cleromancy T5b: Turquet sky facts and authored daily interpretations

**Date:** 2026-08-26
**Status:** implemented; source-level gates passed, full-app integration pending platform reconciliation

## Scope

This slice makes Cleromancy a real consumer of Turquet's public event API,
without making Turquet's Rust values part of Cleromancy's durable schema. It
calculates one explicit UTC civil day for one WGS84 observer and stores a
canonical `SkyDayFacts` record containing selected numerical facts:

- geocentric ecliptic-longitude New Moon;
- caller-policy airless Sun-center dawn and dusk crossings.

The numerical policy is an input, not a default. It names phase and twilight
search controls, the caller-selected altitude threshold, and a disclosed
constant UT1-minus-UTC / zero-polar-motion approximation. The named crossings
retain T4l's deliberately limited meaning. They do not mean conventional
twilight, refraction-adjusted rise/set, human visibility, or local solar
eclipse visibility.

`sky-timeline` is an optional dependency on Turquet 0.17.0 at the exact Git
revision that carries T4l. The existing `analytic-ephemeris` dependency remains
the separate Turquet 0.1.0 chart adapter until a migration is explicitly
chosen.

## Contract

`SkyDayFacts` owns the UTC day, observer, numerical policy, normalized TT
intervals, event kinds, source strings, and a canonical digest. It does not
persist a Turquet type.

`SkyRulePack` is authored data with an id, author, version, rules, and its own
digest. Applying a pack produces `SkyInterpretation` records that carry both
the facts digest and the full pack/rule identity. The operation cannot alter
the facts. A pack is not a graph node in this proof: the stored interpretation
is sufficient to disclose exactly which authored text was applied.

The host stores facts at `cleromancy://sky/facts/{digest}` and interpretations
at `cleromancy://sky/interpretation/{digest}`, with a `GeneratedFrom` edge from
each interpretation to its factual input. Replay verifies the factual binding
before returning a stored interpretation.

## Phases

### 1. Canonical sky and interpretation values

**Done when:** UTC days, WGS84 observers, controls, TT intervals, provenance,
facts, packs, and interpretations validate and serialize deterministically;
the pack can be changed without changing the facts digest.

### 2. Optional Turquet adapter

**Done when:** an explicitly configured Turquet analytical calculation returns
only normalized New Moon, dawn, and dusk records through the public Turquet
API, carrying provider, transform, and Earth-orientation provenance.

### 3. Durable consumer proof

**Done when:** Dallas on 2024-04-08 produces the selected factual records;
facts and an authored interpretation survive close/reopen; the facts digest is
stable across calculation; and a changed pack changes only interpretation
state.

## Findings

- **2026-08-26:** Turquet 0.17.0 is not yet on crates.io. Cleromancy is
  unpublished and already has Git-pinned platform dependencies, so this slice
  uses an optional exact Git revision rather than claiming a registry release.
- **2026-08-26:** T4l's airless caller-threshold result is appropriate input
  for a factual daily timeline, but it must remain distinctly named from
  conventional twilight or visibility policy.

## Progress

- **2026-08-26:** Implemented the value model, Turquet adapter, graph storage,
  and Dallas integration test.
- **2026-08-26:** A temporary sky-only harness path-included the production
  `src/sky.rs` and `src/sky/turquet.rs` files at Turquet
  `30875f30e83909f3030bc01237a5046e14cde29b`:
  `cargo test --manifest-path C:\\t\\cleromancy-t5b-sky-check\\Cargo.toml --features sky-timeline --test t5b_sky -j 1`
  passed 4 tests, including deterministic Dallas 2024-04-08 New Moon, dawn,
  and dusk facts.
- **2026-08-26:** A graph-only harness path-included the production
  `src/host/sky.rs` storage methods:
  `cargo test --manifest-path C:\\t\\cleromancy-t5b-host-check\\Cargo.toml --test t5b_host -j 1`
  passed its factual-record, interpretation-binding, and `GeneratedFrom` edge
  proof.
- **2026-08-26:** The original feature-gated integration command remains
  checked in but could not reach Cleromancy: the ignored local platform patch
  combines current Mere (which calls `LiveryDocument::mutate_dom`) with the
  historical Genet revision required by the removed `genet-layout` crate.
  Reconcile that unrelated platform seam before claiming the app-level
  close/reopen test as measured.

## Stop rule

Do not add positional snapshots, meridian transits, conventional rise/set,
visibility claims, local eclipse circumstances, local-date conversion, pack
installation, generated interpretation, a chart surface, automatic sync, or a
solar tracker here. Those are separate consumer or Turquet event slices.
