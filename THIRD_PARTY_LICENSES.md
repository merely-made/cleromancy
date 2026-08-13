# Third-party licenses

This file records the optional ephemeris feature's distribution boundary. It
is not a substitute for the upstream license texts.

Cleromancy acquires no data file. The JPL DE440s kernel lane, and with it
ANISE and the download machinery, moved to Turquet's opt-in `verify` feature,
where it generates golden vectors as maintainer tooling. Turquet records that
lane's licenses and the kernel's NASA/NAIF provenance.

## turquet 0.1.0

- License: MIT
- Source: <https://github.com/merely-made/turquet>
- Pinned revision: `d29145181191b3f545cceda0b50bdc523c58a1da`
- Provenance: a history-preserving adoption of Saurav Sachidanand's
  MIT-licensed [`astro-rust`](https://github.com/saurvs/astro-rust); see
  Turquet's PROVENANCE.md
- Cleromancy modifications: none

The `analytic-ephemeris` feature adapts Turquet's `apparent` module, which
composes VSOP87D, the partial ELP-2000/82 lunar theory, the analytical Pluto
series, nutation, and precession without any external crate or data file.

