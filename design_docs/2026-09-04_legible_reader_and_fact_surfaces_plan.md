# Cleromancy: legible reader and fact surfaces

**Date:** 2026-09-04
**Status (2026-09-05):** complete. **R0–R4 landed.** Supersedes
nothing; it is the successor slice to
[journal depth](2026-08-08_journal_depth_plan.md), which is landed and green
(see Findings).
**Source:** the maintainer's Claude Design canvas *Cleromancy GUI layout
discussion*, turn 1 (options 1a–1d). Options 1a, 1b, and 1c are annotated
`cambium-near`; 1d is annotated `north-star` and is explicitly out of scope
here.

## Decision

Cleromancy becomes **four surfaces over one fact graph**: `today`, `journal`,
`sky`, `chart`. The graph, the receipts, and the persistence rules do not
change. What changes is that durable material the product already stores
becomes *findable and readable* instead of digest-labelled and hidden.

Two things are ruled out of this slice by the maintainer's own annotations on
the canvas, because they are Cambium primitives rather than Cleromancy
product work — see [Cambium asks](#cambium-asks):

- the ecliptic strip and the dimension-line aspect diagram (vector leaves);
- the range-gated scrubber with pins.

Every surface below is therefore specified in **DOM-legible form**: tables,
sectioned lists, disclosure trees, and detail panels that current Cambium
already ships. A later slice may add the vector leaves without changing any
read model defined here.

## Findings

- **2026-09-04: the journal depth slice is already landed.** All five of its
  acceptance receipts are implemented, and `cargo test --test journal_depth
  --offline` passes 2/2 against `C:\t\cleromancy-headed-target`; the plan doc
  simply carried no Status line and has now been marked landed. Additional `name: value` facts with reserved-name rejection are
  at [`drafts.rs:320`](../src/consultation/drafts.rs); the three-card cast is
  `read_three_card` at [`consultation/mod.rs:358`](../src/consultation/mod.rs);
  receipt comparison is [`compare.rs`](../src/consultation/compare.rs) and its
  journal UI; append-only reflections newest-first are
  [`catalog.rs:107`](../src/host/catalog.rs); the retained DOM proof is
  `tests/journal_depth.rs` with `tests/consultation_authoring_dom.rs`. The
  slice was also overtaken by later work (`Derived`, authored spreads,
  astrology charts, sky facts), all of which landed after it.
- **2026-09-04: session rows are unreadable.** The journal renders
  `Session <8 hex chars>` and nothing else
  ([`journal.rs:59`](../src/ui/view/journal.rs)). There is no date, no context
  label, and no drawn card, so a saved reading cannot be found without opening
  every candidate.
- **2026-09-04: the journal never renders the session's context.** The
  question actually asked is present in `ConsultationDetail.context`
  ([`consultation/mod.rs:47`](../src/consultation/mod.rs)) and is rendered
  nowhere in [`journal.rs`](../src/ui/view/journal.rs).
- **2026-09-04: `sessions()` replays every session on every catalog rebuild.**
  [`catalog.rs:69`](../src/host/catalog.rs) calls `validate()` and
  `replay_session()` per row before returning. This is correct and must stay
  correct for anything that *reads* a session, but it makes the cost of
  drawing a list proportional to the whole archive, on a journal intended to
  grow for years.
- **2026-09-04: there is no sky-facts catalog listing.**
  [`host/sky.rs`](../src/host/sky.rs) exposes `sky_day_facts_for_digest` and
  `sky_interpretation_for_digest` but no enumerator, and
  `ConsultationCatalog` ([`consultation/mod.rs:35`](../src/consultation/mod.rs))
  carries `astrology_facts` but no sky facts. A sky surface cannot list what
  the store holds until this seam exists.
- **2026-09-04: the drafted R3 body table described chart data, not sky-day
  data.** `SkyDayFacts` stores New Moon, dawn, and dusk TT intervals, the
  effective numerical policy, and per-event provenance. It has no body
  longitude, daily motion, retrograde state, frame, supported-range verdict,
  or oracle-parity field. Those claims cannot be projected from the record
  without inventing data or widening the durable schema. `AstrologyChart` and
  `AstrologyFacts` own the stored body positions and sign derivation instead.
- **2026-09-04: concurrence is already the right carrier for chart facts in a
  reading.** `ConsultationDetail.concurrences` exists and is populated, and
  the canvas states the rule explicitly: chosen chart facts are recorded as a
  concurrence, *never* as a cause. No new relation is required for the
  "sky at this cast" strip.
- **2026-09-04: a store with an unreplayable receipt is not constructible
  through the public API, so R0's original done-condition was unprovable.**
  Every insertion path validates before storing: `insert_reading`
  ([`records.rs:174`](../src/host/records.rs)) calls `ReadingEngine::replay`
  and rejects a mismatch, and `insert_session`
  ([`records.rs:287`](../src/host/records.rs)) runs
  `validate_session_bindings` first, which itself replays every placement
  ([`host/mod.rs:418`](../src/host/mod.rs)) and always stores the bound field.
  A saved store therefore cannot hold a session whose receipt fails replay.
  This is the invariant working correctly; the fix is to make R0's guarantee
  structural — a projection function that receives no `Field` cannot replay —
  rather than to test for a state the product refuses to create.
- **2026-09-04: `validate_session_bindings` is where the per-row cost lives.**
  It calls `ReadingEngine::replay` once per placement
  ([`host/mod.rs:418`](../src/host/mod.rs)), and `sessions()` invokes it for
  every row through `replay_session`. Dropping it from the summary path also
  drops the field decode, since the field is only needed to replay.
- **2026-09-04: the screen enum does not gate rendering.**
  `ConsultationScreen` ([`ui/state.rs:14`](../src/ui/state.rs)) has
  `Consultation`, `Reading`, and `Journal`, but `consultation_view`
  ([`ui/view/mod.rs:54`](../src/ui/view/mod.rs)) renders all three regions
  unconditionally and uses the screen only as a `data-screen` attribute.

## Ownership

Unchanged from the [headed local consultation plan](2026-08-07_headed_local_consultation_plan.md).
Restated only where this slice adds a seam:

| Concern | Owner |
|---|---|
| Summary projections over stored sessions | Cleromancy `host` |
| Filter, ordering, and bounds as product policy | Cleromancy `consultation` |
| Surface selection and per-surface view models | Cleromancy `ui` |
| Graph identity, facets, replay | Mere and Muniment |
| Controls, focus, accessibility, layout, paint | Cambium and Genet |
| Numerical sky and chart facts | Turquet, through the existing adapters |

A summary is a **projection of graph truth, not a second store**. It is
recomputed from facets and is never persisted as its own facet.

## Rulings (2026-09-04, with the maintainer)

- **Scope is widened to Cambium.** The vector leaves and the range-gated
  scrubber may be built in `mere/crates/cambium` rather than only recorded as
  asks. That work is Mere's, so it gets its own plan at
  `mere/design_docs/` per doc policy §4 — it is not planned in this file, and
  R3/R4 here stay specified so they land complete without it.
- **R0 starts first.**
- **`Today` carries a bounded recent trail**, not the whole journal region.
  The `Journal` surface is the full reader: every session, filters, context
  rendering, and receipt comparison.
- **R3 renders the sky-day record that exists.** Its ledger is the selected
  civil day's stored New Moon, dawn, and dusk facts in canonical TT-interval
  order. Catalog admission is canonical-address, digest, and structural
  validation, not numerical recalculation. The existing `sky-timeline` tab
  gate remains, while the catalog type may carry stored facts independently
  of the optional Turquet producer.
- **Chart-owned body positions stay in R4.** R4 may render the stored raw
  longitude, derived sign, latitude, and retrograde flag. Daily motion,
  structured frame/range metadata, and per-record oracle parity are absent
  from the current schemas and are not inferred from source strings.

## Phases

### R0. Session summary read model

**Files:** `src/host/summary.rs` (new), `src/host/mod.rs`,
`src/consultation/mod.rs`, `src/lib.rs`, `src/ui/native/mod.rs`,
`tests/session_summary.rs`.

**Implemented: 2026-09-04.** `cargo test --test session_summary --offline -j 1`
passes 2/2. The projection lives in its own module rather than in
`catalog.rs`, because it is a distinct concern from canonical-address
resolution and carries the structural no-replay argument in its own module
doc. `summarize` is the free function specified below; `session_summaries()`
resolves session, context, and readings and calls it. `ConsultationCatalog`
gained `session_summaries`, which required updating the native
`empty_catalog()` literal.

Add a `SessionSummary` carrying exactly what a legible row needs: session id,
`created_at_ms`, context label, placement count, the ordered placement
positions, each placement's card title, and the distinct selection modes used.
Add `CleromancyHost::session_summaries()` returning them newest first with id
as the final tie-break, matching the existing `sessions()` ordering.

The summary path resolves each session's context and readings but does **not**
call `replay_session`. Replay stays on `detail()` and on `sessions()`, which
remain unchanged. A malformed stored facet must still surface as an error
rather than vanishing from the list, so structural validation (`validate()`)
stays on the summary path; only the receipt replay is dropped.

The no-replay guarantee is made **structural rather than behavioural**. The
projection is a free function

```rust
// compile-ready
pub fn summarize(
    session: &ReadingSession,
    context: &ContextSnapshot,
    readings: &[Reading],
) -> SessionSummary
```

which takes neither a `Field` nor a `ReadingEngine` and therefore *cannot*
replay a receipt. `session_summaries()` fetches the session, its context, and
its readings, then calls it. This replaces the behavioural done-condition
originally drafted here; see Findings for why that one was not constructible.

`ConsultationCatalog` gains `session_summaries` and keeps `sessions` until R2
retires the UI's use of the full rows.

**Done when:**

- a summary row's context label, card titles, and modes equal the values on
  the corresponding `ConsultationDetail`;
- a store whose session facet is corrupt fails `session_summaries()` rather
  than silently omitting the row;
- `summarize` takes no `Field` and no engine, so no receipt replay is
  reachable from the summary path, and a test calls it directly with only a
  session, its context, and its readings to demonstrate that boundary;
- ordering matches `sessions()` exactly on a fixture with several occasions;
- summaries survive close/reopen unchanged, proving they are recomputed from
  graph truth rather than stored.

The originally drafted "count the node addresses a summary pass touches"
condition is dropped: counting requires instrumenting the graph accessor,
which is a change to production code made solely to observe a test, and the
structural condition above already forecloses the behaviour it was meant to
detect.

### R1. Four-surface shell

**Files:** `src/ui/state.rs`, `src/ui/view/mod.rs`, `src/ui/action.rs`,
`tests/headed_consultation_dom.rs`.

Replace `ConsultationScreen` with `Today`, `Journal`, `Sky`, and `Chart`, and
make `consultation_view` render only the active surface's regions. `Today`
keeps the current consultation and reading regions plus a bounded recent-trail
column. `Journal` becomes the full reader (R2). `Sky` and `Chart` are
introduced as empty, labelled surfaces in this phase and filled in R3 and R4.

Surface selection uses Cambium's existing `tab_bar` (the `selection_bar`
widget, not the older `tab_strip`); it is a view-local selection and emits no
storage command.

**R1 rulings (2026-09-04, orchestrator; flagged to the maintainer):**

- **Tabs are the only surface switch.** The `present_*` methods stop
  assigning `screen`; a worker result never moves the user to another
  surface. The reflection editor and session rows exist on both `Today` (in
  the bounded trail) and `Journal`, so every landed flow except receipt
  comparison stays reachable without a switch; comparison lives on `Journal`
  only. `SelectionState` is the single source of truth for the active
  surface.
- **The `Chart` tab is always enabled.** Manual chart import has never
  required `analytic-ephemeris` (only the calculate control is gated, at
  `view/consultation.rs:239,305`), and gating the whole tab would make manual
  import unreachable in a default build once R4 moves the controls there.
  Only the `Sky` tab is feature-disabled, on `sky-timeline`, with a visible
  reason. This narrows the done-condition drafted below.
- **`Today`'s trail is bounded to 5 rows**, rendered from
  `catalog.session_summaries`, with the same `session:<id>` controls as the
  Journal so tests and the headed scenario keep one selector.
- New files rather than growth: `src/ui/screen.rs` (surface enum, tab items,
  panel ids) and `src/ui/view/trail.rs`, because `state.rs` sits at 555 of
  the 600-line cap.

**Done when:**

- the retained DOM exposes one `tablist` with four tabs, correct
  `aria-selected`, and arrow-key traversal that skips a disabled tab;
- exactly one surface's regions are in the tree at a time, and `data-screen`
  tracks the active surface;
- switching surfaces issues no `ConsultationAction` and no Redb work;
- a build without `sky-timeline` renders the `Sky` tab `aria-disabled` with
  an accessible explanation naming the feature; `Chart` stays enabled;
- `Today` shows at most 5 trail rows while `Journal` shows every session;
- `headed_consultation_dom`, `consultation_authoring_dom`, `journal_depth`,
  and `session_summary` stay green, and the `portable-core` lib check passes.

### R2. The journal reader

**Files:** `src/ui/view/journal.rs`, `src/ui/state.rs`,
`src/consultation/mod.rs`, `tests/journal_reader_dom.rs`.

Make the archive findable. Each trail row shows the short digest, a relative
date, one pip per placement, and the selection mode — the canvas's
`cb651c82 · today · ▫▫▫ cast`. The row's accessible name is the full,
unabbreviated identification; the pips are decorative and carry
`aria-hidden`, with the placement count in text.

The selected session renders its **context**: label, question, tags, and any
additional facts, above its readings and reflections. Receipt comparison moves
here from `Today` unchanged.

Add filtering over the summaries by tag and by date range, and bound the list
to a page with an explicit "older" control. Filtering and bounding are product
policy in `consultation`, computed over summaries; they never re-query the
graph per keystroke.

#### R2 maintainer rulings

- Relative dates use elapsed UTC-duration buckets through a pure formatter:
  `today` before 24 hours, `yesterday` before 48 hours, then `N days ago`.
- Date filtering is selectable as `Any time`, `Past 7 days`, or `Past 30
  days`; the cutoff instant is included and the range ends at now. A future
  timestamp is labelled `in the future` rather than collapsed into `today`.
- `JOURNAL_PAGE_SIZE` is the explicit product-policy constant and is `20`.
  Older and newer controls page the bounded DOM, which states its shown range
  and page count.
- A mixed session names every distinct selection mode. Each row's accessible
  name contains the full session id, context, date, placement/card count, and
  all modes. Its visible pips are decorative, one per placement, and hidden
  from accessibility while the count remains text.
- `ConsultationCatalog` carries `session_summaries`, not replayed full
  sessions. Session detail and receipt comparison remain explicit replay
  boundaries. UI, worker, and headed-scenario consumers use summaries.

**Done when:**

- a trail row's accessible name identifies the session without abbreviation;
- selecting a session renders its question and tags, and its additional facts
  when present;
- a tag filter and a date range each reduce the list, compose with each other,
  and are clearable;
- the list is bounded and the bound is stated in the DOM;
- comparison behaves exactly as its landed `journal_depth` receipt asserts,
  from its new location.

### R3. Sky surface

**Files:** `src/host/sky.rs`, `src/host/catalog.rs`, `src/ui/view/sky.rs`,
`src/consultation/mod.rs`, `tests/sky_surface_dom.rs`. Feature: `sky-timeline`.

First add the missing enumerator: `sky_day_facts()` over the sky-facts facet,
replay-verified like its siblings, ordered by civil day then digest, surfaced
on `ConsultationCatalog`.

Then three regions, all in current Cambium:

1. **Day record** — a selectable stored civil day, observer, and full digest.
   The civil-day label is a projection over the record; the stored TT
   intervals remain the canonical event values.
2. **Event ledger** — the selected day's New Moon, dawn, and dusk facts as a
   `sectioned_list`, ordered by TT interval. Each row carries the event name
   and raw TT start and end; it does not invent a current-time horizon or an
   applying/separating state absent from the record.
3. **Explanation tree** — nested `disclosure` regions for the effective phase,
   twilight, and Earth-orientation policy and for every event's stored model,
   provider snapshot, transform, and Earth-orientation provenance.

The view reads catalog values only. Invalid or misaddressed sky records are
refused by `sky_day_facts()` before they can render; numerical range failures
belong to the optional producer and cannot masquerade as a stored ordinary
value.

**Done when:**

- `sky_day_facts()` lists stored records and rejects a corrupt one;
- the Dallas 2024-04-08 fixture from `tests/sky_timeline.rs` renders its
  stored facts with no recalculation at view time;
- every event row names its TT time scale and exposes the raw interval;
- the explanation tree exposes the effective numerical policy, each event's
  complete stored provenance, and the record digest;
- Sky controls remain view-local and emit no product action.

### R4. Chart surface and the cast's sky

**Files:** `src/consultation/mod.rs`, `src/ui/view/chart.rs`,
`src/ui/view/reading.rs`, `src/ui/state.rs`,
`tests/chart_surface_dom.rs`. Feature: `analytic-ephemeris` for local
calculation only; manual import, stored charts, and the concurrence strip are
unconditional.

**R4 rulings (2026-09-05, orchestrator):**

- `ConsultationCatalog` gains a read model pairing each replay-verified
  `AstrologyFacts` record with its resolved stored `AstrologyChart`. Catalog
  construction may replay facts to defend graph truth; rendering receives the
  finished values and performs no adapter call or fact derivation.
- A chart has no stored display label or named observer. The saved-moment label
  is therefore its stored UTC instant, and the observer line is a projection
  of its optional stored coordinates (`global` when both are absent). R4 does
  not widen the durable chart schema.
- The reading strip projects the associated `AstrologyFacts.placements`. Raw
  longitude remains on Chart, where the paired stored chart is available.
- An absent retrograde value renders as `unknown`, not `false`.
- Activating the strip's chart link changes the existing view-local surface
  tab and selected chart only. It emits no product action and performs no
  storage work.

Two pieces:

1. **Chart surface** — saved moments as a list (UTC instant, chart digest,
   placement count, coordinate/global observer projection), a placement table
   with stored longitude and latitude millidegrees, labelled sign projection, and retrograde flag, the
   aspects as a data grid with aspect, separation in millidegrees, and orb,
   and a source block naming the stored engine, ephemeris, algorithm, and
   digest. Existing import and calculate controls move here from the
   consultation region.
2. **Sky at this cast** — in the reading region, when a session carries a
   concurrence with chart facts, a compact strip of the associated positions
   plus the concurrence id and a link to the chart surface. It is rendered
   under a heading that names it a concurrence, and its explanatory text
   states that chosen chart facts are recorded as a concurrence and never as
   a cause.

**Done when:**

- the saved-moments list and aspect grid render stored facts with no
  recalculation at view time;
- the source block names the stored engine, ephemeris, algorithm, and digest;
- a session with a chart concurrence renders the strip; one without renders
  nothing rather than an empty frame;
- the strip's copy states the concurrence rule, and no DOM text asserts
  causation;
- moving the chart controls leaves `consultation_authoring_dom` green after
  its selectors are repointed.

## Cambium asks

Recorded from the maintainer's own annotations on the canvas. **These are
Mere-side work and are not in scope for this repository** without an explicit
decision to widen scope:

| Ask | Canvas note | Status |
|---|---|---|
| Ecliptic strip | tier-2 vector leaf, labels stay DOM | V0 landed in Cambium and adopted here; see Mere's `2026-09-06_fact_visualization_leaves_plan.md` |
| Fact explanation tree | disclosure-tree widget | approximated here with nested `disclosure` |
| Range-gated scrubber with pins | stepper exists, scrubber does not | not in Cambium |
| Dimension-line aspect diagram | vector leaf | not in Cambium |
| Event ledger | `sectioned_list` | exists |
| Data grid | `data_grid` | exists |
| Disclosure row, content-addressed select, segment, field, callout | 1a's asks | exist |

R3 and R4 are specified to be complete and legible *without* any of the
missing primitives. If they later land in Cambium, they replace a rendering,
not a read model.

## Stop rules

- No new durable schema. Summaries, filters, and every surface in this plan
  are projections; nothing here adds a facet.
- No generated interpretation, on any surface. Authored packs stay authored.
- The `north-star` 1d graph workspace — sessions as graphlets, spreads as
  arrangements — is not started here, and no seam in this plan may be shaped
  to anticipate it.
- No topocentric positions, houses, local eclipse circumstances, or new
  Turquet events. R3 renders what the sky slice already stores.
- No sync, sharing, or remote surface changes.
- Every handwritten source and test file stays under 600 lines, per the
  headed plan's standing condition.

## Verification wall

Extends the headed plan's wall with this slice's receipts. Note that the
git-ignored `.cargo/config.toml` redirects the committed Mere, Genet, and
NetRender git dependencies to the sibling checkouts under `Code/repos`, so
these commands measure the local platform trees rather than the branch pins in
`Cargo.toml`. That is the intended local-development posture; a receipt
claimed against a specific upstream revision must say so and be measured with
the redirect removed.

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cleromancy-headed-target'
cargo test --test session_summary --offline
cargo test --test journal_reader_dom --offline
cargo test --test sky_surface_dom --features sky-timeline --offline -j 1
cargo test --test chart_surface_dom --features analytic-ephemeris --offline -j 1
cargo test --test headed_consultation_dom --offline
cargo test --test consultation_authoring_dom --offline
cargo test --test journal_depth --offline
```

## Progress

- **2026-09-04:** Plan founded from the design canvas. No code written.
- **2026-09-04:** Verified the journal depth predecessor green (2/2) before
  founding this plan on top of it, in 19m56s of cold compilation.
- **2026-09-04:** R0 landed. `SessionSummary`, `SummaryPlacement`, the
  `summarize` projection, and `CleromancyHost::session_summaries` are in
  `src/host/summary.rs`; `tests/session_summary.rs` passes 2/2. Two
  done-conditions drafted here were corrected during the pass rather than
  worked around — see the two 2026-09-04 Findings on unreplayable stores and
  on address counting. **Cost note for the remaining phases:** a touch to
  Cleromancy's `lib.rs` re-export list invalidated the Mere and Cambium
  compilation units, making this receipt cost 17m03s. R1–R4 are UI phases
  with frequent test runs, so prefer additive module-local changes over
  edits to the `lib.rs` export block where a phase allows the choice.
- **2026-09-04:** R1 landed. Its headed receipt covers all four tabs, their ARIA
  state, the default-build `sky-timeline` refusal, keyboard traversal that
  skips the disabled Sky tab, surface exclusivity, and the five-row Today
  trail against a six-session journal. Surface changes have no product action:
  `tab_bar` mutates only `SelectionState`, its unit view output is mapped to
  `never_tab_action`, and the native host sends work to Redb only by taking a
  recorded `ConsultationAction` after dispatch. The receipt asserts that this
  slot remains empty after pointer and keyboard surface changes. This is a
  structural storage-boundary proof; no production-only Redb instrumentation
  was added solely to observe it. Fresh receipts pass:
  `headed_consultation_dom` 2/2, `consultation_authoring_dom` 1/1,
  `journal_depth` 2/2, `session_summary` 2/2, and the `portable-core` lib
  check. An independent Luna review found no blocking issue against the R1
  done-conditions. It retained two explicit limits: Redb absence is proved by
  the action boundary rather than direct instrumentation, and catalog refresh
  still calls replaying `sessions()` for the compatibility list until R2
  retires it. The receipts ran with unrelated dependency-source pins present
  in the dirty `Cargo.toml`; those pins are outside this slice and excluded
  from its commit.
- **2026-09-04:** R2 landed. The catalog is now summary-only; session detail
  and comparison are the explicit replay boundaries. The Journal applies pure
  tag/date/paging policy over summaries, with a 20-row DOM bound stated even
  for zero matches. Rows expose full accessible identity, decorative
  per-placement pips, relative dates, every selection mode, and selected
  context facts. The new `journal_reader_dom` receipt covers composed and
  clearable filters, paging, local controls, selected detail, and comparison
  and passes 2/2. Journal policy tests pass 2/2; the prior regression set passes
  10/10 across `headed_consultation`, `headed_consultation_dom`,
  `consultation_authoring_dom`, `journal_depth`, and `session_summary`; the
  worker receipt passes 1/1; and the `portable-core` lib check passes. An
  independent Luna review found two bounded issues, both repaired and
  re-reviewed: empty results now state the bound, and past-date ranges end at
  now. Formatting and diff checks pass. The receipts ran with unrelated
  dependency-source pins present in the dirty `Cargo.toml`; those pins remain
  outside this slice and are excluded from its commit.
- **2026-09-04:** R3 landed after correcting a phase-boundary error in the
  draft: body placements, daily motion, frame/range metadata, and oracle
  parity are not `SkyDayFacts` fields. The Sky surface now renders exactly the
  durable record that exists: selectable civil days, observer and digest,
  canonical TT event intervals in a Cambium sectioned list, and disclosed
  numerical policy plus complete per-event provenance. `sky_day_facts()`
  rejects malformed and misaddressed records and sorts by civil day then
  digest without calling the numerical adapter. The feature-gated Dallas DOM
  receipt passes 2/2; its default-build companion passes 1/1 and proves stored
  facts remain catalogued while the Sky tab is disabled. Sky host and timeline
  predecessors pass 3/3, host enumerator guards pass 2/2, feature-enabled shell
  coverage passes 2/2, default shell/journal/persistence/summary regressions
  pass 9/9, and the `portable-core` lib check passes. Independent Luna review
  found no implementation defect; its four receipt-strength gaps were added
  and re-reviewed. Formatting and diff checks pass. The unrelated dirty
  `Cargo.toml` dependency pins remain outside this slice and are excluded from
  its commit.
- **2026-09-05:** R4 landed. `ConsultationCatalog` now pairs each
  replay-verified astrology-facts receipt with its canonically resolved stored
  chart. Chart renders full-identity saved moments, named Cambium placement and
  aspect grids with explicit millidegree units, source identity, and manual
  import in default builds; only local calculation remains feature-gated. A
  reading with an astrology concurrence renders its id and derived placements,
  states the association-only rule, and can open the matching chart through
  view-local state without emitting a product action. Default and
  `analytic-ephemeris` chart receipts pass 3/3 each; moved authoring passes 1/1;
  shell, Journal, journal-depth, session-summary, and default Sky regressions
  pass 10/10; and the `portable-core` lib check passes. The Journal receipt's
  seven-day fixture was moved one hour inside its inclusive cutoff, removing a
  pre-existing millisecond race between fixture time and UI time. Independent
  review found placement semantics, durable-identity, landmark, receipt, and
  unit-label gaps; all were repaired, and the final review is clean. Diff
  checks pass. The unrelated dirty `Cargo.toml` dependency pins remain outside
  this slice and are excluded from its commit.
- **2026-09-06:** Cambium visualization V0 adopted. Chart now projects every
  selected stored longitude into one read-only `AngleStrip` while retaining
  the complete DOM positions grid, source identity, and product-action-silent
  selection. The hidden paint leaf has a labelled DOM sibling with full body
  values; a second-chart receipt guards selection and leaf resynchronization.
  Mere owns the generic normalized geometry under its fact-visualization plan;
  Cleromancy continues to own astrology units, labels, and provenance.
  Chart passes 3/3 in both default and `analytic-ephemeris` builds; the moved
  authoring and headed surface regressions pass 3/3; and the `portable-core`
  check passes. Independent review is clean after the retained-cache and
  second-selection receipt gaps were closed.
