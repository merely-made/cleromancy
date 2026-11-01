# Custody and projection receiver

> **Archived 2026-11-01 (landed).** Received in `89bf653`, qualified in `f62b673` (2026-10-29). Evidence is retained on T-PC only; its paths and SHA-256 receipts are in `archive_docs/receipts/2026-10-29_custody_projection_receiver_receipts.md`. No open points to extract.

Status: Published API migration; focused grant/revocation test passed after updating its fixture requirements.

Cleromancy selects projection-door, local and stdio. Its production and test
AppBrokerClient imports move to `door::projection`; no compatibility alias
remains. Custody clients are not selected by this application's declaration.

The receiving lock selects Mere `c6290142866eefead385e728c5c5da24ac6f624a`,
Genet `565458ea3ab816940484079330eb165a878a22e0`, Dramatis
`051e95a68011072f172b9ec4c0f04d4e5259df5b`, Murm
`1f601488f851c4e54672cd439b7c45728255ea65` and Knot
`704084d064da886dd14cbe8ac3dd8758af105e1d`. The direct/patch Eidetic sources
align to `01ee97b0b95d679430986dffd54c64ce9b3b5ea3`; the previous f599 source
lacked Journal::prefix_digest required by receiving Pandect. Genet's current
events and Taffy contracts also require the receiving cohort. NetRender and
unrelated registry selections were preserved. Resolved lock-edge review found
no changed registry dependency target names or versions.

The focused command is:

`cargo test --locked -p cleromancy --no-default-features --test granted_ambient neighboring_preview_requires_its_own_live_grant_and_clears_on_revocation`

The first multi-receiver run completed Turnstone, then timed out during this
consumer's dependency compilation at the fixed 600-second limit. The test did
not execute. A subsequent attempt expired waiting for the shared machine lock.
Neither is a failing behavioral test or a passing receiver receipt. The runner
retains 3 GiB memory, zero swap, 150 percent CPU and one build job. Further
results will be recorded separately; Windows and GUI runtime are unqualified.

Evidence is retained on T-PC only (the receiver log, the lock-wait record and
the resolved-edge audit). Their paths and SHA-256 receipts are in
[the receipts file](../receipts/2026-10-29_custody_projection_receiver_receipts.md).

## Completed focused qualification

`root-split-cleromancy-final` passed the focused locked test: one passed,
zero failed or ignored. It exercises the admitted projection client, required
ambient grant, large resource assembly and immediate card withdrawal following
revocation. Its log SHA-256 is in
[the receipts file](../receipts/2026-10-29_custody_projection_receiver_receipts.md).
The same 3 GiB, zero-swap, 150 percent CPU limits were retained.

The receiving fixture required three explicit adaptations: its Castellan dev
dependency selects keeper, a Djinn dev declaration selects in-process-custody,
and ResidentReservoir::open receives the existing persona directly. Production
feature declarations are unchanged by these fixture adaptations. Static review
confirmed their dev-only scope and use of the existing temporary root and
in-memory vault. Earlier compiler failures and lock-wait receipts remain retained;
they were not behavioral red tests.

The final fixture lock audit adds credential-exchange-format 0.4.0 and
Djinn-custodian from the same c629014 supplier, plus the selected fixture feature
edges. No existing package identity or dependency edge was removed. No existing
registry version selection changed. The report is retained on T-PC only (path in
[the receipts file](../receipts/2026-10-29_custody_projection_receiver_receipts.md)).
This receipt qualifies the local broker fixture, not the full UI or Windows.
