# Third-party licenses and data

This file records the optional ephemeris features' distribution boundary. It
is not a substitute for the upstream license texts.

## turquet 0.1.0

- License: MIT
- Source: <https://github.com/merely-made/turquet>
- Pinned revision: `95d1724c35ad7344029382f65c47efe1b9c489f2`
- Provenance: a history-preserving adoption of Saurav Sachidanand's
  MIT-licensed [`astro-rust`](https://github.com/saurvs/astro-rust); see
  Turquet's PROVENANCE.md
- Cleromancy modifications: none

The `analytic-ephemeris` feature adapts Turquet's `apparent` module, which
composes VSOP87D, the partial ELP-2000/82 lunar theory, the analytical Pluto
series, nutation, and precession without any external crate or data file.

## ANISE 0.10.6

- License: Mozilla Public License 2.0
- Cleromancy fork: <https://github.com/merely-made/anise>
- Pinned revision: `71e973a245e6701e14a5d4c88a3c4e7dedbf7702`
- Upstream: <https://github.com/nyx-space/anise/tree/0.10.6>
- License text: <https://github.com/nyx-space/anise/blob/0.10.6/LICENSE>
- Cleromancy modifications: none

Cleromancy may link ANISE while retaining its own MIT OR Apache-2.0 license.
If Cleromancy distributes an executable containing ANISE, recipients must be
informed that ANISE is MPL-2.0 software and where its corresponding source is
available. Any modified ANISE source files must remain available under
MPL-2.0.

## SOFARS 0.6.1

- License: MIT
- Source: <https://github.com/astro-xao/sofars/tree/v0.6.1>
- Cleromancy modifications: none

## ureq 3.3.0

- License: MIT OR Apache-2.0
- Source: <https://github.com/algesten/ureq/tree/3.3.0>
- Cleromancy modifications: none

The optional native provisioning action uses ureq with Rustls to acquire the
kernel from the single source listed below. Downloaded bytes are accepted only
after their exact length and SHA-256 digest match Cleromancy's constants.

## NASA/JPL DE440s

- Source: <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp>
- NAIF checksum list: <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/aa_checksums.txt>
- Official MD5: `3917ee56769db332790c751e2168843d`
- Cleromancy SHA-256: `c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2`

The kernel is downloaded from NASA/NAIF and is not committed to Cleromancy.
NAIF cautions users to retain and understand generic-kernel provenance. NASA's
science-data guidance says repository data may carry an explicit license, be
CC0 when it is NASA-led and unmarked, or require source-specific review when
NASA is not the original source. Until release packaging is decided,
Cleromancy records the NASA/NAIF source and checksum and does not redistribute
the kernel.
