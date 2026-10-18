# Cleromancy design-doc index

Rust policy (2026-10-13): working toolchain 1.99.0 and shared MSRV 1.97.1, qualified separately. Canonical policy: `mere/design_docs/2026-08-12_family_composition_thesis_brief.md`, section "Shared Rust toolchain and minimum version".

Murm source adoption (2026-10-13) is tracked in
`murm/design_docs/2026-10-13_murm_family_move_plan.md`. Active consumer roots
use the imported transport supplier and coherent Mere/Dramatis/p2panda
composition patches. Compile and software-test receipts are scoped to this
cutover; previous physical and headed receipts retain their original inputs.

This is the canonical index for active Cleromancy design documents. See the
[documentation policy](DOC_POLICY.md). `PROJECT_DESCRIPTION.md` is reserved for
the maintainer and does not exist in this repository yet; the root README is
the current public overview until it is authored.

Frozen C2 qualification (2026-10-15): C2 consumes Mere `8b08c98b2` through
`graphshell-endpoint::door`, `graphshell-host` and `mere-overmap`.
The [C2 receipt](../receipts/c2-reading-qualification_20261015.json) records
consumer tests, native authoring/reopening and preserved historical evidence.

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
- A resident catalog refresh is not completion of a pending product action.
  Keep it pending until its detail or explicit completion update arrives.
- Verify the headed reading at its ordinary window size. Tall DOM harnesses
  and saved-data checks do not establish that controls fit or that card
  results are visible; acceptance includes painted bounds and native capture.
- Follow the family source convention: canonical Git URL and `branch = "main"`
  for each first-party repository, with exact tested revisions in committed
  lockfiles. Mixing URL spellings or branch/revision references creates extra
  source identities and splits shared types in downstream products.
- Keep font requests separate from local availability and realized glyph metrics.
  Unset roles preserve product defaults; sizes and spacing remain app-owned.
  Failed saves cannot change the applied font, and authored CSS retains ordinary
  cascade authority. A family inventory is not proof of per-glyph coverage.
- Pair a themed primary background with its on-primary foreground in selected
  controls. Check the mounted computed colors under canonical and authored
  themes; preserving arbitrary authored CSS does not replace this app-owned
  semantic styling responsibility.

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

- [Codex handoff (October 15)](2026-10-15_cleromancy_codex_handoff.md): where
  Cleromancy stands after the Mere stack lift: C1/C2 are merged, the canonical
  branch-source manifests and locks are published, and personal-sync's optional
  assertion-basis follow-up completes intake. P4's endpoint receipt is separate
  from removal of the personal-sync port dependency.

- [Shared application appearance acceptance](receipts/tabard_20261011/README.md):
  landed native Tabard choice/workshop/title bar, strict Save/Apply, four modes,
  readable wide/narrow editor and durable fresh reopens; exact CPU/native receipts.
  The selected-radio foreground follow-up has negative controls, default96 and
  native21 tests, seven reviewed native captures and exact focused reopening;
  the broader workshop-resize retry remains explicitly failed evidence.
  [Completed plan](archive_docs/2026-10-11/2026-10-11_tabard_appearance_plan.md).

- [Divination mere and the self-explaining reading](2026-09-23_divination_mere_plan.md):
  **C1 implemented and bounded native/cross-application qualified, 2026-10-14;
  full shared wall pending; no phase accepted.**
  - Published H/f7ec/N composes Cleromancy's product-owned validator into the
    resident. Ordinary Cleromancy attaches or embeds the resident; private
    snapshot/authority paths are retired from its default product. Historical
    receipts remain explicit nondefault harnesses.
  - Five fresh native runs, native Graphshell witness/reopen, actual Chrome
    reopen, retained native-ACK/browser live update and staged archive/read-only
    controls have independent original reviews and separate owner guards.
    Final transient controls/return preserve all45 shared files and the retired
    sentinel, including epochs. Source checkpoints are pushed; the complete
    shared wall, supplements and final documentation verdict remain open.
  - §2 names exact supplier/consumer readiness and scopes. The Mere reservoir
    plan owns V1–V5 status; bounded C1 proofs do not close full V2b/one-tree UI
    qualification. Graphical unlock handling remains unqualified; the controlled
    production terminal lane is explicitly scoped.
  - C2 narrates the reading's computation. Then, in Mark's ruled order: typed
    cards across decks, card ambient view, shuffled deck/reversals, situation,
    labelled generated inferences and additional systems. The journal-of-occasions
    scene and focused card constellation remain proposals for those later gates.
  - The mere starts fresh with no import. Appearance/font-role publication and
    historical F/G qualification receipts retain their separate scopes. All
    source, lifecycle, authority and later narrative gates preserve that order.
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

Granular appearance continuation (2026-10-12): the existing
[appearance plan](archive_docs/2026-10-11/2026-10-11_tabard_appearance_plan.md#granular-font-roles--2026-10-12-continuation)
records shared font-family roles, licensed bundled faces, explicit preference
Apply, and current coherent supplier adoption. Consumer macOS qualification is complete: 101 root tests, 17 portable-core tests,
two native processes, nine reviewed captures and unchanged exact font requests
and durable IDs on reopening. Prior theme captures retain their original revision
scope; other consumer/platform and glyph/frozen-face gates remain separate.

C1 qualification (2026-10-14):
[the qualification ledger](../receipts/c1-resident-qualification_20261014.json) records
published exact-float supplier H, portable validator f7ec and native N,
immutable CPU/owner gates and 19 independently reviewed fresh H native
captures. Controlled process-local terminal fallback has a separate declared
scope; graphical unlock remains unqualified. The F/A3/D checkpoint is preserved.
The divination mere plan retains the full C1 done-conditions. Current H native
Graphshell witness/reopen has six independent original reviews and owner guards;
the mounted browser witness has five. Fresh Chrome/live/archive proofs also pass with independent reviews and whole
owner guards. The full shared wall and final documentation remain pending.

Main integration (2026-10-15): the resident/C2 lane and superseded portable
branch are merged; [integration checks](../receipts/main-integration_20261015.json)
pass with the qualified product tree unchanged. The later coordinated round
and branch-source convention are published; the handoff owns the personal-sync
source follow-up and intake checks. Endpoint registration/admission uses graphshell-host;
personal-sync still uses the Graphshell port.

Family software intake (2026-10-15): `9aae258` supplies the optional assertion
basis in selected-sync exports. The
[family intake receipt](../receipts/family-intake_20261015.json) records one
canonical source per repository, 92 default and 130 historical tests, core
18/19/20, active sky/sync fixtures and both all-target checks. Prior native
qualification retains its source/capture scope; Mere's lift plan records the
bounded P4 endpoint consumer proof.

C3 software and bounded native qualification (2026-10-16): the divination plan and
[software receipt](../receipts/c3-software-qualification_20261016.json) record
two complete 78-card text decks, typed identity independent of printed number,
cited Golden Dawn/Thierens overlays, non-writing crosswalk inspection and saved
content revisions. Gates pass at 100 default, 139 historical and core 24/25/26.
Existing fields and all prior receipts retain their identities. Fixed minor
prompts disclose model-assisted editorial provenance. The
[native receipt](../receipts/c3-native-qualification_20261016.json) records
107 creation and 55 reopening frames, eight reviewed originals, exact IDs and
byte-equal card payload. The [publication ledger](../receipts/c3-publication-intake_20261016.json)
records published C3 source and Mere’s separately qualified receiving intake.
C4/C5 and shared-wall acceptance remain separate.

Bounded shared natal qualification (2026-10-17): the [divination plan](2026-09-23_divination_mere_plan.md) and [C8 natal receipt](../receipts/c8-natal-qualification_20261017.json) bind shared birth resources, exact natal inputs, software gates, native captures and offline reopening. Earlier C7/C8 receipts retain their original scope.

The [hardened natal intake](../receipts/c8-natal-hardened-intake_20261017.json) records the actual-vault admission guard, unchanged history and source-bound consumer qualification on Mere d03e367bd.

Bounded Whole Sign/Equal houses and ascendant: [qualification receipt](../receipts/natal-houses-qualification_20261017.json) and [evidence notes](../receipts/natal_houses_20261017/README.md). Numerical authority is Turquet; original natal receipts and the planetary edition retain their source identity.

Bounded natal motion, observed Earth orientation and Placidus: [plan](2026-10-17_natal_cap_plan.md), [qualification ledger](../receipts/natal-cap-qualification_20261017.json), and [evidence notes](../receipts/natal_cap_20261017/README.md). Earlier planetary and house editions remain immutable.

Natal aspects, graphical unlock and macOS packaging: [plan](2026-10-17_aspects_unlock_packaging_plan.md). Explicit per-aspect rules retain geometric results separately from the original natal edition; packaging and graphical-startup evidence have separate acceptance boundaries.

Natal-aspects [qualification ledger](../receipts/natal-aspects-qualification_20261017.json) and [evidence notes](../receipts/natal_aspects_20261017/README.md): 150/199 product tests, core 59/60/61, ten reviewed native originals and exact reopen, with graphical/private-key access blockers and the ad-hoc candidate recorded separately.

Graphical unlock [qualification](../receipts/graphical-unlock-qualification_20261017.json) and [evidence notes](../receipts/graphical_unlock_20261017/README.md): final extracted package-04 launcher, actual production dialog and retry, zero terminal answers, six exact readings/two reflections and one reviewed original. Earlier failed diagnostics remain intact; Developer ID signing and public release remain open.

Developer ID signed native [qualification](../receipts/developer-id-native-qualification_20261017.json) and [evidence notes](../receipts/developer_id_native_20261017/README.md): Apple-anchored timestamped signatures/hardened runtime, two graphical processes, six reviewed originals, exact readings/reflections/shared resources and unchanged earlier receipts. Notarization and app distribution remain separate.

Duplication reduction: [plan](2026-10-18_duplication_reduction_plan.md). Consolidate repeated mechanisms with explicit authority and replay boundaries; first remove duplicate aspect geometry, then qualify shared resident materialization across real consumers.

Shared resident materialization: [consumer evidence](../receipts/resident_materialization_20261018/) and [read-only verifier](../receipts/verify_resident_materialization.py). Cleromancy uses the shared complete reader and has bounded native lifecycle/fresh-process reopening evidence; supplier and Knot status live in `mere/design_docs/2026-10-18_resident_materialization_plan.md`. Earlier qualification and signed packages retain their frozen identities.
