# Third-party licenses

This file records the optional ephemeris feature's distribution boundary. It
is not a substitute for the upstream license texts.

Cleromancy acquires no data file. The JPL DE440s kernel lane, and with it
ANISE and the download machinery, moved to Turquet's opt-in `verify` feature,
where it generates golden vectors as maintainer tooling. Turquet records that
lane's licenses and the kernel's NASA/NAIF provenance.

## turquet 0.1.0

- License: MIT
- Source: crates.io, <https://github.com/merely-made/turquet>
- Pinned version: `=0.1.0`, taken from the registry rather than Git so the
  exact bytes stay retrievable if the repository moves
- Provenance: a history-preserving adoption of Saurav Sachidanand's
  MIT-licensed [`astro-rust`](https://github.com/saurvs/astro-rust); see
  Turquet's PROVENANCE.md
- Cleromancy modifications: none

The `analytic-ephemeris` feature adapts Turquet's `apparent` module, which
composes VSOP87D, the partial ELP-2000/82 lunar theory, the analytical Pluto
series, nutation, and precession without any external crate or data file.

## turquet 0.17.0 (`sky-timeline`)

- License: MIT
- Source: Git, <https://github.com/merely-made/turquet>
- Pinned revision: `30875f30e83909f3030bc01237a5046e14cde29b`
- Provenance: optional, unpublished consumer dependency pinned to an exact
  repository revision; this is separate from the registry `turquet 0.1.0`
  chart adapter above
- Cleromancy modifications: none

The `sky-timeline` feature consumes Turquet's public event APIs and stores
Cleromancy-owned, serde-safe facts rather than Turquet values. It does not
turn caller-selected airless altitude crossings into a default visibility,
refraction, limb, horizon-dip, or weather model.
