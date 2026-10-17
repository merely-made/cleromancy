# Natal motion, observed Earth orientation, and Placidus

**Status (2026-10-17):** implementation in progress; no new qualification is claimed.

## Scope

Mark authorized retrograde calculation, observed orientation, and Placidus as
the next slice after qualified Whole Sign and Equal House. Observed orientation
is being implemented as retained IERS UT1-minus-UTC and polar-motion observations.
The existing planetary and house editions remain immutable and replayable.

Turquet owns apparent longitude rates, observed Earth-orientation parsing and
interpolation, polar-motion horizon geometry, and Placidus semi-arc roots.
Cleromancy owns explicit choices, source-text custody, sealed supplementary
receipts, readable presentation, admission, and native qualification. No new
calculation silently fetches a data source or substitutes another house system.

## Acceptance

- Signed longitudinal motion retains numerical interval and a disclosed
  near-station threshold; independent fixtures and wrap/station cases pass.
- Retained final IERS observations reproduce UT1 and polar motion within
  coverage. Predictions, gaps, leap discontinuities and extrapolation refuse.
- Placidus agrees with independent reference cusps within a recorded bound;
  unsolved/polar cases remain unavailable with no fallback.
- Earlier receipt paths remain byte-identical and original natal/house records
  still replay against their original numerical editions.
- The portable resident validator rejects altered motion, observations and
  cusps before writing, including writes from another application.
- New native occasions expose motion, Placidus and observed provenance in wide
  and compact layouts. Fresh offline processes preserve exact saved records
  and reflections. Software, native, publication and wider platform acceptance
  remain separate claims.
