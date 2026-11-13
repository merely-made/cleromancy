# Fortune teller: a daily game for Reddit

**Status (2026-11-11): plan.** Design rulings recorded; nothing built. Platform
feasibility is unverified (see Findings), so no phase past F0 is entered yet.

A cozy fortune-teller game. A customer sits down; cards are drawn for them; the
player composes the reading by filling a templated fortune from a word bank.
The customer reacts to how the reading sounds and to whether its words suit the
cards. It is a candidate entry for Reddit's "Game of the Day" hackathon.

## The occasion

Source: `https://redditgameofday2026.devpost.com/rules`, read 2026-11-11.

- Submission period 2026-11-11 12:00 PT to 2026-12-08 18:00 PT. Winners around
  2026-12-17.
- Build a new daily game on Reddit's Developer Platform (Devvit Web,
  Interactive Posts). "Apps built around a bite-sized daily experience", "easy
  to get the hang of", with "a satisfying daily loop".
- Four equally weighted criteria: Delightful UX, Polish, Reddity, Daily Content.
  A first pass/fail stage checks that the project fits the theme.
- Submit: app listing link, `README.md` in the app root, a demo post in a public
  subreddit with fewer than 200 members, a text description; optionally a video
  under one minute and a public repository.
- Existing work is allowed if significantly updated in the window. Submissions
  stay the entrant's property, subject to Reddit's Developer Terms (not read).

## The game as ruled

- **Customer.** Three axes. The *planet* is the classical planet closest to
  the Moon in the real sky that day and sets which sounds the customer likes.
  The *sign* is the customer's own, from the daily seed, and sets what they
  came to ask about. The *element* is that of the sign the real Moon is in on
  the day, and sets their mood. "Mars x Water x Libra" is a Libra visiting
  under a water Moon that stands nearest Mars.
- **Reading.** Cards are drawn from the day's public seed, so every player sees
  the same draw. The spread grows through the week, one to five cards, and
  its size is the difficulty. The fortune is a template with blanks, filled from a word bank
  the cards supply.
- **Judgement.** Two axes: Mora scores the sound of the finished reading
  against the customer's taste; a tag rule checks the chosen words suit the
  cards. Meter is one taste among several, graded by how much of a line fits,
  never pass/fail.
- **Daily loop.** The first customer of a session is today's daily. Each
  customer after that is the previous day's daily, walking back through the
  archive.

## Phases

### F0. Feasibility and shape

Done when:

- Devvit Web's client and server limits are read from a primary source and
  recorded in Findings: whether a webview can load WebAssembly, bundle and
  asset size limits, server runtime, storage, scheduling of a daily post.
- A WebAssembly module loads and runs inside a Devvit playtest post (D9). If
  it cannot, the fallback in D9 is taken and recorded.
- The seeded shuffle and draw live in a dependency-light crate inside
  Cleromancy (D11) that compile-checks for `wasm32-unknown-unknown`, the
  application uses it, and `tests/derived_selection.rs` still passes.
- `repos/soothsayer` exists (D10, D12).
- Reddit's Developer Terms are read and anything that bears on ownership or
  licensing is in Findings.

### F1. One customer, scored

Done when:

- A customer is derived from a date alone, and the same date gives the same
  planet, sign, element and draw on a second independent run.
- A filled template is scored on both axes, and a test shows a reading that
  suits the customer's taste outscoring one that does not, for at least one
  customer per planet.
- Every word in the bank has a pronunciation from a source whose licence is
  recorded.

### F2. The booth

Done when:

- A player can meet the daily customer, see the draw, fill the fortune and get
  the customer's reaction inside a Reddit post.
- The next customer is the previous day's daily, and so on back to the first
  day the game ran.
- A screenshot of the whole booth at phone width is in the receipts file.

### F3. Reddit-native and polish

Done when:

- A new daily post appears without manual action.
- A player can share their reading where other players see it.
- The submission checklist in "The occasion" is complete.

## Open questions

Each is Mark's to rule.

- Which day boundary sets the weekday and the Moon's sign (UTC, as
  `sky-timeline` uses, or something else).
- What the positions of the two-, four- and five-card spreads are called.
- When the Moon itself is the customer's planet under D21 (for example when
  no planet is within some distance of it), and what "closest" measures
  (ecliptic longitude or angular separation).
- Which sky events count for D19, what spread each brings, and whether a
  special replaces or joins the day's ladder spread.
- How the 78 card faces of D20 are made, and what a reversed card looks like
  at about 100 px wide.
- What the game calls the deck (see Findings on the "Rider-Waite" name).
- Which guesser fills D34's words, and how a guessed word is shown.
- What a card's own associations are (the roots D27 measures distance from),
  for 78 cards upright and reversed, and who writes them.
- Whether card backs show definitions, which the measured 1.6 MB leaves out.
- The LibriSpeech lexicon's "public domain" listing is the page's word; no
  licence file was read.
- Where card tags come from: existing deck content, or newly authored.
- What the seven planetary tastes are, sound by sound, and what each element
  does to mood.
- Whether the sign's topic changes the word bank, the template, or only the
  weighting of card tags.
- Whether Reddit features beyond comments (flair, streaks, snoovatars) are in
  scope.

## Decisions

Recorded in the words given, 2026-11-11.

- **fortune_teller D1.** Which core to pursue. "1 seems doable, 2+3 (relating
  those two seems the core of any puzzle), and 3 (a lil blank canvas is not bad
  for a new target, no?)… i might even think like, what if you had to explain a
  tarot reading to someone, and your words were (generated) templated like
  madlibs but evaluated by mora against the customer's profile, drawn from the
  sky today. I'm maybe thinking of something like a cozy fortune teller
  simulator game…" Outcome: new.
  Links: area game · summary cozy fortune-teller: templated reading, scored by Mora against a sky-drawn customer
- **fortune_teller D2.** What the customer judges. "Sound plus card tags".
  Outcome: new.
  Links: area game · draws on fortune_teller D1 · summary two axes: Mora sound score, card-tag fit
- **fortune_teller D3.** The daily unit. "First customer of the day is the
  daily. Each subsequent customer is the previous day's daily". Outcome: new.
  Links: area game · summary today's daily first, then the archive backwards
- **fortune_teller D4.** How central meter is. "One taste among several".
  Outcome: new.
  Links: area game · draws on fortune_teller D2 · summary meter is one customer taste, graded
- **fortune_teller D5.** Base profiles. "Seven by planet for matching the week.
  Then we should apply the other two criteria as axes of variance like Mars x
  Water x Libra". Outcome: new.
  Links: area game · summary seven planets by weekday, crossed with element and sign
- **fortune_teller D6.** Where each axis comes from. "Element from today's
  Moon": planet from the day of the week, sign from the daily seed, element
  from the sign the real Moon is in. Outcome: refines D5.
  Links: area game · amends fortune_teller D5 · summary planet=weekday, sign=seed, element=Moon's sign
- **fortune_teller D7.** What each axis controls. "Planet=sound, sign=topic,
  element=mood". Outcome: new.
  Links: area game · draws on fortune_teller D5 · summary planet sets sound taste, sign the topic, element the mood
- **fortune_teller D8.** Where the design is written. "Inside
  cleromancy/design_docs". Outcome: new.
  Links: area docs · summary this plan lives in Cleromancy's design docs

- **fortune_teller D9.** Build shape. "Let's try 2 with a fallback of 1. No
  reason why 2 shouldn't work in principle, right?" Option 2 was Mora, Turquet
  and the seeded draw all compiled to WebAssembly and computing each day live;
  option 1 was Mora in WebAssembly with each day's customer and cards
  precomputed natively to a JSON table. Outcome: new.
  Links: area build · summary everything in wasm live; fall back to Mora in wasm plus a precomputed day table
- **fortune_teller D10.** Where the code lives. "New repo under repos/". The
  plan stays here (D8). Outcome: new.
  Links: area build · draws on fortune_teller D8 · summary the game's code gets its own repo under repos/
- **fortune_teller D11.** How the game reaches the seeded draw. "Cut a small
  draw crate": lift the seeded shuffle and draw into a dependency-light crate
  inside Cleromancy that the app and the game both use. Outcome: new.
  Links: area build · draws on fortune_teller D9 · summary one shared draw crate, cut out of Cleromancy
- **fortune_teller D12.** Working name of the game's repository. "soothsayer".
  Outcome: new.
  Links: area build · draws on fortune_teller D10 · summary the repo is repos/soothsayer
- **fortune_teller D13.** The kind of draw. "Single card, past present future,
  situation action outcome, three options, celtic cross… seems if the word
  banks come from the cards, then more cards means a bigger, more complex
  response. Could be difficulty!" Outcome: new.
  Links: area game · draws on fortune_teller D2 · summary spread size is the difficulty
- **fortune_teller D14.** What decides the spread. "We need two more then…
  seven spreads? Plus, big jump between 3 and celtic. Surely we could do 1, 2,
  3, 4, 5 card spreads? If not, 2 is a better fit, in terms of diff cadence
  (1, 3, celtic)". The weekly ramp was the option answered; "2" was the Moon's
  element deciding. Then, on the ladder: "1, 2, 3, 3, 3, 4, 5", Monday easiest
  to Sunday hardest: single card; a two-card pair; past/present/future,
  situation/action/outcome and three options midweek; four and five cards at
  the weekend. Outcome: new.
  Links: area game · draws on fortune_teller D13 · summary weekly ramp of seven spreads: 1, 2, 3, 3, 3, 4, 5 cards
- **fortune_teller D15.** Spreads in the hackathon cut. "One and three now,
  cross later", answered before D14's ladder; the ladder keeps the Celtic
  cross out and adds the two-, four- and five-card spreads. Outcome: refined
  by D14.
  Links: area game · draws on fortune_teller D14 · summary Celtic cross is after the hackathon cut
- **fortune_teller D16.** The deck. "Rider waite seems standard." Outcome: new.
  Links: area game · summary the deck is Rider-Waite-Smith
- **fortune_teller D17.** Reversals. "Reversals are in." Outcome: new.
  Links: area game · summary cards can come reversed
- **fortune_teller D18.** The ramp's first day. "Monday is the first day of the
  week." Outcome: applies D14.
  Links: area game · draws on fortune_teller D14 · summary the weekly ramp starts on Monday
- **fortune_teller D19.** Occasional extra spreads. "include extra spreads
  occasionally", triggered by "Sky events": a full Moon, eclipse, solstice or
  similar brings a special customer with a special spread. Outcome: new.
  Links: area game · draws on fortune_teller D14 · summary real sky events bring special spreads
- **fortune_teller D20.** Card pictures. "New art in the game's own style".
  Outcome: new.
  Links: area art · draws on fortune_teller D16 · summary original card faces, not scans
- **fortune_teller D21.** Taste and difficulty. "It would be nice if we could
  carry taste and difficulty", then "Vary them independently", then "Planet
  from the real sky": the weekday keeps the spread ramp, and the customer's
  planet is the classical planet closest to the Moon in the sky that day.
  Outcome: contradicts D5 and D6 on the planet's source; put back and ruled.
  Links: area game · amends fortune_teller D5 · amends fortune_teller D6 · summary planet = classical planet nearest the Moon; weekday keeps the ramp
- **fortune_teller D22.** Art style and how the faces are made. "Microvoxel,
  pseudo pixel style", made as "Voxel kit, majors bespoke": minors and courts
  are assembled from a small set of voxel pieces by data, and the 22 majors
  are hand-built scenes. Outcome: refines D20.
  Links: area art · draws on fortune_teller D20 · summary microvoxel pseudo-pixel; a voxel kit for minors and courts, bespoke majors
- **fortune_teller D23.** Rendering. "Real voxels, pixel-snapped": small voxel
  models rendered at low resolution with hard pixels, able to tilt, flip and
  turn over. Outcome: new.
  Links: area art · draws on fortune_teller D22 · summary cards are live 3D voxel models, pixel-snapped
- **fortune_teller D24.** The opening of a session. "Lil pixelesque generated
  customer walks in to the table, you look at the orb and it expands to show
  customer and circumstantial detail. Then the first spread is the daily, you
  get a random draw on the spread, and you have to describe it/their fortune
  pleasingly." Outcome: new. Read with D3: the draw is the day's seeded draw,
  the same for every player; Mark has not said otherwise.
  Links: area game · draws on fortune_teller D3 · summary customer walks in; the orb shows who they are; then the daily spread
- **fortune_teller D25.** Composing the reading. "Ui for that looks like
  flipping a card in the spread to see its word associations on the back,
  which you can click to open up a related word graph from the wordnet
  dictionary, right? You get the skeleton of the response from the customer's
  concern and you have to figure out what to say. The benchmarks your response
  would be judged by can allow us to offer a lil guidance, maybe". Outcome:
  refines D1 and D2.
  Links: area game · draws on fortune_teller D1 · draws on fortune_teller D2 · summary flip a card for its words; open a WordNet graph from any word; the concern gives the skeleton; scoring can guide
- **fortune_teller D26.** How words enter the fortune. "I'm ok with 1 but why
  limit wordnet? Why a subset? And there's likely pronunciation data
  available… seems like we're foreclosing covering more words!" Option 1 was
  picking words by walking the graph outward from a card's associations. The
  graph is the whole of WordNet, not a subset. Outcome: new.
  Links: area game · draws on fortune_teller D25 · summary words are picked from the full WordNet graph, not a subset
- **fortune_teller D27.** Card fit. "Graph distance": a word scores by how few
  steps it is from the card's own associations. Outcome: refines D2.
  Links: area game · amends fortune_teller D2 · summary card fit is graph distance from the card's words
- **fortune_teller D28.** Guidance. "Only after delivery": nothing while
  composing; a breakdown once the fortune is handed over. Outcome: refines
  D25.
  Links: area game · draws on fortune_teller D25 · summary no hints while composing; breakdown after delivery
- **fortune_teller D29.** Where the word graph lives. "Measure first": compact
  the full WordNet, measure its size and CMUdict's coverage of it, then
  choose between the client and the app's server. Outcome: new.
  Links: area build · draws on fortune_teller D26 · summary graph home decided after measuring
- **fortune_teller D30.** Words CMUdict cannot pronounce. "Add a second
  source, then a fallback": further open pronunciation data, phrases
  pronounced word by word, then a rule-based guess marked as a guess.
  Outcome: new.
  Links: area build · draws on fortune_teller D26 · summary widen pronunciation coverage; guess the rest, marked
- **fortune_teller D31.** Where the word graph lives, after measuring. "In the
  client": about 2 MB gzipped downloads with the game. Outcome: applies D29.
  Links: area build · draws on fortune_teller D29 · summary the full graph and pronunciations ship in the client
- **fortune_teller D32.** Finding the second pronunciation source. "Survey
  open sources first": check licences and measure how much of the gap each
  closes, then bring the numbers back. Outcome: applies D30.
  Links: area build · draws on fortune_teller D30 · summary survey open pronunciation data before choosing
- **fortune_teller D33.** The second pronunciation source. "LibriSpeech
  lexicon": listed public domain, joined to CMUdict. Outcome: applies D30.
  Links: area build · draws on fortune_teller D32 · summary CMUdict plus the LibriSpeech lexicon
- **fortune_teller D34.** Words no source pronounces. "Guess them, marked": a
  spelling-to-sound guesser fills the rest at build time and the game marks
  those words as guessed. No share-alike data ships. Outcome: applies D30.
  Links: area build · draws on fortune_teller D30 · summary remaining words get a build-time guess, shown as guessed
- **fortune_teller D35.** Entering the build. "This coverage, knot could also
  have :D nice, improving the stack through new development, pretty cool! I'm
  down for the action!" Outcome: new.
  Links: area build · summary go-ahead to build; stack improvements are welcome
- **fortune_teller D36.** Where the combined pronunciation source is built.
  "In Knot, Soothsayer consumes it": Knot's pronunciation provider grows to
  carry both dictionaries and the marked guesses, and the game pins it.
  Outcome: applies D33 and D34.
  Links: area build · draws on fortune_teller D33 · draws on fortune_teller D34 · summary the wider lexicon is built in knot-editor and consumed by the game
- **fortune_teller D37.** The draw crate's name. "Keep cleromancy-draw".
  Outcome: applies D11.
  Links: area build · draws on fortune_teller D11 · summary the shared draw crate is cleromancy-draw
- **fortune_teller D38.** LibriSpeech after the overlap finding. "Hmm. Could
  we do better guesses? Measure wiktionary, too". Outcome: reopens D33; the
  second source is not settled until guessers are compared.
  Links: area build · amends fortune_teller D33 · summary LibriSpeech's entries are guesses; look for better ones
- **fortune_teller D39.** Where guessing happens. "Both, table first": a
  precomputed table for the hackathon, then a guesser in the stack that
  replaces it when it measures at least as well. Outcome: refines D34.
  Links: area build · amends fortune_teller D34 · summary precomputed guess table now, in-stack guesser after
- **fortune_teller D40.** Finding a better guesser. "If there's a rust guesser
  (unless the prior art is also crosslanguage), trial. If not, let's do our
  own". Read as: trial an existing Rust guesser, or cross-language prior art
  in any language; with neither, write our own. Outcome: new.
  Links: area build · draws on fortune_teller D38 · summary trial Rust or cross-language guessers; otherwise build one
- **fortune_teller D41.** Wiktionary's role. "Yardstick only": used to measure
  and choose guessers; nothing derived from it ships. Outcome: settles the
  share-alike question left open on 2026-11-11.
  Links: area build · draws on fortune_teller D32 · summary Wiktionary measures guessers and is not shipped

## Findings

- 2026-11-13. The misaki lexicons checked as a third dictionary (version
  0.9.4 as bundled in `loqui-g2p` 0.4.0; receipts file):
  - Provenance is undocumented. The misaki repository is Apache-2.0 and says
    nothing about where the "gold" and "silver" dictionaries came from, what
    the tiers mean, or whether the data has its own licence (reported by a
    web lane from the README, LICENSE, EN_PHONES.md and data listing).
  - Size: gold 89,359 usable US words, silver 93,361, 4,321 in both. They
    carry primary and secondary stress marks.
  - They are not a copy of CMUdict or of Wiktionary. On words CMUdict has,
    gold matches CMUdict 77.9% and Wiktionary 79.8%, the level two
    independent human sources reach (82.1%), and puts primary stress on the
    same vowel as CMUdict 96.0% of the time (silver 97.0%).
  - On words CMUdict lacks, against Wiktionary: gold 68.4% exact, 5.6%
    phone error; silver 73.3%, 5.4%. LibriSpeech's guesses score 52.0%,
    11.7% on their list. So these read as dictionary-grade, not guesses.
  - Coverage of WordNet's 36,701 single words missing from CMUdict: gold
    57.6%, gold or silver 65.5%. CMUdict with gold reaches 77.4% of single
    words; with silver 81.6%; with LibriSpeech's guesses as well, 84.1%.

- 2026-11-13. Guesser trial (D40), 11,036 LibriSpeech-only words that
  Wiktionary US also has, stress ignored (receipts file):
  - LibriSpeech's own entries: 52.0% exact, 11.7% phone error.
  - `grapheme_to_phoneme` 0.1.0 (Rust port of a 2020 neural model, ARPAbet
    with stress, BSD-4-Clause): 43.7%, 18.2%. Worse than the mark.
  - `loqui-g2p` 0.4.0 (MIT; misaki lexicons then a small neural model, both
    Apache-2.0 by its NOTICE): 65.2%, 7.8% overall, but 7,840 of the words
    are in its lexicons. On the 3,194 words outside them, where its guesser
    works alone: 48.0%, 15.1%, against LibriSpeech's 44.7%, 15.7% on the
    same words. A small gain, not a different class.
  - `g2p2-core` 0.3.0 (pure Rust joint n-gram, 100 languages) ships no
    models and its model repository could not be reached; not trialled.
  - Where LibriSpeech and `grapheme_to_phoneme` give the same phones (5,039
    words) they are right 75.1% of the time, near the 82.1% two human
    sources reach; where they differ, 32.6% and 17.3%. Agreement between
    independent guessers is a usable confidence signal.
  - The misaki lexicons inside `loqui-g2p` hold 7,840 of these words with
    stress marks. Their provenance and what "gold" and "silver" mean are not
    yet verified; if sound, they are a third dictionary, not a guesser.
  - No stress accuracy has been measured for any guesser: Wiktionary's files
    lack stress, and the pretrained models have seen CMUdict.

- 2026-11-13. Measured, the LibriSpeech lexicon against CMUdict (receipts
  file): 66,016 words are in both, and 97.0% of those share an identical
  pronunciation; 133,984 words, two thirds of the file, are in LibriSpeech
  only. Its page says "some of which G2P auto-generated" and gives only a
  page-level "License: Public domain", with no licence file and no statement
  of where the rest came from. The LibriSpeech-only words are taken to be
  the generated ones. This reopens D33, which chose it as a second
  dictionary.
- 2026-11-13. Measured, Wiktionary pronunciations (WikiPron, US and UK broad
  files, CC BY-SA 4.0 material) as a yardstick and a source:
  - The files carry no stress marks, so they cannot feed Mora as they are;
    stress would have to be supplied some other way.
  - Yardstick, stress ignored, best of listed variants, against Wiktionary
    US. Control, CMUdict's own words: 82.1% exact, 3.6% phone error.
    LibriSpeech-only words: 51.9% exact, 11.8% phone error (53.3% and 10.0%
    on the WordNet words among them). That is the mark a better guesser has
    to beat, and it says nothing yet about stress.
  - Coverage of WordNet's 36,701 single words missing from CMUdict:
    Wiktionary US has 9,972 (27.2%), US or UK 11,248 (30.6%). Of the 23,126
    in neither CMUdict nor LibriSpeech, Wiktionary has 5,358 (23.2%).
    CMUdict with Wiktionary reaches 63.0% of single words; adding the
    LibriSpeech guesses, 74.2%.

- 2026-11-11. The draw crate is cut (D11): `crates/cleromancy-draw` holds the
  shuffle record, the permutation and orientations, and `canonical_digest`;
  it depends on `blake3`, `serde` and `serde_json` only and compile-checks
  for `wasm32-unknown-unknown`. `src/reading/shuffle.rs` keeps field
  validation, receipts and sealing, and reaches the method through the
  `ReplayDeck` trait. `tests/shuffle_golden.rs` pins five cards captured from
  the code before the cut and passes after it.
- 2026-11-11. `tests/derived_selection.rs` is gated on the `legacy-receipts`
  feature and runs no tests by default, so F0's done-condition names a test
  that only proves anything with that feature on.

- 2026-11-11. Measured, Open English WordNet 2025 against Knot's bundled
  CMUdict (run details in the receipts file):
  - 127,311 distinct lemmas, 107,519 synsets, 234,810 synset relations,
    120,254 sense relations.
  - Of 68,869 single plain words, 32,168 (46.7%) have a CMUdict
    pronunciation. Of 58,442 phrases and hyphenated forms, 43,834 (75.0%) are
    pronounceable part by part. Overall 59.7%. The gaps include ordinary
    words ("gargle", "expository", "conjunct" were confirmed absent from the
    dictionary file by hand), so D30's second source is needed, not optional.
  - The whole graph as integer-coded structure (lemmas, synset membership,
    typed synset relations; no definitions or sense relations) is 5.8 MB, or
    1.6 MB gzipped. The pronunciations for every covered word are 1.0 MB, or
    0.3 MB gzipped. Both fit in the client with room to spare; D29's choice
    is back with Mark.
- 2026-11-11. Measured, coverage of WordNet's 68,869 single plain words
  (receipts file):
  - CMUdict 46.7%; MFA US ARPA 60.0%; LibriSpeech lexicon 60.1%.
  - CMUdict with either one 66.4%; all three together also 66.4%. MFA and
    LibriSpeech cover almost exactly the same words, so one of them is
    enough. With phrases counted part by part, the best combination reaches
    73.6% of all 127,311 lemmas.
  - "gargle", "expository" and "conjunct" are in both new sources.
  - 23,126 single words remain in none of the three. A sample is mostly
    technical or rare ("anaplasmosis", "pentoxifylline") with some ordinary
    ones ("lowercase", "homograph", "timpanist", "semiskilled"). These are
    what D30's guess, or a Wiktionary-derived source, would have to supply.
  - The LibriSpeech page says some of its pronunciations are themselves
    machine-generated; which ones is not marked in the file.
- 2026-11-11. Survey of open pronunciation sources (D32), reported by a web
  lane from project pages and licence files; nothing downloaded, no coverage
  measured, and not re-read by hand:
  - Candidates. The Montreal Forced Aligner "English (US) ARPA" dictionary
    v3.0.0: 199,858 words, ARPAbet with stress, CC BY 4.0; where its
    pronunciations come from is not stated. The LibriSpeech lexicon (OpenSLR
    11): listed "Public domain", 5.6 MB, a 200K-word vocabulary, "some of
    which G2P auto-generated". Moby Pronunciator II: public domain by the
    author's grant, its own notation with stress.
  - Share-alike. WikiPron and other Wiktionary extractions are CC BY-SA 4.0
    material in IPA; shipping anything derived from them carries that
    licence.
  - Excluded. Unisyn (non-commercial only), CELEX and PRONLEX (licensed
    corpora), espeak-ng (GPL-3.0), ipa-dict en_UK (GPL-3.0 source); ipa-dict
    en_US and cmudict-ipa are CMUdict conversions and add no words.
  - Guessers for words no dictionary has: the MFA US ARPA model (CC BY 4.0,
    17.69% word error rate on held-out words) and g2p-en (Apache-2.0,
    ARPAbet with stress).
- 2026-11-11. The stack already holds both lexical sources D25 needs, in
  `knot-editor`. `crates/mora-cmudict` bundles CMUdict (135,010 lines,
  3.6 MB, BSD-2-Clause, terms in `LICENSE-CMUDICT`) as a pronunciation
  provider for Mora. `crates/knot-composition` has `knot-wordnet-import`,
  which converts a caller-supplied Open English WordNet file into a JSON
  subset of chosen lemmas and their one-hop neighbours, "without recursive
  graph expansion", carrying the CC BY 4.0 attribution. It fetches nothing;
  the WordNet XML must be supplied locally.
- 2026-11-11. D25 opens the word bank beyond a few hundred words, which
  withdraws the earlier finding that only a closed bank needs pronunciations.
  Every word a player can choose needs a CMUdict pronunciation for Mora to
  score it. Devvit's client cannot fetch from outside, so the word graph and
  pronunciations ship in the app or come from its own server (10 MB response
  limit).
- 2026-11-11. A word graph rooted at each card's associations offers a rule
  for D2's card fit that needs no separately authored tags: how near a chosen
  word is to the card's own words. Not ruled.

- 2026-11-11. D21 as worded cannot produce the Moon as a customer's planet,
  since the Moon is one of the seven and is the reference. How often each of
  the other six comes up, and how long the runs are, has not been measured;
  Turquet can measure it over a year before the rule is fixed.

- 2026-11-11. Cleromancy ships no card art: `assets/` holds fonts only, and
  both decks are text. D20 therefore means 78 new faces, the largest art job
  in the window; how they are made is an open question.
- 2026-11-11. From memory, not checked against sources: the 1909
  Waite-Smith art is public domain in the US and, since the end of 2021, in
  life-plus-70 countries; U.S. Games Systems claims its 1971 recolouring and
  holds marks on the "Rider-Waite" name. What the game calls the deck should
  be checked before submission.

- 2026-11-11. The root `README.md` (line 80) says authored spread templates
  take 1 to 12 positions. The bound was not found in source by a quick search,
  so the two-, four- and five-card spreads of D14 are unverified against the
  code. The fixed three-card spread and the seeded full-deck shuffle with
  upright and reversed orientation are in `src/reading/shuffle.rs`.

- 2026-11-11. `cargo check --lib --target wasm32-unknown-unknown --locked` in
  `crates/turquet` finishes clean (six lint warnings). This is a compile
  check; nothing was run in a browser. Turquet's dependencies are `hifitime`
  and `sofars` only.
- 2026-11-11. The seeded draw is not separable as it stands. It lives in
  `src/reading/shuffle.rs` (320 lines) and `src/reading/engine.rs` (472
  lines), which import `ContextSnapshot`, `Field`, `SealedEnrichment` and the
  `moirai` modules, and the package that builds them (`core/Cargo.toml`)
  depends on Mere, Dramatis, Eidetic's `muniment` with `redb`, Graphshell and
  Seiche. D9's option 2 needs the draw reachable without that cone; how is
  an open question below.

- 2026-11-11. Mora (`knot-editor/crates/mora/README.md`) scores sound only:
  syllables, weight, meter with partial fit ("Scansion reports how much of a
  line fit"), perfect and slant rhyme, assonance, consonance, alliteration. It
  is `no_std` with zero dependencies and ships no pronunciations; it parses
  CMUdict's format. It is "not a semantic analyzer", which is why the card side
  needs its own tag rule (D2).
- 2026-11-11. A templated fortune draws on a closed word bank, so the game
  needs pronunciations only for that bank, not a whole lexicon. The licence of
  any pronunciation source is not yet checked.
- 2026-11-11. The Moon changes sign about every 2.3 days and its phase cycle is
  29.5 days. A taste read from the sky alone would be nearly the same on
  neighbouring days, and D3 makes a session's customers consecutive days; this
  is why profiles cycle and the sky seasons them (D5, D6).
- 2026-11-11. Reported by a read-only sweep and not yet verified against the
  code: public-seed derived readings replay byte-equal
  (`tests/derived_selection.rs`), and Turquet names no WebAssembly target.
- 2026-11-11. `developers.reddit.com` could not be read from this session
  (blocked for both the fetch tool and the browser pane). The facts below come
  from the docs sources at `github.com/reddit/devvit-docs` (main, read by a
  fetch lane the same day; not re-read by hand).
  - WebAssembly is never named. The client requirement is "HTML/CSS/JS only".
    The Unity quickstart ships `SampleGame.wasm.unityweb` as a client asset and
    names Unity games already running, so a wasm module in the client is
    indicated, not stated. Proving it is an F0 item: load one in a playtest.
  - Client: no external requests ("locked down to the webview domain via
    CSP"); calls go to the app's own `/api/` endpoints. `localStorage` clears
    on every app update, so streaks and history belong in Redis.
  - Server: a serverless Node endpoint, CommonJS bundle, "cannot use `fs` or
    external native packages". 30 s per request, 4 MB payload, 10 MB response.
    Whether wasm runs there is not stated.
  - Storage: Redis (strings, hashes, sorted sets), siloed per subreddit
    installation; 100 MB unpublished, 5 GB published. Leaderboards are sorted
    sets.
  - Daily post: a `scheduler` task in `devvit.json` with a five-part cron
    calling `reddit.submitCustomPost()`. Cron time zone is not stated.
  - Sharing: a score comment must be made by the user, on an explicit button
    press, as a reply to a single stickied comment, or top-level only with a
    custom message from the user. Posting as the user works only after app
    approval; before it, content comes from the app account.
  - Layout: inline posts are 320 px or 512 px tall, about 288 to 880 px wide;
    no scrolling inside an inline webview; expanded mode only on a user
    action; a custom first screen is required.
  - Review: playtest and private upload need no approval; a published version
    is unlisted by default and each version is reviewed again. Stated review
    time is "1-2 business days" on one page and "approximately one week" on
    another. Using external fetch adds a domain review plus Terms and a
    Privacy Policy.
  - Rules: no third-party IP or clones, no Reddit trademarks or Snoo, no
    gambling apps; delete stored user data when a user's content or account
    is deleted. `devvit publish` sends a source archive; Rust sources outside
    the project are included with `additionalSourceRoots`.

## Progress

- 2026-11-13. F0, draw crate landed: `crates/cleromancy-draw` cut and the
  application switched to it. With `legacy-receipts` on, `derived_selection`
  3, `shuffled_deck` 4 and `shuffle_golden` 1 pass; the `core` package's
  library compiles; clippy, the line ceiling and the dependency bans pass.
  `repos/soothsayer` exists locally with a README only. Not run:
  `embedded_authority_guest`, whose helper builds the authority guest with
  `--locked` now that no lock is tracked. `core`'s test targets not
  compiling is the divination plan's finding of 2026-11-12, not this change.

- 2026-11-11. Plan founded from one design session; D1 to D34 recorded.
  WordNet and pronunciation coverage measured.
  Devvit limits read; Turquet compile-checked for wasm. Nothing built.
