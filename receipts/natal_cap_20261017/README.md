# Natal motion, observed Earth orientation, and Placidus

The implementation source is Cleromancy `da495cfc75b2ee6bbb9476c8c03386e9057c36d3`.
New numerical facts use immutable Turquet
`8e3755bbd0d40343eb4750b072801c941ff32802`. Original planetary facts retain
`30875f30`; original Whole Sign/Equal receipts retain `e2f65c0`. The new optional
supplement is omitted from old records, preserving their exact representation
and identity. Nothing imports or rewrites an earlier reading.

## Product behavior

New occasions retain apparent signed geocentric tropical longitude rates for
ten bodies. Turquet samples at ±864 TT seconds. Cleromancy rounds each rate to
integer microdegrees per TT day before applying the inclusive ±1000 near-station
band. This is a disclosed small-rate policy, not an exact station or an accuracy
bound. The signed rate remains available, including within that band.

Whole Sign, Equal House and Placidus are explicit choices. Placidus divides
semi-arcs and refuses circumpolar/unsolved geometry without another system being
substituted. Planetary house membership uses the original retained longitudes
and excludes ecliptic latitude. No new symbolic interpretation is added.

Observed orientation here means Earth orientation. A person explicitly pastes a
bounded IERS finals2000A excerpt. The exact text, BLAKE3 digest and interpolated
UT1-minus-UTC/xp/yp values are sealed into the supplement. The parser accepts final
Bulletin B values and rejects relevant prediction flags, malformed records,
gaps, extrapolation and interpolation across leap-table discontinuities. No
calculation downloads data. Blank input retains the explicit UT1=UTC approximation.
Source labels are assertions about the supplied text, not an authentication of
its origin. Celestial-pole offsets, refraction and EOP uncertainty propagation
are outside this edition.

## Evidence

The top-level [qualification ledger](../natal-cap-qualification_20261017.json)
records actual software exits/counts and the separate native and receiving-owner
source identities. Turquet's default suite passes 163 tests/doctests. It retains
31 motion oracle rows, 259 Placidus cases, 24 unavailable cases, six observed
geometry references and the primary IERS excerpts/format documentation. Optional
JPL verification is not renewed.

Five native processes pass with 22 reviewed original captures:

1. `native-setup-01`: shared contact, missing-place natal occasion with motion,
   reflection, then a new saved placed birth revision.
2. `native-create-01`: explicit selection of the saved placed revision whose resource identity was recorded;
   approximate Placidus, a second reflection, then observed Placidus. Wide,
   provenance and 420×760 compact captures. A third, polar birth revision is saved.
3. `native-reopen-01`: actual fresh offline reopening preserves all three readings
   and both reflections exactly. The shared catalog supplies the polar resource
   identity used by the next selection.
4. `native-polar-01`: explicitly selected polar birth creates an unavailable
   Placidus result while preserving motion. Wide and compact captures.
5. `native-reopen-final-01`: actual fresh offline reopening preserves all four
   readings and both reflections exactly, including the observed and unavailable
   outcomes. Saved birth resources and subjects remain exact across these stages.

Every run uses the production encrypted-vault prompt in an owned PTY, with
PATH=/bin selecting real terminal fallback. Outbound IP is denied in all five
processes; no model is configured. Each process must exit successfully and produce
an `ok` receipt. The exact native binary and 111 local depfile inputs are bound
to the committed source. All original PNGs were viewed; dimensions and hashes are
in `capture-reviews.json`. Some captures begin at a section/viewport edge, with
lower rows available by scrolling. Full database byte equality, startup/performance,
graphical unlock, browser cutover and release acceptance are separate claims.
Creation UI association caches are not treated as a complete fresh shared catalog.

Independent source review found two consumer boundary issues: a collapsed TT
Julian date could precede an exact midnight observation; rounded motion values
could disagree with an unrounded classification. The new edition uses separate
midnight/time-scale components and classifies the retained integer. Singleton
midnight and post-leap-midnight regressions plus positive/negative threshold
regressions cover the fixes. The reviewer closed both findings by source inspection;
its report does not claim independent execution of tests.

## Preservation and diagnostics

All 1046 preexisting Cleromancy evidence files and 39 Turquet receipt/vector files
match their pre-task SHA-256 values. Old natal and house records replay in the
focused tests. The earlier evidence verifier is run separately.

The retained initial combined supplier test overlapped a house-geometry correction
and failed; the final frozen default suite passes. The first EOP-tamper test had
an ineffective string replacement; its failing log is retained, and the corrected
test changes an exact fixed-column field and verifies that the input differs.
The first public-fixture provisioning attempt used the wrong marker; it refused
before provisioning, and the corrected marked fixture passed. Initial shared
Cargo updates selected an unrelated unpublished renderer reference. The successful
Cargo-generated narrow lock uses an isolated cache with existing supplier refs;
no shared Git refs or manifests were changed. All other supplier revisions remain
exact. Three registry dependency edges normalize between existing package versions;
the complete lock diff and solver report are retained. These diagnostics are not
relabeled as successful gates.
