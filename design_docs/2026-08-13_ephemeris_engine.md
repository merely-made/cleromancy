# Cleromancy ephemeris engine

**Date:** 2026-08-13  
**Scope:** calculate source-qualified tropical chart positions without Swiss
Ephemeris.

## Decision

Cleromancy uses the public JPL DE440s SPK kernel through a narrow Rust adapter.
The `merely-made/anise` MPL-2.0 fork, pinned at
`71e973a245e6701e14a5d4c88a3c4e7dedbf7702`, supplies SPK parsing, UTC/TDB
conversion, light-time correction, and stellar aberration. SOFARS supplies the
IAU 1976 precession and IAU 1980
nutation matrices. Cleromancy converts the resulting Earth-centered apparent
state into ecliptic-of-date longitude and latitude, rounds once to the existing
integer chart contract, and derives retrograde state from positions twelve
hours on either side of the requested instant.

The `ephemeris` feature is optional. The default and Wasm builds do not acquire
a kernel or link the astrodynamics stack. In a feature-enabled native build,
the headed chart surface offers an explicit 31 MiB download from NASA/NAIF.
The download is written beside the final path, length- and SHA-256-checked,
synced, and renamed into place. An invalid existing file is preserved with a
`rejected` name. Nothing downloads at startup. `JplEphemerisAdapter::open_de440s`
independently verifies the installed kernel and records its digest in every
chart.

## Ownership and licenses

- `src/ephemeris.rs`, the astrology receipt, graph projection, and golden tests
  are Cleromancy code under `MIT OR Apache-2.0`.
- `merely-made/anise` retains ANISE history and is pinned to the revision above
  under `MPL-2.0`. Cleromancy can link it without
  changing Cleromancy's license. Modified ANISE files must remain MPL-2.0.
- SOFARS 0.6.1 is MIT licensed.
- DE440s is obtained directly from NASA/NAIF. The 31 MiB kernel is not committed
  to this repository. Its official NAIF MD5 is
  `3917ee56769db332790c751e2168843d`; Cleromancy additionally pins SHA-256
  `c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2`.

The fork currently matches upstream at the pinned revision. Cleromancy chart
metadata names the fork and revision. The first source modification will stay
in that repository with its MPL notices and source availability intact.

## Donor audit

XALEN was rejected as the base. Its core tests pass, but the ephemeris crate
pulls houses, stars, ayanamsa, and Vedic layers into the calculation boundary.
Its analytical Moon and Pluto use abridged coefficient tables transcribed from
Meeus, and several advertised external checks skip when local data is absent.
The useful astronomy code is not cleanly separated enough to justify taking
the whole fork.

ANISE stays below the astrology boundary. It has a smaller responsibility,
active CSPICE comparison workflows, and no interpretation catalog.

## Golden proof

`tests/ephemeris_golden.rs` compares three complete ten-body charts against
NASA/JPL Horizons observer-table quantity 31:

1. J2000, `2000-01-01T12:00:00Z`;
2. the 2024 total solar eclipse, with Dallas coordinates retained in the chart;
3. `2026-08-13T12:00:00Z`.

Horizons quantity 31 is observer-centered IAU 1976/1980 ecliptic-of-date,
including light time, gravitational deflection, and stellar aberration. The
adapter currently applies light time and stellar aberration but not solar
gravitational deflection. The selected vectors agree after millidegree
normalization with a maximum residual of one millidegree. The test permits two
millidegrees so it reflects the stored chart precision rather than claiming
sub-arcsecond equivalence.

Run the real-kernel proof with:

```powershell
$env:CLEROMANCY_DE440S = 'C:\path\to\de440s.bsp'
cargo test --features ephemeris --test ephemeris_golden -- --ignored --nocapture
```

## Native consumer

The feature-enabled consultation worker owns kernel acquisition and calculation
off the window thread. The chart form has two explicit actions: install the
verified NASA ephemeris, then calculate and save a chart for a UTC instant and
optional location. The resulting chart and facts use the existing graph truth,
catalog, concurrence, replay, and persistence paths. Manual source-qualified
chart import remains available.

`tests/ephemeris_consultation.rs` proves that a calculated chart survives a
Redb close/reopen. The ignored provisioning network test performs a real NAIF
download and accepts it only after the pinned checksum passes.

## Stop rule

This gate proves a fork-pinned ephemeris adapter, explicit verified kernel
provisioning, source-qualified chart vectors, and a durable native chart
consumer. It does not calculate houses, use the supplied location for parallax,
generate interpretations, or claim an independent ANISE implementation.
