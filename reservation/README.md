# cleromancy

Name reservation for **cleromancy**, a local-first divination journal.

Cleromancy makes a reading and keeps its complete workings. A reading is
selected in one of three disclosed ways: **Calculated** takes the highest
disclosed qualified weight, **Cast** draws once from operating-system
cryptographic randomness, and **Derived** hashes a public seed and domain
into a replayable choice, which is reproducible pseudorandomness rather than
fresh entropy.

Every reading seals a receipt naming its context digest, field digest,
qualified weights, algorithm, and selected candidate, so it replays from
graph truth alone after the caller that asked for it is gone. Saved
occasions, immutable reflections, authored spread layouts, and
source-qualified astrology charts are ordinary graph nodes beside it.

What the receipts prove is calculation and replay. They do not claim
supernatural causation, and grouping a chart with a reading records only that
the two were consulted together.

The boundaries are the point: not an oracle (interpretation is authored
content, never generated), not a service (the default data root is private
and local, and personal sync is opt-in and off until a device consents), and
not an ephemeris (celestial positions come from
[turquet](https://crates.io/crates/turquet), measured against NASA/JPL
Horizons, and Cleromancy owns only the chart contract).

## Status

The application exists and runs; see the
[repository](https://github.com/merely-made/cleromancy). It is not published
here because its engine, graph, and shell dependencies are unpublished, and a
release that resolved them from the registry would not build. This crate
holds the name until that changes.

Written with AI assistance.
