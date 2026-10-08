# Cleromancy: the divination mere, and a reading that explains itself

**Date:** 2026-09-23
**Status (2026-10-08):** C1 preparation in progress; no phase is accepted.
The existing local
consultation works, but still owns one snapshot store and its own authority.
Mere's reservoir V1 and V2, and V2b's component, harness and route adapter,
have landed. C1 still needs the remaining reservoir contracts and Cleromancy's
domain-authority integration. The dependency map below replaces an opaque
"wait for V1–V5" handoff with named acceptance gates. §7's 2026-09-23 rulings
and the execution order remain in force.
**Scope:** move Cleromancy onto its data domain's mere in the identity's
reservoir, then rebuild the reading around Mark's 2026-09-23 rulings: a
reading that narrates its own computation, typed cards with every tradition's
correspondences, an ambient tier around whatever card is in focus, and a draw
that behaves like a shuffled deck.

**Related:**
- `repos/mere/design_docs/mere_docs/design/2026-09-23_ambiance_design.md`: the
  full record of the rulings, the ambient tier, and the attention and keeping
  axes.
- `repos/mere/design_docs/mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md`:
  the Mere side. Meres are held by the device resident, and any of an
  identity's applications can open them.
- [Legible reader and fact surfaces](2026-09-04_legible_reader_and_fact_surfaces_plan.md),
  the predecessor slice. Its north-star line, "spreads as arrangements", is
  revised below.
- `repos/mere/design_docs/mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md`:
  V2b's Graphshell cutover, distinct from reservoir storage and lifecycle.
- `repos/mere/design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`:
  shared motion terms, channel bindings, compositions, persistence and replay.
- `repos/mere/design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md`:
  shared projection authoring and editing; its status owns those capabilities.

## 1. Rulings

All given by Mark on 2026-09-23. The ambiance design records them verbatim.
Those that were choices among offered options are marked (o).

**Domain and storage.**
- Cleromancy's data domain is "Divination, journaling, rng". Its neighbours
  are Knot (notes, seeds, prompts), Redshank (notes) and Isocosm (seeds).
- The domain gets its own mere in the reservoir, and the Mere change comes
  first (o).
- The mere shows the full session lifecycle: mint, switch, fork and trash (o).
- Cleromancy's domain authority is composed into the resident (o).
- Cleromancy's `ReadingSession` keeps its name and is always written "reading
  session" in prose (o).
- No existing store needs importing (o).

**What a reading owes the person.** Of the old constraints, only "everything
replayable" holds. "No generated interpretation", "authored packs only" and
"the sky is never causal" were not Mark's constraints. Auditable "is not the
same thing as putting a big block of unstyled text under a reading". Other
presentation mistakes Mark named:
- making basic provenance a feature;
- separating the local context from the reading, "opposite of what people might
  want!";
- blank authoring boxes instead of "an example to riff on and consider in
  context".

The model a reading should follow is his own words: "I'm running this
computation on this data to derive a card from this deck and place it in this
orientation in this position in this spread, the nth of m cards i've drawn so
far". Position meanings show on hover. Clicking a card shows similar cards
"along the axes of meaning (suite, number, arcana, decks, other systems)".

**Generation.** "It's necessary to be honest when things are randomly or
otherwise generated, and letting people choose from what, and how random."
- Formula-derived inferences recompute exactly (o).
- Model-generated ones are stored when produced, with inputs, model identity
  and output, and labelled (o). The keeping level they enter depends on the
  model and the purpose.

**Relations.** Tensions and synergies come from traditional methods, from
user-configurable methods, from the person's history, and from real data:
"Tarot, weather, lunar/solar cycles, seasons, divination". The typed
dimensions are handled "atomically, not blobbed together on a canvas".
Readings, not spreads, are what get arranged: "there is no value to arranging
spreads relative to each other".

**The draw.** A spread draws like a physical deck: one shuffle per reading, no
repeats, orientation from the shuffle, and the shuffle method in the receipt
(o). Reversals belong in this work (o).

**Situation.**
- Time is always recorded; place is opt-in (o).
- Local-first holds: network sources such as weather are opt-in, and what they
  return is stored with the reading (o).

**Systems.** A choice of tarot decks comes first. Dice and horoscopes follow.
Other systems enter only on argued merit.

**Order.** After the mere comes the reading narrative (o). Then, in order (o):
- typed cards and decks;
- the ambient card view;
- the draw;
- situation;
- generated inferences;
- more systems.

## 2. Findings

### Current source and native review (2026-10-07)

Reviewed Cleromancy `32a4b948` and Mere `cd3ebf26d`, both fetched from their
published main branches. This refresh changes documentation only.

- **The reservoir consumer has not landed.** `CleromancyHost` still stores
  `PersistedHost` (graph, facets and projection counters) in one muniment slot
  ([`host/mod.rs`](../src/host/mod.rs)). `CleromancySessionAuthority` still
  owns its application graph ([`admitted.rs`](../src/admitted.rs)). C1's
  resident-held divination domain is not established by the existing native
  reopen proof.
- **The scene is a bounded adapter.** [`reading_scene.rs`](../src/reading_scene.rs)
  interns reading sources with separate placement occurrences, but manually
  positions three columns. The DOM realization uses those coordinates
  ([`ui/view/reading_scene.rs`](../src/ui/view/reading_scene.rs)); there is no
  editable relation neighborhood or dynamics binding. The manifest pins Mere
  `8106c7c2`, so current platform capabilities need a tested adaptation.
- **Authored positions lose their meaning in the reader.** The renderer
  derives labels and stock rationales from position ids instead of the saved
  template ([`ui/view/reading.rs`](../src/ui/view/reading.rs),
  [`reading_scene.rs`](../src/reading_scene.rs)). A saved label "What I must
  release" under `foundation` displayed "Foundation" and the built-in
  grounding rationale. C2 must resolve the saved template; C3 must carry its
  meaning as versioned data.
- **Audit presentation needs the reading narrative.** The expanded Cambium
  detail panel has no host styles for its label/value classes, so labels and
  explanations run together. Context authoring, large status identifiers and
  header space compete with the focused reading. Chart entry exposes UTC
  strings and microdegree/millidegree units. These are C2 presentation findings,
  not evidence that a new storage layer fixes the interface.
- **The old draw behavior remains.** The built-in field has 22 upright Major
  Arcana, and each position casts independently over the full field
  ([`tarot.rs`](../src/tarot.rs), [`host/spreads.rs`](../src/host/spreads.rs)).
  C5's shuffled deck, non-repetition and reversals have not landed.
- **Verification is bounded.** The four-feature suite
  `cargo test --features analytic-ephemeris,sky-timeline,personal-sync,graphshell-admission`
  passed. Native tarot first/reopen and astrology lanes passed; the reading's
  ids matched and its card previews were byte-identical after a fresh process.
  Three 2320×1520 captures were reviewed at the default 1160×760 window.
  The standalone core's documented locked gate fails before compilation:
  its manifest pins Mere `8106c7c2` while its lock retains `876320fd`.
  `cargo fmt --all -- --check` also fails. These failures remain open.
  The local review receipt and captures are retained under
  `Code/testing/cleromancy/review-20261007/`; narrow windows, high zoom and
  release packaging were not qualified.

### Dependency and readiness map (reviewed 2026-10-08)

The reservoir plan owns reservoir status; the one-tree plan owns Graphshell's
cutover; the dynamics and Scenograph editor plans own their shared contracts.
Their historical receipts do not count as Cleromancy consumer acceptance.

| Cleromancy target | Required contract or proof | Readiness and remaining work |
| --- | --- | --- |
| C1: resident-held domain and lifecycle | Reservoir V1–V2; composed Cleromancy domain validator | Shared index, routes, sessions and journal exist. Every write entrance still needs Cleromancy replay/record validation, with cross-application refusal tests. |
| C1: common session view | Reservoir V2b | Component and route adapter exist; Graphshell cutover/panel acceptance and Cleromancy embedding remain separate unfinished steps. |
| C1: session archive | Reservoir V3 | Codicil helpers exist; reservoir save/open/fork/compose integration and its receipts remain open. |
| C1: access and ambient crossing | Reservoir V4 | Initial first-party route grants exist; recorded denials and per-mere ambient consent remain open. |
| C1: standalone ownership | Reservoir V5 | Qualify embedded operation without Djinn, client attachment with Djinn, and refusal of a second owner. |
| C2: computation narrative and authored position meaning | Stored reading, field, context and template definitions | Source analysis, narration design and template-resolution preparation can proceed against existing receipts; they do not complete C1 or change the ruled execution order. |
| C3: typed cards, decks and interpretations | Cleromancy-owned schemas, versioned content and cited traditions | Prepare the domain binding and remove view-owned meaning when this phase executes. Mere does not supply divination semantics. |
| C4: interactive contextual neighborhood | C3 binding; shared projection definitions/compiler and scene editing | Prepare explicit relation families and a consumer fixture. Focus and curation use owner actions; navigation never implicitly keeps an item. |
| C4: authored dynamics and retained composition | Shared dynamics grammar, especially G4; G2 for its unified channels | G2 and G4a's portable spec core have since landed; G4b's runtime binding and end-to-end choice retention remain separate gates. The grammar plan owns their current status. Do not introduce an app-private spec or treat a pin bump as adoption. |

Preparation means schemas, mappings, source-backed fixtures and acceptance
design. It must not install a second persistence owner or claim an unfinished
phase complete. The full C1 gate still includes V1–V5 under the existing ruling.

### Authorized orchestration (2026-10-08)

Mark authorized publishing this refresh and proceeding with the plan. The
first implementation wave runs the independent reservoir contracts in separate
lanes: V3 archive, V4 recorded denials and ambient grants, and V5 embedded/client
ownership. Integration follows their scoped tests and real-process receipts.
Graphshell's one-tree cutover and mounted session panel remain V2b gates; this
wave does not bypass them or move C2 ahead of C1.

Cleromancy supplies a read-only complete-candidate validator in
[`host/domain.rs`](../src/host/domain.rs). It checks canonical record identity,
supported domain facets, sealed reading replay and stored dependencies using
the existing product decoders. It opens no store and leaves the candidate
unchanged. The shared resident must install it before any graph, journal,
archive or imported-session write; a helper's presence does not establish that
composition. A coordinated integration pin follows the shared lanes, then the
client and shared mere view replace the private authority and snapshot store.

The core lock is aligned to the manifest's existing Mere revision. The locked
sky-timeline gate now passes; the dated failure above remains historical.
Domain-specific refusal tests run in the portable core. These are source
checks; the new resident path still needs native lifecycle, cross-application
visibility and fresh-process reopen qualification on the integrated revision.

### Historical findings (verified 2026-09-23)

The following records the original assessment, including its then-current
store and consumer observations. Recheck consumer revisions before C5; the
stale Isometry/Isocosm checkout identified during the 2026-10-07 review is not
current compatibility evidence.

- **Storage.** `CleromancyHost` holds one Mere kernel `Graph` and saves it
  whole, as one snapshot document in one muniment slot
  ([`host/mod.rs`](../src/host/mod.rs)). It has no pandect sessions, no graph
  journal and no codicils. It runs its own resident authority,
  `CleromancySessionAuthority` ([`admitted.rs`](../src/admitted.rs)). The
  reservoir plan's device-resident model replaces that authority.
- **No data to migrate.** No store exists at the default data root,
  `%LOCALAPPDATA%\cleromancy`, on the primary development machine, and Mark
  confirms there are no real readings elsewhere. Headed test stores live under
  `C:\t\` and regenerate.
- **Spreads repeat cards.** Every position is an independent cast against the
  full field ([`host/spreads.rs`](../src/host/spreads.rs), both the three-card
  and template paths). Over the 22 Major Arcana, a three-card spread repeats a
  card about 13% of the time, and a ten-card spread about 91%.
- **Interpretation lives in four carriers.**
  - `Candidate.interpretation` is sealed into the field.
  - `SkyRulePack` carries a pack id, author, version and digest
    ([`sky.rs`](../src/sky.rs)).
  - The edition-1 chart prompts are `match` arms in a view
    ([`ui/view/astrology_reading.rs`](../src/ui/view/astrology_reading.rs)).
  - Position rationales are another view-level `match`
    ([`reading_scene.rs`](../src/reading_scene.rs)).
- **Cards carry no typed axes.** The pack is upright-only by A6's decision, and
  a card is `{id, title, prompt, 3 tags, base_weight}`. Its number exists only
  inside the id string ([`tarot.rs`](../src/tarot.rs)). It has no typed suit,
  arcana, element, planet or sign.
- **Isocosm consumes derived selection.** Its games-wing plan pins Cleromancy
  at `3b321539c5bc854403623087d8af18dcef6f2b53`
  (`repos/isometry/design_docs/2026-09-09_games_wing_consolidation_plan.md`).
  Receipt changes need a path that keeps that consumer working.

## 3. Phases

### C1. Cleromancy's mere

Once the reservoir plan's V1–V5 land, Cleromancy's graph moves out of its
single snapshot slot into the divination mere. Reading sessions, readings,
contexts, fields, charts, sky facts and reflections become records inside the
mere's sessions. The graph journal records every move, and Eidetic archives
sessions as codicils.

Use the readiness map in §2 to name the missing contracts and their proofs.
V1–V2's completion does not install Cleromancy's validator in the resident.
V2b's route adapter does not establish the product's embedding, V3 archive
integration, V4 denials/ambient grants, or V5 standalone ownership behavior.

Cleromancy's domain authority, meaning replay before write, receipts and
record validation, is composed into the resident the way Knot's was in the
device resident plan's R3. Every application's writes to the divination mere
pass through it. `CleromancySessionAuthority` and Cleromancy's own persistence
path retire. Standalone Cleromancy embeds the resident library, domain
authority included. Nothing is imported: the mere starts empty.

**Done when:**
- Cleromancy opens its mere through the resident and never opens the
  reservoir's files;
- mint, switch, fork and trash work from Cleromancy's interface, through the
  shared Cambium mere view (reservoir plan V2b), not a view of Cleromancy's
  own;
- a reading made in Cleromancy is visible to another application, such as
  Graphshell, opening the same mere;
- every receipt still replays, and a record that fails replay is refused
  before any write, including a write from another application;
- the proof wall passes after the old authority and persistence path are
  removed.

### C2. The reading narrative

The reading explains its own computation in plain words, card by card. Each
card's narration says which deck, which method, what randomness and from where,
which card, which position, and its place in the draw. Position meanings appear
on hover. The context appears with the reading, not in a separate region.
Reflection and context entry offer examples drawn from the reading instead of
blank boxes. The receipt stays available, but auditability is the narration,
not a raw block of workings.

Resolve each position's label and meaning from the saved template rather than
its id. Normal date/time and angle entry belongs in the product surface;
serialized UTC strings, integer units and import details remain inspectable
at the calculation boundary. Any expanded detail panel supplies readable
label/value geometry through the host's styling contract.

The narration says only what the receipt supports. Until C5's shuffled deck
lands, it says each position is drawn independently, so a card can recur, and
that every card is upright.

**Done when:**
- a headed reading at ordinary window size narrates every card with deck,
  method, source of randomness, position and index, in plain text that
  accessibility tools can read;
- hovering a position shows its meaning;
- the context and the reading appear together;
- the reflection box opens with at least one example taken from the reading it
  belongs to;
- an authored position retains its saved label and meaning in the scene,
  focused reading and accessible narration, including a custom label on a
  built-in position id;
- the narration states the current draw's independence and upright-only
  orientation, and changes when C5 lands.

### C3. Typed cards and several decks

Cards carry typed axes: number, suit, arcana, and, from each tradition,
element, planet and sign. Several tarot decks are available as typed data.
Each tradition's correspondences are recorded with their source and date and
shown side by side. The four interpretation carriers become typed, versioned
data, so nothing interpretive is hardcoded in a view.

**Done when:**
- two decks are loaded and a card crosswalks between them;
- every correspondence names its tradition, source and date;
- no view contains interpretive text.

### C4. The ambient card view

Around a card in focus, registered engines generate its ambient tier. Each item
carries its reason, method and engine version. The ambient tier holds:
- cards near it on each axis;
- the same card in other decks and systems;
- relations between the cards of a spread, by named methods;
- its earlier appearances and what was written about them;
- the sky at the draw.

The ambiance design's two axes apply. Examining an item puts it in short-term
memory, keeping promotes it, and focus never promotes.

**Scene and dynamics refinement (2026-10-07; proposed consumer design).**
Cleromancy supplies typed facts, cited correspondences, interpretation versions,
reflections and permitted actions. Mere supplies projection authoring,
arrangements, navigation, motion and shared editing. Foreground/background
emphasis, provenance, and seeded/anchored/pinned placement are independently
editable dimensions. Both foreground and background remain interactive.

The first dynamics proof uses explicit relation families: correspondence under
a named tradition, earlier appearances/reflections, and saved sky observations.
Those families can be inspected or chosen separately. Inferred similarity may
join them with its inputs, method and version disclosed; it is not a
prerequisite for the first explicit-relation fixture. Material from other
meres enters the ambient level only through V4's per-mere, per-app consent.

Compose shared terms and targets rather than implementing a Cleromancy force
catalog. A settling composition separates items and gathers related material;
a living composition may retain circulation or other sustained motion. The
person can pause, use a still/reduced-motion presentation, disturb an item,
anchor it or pin it. A timeline's encoded time axis stays held while motion
uses the remaining freedom. Available shared terms and permitted actions set
the supported choices; unsupported terms or channels are refused explicitly.

Consume G4's versioned `DynamicsSpec` when its persistence contract lands.
The save records the recipe, relation/channel bindings, placement roles and
curation separately from the sealed reading. Reopening must preserve the draw,
position meanings, interpretation revisions and reflections. Scene dynamics
do not silently draw again, rewrite an interpretation or promote keeping.
Cleromancy must demonstrate its own save/reopen and moving-selection behavior;
Graphshell or another product's receipts are not consumer adoption.

A dynamics-based cast is a later exploration candidate, not this scene's draw
method or a C5 requirement. It would require an explicitly chosen selection
method and a replayable receipt with engine revision, parameters, initial state
and recorded inputs/checkpoints. Mere's G8 determinism and G10 input-replay
contracts must be qualified for that method; a random seed alone is not proof.

**Done when:**
- a focused card shows at least three relation families;
- each ambient item names its engine and version;
- examining and keeping move items between keeping levels and appear in the
  journal;
- the same neighbourhood can be shown in at least two scenes, with
  force-directed as the default;
- the first fixture shows one saved reading, a focused card and at least three
  disclosed relation families; changing the lens or disturbing the scene lets
  the person inspect the relationships and their named methods;
- scene and supported dynamics choices save and reopen with focus, roles and
  foreground/background presentation intact, while sealed reading data stays
  unchanged;
- selection and inspection remain usable during motion, and keyboard plus
  still/reduced-motion operation expose the same meaningful actions;
- keeping is explicit, and an opted-out neighboring mere contributes no
  ambient material.

### C5. The shuffled deck

One shuffle per reading; cards come off the top in order with no repeats;
orientation, now including reversals, comes from the shuffle. The receipt
records the shuffle method and its source of randomness, either the operating
system or a public derived seed.

**Done when:**
- a spread never repeats a card;
- the receipt replays the whole draw from the shuffle record;
- every receipt from the current schema still replays;
- Isocosm's derived selection still produces byte-equal receipts at its pinned
  revision;
- C2's narration reports the shuffle and each card's orientation.

Establish that consumer's current checkout and integration pin before running
the compatibility gate; the historical pin in §2 is not a current acceptance
result.

### C6. Situation

Time is recorded with every reading. Place is opt-in. Sky facts come from
Turquet: phase and sign positions always, and place-dependent facts (season,
rise and set, planetary hours) only with place. Weather is an opt-in network
source, and what it returns is stored with the reading.

**Done when:**
- a reading with place off says which facts it could not use and why;
- a reading with weather on stores the observation it used, and replays
  offline.

### C7. Generated inferences

Formula-derived prompts and synergistic readings over the person's history
recompute exactly. Model-generated ones are stored when produced and labelled,
and enter the keeping level their model and purpose set.

**Done when:**
- every generated inference names its method, or its model identity and
  inputs;
- formula-derived ones regenerate byte-equal;
- model-generated ones reopen from storage with their label intact.

### C8. More systems

Dice, then horoscopes; horary charts need houses, which is Turquet work. Other
systems enter one at a time, each on an argued case. The case made on
2026-09-23 ranked, strongest first: geomancy, the I Ching, Lenormand, and
sortes for drawing from a chosen work. It argued against runes. It is not
ruled.

## 4. Stop rules

- No app-local copies of stack capabilities. Sessions, the journal, codicils,
  the reservoir and engine registration are Mere's.
- Everything stays replayable. A record that fails replay is refused before it
  is written.
- Generated material is always labelled, formula-derived or model-generated.
- Place and network sources never switch on by default.
- The narration never claims more than its receipt supports.

## 5. Verification wall

The existing proof wall stays green through every phase. Each phase adds its
done-conditions as receipts. Headed receipts run at the ordinary window size,
with painted bounds and native capture, as the working principles require.

## 6. North-star revision

The 09-04 plan recorded the north-star as "sessions as graphlets, spreads as
arrangements". Mark revised it on 2026-09-23: spreads stay spreads, and what
needs arranging is readings, in relation to prior readings, the available cards,
orientations and positions, and interpretations.

## 7. Decisions (ruled 2026-09-23)

1. **Who validates writes to the divination mere.** Cleromancy's domain
   authority is composed into the resident, as Knot's was (C1). The
   alternatives, accepting only Cleromancy's own writes or a generic reservoir
   validator hook, were not chosen.
2. **The order after C2.** Typed cards and decks, then the ambient card view,
   then the draw, situation, generated inferences and more systems. The ambient
   payoff arrives before the draw changes.
3. **Existing data.** None; the mere starts fresh, with no importer.

## 8. Progress

- 2026-09-23: Plan written. No code.
- 2026-09-23: §7 ruled the same day. The reservoir plan's own decisions were
  ruled too: reservoir under the shared root per persona, stable domain
  identifiers, and Turnstone's lifecycle moved into pandect for every
  application.
- 2026-10-07: source/native review at Cleromancy `32a4b948` and shared-contract
  refresh against Mere `cd3ebf26d`. Recorded the available V1–V2/V2b substrate,
  named the outstanding C1 contracts, and separated preparatory work from phase
  completion without changing the ruled order. Added authored-position and
  native presentation findings, bounded test/capture evidence, and proposed
  C4 scene/dynamics acceptance. No source, pin, store or runtime changes.
- 2026-10-08: Mark authorized push and orchestration. Published the refresh,
  started separate V3/V4/V5 implementation lanes, and prepared the product-owned
  full-candidate replay validator. Aligned the core lock with its declared pin
  and passed its locked sky-timeline gate. C1's resident composition, consumer
  pin, common view and private-store retirement remain unaccepted; C2–C8 keep
  their ruled order.
