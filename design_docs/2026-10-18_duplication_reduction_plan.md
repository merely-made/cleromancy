# Reduce duplicated mechanics without moving product authority

**Status (2026-10-18):** bounded source audit and first aspect-geometry consolidation complete; shared resident materialization is implemented in Cleromancy with bounded native reopening qualified; Knot native presentation qualification remains open; dependency-boundary work remains planned. Notarization is deferred at Mark's request; its prior account refusal and signed/native receipts retain their original identities.

## Purpose and boundary

Mark asked to reduce complexity and improve stack reliability by removing unnecessary duplication. Start at the Cleromancy/shared-stack seam and distinguish repeated infrastructure from deliberate product semantics. The objective is fewer independently maintained contracts and failure paths, rather than simply fewer lines or crates. Shared code should own a mechanism only where its consumers really agree about the contract. Reading interpretation, correspondence policy, replay editions and durable domain admission remain product responsibilities.

This is a bounded audit, not a whole-stack coverage claim. Cleromancy baseline is `5590a2a05f47b79d93d9688506be07710afa0223`, fetched and clean before edits. The inspected primary Mere checkout is `dd9d5c523c092baf72be31cdbc1d28c4ae179fa0` and contains unrelated source-edit work; it is read only for this investigation. The Knot source is comparison evidence, not renewed Knot qualification. No supplier intake or native acceptance is inferred from the audit.

## Findings (2026-10-18)

| Candidate | Concrete duplication and risk | Direction and relative effort |
| --- | --- | --- |
| Resident session/archive reading | Cleromancy `src/ui/resident.rs::refresh`, Knot `crates/knot-editor/src/mere_overlay.rs::read`, and Mere `crates/cambium/mere-overmap/src/route_session.rs::MereRouteSession` each sequence session/graph/archive materialization. Independent sequencing can drift around archive cards, unavailable state and refresh. The implementations are not yet proven interchangeable. | Highest architectural priority; medium-to-large effort. Specify a shared verified materialization contract and adapt two real consumers before retiring their duplicated portions. |
| Aspect geometry | `src/astrology.rs` and `src/natal_aspects.rs` repeat the exact-angle table and shortest circular separation. An edit in one can silently diverge from the other. | First bounded change; small effort. Reuse `AspectKind` ordering/angles and one `AstrologyPosition` separation method. Keep each edition's selection policy unchanged. |
| Default Graphshell-port dependency | Direct `graphshell::` imports are feature-gated personal-sync uses, but `cargo tree --locked --offline -i graphshell` shows both Cleromancy and Djinn as default incoming edges. Making only the product dependency optional would leave the port in the closure. | Investigate the Djinn/port boundary before dependency pruning; medium effort. Require actual default-graph removal and preserved sync/door behavior, not a manifest-only claim. |
| Portable core | `core/Cargo.toml` points its library at `../src/lib.rs`. The resident validator and desktop already share product implementation source. | Preserve this arrangement. Another core extraction or decoder rewrite would add complexity rather than remove source duplication. |
| Versioned astronomy adapters | Three exact Turquet sources implement retained reading editions. Houses/cap adapters have similar conversions, but dependencies and numerical contracts differ. | Preserve until explicit replay-equivalence evidence exists. A latest-version pin replacement is not a deduplication proof. |

The resource checks in Cleromancy are not removed merely because newer shared clients validate resource chunks. The pinned consumer closure must be inspected and its full hash/session/refusal contract demonstrated before declaring a product check redundant. Product validation remains necessary after transport validation: a correctly hashed payload can still be invalid divination data.

## Phases and done conditions

### Shared aspect facts

Use one typed aspect-angle definition and one shortest-circle-distance calculation. The original astrology edition still selects the nearest eligible kind, with its existing tie break and traversal order. The optional natal edition still applies independent saved orbs, retains overlapping matches and emits its existing sorted order. No schema, method identity, preset, numerical unit or saved record changes.

Done when the focused natal suite replays all six actual saved native records, including original planetary/house/cap editions and optional aspects, without changing any retained JSON values or seals. Existing aspect tests must still cover custom rules and overlap behavior. Relevant product and portable-core gates must pass. Existing evidence remains frozen; this source refactor does not renew the signed binary's native qualification.

### Shared resident materialization

First compare the contracts for sessions, attached graph, archive cards, application-owned view state, unavailable refresh and read-only transient inspection. Select an existing shared owner rather than adding another generic graph cache. Keep the complete graph revision, session identity, application identity, content hashes and product replay validator checked before publishing candidate state.

Done when two consumers use the same materialization mechanism, tests demonstrate stale/mismatched resource refusal and safe failure publication, and native fresh-process/save/archive/view isolation proofs pass for each affected product. A source helper without consumer adoption is incomplete. Supplier work uses an isolated checkout and preserves the active source-edit lane.

## Progress

- 2026-10-18: Completed the bounded comparison above. First source consolidation removes the natal copy of angle constants and circle-distance arithmetic. Added a regression that replays and round-trips all six original signed-native reading values using the existing committed receipt, without copying or rewriting those fixtures. Software validation is in progress.

- 2026-10-18: Software validation passes: portable core 60 tests, desktop natal 18 tests, zero failures/ignores. The [source-bound receipt](../receipts/duplication-reduction_20261018.json) records test outcomes, changed-file hashes, unchanged manifests/locks and preservation of all 1275 prior tracked receipts. The desktop debug linker emitted its large compact-unwind-table warning; no runtime/performance claim follows. No new native acceptance or signed-package replacement is claimed.

- 2026-10-18: Mark authorized shared resident materialization and both consumer migrations. The shared contract and qualification criteria now live in `mere/design_docs/2026-10-18_resident_materialization_plan.md`. Cleromancy consumes its complete mode and retains product replay validation; Knot consumes projection-only mode above its own catalog. Isolated supplier/Knot checkouts preserve active source-edit work. Source integration and qualification are in progress; no prior native receipt is relabeled.

- 2026-10-18: Cleromancy source `420bcc3` consumes the shared complete reader from Mere `9b8c161a8`, with product replay validation retained. Native qualification exposed an asynchronous completion race: an unrelated publication could clear the pending Mere command, allowing Restore to be observed before its acknowledgement. The worker now publishes model and restored selection atomically; background publications do not acknowledge commands. The original failed attempt remains preserved. The unchanged lifecycle and fresh-process reopen scenarios then pass with eleven reviewed originals, identical session/reading/reflection IDs and byte-identical three-card previews. The new [receipt packet](../receipts/resident_materialization_20261018/) keeps the exact source, binary, controller, failed attempt, successful processes and preservation manifests. Compact clipping and below-fold archive controls remain explicit capture limits. Shared two-product completion still requires Knot native archive/reopen qualification; source/software and real-broker tests alone do not close that gate.
