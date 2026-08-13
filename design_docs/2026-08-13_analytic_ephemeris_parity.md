# Cleromancy analytic ephemeris parity

**Date:** 2026-08-13
**Scope:** a second chart engine that reaches neither the filesystem nor the
network, measured against the same reference vectors as the DE440s engine.

## Decision

Cleromancy carries two ephemeris engines behind the existing A14
`AstrologyAdapter` seam. The kernel engine stays the precision reference. The
analytic engine evaluates VSOP87D directly and is available wherever a 31 MiB
kernel download is not.

The two share the SOFARS IAU 1980 nutation, so a comparison between them
isolates the source of planetary positions rather than mixing in a second
convention. Every chart already records its algorithm, engine, and ephemeris
string, so a reader can always tell which engine produced a saved chart, and
the two produce distinct chart digests by construction.

| | `ephemeris` | `analytic-ephemeris` |
| --- | --- | --- |
| Positions from | JPL DE440s kernel via pinned ANISE fork | VSOP87D series, plus a truncated series for Pluto |
| Acquisition | explicit verified 31 MiB NAIF download | none |
| Bodies | ten, including the Moon | nine; the Moon is absent |
| Measured worst residual | 1 millidegree | 14 millidegrees |
| Proof runs | only with a kernel present | in the ordinary suite |

## Measured parity

`tests/analytic_parity.rs` runs the analytic engine against the same NASA/JPL
Horizons quantity 31 vectors as `tests/ephemeris_golden.rs`: J2000, the 2024
total solar eclipse, and 2026-08-13. It needs no kernel, so it is an ordinary
test rather than an ignored one.

The Sun and all eight planets reproduce the Horizons millidegree value
exactly across all three charts, with one exception: Uranus at 2026-08-13 is
one millidegree low, which is a rounding boundary rather than a modelling
difference. Pluto lands within 14 millidegrees.

That was not the expected result. The working assumption when this engine was
proposed was arcminute-scale agreement, on the order of 17 millidegrees. Exact
agreement for the planets means the truncation in VSOP87D sits below the
millidegree rounding the chart contract already applies.

For the product the margin is large either way. A sign is 30 degrees and the
narrowest orb the facts derivation uses is measured in degrees, so a
14-millidegree worst case cannot move a placement or an aspect. The test
asserts a 20-millidegree ceiling, which is a measured bound rather than an
accuracy claim.

## The Pluto frame finding

Pluto is outside VSOP87 and carries Meeus's truncated series. That series is
referred to the standard equinox of J2000.0, while VSOP87D is referred to the
equinox of date. Subtracting one from the other unrotated leaves a pure
precession error: it measured 3 millidegrees at J2000, 351 at 2024, and 375 at
2026, which is general precession at roughly 14 millidegrees per year.

`precess_ecliptic_from_j2000` rotates the J2000 vector onto the mean ecliptic
of date by way of the equatorial frame, using the SOFARS obliquity and IAU
1976 precession matrix rather than a second set of transcribed
ecliptic-precession terms. Residuals fall to 3, 14, and 7 millidegrees.

The residual that remains is the series itself, which was fitted against an
earlier planetary integration than DE440. The near-zero J2000 residual is what
establishes that the coefficient table is transcribed correctly, since at that
instant the frame rotation is the identity.

## The Moon

The Moon needs a lunar theory rather than a planetary one and is absent rather
than approximated. `tests/analytic_parity.rs` asserts its absence, so a later
partial implementation cannot appear as a silently wrong placement.

The closable path is `saurvs/astro-rust` at `c62ffdc`, MIT licensed, whose
`lunar::geocent_ecl_pos` returns a geocentric ecliptic position already
referred to the mean equinox of date. That is the frame this pipeline wants,
so the integration is the nutation step alone. Adopting it is a separate gate:
the crate has been dormant since 2017, which argues for a fork rather than a
dependency, and a fork should carry the same Horizons measurement before the
Moon is offered in a chart.

## Ownership and licenses

- `src/ephemeris/analytic.rs` and the parity test are Cleromancy code under
  `MIT OR Apache-2.0`.
- `vsop87` 3.0.0 is `MIT/Apache-2.0`. It is taken with default features off,
  which leaves its SIMD path disabled.
- SOFARS 0.6.1 is MIT and is shared with the kernel engine.
- The Pluto coefficients are Meeus's published truncated series.

No data file is acquired, committed, or read.

## Stop rule

This gate proves a second engine, its measured agreement with the same
reference vectors, and a correct frame rotation for Pluto. It does not add the
Moon, choose which engine a headed consultation uses by default, calculate
houses, or claim a verified Wasm build.
