# Cleromancy

Cleromancy is a local-first divination journal (Tarot, dice, astrology
context) built as a Graphshell product cut over Mere. Every reading is
deterministic, securely cast, or derived from a public seed, and every result
keeps a replayable receipt that separates context, qualification, selection,
and interpretation.

## Status (2026-10-16)

The ordinary product opens the resident-held divination mere; its old private
snapshot authority is confined to the explicit `legacy-receipts` harness.
C1/C2 are merged on main and consume the extracted application door, serving
views and mere route from their stack crates. Family dependencies use canonical
Git URLs with `branch = "main"`; the committed locks record the tested revisions.
The original native C2 intake at Mere `8b08c98b2` and the later family software
checks have separate scopes. Full shared cutover qualification remains open. The
[family intake receipt](receipts/family-intake_20261015.json) records the
completed software checks and personal-sync compatibility follow-up.

C2 narrates each card's computation, keeps authored position labels and
meanings, shows context with the reading and offers reflection examples.
Cards reflow at compact widths. Chart entry accepts ordinary UTC dates/times
and decimal degrees, converted exactly at the calculation boundary. Bounded
native authoring and fresh-process reopening preserve the saved reading and
reflection identities. The [C2 receipt](receipts/c2-reading-qualification_20261015.json)
records source, gates, capture reviews and qualification limits; the
[divination plan](design_docs/2026-09-23_divination_mere_plan.md) owns phase scope.

C3 is implemented and bounded native qualification passes: complete Rider–Waite–Smith and
Marseille-structure text decks, 78 cards each, with typed identities, suits,
numbers and ranks. A fresh reading session starts with RWS; **Add tarot decks**
adds the second choice. A focused card discloses Golden Dawn / Book T and
Thierens correspondences separately, with source dates and untranscribed axes
explicit. Cross-deck inspection preserves the saved reading. New fields retain
their full card and position-definition editions; previous fields keep their
original encoding. The new fixed minor prompts disclose model-assisted
editorial provenance. The [C3 software receipt](receipts/c3-software-qualification_20261016.json)
records passing software gates. The [C3 native receipt](receipts/c3-native-qualification_20261016.json)
records creation, compact presentation, crosswalk inspection and fresh-process
reopening: eight reviewed originals, exact reading identities and byte-equal
card payload. Publication and standalone-daemon validator intake follow separately.

- The A0-A25 proof ledger is complete (2026-08-02 through 2026-08-07):
  single readings, sealed receipts with replay, field provenance, a 22-card
  Major Arcana pack, a fixed three-card spread, reading sessions and
  reflections, astrology chart and facts nodes, cross-system "consulted
  together" occasions, a resident multi-session authority, explicit
  persistence, and opt-in selected personal sync with durable local consent.
- The headed local consultation now presents Today, Journal, Sky, and Chart
  surfaces over one local fact graph. The Journal opens complete saved
  readings; Sky and Chart expose provenance-bearing facts in DOM-legible
  tables and lists.
- Chart pairs replay-verified astrology facts with their stored chart, keeps
  manual import available in default builds, and renders longitudes through
  Cambium's read-only ecliptic strip while retaining the full positions grid.
  A view-local aspect selector adds a read-only dimension projection beside
  the complete aspects grid, and a controlled stored-chart range scrubber adds
  UTC/digest pins. Scrubber selection is retained application UI state across
  rerender and tab switches, not durable graph storage.
- Derived readings replay byte-equal on independent engines from a public
  seed and domain (2026-08-08 contract).
- Authored spread templates (1-12 positions, reusable, content-addressed)
  with selected sync and explicit astrology chart input are landed.
- The `analytic-ephemeris` feature calculates Sun, Moon, and planetary
  positions locally through Turquet, saving the engine identity with each
  chart. The chart calculator needs no downloaded kernel.
- Today starts in Cast mode. Read sits above optional context metadata;
  derived seed controls appear when selected, and layout authoring is folded
  away. Results show the saved question, selectable placements and a focused
  card interpretation.
- The optional `sky-timeline` feature is the first daily astronomy consumer.
  It normalizes Turquet's New Moon and caller-threshold airless solar
  twilight facts for a UTC civil day and WGS84 observer. Numerical facts keep
  their policy and provenance; authored interpretation packs remain a
  separate, replayable layer. Refraction, limb, horizon-dip, weather, and
  general visibility policy remain caller-owned.

The canonical plan index is [design_docs/DOC_README.md](design_docs/DOC_README.md).
The current ephemeris proof does not implement houses or topocentric positions.
Generated interpretations are planned in C7, with their inputs and method/model
identity retained; they are not prohibited by the product's standing rules.

## Use

Building requires access to the merely-made mere, genet, and netrender
repos (git dependencies).

```powershell
cargo run --bin cleromancy         # the local consultation window
cargo run --features analytic-ephemeris,sky-timeline --bin cleromancy # chart calculator and stored Sky facts
cargo test --features sky-timeline --test sky_timeline # daily timeline consumer
cargo test                         # the proof wall

# Per-slice receipt binaries, e.g.
cargo run --features legacy-receipts --bin cleromancy_receipt -- receipts/a0.html

# Historical sync-consent receipt harness (ordinary consent is resident-owned)
cargo run --features legacy-receipts,personal-sync --bin cleromancy -- sync-consent show
```

`MERE_ROOT` selects the shared reservoir and identity root;
`CLEROMANCY_VAULT_DIR` selects its vault. `CLEROMANCY_ROOT` selects local
application preferences and the historical harness directory. Normal resident
startup requires a provisioned persona; qualification uses an isolated public
synthetic identity.

## License

Cleromancy is MIT OR Apache-2.0. The optional `ephemeris` and
`sky-timeline` features link third-party dependencies; see
`THIRD_PARTY_LICENSES.md`.

---

*This README was generated by AI and will be edited by the author upon
release.*
