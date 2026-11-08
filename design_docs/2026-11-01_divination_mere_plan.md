# Cleromancy: the divination mere, open work

**Date:** 2026-11-01
**Status (2026-11-01):** cut from [the divination plan](archive_docs/2026-11-01/2026-09-23_divination_mere_plan.md), archived whole with the history, rulings and receipts. C1 to C8 (first die) and the natal, aspects and packaging slices have bounded qualification; **no phase is accepted**, which is Mark's verdict to give. Open work follows.

## Decisions

### divination D1: cut the plan, draft first (2026-11-01)

Mark, 2026-11-01: "Cut it, draft first". The 1,319-line divination plan is cut under DOC_POLICY §8: its open, deferred and follow-on items become this plan; the old plan is archived whole after this draft is reviewed.

Links: area divination mere · replaces 2026-09-23_divination_mere_plan · summary old plan cut to open work and archived whole

### divination D2: one plan for every open Cleromancy item (2026-11-01)

Mark, 2026-11-01: "New divination plan". The extracted leftovers (headed items 5-7, the Forms-family gate, legible scroll jank, spatial authoring and camera persistence, handoff L13/P8) do not get their own plans. L13/P8 sits in the duplication plan beside its Graphshell-port row (handoff D1); everything else is below.

Links: area divination mere · draws on headed D1, handoff D1 · summary all open Cleromancy items live in this plan

### divination D3: receipts leave the repository (2026-11-01)

Mark, 2026-11-01: "Move everything", refined after compiled-in scenarios were found to "Scenarios to repo dir, rest out". The 76 `.scn` files are source: they live in `scenarios/` (14 are `include_str!`-ed by `src/ui/native/scenario_driver.rs`, 7 are read by `appearance_tests.rs`). The other 1,281 tracked receipt files and the 175-file tabard folder live in `Code/testing/cleromancy/receipts/` and `Code/testing/cleromancy/tabard_20261011/`; `receipts/` is a gitignored local output directory.

Links: area receipts · applies DOC_POLICY §8 · summary scenarios to scenarios/, all other receipts to Code/testing/cleromancy

### divination D4: old hashes (2026-11-01)

Mark, 2026-11-01: "Rewrite live docs only". Live plans cite hashes as they are on main; archived plans and receipts keep the hashes they were written with. The mapping from each pre-merge hash to its main hash is a table in `archive_docs/receipts/2026-09-23_divination_mere_plan_receipts.md`.

Links: area documentation · summary live docs cite main hashes; archived docs keep old ones; mapping table in the receipts file

### divination D5: frozen-replay fixtures stay in the repository (2026-11-01)

Mark, 2026-11-01: "tests/fixtures/natal/". Three receipt files are read at compile time by `tests/natal.rs`; they moved to `tests/fixtures/natal/` (flattened, names kept recognizable) instead of leaving with the receipts. They are also in the Code/testing copy.

Links: area receipts · refines divination D3 · summary three natal replay fixtures live in tests/fixtures/natal/

### divination D6: the move to a causal-log mere starts fresh (2026-11-06)

Mark ruled in Mere's domain contract (`mere/design_docs/mere_docs/technical_architecture/2026-11-06_domain_contract.md`, O8 and C1) that every mere becomes one Stickleback causal operation log, and that Cleromancy's existing `divination` and `personal-facts` meres are not converted: "Fresh start". When Mere lands the causal-log store, the new meres begin empty, and the linear-journal stores stay on disk untouched and are no longer opened. Retiring them is Mark's call. `validate_divination_graph` carries over unchanged as the admission policy.

The contract check (same document, "Cleromancy against the contract") found Cleromancy conforming through the route, with two changes on this side: the theme choice in `appearance.json` moves to the person's settings (Mere adoption plan D7), and the `reading-workspace` view splits so its layout replicates while selection and focus stay on the device (contract C7). Neither is scheduled.

Links: area divination mere · follows mere domain contract O8, C1 · summary new causal-log meres start empty; old stores kept, not converted

## Open work

Every item is open as of 2026-11-01. Constraints that bind it: no app-local copies of stack capabilities (sessions, journal, codicils, reservoir and engine registration are Mere's); everything stays replayable and a record that fails replay is refused before it is written; generated material is always labelled; place and network sources never switch on by default; narration claims no more than its receipt supports. The existing proof wall stays green, and headed receipts run at the ordinary window size with painted bounds and native capture.

### O1. Shared one-tree wall (reservoir V2b)

Mere owns it: `mere/design_docs/mere_docs/implementation_strategy/2026-10-28_reservoir_plan.md` and, for the Graphshell cutover, `2026-10-28_graphshell_host_plan.md` beside it (their predecessors, with the V1-V5 history, are under `mere/design_docs/archive_docs/2026-10-28/`). At the last count (2026-10-14) 21 of the 164 original canonical cases were closed, with 43 reviewed originals; the other 143 and the mandatory supplements are open. Cleromancy's bounded C1 proofs (five fresh native runs, Graphshell witness and reopen, Chrome reopen, staged archive and read-only controls) do not close it.

**Done when:** the complete 164-scene wall and its mandatory supplements pass on the current published supplier, with independent reviews and unchanged owner guards, recorded in Mere's plan and cited here.

### O2. Startup, cold-start and performance

Cold startup is bounded and imperfect: a thirty-second inspector timeout and the combined-flow presentation stall (C6, 2026-10-16) remain explicit, as does the first cold epoch 8 to 10 byte failure; successful startup in every environment is unproven; native scroll and frame time are unmeasured (see O6a). Graphical unlock and signing are owned by `2026-10-17_aspects_unlock_packaging_plan.md`; notarization and public release stay there.

**Done when:** Mark names a cold-start budget, and repeated fresh-process cold and warm opens meet it with no timeouts, recorded as a receipt.

### O3. Further systems (C8, not ruled)

Dice is the first system. The 2026-09-23 case ranked, strongest first: geomancy, the I Ching, Lenormand, and sortes for drawing from a chosen work; it argued against runes. None is ruled, and horary charts wait on Turquet houses work.

**Done when:** Mark rules a system in, after which it takes its own dated gate with done-conditions.

### O4. Journal of occasions, card constellation, and deck swatch

Proposals, not gates, recorded in the archived plan's C4 section: the reading-centered scene (2026-10-13), a journal of reading occasions with a contextual constellation around the selected occasion, and a bounded card neighborhood (history, correspondence, situation) on selecting a card; and Mark's deck swatch and dealing proposal (2026-10-16), a deck as an embedded swatch with a floating comparison surface and an interactive reveal backdrop. A dynamics-based cast is a later exploration candidate that would need Mere's G8 and G10 determinism contracts. These depend on shared adoption gates S2 and B1 in Mere and on the dynamics grammar plan (`mere/design_docs/mere_docs/implementation_strategy/2026-10-28_dynamics_grammar_plan.md`).

**Done when:** Mark rules each proposal in or out; if in, it takes its own dated gate.

### O5. Phase acceptance

C1 through C8 (first die) and the natal editions each have a bounded ledger under `Code/testing/cleromancy/receipts/` (indexed in the archived receipts file). None is accepted.

**Done when:** Mark accepts or rejects each phase, recorded as a Progress line here.

### O6. Items extracted from archived plans

- **a. Scroll jank** (legible plan): native scrolling jank in the reading scene is an open performance question. 48 bounded wheel steps pass without reversal or jump; that is not a GPU frame-time measurement. **Done when:** frame time is measured on the native host at the ordinary window size and the jank is bounded or fixed.
- **b. Spatial arrangement authoring and camera persistence** (legible plan): the reading scene uses three fixed columns in authored position order. Per Mark's 2026-09-23 revision, readings are what get arranged, not spreads. **Done when:** a reading arrangement can be authored and its camera persisted, replayably and without changing sealed reading semantics.
- **c. Pattern** (headed item 5): project lived sessions, facts, reflections and concurrence through Scenograph; statistical views disclose method, sample size and missing data; concurrence keeps its narrower meaning. **Done when:** a consumer-backed view passes with those disclosures.
- **d. Private sync controls** (headed item 6): render the existing Cleromancy sync selection once the shared `SettingsProvider` renderer has two converted consumers. **Done when:** that precondition holds and the controls pass a headed receipt.
- **e. Resident catalog** (headed item 7): integrate the installed product route generically with Knot and Cleromancy, retaining Personae and Graphshell authority. **Done when:** both products use one route integration with their receipts unchanged.
- **f. Forms-family adoption** (headed "separate gate"): mechanical compatibility with the existing public pins qualified on 2026-10-08. Adopting Mere's new Forms family needs its own coherent published pins and consumer gates; no labels are added to currently unnamed fields. The F5 scope ruling is `genet/design_docs/2026-10-07_forms_value_validation_submission_plan.md`; the F6 accessibility projection is `mere/design_docs/cambium_docs/technical_architecture/genet-compatibility.md`. **Done when:** a coherent published family is pinned and the affected native classification and headed DOM fixtures pass on the locked graph.

### O7. Existing automated consultation and ambient-route failures

The Windows baseline for the [H7 source pass](2026-11-03_source_splits_plan.md) finds an absent aria-expanded false attribute in consultation_authoring_dom and a declared app-broker route open returning NotFound in granted_ambient. Both precede and survive the source moves; the existing public-seed golden only needed canonical LF checkout bytes.

**Done when:** the two named integration cases pass on the locked graph while retaining authored-action and live-grant/revocation behavior, with the source plan's external evidence used as the control. This is product follow-up and does not imply phase acceptance.

## Findings

- 2026-11-01: Isocosm no longer depends on Cleromancy: its commit `9c991e87` ("Remove Cleromancy from the VTT") removed the dependency, and its `Cargo.toml` has no Cleromancy entry. The headed plan's second-consumer item is therefore closed, not open.

## Progress

- 2026-11-01: Plan cut from the divination plan; decisions D1 to D5 recorded. Receipts and old-hash mapping live in `archive_docs/receipts/2026-09-23_divination_mere_plan_receipts.md`.
- 2026-11-08: Mere cycle breaks P3 steps 1 and 2 (the validator as a wasm guest; the plan and the measurements are in Mere's `2026-11-06_cycle_breaks_plan.md`, Findings, "P3 spike"). `core/Cargo.toml` no longer names `muniment`'s `redb` feature (only the receipt binary in the root package opens a redb store), which was the whole wasm blocker on `wasm32-wasip2`. `tests/authority_corpus.rs` is the corpus of the seven tests that move from Mere's `embedded_reservoir_validation`, with native verdicts; `spike/wasm-authority` is the throwaway component and its wasmtime harness (own lock root). Not the real guest, held for Mark's review (D15 (h)).
