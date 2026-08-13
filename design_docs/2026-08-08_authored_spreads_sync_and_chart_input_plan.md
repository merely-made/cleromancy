# Cleromancy: authored spreads, selected sync, and chart input

**Date:** 2026-08-08
**Scope:** extend the headed local consultation from one fixed three-card cast
to reusable, authored spread layouts; make their graph truth travel through
the already-selected personal-sync lane; and accept an explicit, source-
qualified astrology chart as a local input.

## Decision

This is an authored-layout model, not an unbounded plug-in DSL.

`SpreadTemplate` is an immutable, content-addressed layout: a label, ordered
position names and labels, and optional typed relations among those positions.
`Spread` binds one template to one saved cast session. The template and the
instance are graph nodes, so replay never depends on a current UI draft. A
spread always casts one independent card per position against one context and
field. Calculated and enriched spread modes remain unsupported.

The existing A8 `ThreeCardSpread` stays addressable and replayable as the
legacy, fixed product record. New layouts use the generic facet; there is no
migration or reinterpretation of prior A8 receipts.

The chart form is an import boundary, not an ephemeris. It requires the named
algorithm, engine, ephemeris/source, UTC instant, optional coordinates, and
explicit integer positions. Cleromancy validates, stores, and derives the
facts. It does not calculate positions, choose a license, infer a timezone,
or create astrology prose. A selected facts record can be grouped with a newly
saved session in the existing non-causal `Concurrence` model.

## Ownership and seams

| Seam | Change | Does not own |
| --- | --- | --- |
| `src/spread.rs` | immutable template and session-bound generic spread values | Tarot content or interpretation generation |
| `src/host.rs` | graph storage, replay, and projection edges for templates/spreads | mutable UI draft state |
| `src/sync.rs` | selected export/import validation of template and spread dependencies | pairing, identity, carrier, or automatic publishing |
| `src/consultation.rs` | parse draft layouts and source-qualified chart input; persist a selected concurrence | ephemeris math |
| `src/ui/` | edit/select layouts, record charts, select verified facts, show the resulting binding | direct Redb access or background sync |
| `src/sync_settings.rs` | unchanged local-only consent authority | graph synchronization of the consent choice |

## Generic spread contract

1. A template has 1 to 12 ordered position names, an optional visible label
   per position, and at most 24 relations. Names use the same bounded stable
   identifier grammar as explicit context facts. Relations use only Mere
   semantic kinds exposed by the layout editor.
2. A cast creates one sealed `Reading` per template position, one ordered
   `ReadingSession`, and one `Spread` that names both the template digest and
   session ID. The graph exposes the template, instance, session, cards, and
   authored semantic edges.
3. Replaying a spread verifies the stored template, session bindings, and
   every sealed reading. Changing any layout identity, card binding, or
   relation fails before it is accepted.
4. The personal-sync `ContextsAndReadings` selection carries both generic
   spread facets with their context, field, reading, and session dependencies.
   Import validates all dependencies before mutating local graph truth.

## Chart input contract

1. The local form records an already-calculated chart with clear source
   metadata and `body | longitude millidegrees | latitude millidegrees |
   retrograde` rows.
2. Invalid source fields, partial coordinates, duplicate bodies, or invalid
   positions fail before persistence. The stored facts are recomputed from the
   stored chart and selected orb.
3. Selecting a verified facts record when saving a reading creates a separate
   immutable `Concurrence`. It means only “consulted together,” and does not
   change Tarot weights or claim causation.
4. Existing `ContextsAndReadings` sync already selects chart/facts and
   concurrences; this gate proves that the headed consumer reaches those
   existing sync values without expanding consent or starting transport.

## Acceptance receipts

1. A reusable four-position template produces four independently sealed casts,
   replays after Redb reopen, and leaves a prior template unchanged when a new
   template is authored.
2. A two-replica selected-sync receipt exports and imports a generic spread
   with its template, session, readings, and semantic relations intact.
3. A source-qualified chart form stores a chart and verified facts, then a
   selected facts record and reading create a separately replayable
   concurrence.
4. The retained DOM can author a layout, select it, make a cast, record a
   chart, select its facts, and show the non-causal association.
5. The existing A8 and A22 sync receipts remain green as compatibility
   evidence.

## Stop rule

Stop after generic template/spread persistence, selected sync, chart import,
and the headed controls are covered. Do not choose or bundle an ephemeris,
calculate houses, add astrological correspondences or interpretation prose,
make sync automatic, add pairing controls, or create a general plug-in SDK.
