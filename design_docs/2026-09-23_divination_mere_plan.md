# Cleromancy: the divination mere, and a reading that explains itself

**Date:** 2026-09-23
**Status:** plan. Nothing implemented. C1 waits on Mere's reservoir plan
(V1–V5). §7's decisions were all ruled on 2026-09-23.
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

## 2. Findings (verified 2026-09-23)

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

**Done when:**
- a focused card shows at least three relation families;
- each ambient item names its engine and version;
- examining and keeping move items between keeping levels and appear in the
  journal;
- the same neighbourhood can be shown in at least two scenes, with
  force-directed as the default.

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
