# Cleromancy design-doc index

This is the canonical index for active Cleromancy design documents. See the
[documentation policy](DOC_POLICY.md). `PROJECT_DESCRIPTION.md` is reserved for
the maintainer and does not exist in this repository yet; the root README is
the current public overview until it is authored.

## Working principles

- Keep numerical facts, interpretation (authored or generated, labelled which),
  and graph relationships as separate inspectable values. A prose rule never
  mutates a calculated fact.
- Make time scale, observer, numerical policy, and source identity explicit at
  a calculation boundary. Do not rename a geometric result as a human
  visibility result.
- Store content-addressed factual inputs before values derived from them, then
  replay the declared binding when reopening persistent state.
- Treat a consumer as an acceptance proof, not as permission to grow an
  unbounded algorithm catalogue. Add the next event only when a real consumer
  exposes a vocabulary gap.
- Everything stays replayable. It is the one standing constraint (Mark,
  2026-09-23); "no generated interpretation", "authored packs only" and "the sky
  is never causal" were never his.
- Be honest when anything is generated: say what was computed on what, and how
  random, and let people choose both. Formula-derived inferences recompute
  exactly. Model-generated ones are stored when produced, and labelled.
- Local-first holds. Network sources such as weather are opt-in, and what they
  return is stored with the reading. Time is always recorded; place is opt-in.
- A reading narrates its own computation in plain words: which deck, which
  method, what randomness, which card, orientation and position, the nth of m.
  That narration is the audit, and a raw block of workings is not. Present the
  context with the reading, and offer examples to riff on instead of blank
  boxes.
- Arrange readings, not spreads. Typed dimensions stay atomic, and every
  relation names the method and engine that produced it.
- Keep semantic provenance, foreground/background emphasis and motion roles
  independent. Shared scene and dynamics capabilities arrange disclosed facts;
  Cleromancy owns their meaning and permitted actions. A saved presentation or
  another host's receipt is not proof of this product's adoption.
- Cleromancy's data is its domain's mere (divination, journaling, RNG), held by
  the device resident. Use Mere's own sessions, graph journal, codicils,
  reservoir and ambient tier, and keep no app-local copies of them.
- Verify the headed reading at its ordinary window size. Tall DOM harnesses
  and saved-data checks do not establish that controls fit or that card
  results are visible; acceptance includes painted bounds and native capture.
- Keep the Mere and Genet dependency family on one immutable integration pin
  when adopting a platform consumer; branch references can create a second
  source identity and split shared types in downstream products.

## Product, graph, and reading proof ledger

- [A0 product cut](2026-08-02_a0_graphshell_product_cut.md): initial
  local-first Graphshell reading graph boundary.
- [A1 Turnstone enrichment](2026-08-02_a1_turnstone_enrichment.md): browsing
  context as disclosed enrichment rather than hidden personalization.
- [A2 sealed qualification](2026-08-02_a2_sealed_qualification.md): versioned
  external evidence and replayable qualification receipts.
- [A3 bound intents](2026-08-03_a3_bound_intents.md): authenticated local
  Graphshell intent contract.
- [A4 personal sync](2026-08-03_a4_personal_sync_adapter.md): selected,
  consent-gated personal sync boundary.
- [A5 field provenance](2026-08-03_a5_field_provenance.md): durable candidate
  fields and qualification provenance.
- [A6 Major Arcana pack](2026-08-03_a6_major_arcana_pack.md): the first bounded
  authored Tarot consumer.
- [A7 reading sessions](2026-08-04_a7_reading_sessions.md): saved reading
  occasions and their replay dependencies.
- [A8 three-card spread](2026-08-04_a8_three_card_spread.md): fixed
  three-position graph frame.
- [A9 three-card intent](2026-08-04_a9_three_card_intent.md): hosted action for
  creating the fixed spread.
- [A10 generic composer](2026-08-04_a10_generic_composer.md): explicit layout
  and selection composition.
- [A11 field selection](2026-08-04_a11_field_selection.md): stored-field choice
  at the intent boundary.
- [A12 field composer](2026-08-04_a12_field_composer.md): authored field draft
  contract.
- [A15 pattern occasion](2026-08-05_a15_pattern_occasion.md): a non-causal
  grouping of independently saved values.
- [A16 pattern selection](2026-08-05_a16_pattern_selection.md): selected
  relationship creation through the existing graph surface.

## Astrology and ephemeris

- [A13 astrology facts](2026-08-04_a13_astrology_facts.md): positions to
  source-qualified placements and aspects.
- [A14 astrology adapter graph](2026-08-04_a14_astrology_adapter_graph.md):
  narrow adapter and durable chart/facts nodes.
- [Analytic ephemeris parity](2026-08-13_analytic_ephemeris_parity.md):
  Turquet-based analytical chart-position comparison work.
- [Celestial fact engine direction](2026-08-13_celestial_fact_engine_direction.md):
  historical product direction; its incubation assumptions are superseded by
  Turquet's standalone roadmap.
- [Ephemeris engine](2026-08-13_ephemeris_engine.md): optional kernel-backed
  chart lane and its source/licensing boundary.
- [T5b sky timeline](2026-08-26_sky_timeline_plan.md): current Turquet event
  consumer, durable sky facts, and authored daily interpretations.

## Consultation, projection, and residency

- [Divination mere and the self-explaining reading](2026-09-23_divination_mere_plan.md):
  **C1 preparation in progress, 2026-10-08; no phase accepted.**
  - C1 moves Cleromancy onto its domain's mere in the identity's reservoir,
    held by the device resident with the full session lifecycle. §2 names the
    available shared substrate and the remaining authority, view, archive,
    grant and standalone-ownership acceptance gates; the Mere reservoir plan
    owns their status.
  - C2 is the reading that narrates its own computation.
  - Then, in the order Mark ruled: typed cards across several decks, the
    ambient card view, the shuffled deck with reversals, situation (time,
    opt-in place and weather), labelled generated inferences, and more
    systems.
  - Mark's rulings are recorded in Mere's
    `repos/mere/design_docs/mere_docs/design/2026-09-23_ambiance_design.md`.
  - All decisions ruled 2026-09-23: Cleromancy's domain authority is composed
    into the resident; the order is as above; the mere starts fresh, with no
    import.
  - The refresh records the existing snapshot store, native first/reopen
    evidence and limits, authored-position meaning bug and the outstanding
    format gate. The core lock is repaired and the portable full-candidate
    validator is prepared; resident composition and consumer adoption remain
    open. C4 includes proposed foreground/background and dynamics
    acceptance using shared contracts; no private spec or physics adoption is
    claimed. Preparation is distinguished from changing the ruled phase order.
- [Headed local consultation](2026-08-07_headed_local_consultation_plan.md):
  native journal-window product plan.
- [Journal depth](2026-08-08_journal_depth_plan.md) — **landed 2026-09-04**:
  disclosed additional context facts, the fixed three-card cast, receipt
  comparison, and append-only follow-up reflections. All five receipts
  implemented; `journal_depth` passes 2/2.
- [Legible reader and fact surfaces](2026-09-04_legible_reader_and_fact_surfaces_plan.md):
  **R0-R4 and Cambium V0-V2 adoption landed 2026-09-06**: four surfaces over one
  fact graph (`today`, `journal`, `sky`, `chart`); a replay-free session
  summary projection; a findable journal reader; DOM-legible sky and chart
  facts; a reusable read-only ecliptic strip above the complete positions
  grid; a view-local aspect dimension projection beside the complete aspects
  grid; and a controlled stored-chart range scrubber with UTC/digest pins.
  Scrubber persistence means retained application UI state across rerender and
  tab switches, not durable graph storage or a new schema. Later visualization
  primitives remain Mere-side work. Its north-star, "spreads as arrangements",
  was revised by Mark on 2026-09-23: readings are what get arranged (divination
  mere plan §6).
- [Derived selection](2026-08-08_derived_selection_plan.md): public-seed
  deterministic selection direction.
- [Authored spreads, sync, and chart input](2026-08-08_authored_spreads_sync_and_chart_input_plan.md):
  later combined product plan.
- [A17 retained action draft](2026-08-06_a17_retained_graphshell_action_draft.md):
  retained action semantics.
- [A18 admitted endpoint](2026-08-06_a18_admitted_endpoint.md): session
  admission boundary.
- [A19 resident catalog](2026-08-06_a19_resident_endpoint_catalog.md):
  resident endpoint catalog behaviour.
- [A20 resident authority](2026-08-07_a20_resident_session_authority.md):
  one durable authority with session-local projections.
- [A21 resident persistence](2026-08-07_a21_resident_persistence.md):
  close/reopen ownership and persistence rules.
- [A22 resident pattern sync](2026-08-07_a22_resident_pattern_sync.md):
  selected export/import of saved graph truth.
- [A23 sync consent](2026-08-07_a23_resident_sync_consent.md): durable local
  consent boundary.
- [A24 sync startup](2026-08-07_a24_resident_sync_startup.md): explicit
  initialization and recovery path.
- [A25 sync consent command](2026-08-07_a25_sync_consent_command.md):
  user-facing consent action.
