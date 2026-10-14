# Cleromancy: the divination mere, and a reading that explains itself

**Date:** 2026-09-23
**Status (2026-10-14):** C1 implementation and bounded native qualification are
complete on the frozen candidate; the shared browser cutover wall and final
documentation publication remain pending. No phase is accepted yet. Shared appearance
and font roles retain their separate published receipts.

The ordinary product attaches to the resident-held divination mere. Its private
snapshot and admitted authority APIs are confined to the nondefault
`legacy-receipts` harness. Daemon and embedded owners install the same immutable
portable Cleromancy validator. The [C1 qualification ledger](../receipts/c1-resident-qualification_20261014.json)
binds current H/N source, locked CPU gates, 19 original H captures, independent
visual reviews, fresh-process recovery, numerical compact control bounds and
controlled terminal-fallback unlock/embedded ownership. The historical F/D
checkpoint retains its separate images, bytes and source identity. V2b still requires the complete 164-scene one-tree
wall and its mandatory supplements; bounded product acceptance does not close
that gate. §7's rulings and execution order remain in force.
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

### Historical F/A3/D qualification checkpoint (2026-10-14)

This subsection records the preserved F/A3/D checkpoint. The H/f7ec/N intake
and actual fresh H qualification are recorded separately below.

Mere supplier `c51e1913cc52cc161815b97939a46452495747e0` and portable
Cleromancy validator `a3d96b62f2f4f908cc0267cbb7bbc29683d9a957` are published on
`codex/c1-resident-divination-20261014`. Native source is locally committed as
`400ce312fd2b2c135449282cbb9e7a1463ab8e97` and awaits publication. The root has one immutable Mere/Genet family, with no local family
overrides. Cargo 1.99 drives the explicit Rust/Rustdoc 1.97.1 CPU gates: 87 default
and 125 historical-feature tests pass; both all-target checks pass. Portable
core default, legacy and sky gates pass 18, 19 and 20 tests respectively.

Final native binary `3510d3edffaf42a15d800b5942fdfc978145b06507e023771ff4ab1120be0026`
passes seed (66 frames/seven captures), fresh reopen (39/four), compact controls
(46/four), ordinary embedded startup (20/two), and a separate stable retained-state run (20/two). The saved synthetic question,
Hierophant/Tower/Hermit cast, reflection and domain IDs survive each accepted
reopen; card data is byte-identical. All 19 original captures have independent
visual reviews. At 420×760, actual scrolling exposes Fork, Trash, Save to archive
and Open archive with painted rectangles equal to their recorded visible
rectangles. Long IDs and horizontal card content still clip: C2 owns that
presentation work.

The first reopen loaded its accessibility tree but stalled with 1,328 redraws,
zero presented frames and a 60-second timeout. Its passing retry does not prove
consistently successful startup presentation. Ordinary startup uses the real
production terminal passphrase prompt on a dedicated PTY; graphical unlock is
not qualified. Same-disk handoff restores the identical eight records and four
live sessions/one archive. Cold cross-application hydration increments only the
Graphshell projection epoch; those failed all-bytes-equal controls remain
preserved. Warm inspections and the second-owner refusal preserve durable
bytes. Read-only focus/layout are local inspection state; they neither fork nor
write a session view, and entering another session clears them.

Final native Graphshell witnesses the same new reading on the same resident
session: 49 frames/five captures, followed by fresh reopen with 19/one.
Its six originals are independently reviewed. Graphshell saves its own Grid
layout and Reading session focus; Cleromancy retains its Hierophant focus.
Exact post-run inspection preserves all durable bytes and Cleromancy-owned
view state. These final executions supplement the source-bound earlier B590
witnesses; browser live/archive and the full wall remain separate gates.

The imported owner capsule separately passes 23 active tests in each stock and
embedded mode, with five intentionally ignored process-child entry points
launched by parent tests. Its dedicated three-owner replay/tamper test passes.
The final C51 browser witness, actual fresh reopen, connected live update and
read-only archive stages pass within their bounded scopes. The full shared
cutover wall remains pending after its first diagnostic failures. The web-only
snapshot repair at `4a5099820` passes its unchanged D3 and selected Bare cases
with independent capture reviews; the remaining full wall and mandatory
supplements are still required. Source qualification, native acceptance, branch publication and
main-branch integration remain distinct ledger entries.

### Dependency and readiness map (reviewed 2026-10-14)

The reservoir plan owns reservoir status; the one-tree plan owns Graphshell's
cutover; the dynamics and Scenograph editor plans own their shared contracts.
Their historical receipts do not count as Cleromancy consumer acceptance.

| Cleromancy target | Required contract or proof | Readiness and remaining work |
| --- | --- | --- |
| C1: resident-held domain and lifecycle | Reservoir V1–V2; composed Cleromancy domain validator | Current H/f7ec/N intake qualifies the registered product-owned validation rules and exact consumer closure. Revision-checked replay/tamper/refusal and five fresh native Cleromancy runs pass. H native Graphshell witness/fresh reopening and separate own-view/byte guards pass; actual H browser witness/fresh reopening/live/archive proofs now pass within their bounded scopes; the full cutover wall and final documentation publication remain open. Historical F/A3/D results retain their original scope. |
| C1: common session view | Reservoir V2b | The common component is mounted in Cleromancy, native Graphshell and the browser tree. Fresh H Cleromancy captures and numerical compact control bounds pass, as do H native Graphshell witness/reopen. Historical F browser/reopen/live/archive proofs are preserved; current H browser witness/fresh reopening/live/archive proofs pass; the full 164-scene wall/supplements remain open. H passes the unchanged original Scene Save/Reopen contract after the exact-float repair. |
| C1: session archive | Reservoir V3 | Fresh H native lifecycle/archive and process-reopen receipts pass through the resident, including legitimate archive thaw creation. Read-only focus/layout remain transient and byte-preserving in exact owner regressions and warm inspections. Historical F staged browser controls pass; H staged Save/exact Open/read-only controls and every-byte guards now pass. |
| C1: access and ambient crossing | Reservoir V4 | Persistent denial, independent ambient consent, read-only scope and idle/live revocation pass resident/process fixtures. C1 preserves that enforcement; trusted owner controls and Cleromancy's card-ambient presentation remain separate later product gates. |
| C1: standalone ownership | Reservoir V5 | Daemon and embedded compositions preserve the same product validation rules. Fresh H live-client and controlled PATH=/bin production terminal fallback qualify encrypted unlock/embedding; second-owner refusal and clean same-disk H helper handoff pass separate native/API receipts. The host graphical backend is available, but graphical prompt handling remains unqualified. |
| C2: computation narrative and authored position meaning | Stored reading, field, context and template definitions | Source analysis, narration design and template-resolution preparation can proceed against existing receipts; they do not complete C1 or change the ruled execution order. |
| C3: typed cards, decks and interpretations | Cleromancy-owned schemas, versioned content and cited traditions | Prepare the domain binding and remove view-owned meaning when this phase executes. Mere does not supply divination semantics. |
| C4: interactive contextual neighborhood | C3 binding; shared projection definitions/compiler and scene editing | Prepare explicit relation families and a consumer fixture. Focus and curation use owner actions; navigation never implicitly keeps an item. |
| C4: authored dynamics and retained composition | Shared dynamics grammar, especially G4; G2 for its unified channels | G2 and G4a's portable spec core have since landed; G4b's runtime binding and end-to-end choice retention remain separate gates. The grammar plan owns their current status. Do not introduce an app-private spec or treat a pin bump as adoption. |

Preparation means schemas, mappings, source-backed fixtures and acceptance
design. It must not install a second persistence owner or claim an unfinished
phase complete. The full C1 gate still includes V1–V5 under the existing ruling.


### Exact replay repair intake and fresh H qualification (2026-10-14)

The original shared Scene Save/Reopen contract exposed decimal JSON facet
floats changing by one and two ULPs during journal replay. Mere H
`a1309431f935427607da35844376a7511656fea4` enables exact float parsing without
weakening the whole graph/facet guard. Its new exact-bit replay and persisted
scene regressions pass, and the unchanged original headed Scene contract passes
on a fresh H physical profile. Supplier H is pushed on the C1 branch. Four H
shipping packages and seven native binaries have rebuilt; CPU Viewer passes
its unchanged delivery caps. Previous F/G artifacts retain their identities.

Portable validator `f7ec0db556ca69a4f1d5a2b917409d7b2d861ec8` is published on
`codex/c1-float-roundtrip-core-20261014`, with only its manifest and lock changed.
Its default, explicit legacy and legacy-plus-sky gates pass 18/19/20 tests with
zero failures or ignores. Native N
`04bd24094301b91f1d39ee541554be9f1b9c5081` pins H and that exact validator; its
87 default and 125 historical-feature tests, both all-target checks and native
build pass. Cargo 1.99 drives explicit rustc/rustdoc 1.97.1. All 145 recorded
compiled inputs match N Git blobs. The exact native binary is
`537c6d7bbc870190b297b42f547dad4d8b7be9b7ae856d2ebac29e592d1a5cf1`.
The exact imported H/validator owner capsule passes 24 stock and 24 embedded
tests, with five intentional child-entry ignores per mode and a separate
three-owner replay/tamper pass. All 846 common registry names match versions.

A fresh public H reservoir is established through the product, with no copied
owner data. Its Fool/Hermit/Temperance cast, reflection and record IDs survive
actual fresh processes. Five positive Cleromancy processes have 19 independently
reviewed originals: seed 7, reopen 4, compact controls 4, retained 2 and ordinary 2.
The compact reveal preserves exact painted/visible control rectangles while
third-card and long-ID clipping remains C2 presentation debt. Reopen performs
one legitimate archive thaw, giving four live sessions, eight domain records,
six edges and one archive. Retained GUI selection is that fourth thaw; explicit
original-session inspector attachment is a separately scoped operation.

Ordinary startup uses the real encrypted vault and production NativeOrTerminal
prompt in an owned PTY, with process-local PATH=/bin making the graphical
provider unavailable. The public fixture passphrase is typed into that actual
terminal, with no credential environment variable, provider stub or source
change. Its 2,421 frames include the acceptance-only 2,400-frame owner hold and
supply no performance claim. A second owner refuses with exit 101, and separate
warm original/read-only inspections preserve all durable bytes and retired
store bytes. Clean app exit followed by the verified H helper restores the
same disk and app views. Cold-open changes are recorded separately.

The host's graphical backend was actually available in the first ordinary
attempt, but its unbundled identity-dialog process could not be targeted by
the available UI automation. That zero-frame/no-password failure remains
preserved. The controlled terminal pass does not qualify graphical unlock
handling or prove consistently successful startup in every environment.

The actual H native Graphshell witness and fresh reopening pass, with six
independently reviewed originals and separate own-view/every-byte guards. Its
Graphshell Reading-session/Grid view is restored; Cleromancy's complete Fool
view map remains unchanged. The actual H mounted browser witness also passes,
with five independently reviewed originals and a separate owner guard: only
Graphshell's view history appends, while the other 35 files and sentinel remain
unchanged. Fresh Chrome reopening also passes with one original review and all 36 files
plus sentinel unchanged. Retained live update also passes: acknowledged native Spiral reaches generation 7
in the same browser before its companion click, without Refresh/Switch/rejoin.
Staged Save produces exact codicil `11119317…`; exact Open creates read-only
thaw `a8066413…`, with no implicit fork. Nine original archive captures pass
review. Context/Grid/Spiral/Refresh/purge/rejoin and return to writable `be49`
preserve all 45 post-Open shared files plus sentinel, including epochs. Both
complete application view maps and all domain IDs/facets remain exact.

The full 164-scene wall and mandatory supplements remain open; the
accepted H Scene row is retained without rerun/reset, and no G pass is relabeled
H. Native N is pushed and its exact remote revision verified; canonical
documentation publication remains pending. No C1 phase
acceptance or main-branch integration is inferred from the published supplier
and portable-core checkpoints.


The H controls batch now closes 21 of 164 original canonical cases, with
43 independently reviewed originals: 20 fresh profiles plus the retained Scene
profile. The other 143 cases and mandatory supplements remain open. A final
clean same-disk H resident restart preserves both complete application views
and all domain IDs, with five sessions/two archives. Separate warm original
and thaw guards preserve all45 shared files plus sentinel, epochs included.
Release/startup database changes and the first cold original epoch8→10 byte
failure remain recorded separately; the warm passes do not erase them.
The documentation audits retain exactly the pre-edit F mechanical findings
and D2 errors, with no new findings; inherited debt remains unresolved.

### Authorized orchestration (2026-10-08)

Mark authorized publishing this refresh and proceeding with the plan. The
first implementation wave ran the independent reservoir contracts in separate
lanes: V3 archive, V4 recorded denials and ambient grants, and V5 embedded/client
ownership. Mere's integrated source at `5f611cfb3` passes scoped tests and
real-process receipts; the reservoir plan owns their exact counts and limits.
Graphshell's one-tree cutover and mounted session panel remain V2b gates; this
wave does not bypass them or move C2 ahead of C1.

Cleromancy supplies a read-only complete-candidate validator in
[`host/domain.rs`](../src/host/domain.rs). It checks canonical record identity,
supported domain facets, sealed reading replay and stored dependencies using
the existing product decoders. It opens no store and leaves the candidate
unchanged. The shared resident must install it before any graph, journal,
archive or imported-session write; a helper's presence does not establish that
composition. A coordinated integration pin follows the shared and V2b gates, then the
client and shared mere view replace the private authority and snapshot store.

The core lock is aligned to the manifest's existing Mere revision. The locked
sky-timeline gate now passes; the dated failure above remains historical.
Domain-specific refusal tests run in the portable core. These are source
checks; the new resident path still needs native lifecycle, cross-application
visibility and fresh-process reopen qualification on the integrated revision.

### Font adoption and scene review (2026-10-13)

Refreshed the clean product checkout to published `3e101cd087ea7a6dbec6f0094f2282fb51a32286`.
The granular-font adoption is already published at `6cf1e405042455dd5d0ca314b45e92721997ca9d`;
it does not need another implementation lane. The
[font receipt](receipts/tabard_20261011/font-roles/qualification.json) records
101 root and 17 portable-core tests, and native runtime `c1fb4ce7f28f0f599e3c3ce876bfcf128046fb1b`
against Mere `addccab32`. Re-ran its read-only verifier and independently
reviewed all nine original captures: two accepted processes, 123 presentations,
unchanged preference/library bytes, card data and exact session/reading/reflection
IDs after fresh reopening. The compact capture shows the Reading field in the
scrolling panel, not every font control. Sizes and spacing remain host-owned.

The three subsequent commits adopt Rust policy and the Murm supplier cohort;
they change manifests, configuration, CI and documentation, with no product
source, test, font-byte or scenario changes. Their compile/software receipts
remain separate from the earlier headed font receipt. This review does not
renew native acceptance on the new supplier graph.

That published revision retained the private `HOST_SLOT` snapshot and application
authority. The C1 working source subsequently retired their ordinary paths to the
explicit historical harness; acceptance is recorded separately below. The manually
placed three-column reading scene and id-derived position labels remain C2
findings. The font receipt closes neither the reservoir migration nor C2.
Reviewed published Mere `e84cfbd95b1660046d646580a8707af020a06e4b`'s reservoir
and one-tree plans: they still distinguish available resident contracts from
the unfinished V2b cutover/panel and Cleromancy consumer gates. The Mere plans
own the detailed supplier status.

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
V2b's route adapter does not establish the product's embedding. The shared
V3–V5 resident contracts now have process fixtures; those do not establish
Cleromancy's actual archive or standalone startup acceptance. Shared V4 access
enforcement belongs to C1; the card-ambient product scene belongs to later C4.

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

**Reading-centered scene proposal (2026-10-13; not a new ruling).**
Use a journal of reading occasions with a contextual constellation around the
selected occasion. Each occasion groups its question, saved cast, draw-time
context and subsequent reflections. A reading session contains individual
sealed readings at distinct position occurrences; grouping them visually does
not merge those records or replace their receipt identities. A dated journal
is the legible overview. The constellation is the relation view, with a
force-directed arrangement as the existing default and a still presentation
available from the outset.

Selecting an occasion opens its readable narrative and ordered spread.
Selecting a card opens a bounded neighborhood around that occurrence:

- **History:** prior occurrences of the same card, their questions and saved
  reflections. Distinct occurrences remain distinct even when a card repeats.
- **Correspondence:** related cards, deck crosswalks and traditional axes,
  each naming its tradition, source and version. Admit only the typed data
  available after C3; no view invents missing correspondences.
- **Situation:** the facts actually recorded at the draw. Sky facts and
  opted-in observations remain distinguishable from interpretation; this lens
  does not silently fetch current weather or enable place.

Choose relation families independently. A connection explains why it exists;
distance is an arrangement aid, not a numerical claim about symbolic meaning.
The focused question, position meaning and card narration stay readable while
the surrounding neighborhood changes. A saved cross-reading relation or
reflection uses a Cleromancy owner action. View pinning and focus alone never
keep an ambient item or alter a reading. Material from another mere requires
its explicit ambient grant and disappears from this projection on revocation.

Graphshell supplies the shared navigation, scene editing, toolbar and
accessible alternative, with the common mere session view composed alongside.
Cleromancy supplies the domain projection and typed actions; the resident owns
durable graph/session/archive operations and replay validation. Do not mount
another editable generic graph over an app-local copy. A browser Graphshell
opening the same admitted divination mere is a useful C1 cross-application
acceptance witness, separately from the Cleromancy native scene.

The first fixture is one saved three-position occasion, two earlier occasions,
saved reflections and recorded sky facts, followed by cited correspondence data
when C3 permits it. It deliberately includes a repeated card and a custom
position label. The flow selects the current occasion, follows an earlier
appearance, returns to the focused card, inspects a named relation, explicitly
keeps one ambient item, and saves view curation. Fresh reopening checks exact
sealed readings and reflection bytes, saved template labels, focus, curation
and placement roles. A revoked neighboring mere yields no ambient material.
Wide and compact native captures must show readable narrative and reachable
controls under shared font roles; hit targets follow realized geometry.
Keyboard and still/reduced-motion operation expose the same actions. Any
motion composition uses the shared grammar and has its own save/reopen gate.

Prepare the projection binding and synthetic fixture now; implement the
resident-backed scene in the ruled order. C1's shared lifecycle and authority
cutover remains the next product implementation gate. C2 supplies the narrative
and authored-position meaning; C3 supplies typed correspondence data; C4 then
qualifies this neighborhood. This proposal supplies a concrete consumer for
the shared Graphshell work without treating a prototype or an upstream receipt
as completion of those phases.

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
- 2026-10-08: the first shared wave is integrated in Mere at `5f611cfb3`.
  Archive lineage and read-only first-edit forks, recorded denial/ambient
  route enforcement, and client-first embedded ownership pass combined
  source and real-process fixtures. The reservoir plan records exact counts,
  revisions and the unresolved expanded personal-sync reopen test. Its
  permission API is owner-side; admitted products cannot clear their own
  denial. Rechecked this readiness map against that source. No Cleromancy
  repin, authority registration, private-store retirement, native consumer
  proof or phase acceptance is implied by the shared implementation.
- 2026-10-13: Refreshed the product checkout to published `3e101cd`, verified
  the already published font adoption's committed receipt and all nine original
  images, and separated its frozen native inputs from the later Murm/toolchain
  adoption. Reviewed the current published Mere reservoir/one-tree plans and
  recorded a proposed reading-centered journal/constellation with a concrete
  fixture, owner boundaries and acceptance flow. No product code, dependency,
  persistence owner or C1–C8 acceptance state changed.

- 2026-10-13: Mark authorized C1 through qualification and push in lockstep
  with shared Graphshell/reservoir work. The integrated implementation replaces
  the ordinary private-store worker with client-first resident attachment, a
  complete revision-bound graph resource and staged replay-checked transactions.
  Daemon and embedded compositions share the product's portable validator.
  The common mere view adds lifecycle/archive controls, and Graphshell supplies
  a separate native consumer witness. Private snapshot/admitted-authority APIs
  are confined to the explicit nondefault historical receipt harness; there is
  no ordinary-app importer or fallback. Development path patches and provisional
  pins are temporary integration inputs, not published acceptance. CPU tests,
  native/control capture, embedded standalone startup, cross-application
  visibility and frozen fresh-process reopen remain gates. Shared browser
  presenter retirement and full V2b qualification proceed separately in Mere;
  their status is not inferred from native C1.

- 2026-10-14: Frozen C51/A3 immutable intake and bounded native C1 qualification
  pass with the exact source and capture ledger above. Normal terminal unlock,
  embedded owner exclusion and same-disk handoff are established separately.
  The first presentation stall, cold projection-epoch writes and C2 clipping
  remain explicit. Full browser cutover qualification and native publication
  continue in lockstep with the shared Graphshell/reservoir lane.

- 2026-10-14: The final C51 browser witness, actual fresh Chrome reopen and
  retained connection live update pass. A native Spiral ACK reaches the browser
  at generation 6→7 without Refresh/Switch/reconnect. Staged actual Save/Open,
  transient read-only controls, purge/rejoin and return-to-original proof pass;
  every durable byte, including epoch and both apps' views, matches across
  read-only inspection. The failed packet-selector extension remains preserved.
  The first full-wall attempt exposes the unselected-face snapshot defect and
  viewport drift: four diagnostic runs, zero canonical accepted cases. The
  web-only repair is committed as `4a5099820` and is being rebuilt; native C51/A3/D
  receipts retain their original identities until the browser stack settles.
