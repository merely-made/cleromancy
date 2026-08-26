# Cleromancy design-doc index

This is the canonical index for active Cleromancy design documents. See the
[documentation policy](DOC_POLICY.md). `PROJECT_DESCRIPTION.md` is reserved for
the maintainer and does not exist in this repository yet; the root README is
the current public overview until it is authored.

## Working principles

- Keep numerical facts, authored interpretation, and graph relationships as
  separate inspectable values. A prose rule never mutates a calculated fact.
- Make time scale, observer, numerical policy, and source identity explicit at
  a calculation boundary. Do not rename a geometric result as a human
  visibility result.
- Store content-addressed factual inputs before values derived from them, then
  replay the declared binding when reopening persistent state.
- Treat a consumer as an acceptance proof, not as permission to grow an
  unbounded algorithm catalogue. Add the next event only when a real consumer
  exposes a vocabulary gap.
- Preserve the local-first and no-generated-interpretation boundaries. Packs
  are authored, versioned content and remain independently replaceable.

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

- [Headed local consultation](2026-08-07_headed_local_consultation_plan.md):
  native journal-window product plan.
- [Journal depth](2026-08-08_journal_depth_plan.md): next durable journal
  surface work.
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
