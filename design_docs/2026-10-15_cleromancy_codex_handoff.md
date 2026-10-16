# Cleromancy handoff for Codex, 2026-10-15

**C6 qualified (2026-10-16):** native Read records time and pinned sky facts,
with place/weather independently off by default. Explicit weather stores the
actual response and attribution; exact fresh-process reopens pass with outbound
IP denied and future consent still off. The
[C6 ledger](../receipts/c6-situation-qualification_20261016.json) and
[evidence notes](../receipts/c6_20261016/README.md) bind the two save-source and
reopen-source identities, 120/160 product tests, portable-core 36/37/38, seven
receiving owner tests and ten reviewed originals. All six earlier occasions,
complete C4 views/memory and all 424 older receipt/scenario paths remain exact.
A narrow supplier intake separates endpoint discovery from checked domain
opening. Startup timeouts and the combined-flow presentation failure remain
recorded; bounded feature proof does not establish consistent startup or release
performance. C7 is next. Publication is recorded separately; dated C5 and older
notes below retain their original scope.

**C5 published (2026-10-16):** shuffled-deck receipts, reversals and seeded
multi-position tarot reads are implemented. The
[C5 evidence notes](../receipts/c5_20261016/README.md) and
[qualification ledger](../receipts/c5-shuffled-qualification_20261016.json)
retain source-bound software, ten original native captures, exact fresh-process
reopening and receiving-owner proof. Current Isocosm has removed its
Cleromancy dependency; historical scalar derived bytes remain tested. All 333
C4-and-earlier receipt/scenario paths remain exact. C6 is the next product
phase. Product `e94a47c` and Mere `81e6805cf` are published; the
[C5 publication ledger](../receipts/c5-publication_20261016.json) records
independent retrieval and Knot's isolated compatibility checkpoint. C4's
publication record and all older receipts remain unchanged below.

**Current continuation (2026-10-16):** published C3 remains intact. C4 is now
implemented, software-qualified and bounded native-qualified at product `d6ea9f7`, frozen supplier
`72f9c5c73`, with eleven reviewed originals and exact sealed-reading/full-view/
memory reopening. The [C4 evidence notes](../receipts/c4_20261016/README.md)
link the software, native and receiving ledgers. Final product gates pass
108 default/147 historical tests, both all-target checks, the active sky fixture
and portable-core 27/28/29. Integrated Mere `8111fa2ba`
receives the same portable validator and passes five active owner fixtures
with one intentional process-child entry. All 267 prior receipt/scenario paths
remain exact. Mark authorized C4 publication and proceeding. Product `29d4f3b` and integrated
Mere `9b6da81db` are published; exact checkpoint objects were retrieved from
GitHub in fresh bare repositories. The additive
[publication ledger](../receipts/c4-publication_20261016.json) records verification. The
coordinated per-repository plan is in the
[receiving ledger](../receipts/c4-resident-intake_20261016.json). C5 is the next
product phase. The dated handoff and older completion notes below retain their
historical scope.

**Status (2026-10-15 continuation):** C1/C2 lane published and merged on main;
integration CPU checks pass. The coordinated family round and branch-source
manifest/lock switch have since landed. Personal-sync compatibility and all
requested software intake checks pass at `9aae258`. Historical sections retain
their inputs; the original native qualification keeps its frozen supplier scope.

**Original status:** handoff, written for the Codex lane to execute in order. It covers
what the Mere stack lift changed under Cleromancy on 2026-10-15 and what
Cleromancy still owes. Every ruling quoted here is Mark's and is settled. If
a task needs a decision that isn't here, stop and ask Mark, giving concrete
options.

## 1. Historical context at handoff creation

### `main` is behind the lane

`origin/main` is `d18590f`, on the old family:

- Mere `0b9b7a9b`;
- Genet `7a8c5b8`;
- `graphshell::native` and the port's `graphshell::lifecycle` paths.

### The lane holds the current work

The Codex lane in Mark's checkout (`/Users/markik/Code/repos/cleromancy`) is
on branch `codex/c1-resident-divination-20261014`.

- **Pushed to that branch:** through `8e395cd`, which is 6 commits ahead of
  `main` and 1 behind it.
- **Local only, not pushed:**
  - `fcfb1a4`: "Consume the extracted Mere stack and narrate saved readings";
  - `b153672`: "Record C2 stack intake and bounded native qualification" (2026-10-15 18:02).
- **Separate branch:** `codex/c1-float-roundtrip-core-20261014`, at
  `f7ec0db`, 3 commits ahead.

### What `b153672` already pins

The lane has already consumed most of the lift.

| Supplier | Pinned at `b153672` |
|---|---|
| Mere | `8b08c98b2`, for all 160 Mere rows |
| Genet | `ae3d9f5d` |
| Murm | `38e9c9b8` |

Imports:

- `graphshell_host::{endpoint_catalog, lifecycle, view}`;
- `graphshell_client::{RetainedEndpointSession, ResolvedContent}`;
- `graphshell_endpoint::{local::LocalCarrier, stdio::StdioCarrier}`;
- `djinn`, `pandect`;
- `graphshell-admission = ["graphshell-host/native"]`.

What still comes from the Graphshell port: only `graphshell::personal_sync`,
behind the `personal-sync` feature. That's lift L13. It moves once Mere's
facet dissolution brief rules, so Cleromancy keeps `graphshell` until then.

The lane already solved one-source resolution with a double-slash Mere URL
(`https://www.github.com/merely-made//mere.git`). It is a redirect-equivalent
spelling that patches both historical Mere spellings to one immutable
replacement, because Cargo refuses to patch a source to another revision of
itself. That was the frozen C2 workaround; Task 2 below supersedes it. Current
family manifests use one canonical URL and branch reference, and no alias bridge.

### What changed in Mere and Murm since `8b08c98b2`

All of it is on Mere `main`. As of this handoff, Mere `main` is `61fba39d9`.

- **P5:** Graphshell's brokers moved to Djinn (`djinn::door`, `djinn::sync`),
  and the line codec moved to `chirograph::line`.
- **Follow-ups:**
  - `view`, `sessions` and `canary` moved to `graphshell-host`;
  - `browser_carrier`, `identity_endpoint` and `identity_projection` moved to
    `djinn::door`;
  - `resident_mere_proxy` moved to `graphshell_endpoint::door`.
- **Merge-in:** branches; `MereViewRequest` and `SessionStep` are now
  `#[non_exhaustive]`.
- **Rename:** `edit-history` is now `backstep`, published as 0.1.0. sceno and
  scenomise are bumped to 0.0.5.
- **Murm:** one line on `main`, at `fb6360c12b`. It merges the C1 callback
  fix `3d9e799f` and the in-process kind `38e9c9b8`. The old Murm branch
  names are deleted, and both commits are reachable from `main`.
- **Merge driver:** `weave` was removed from every family `.gitattributes`,
  including Cleromancy's (`d18590f`). User git now uses `zdiff3` and
  `rerere`.

### Mark's rulings that bind this work

- **Git:** land work on `main` as it goes green. Merge lane branches into
  `main`, don't stack new ones. Fetch first, and never rebase or force-push.
  Mark has authorized pushing.
- **Platform family (superseded by the later source-convention ruling):**
  the frozen qualification used one immutable supplier family. Current manifests
  use canonical URLs with `branch = "main"`; committed locks preserve the exact
  tested revisions. See Task 2 and Mere's family composition brief.
- **Remote mere route:** the dialable host starts only once a trusted root
  other than the persona's exists. Device roots derive from the persona, and
  pairing issues grants. Place roots come from the place record. See
  `mere/design_docs/mere_docs/technical_architecture/2026-10-15_remote_mere_route_spec.md`
  §11.

## 2. Tasks, in order

### Task 1: land the lane

1. In Mark's checkout, confirm the working tree is clean. Push `fcfb1a4` and
   `b153672` to `codex/c1-resident-divination-20261014`.
2. Decide whether `codex/c1-float-roundtrip-core-20261014` is contained in
   the resident-divination line. Its `a3d96b6`/`e2e9c27` commits appear in
   both histories, so check with `git branch --contains` and `git cherry`.
   - If it is contained, it's superseded.
   - If not, merge it first.
3. Merge the lane into `main`: `git switch main`, `git merge --ff-only
   origin/main`, then `git merge --no-ff codex/c1-resident-divination-20261014`.
   - Resolve the `.gitattributes` deletion from `d18590f` in `main`'s favour.
   - Resolve `DOC_README.md` by keeping both sides' entries. This handoff's
     index entry is on `main`.
   - Run the checks below and push `main`.
4. Delete the merged branches, local and remote.

**Done when:**

- `main` holds the lane's work and this handoff;
- the checks pass;
- no `codex/c1-*` branch is left unmerged.

### Task 2: switch to the family's `branch = "main"` convention

**Status: complete 2026-10-15.** The manifest/lock switch landed in `a4364de`
and `22df4c9`; the product source follow-up is `9aae258`. All requested software
checks pass; see the [intake receipt](../receipts/family-intake_20261015.json).

**Manifest/lock checkpoint (Claude, at Mark's direction):** Cleromancy's rows for Mere, Murm,
Genet, netrender, Knot, Cleromancy (its own package, patched by path for
Mere's Djinn pin) and dramatis name their repository as `git =
"https://github.com/merely-made/<repo>.git", branch = "main"`; the www and
double-slash spellings and every same-repository bridge table are gone.
`Cargo.lock` and `core/Cargo.lock` resolve one source per family repository:
Mere `59f88e63`, Murm `cba46d85`, Genet `e37762d7`, netrender `f4f9c0d8`, Knot
`11b07360`, dramatis `5c89001a`. A repin from here is `cargo update -p
<package>` (use the full package id when a name is ambiguous, for example
`cargo update -p 'https://github.com/merely-made/mere.git#mere-kernel@0.0.2'`).

**Historical failure at the manifest/lock checkpoint (fixed by `9aae258`):**
No product code was touched in the switch. At that state `cargo check --all-targets`,
`--features graphshell-admission --all-targets`, the `core/` check and
`cargo test` pass, but `cargo check --features personal-sync --all-targets`
fails: 15 `E0063` errors, "missing field `basis` in initializer of
`PersonalGraphEvent`" (a field Mere added), in the library and its tests. Fix
those initializers (the earlier list below still applies where a hit is left:
`graphshell_client::line` to `chirograph::line`, `edit_history` to `backstep`,
wildcard arms on `MereViewRequest` and `SessionStep`, moved Graphshell module
paths), then run the checks and record the counts in the divination plan.

**Checks** (from the README):

- `cargo test`, the proof wall;
- `cargo test --features sky-timeline --test sky_timeline`;
- `cargo check --features personal-sync --all-targets`;
- `cargo check --features graphshell-admission --all-targets`;
- the `core/` portable package's own tests.

Use `TMPDIR=/tmp` for anything that opens a Unix socket. macOS limits socket
paths to 104 bytes, and the default temp dir is longer.

**Done when:** pushed on `main`, with the counts recorded in the divination
plan's progress section.

### Task 3: record P4's second-host receipt for Mere

The Mere lift plan's P4 done condition names Cleromancy: "Cleromancy
registers an endpoint with `ResidentEndpointCatalog` and
`AdmittedEndpointContext` without `graphshell`." The lane already does this
through `graphshell-host`.

Point a test, or a receipt line, at that registration, and confirm the
endpoint path names no `graphshell` package. The port remains only for
`personal_sync`. Send the commit to the Mere lift plan's owner, or add a
Progress line there yourself if you can push Mere.

### Not for Codex yet

- **L13 / P8:** Cleromancy builds with no `graphshell` package once
  `personal_sync`'s replica, event and projection types have a stack home.
  That waits for Mere's facet dissolution brief, which is waiting on graph
  semantics' owner.
- **Native foreground runs:** these are serialized with the Woodshed and Knot
  lanes. Ask before taking the screen, and check it's unlocked:
  `ioreg -n Root -d1 -a | grep -A1 CGSSessionScreenIsLocked`.

## Execution continuation (2026-10-15)

- Published `fcfb1a4` and `b153672` on the resident-divination branch.
- Fast-forwarded local main to published `9d8ebda`, then merged that lane at
  `1e2fe7f`. Main's removal of `.gitattributes` and both index entries
  are retained. Product source, tests and both manifests/locks are byte-identical
  to `b153672` after integration.
- `git cherry` identifies portable `f7ec0db` as patch-equivalent to the
  resident line. Its only changes are core manifest/lock pins, superseded by
  C2's newer qualified intake. Merge `0f775a0` records its ancestry with the
  current tree retained. No old receipt or dependency is reverted.
- Re-ran the existing read-only C2 verifier: 12 original capture hashes, 145
  source inputs, exact reopened IDs/card bytes and 213 historical tracked
  receipts pass. This preserves the frozen native source identity; merging
  documentation is not a new native qualification.
- The Knot checkout has the coordinated repin round in progress. R2 is not
  yet identified as published. The required choice to wait or consume current
  Mere main is pending with Mark; no independent supplier repin has begun.

- Main integration checks pass: default 92, portable core 18, endpoint
  admission/catalog 2, and the active historical sky-timeline fixture 1.
  Personal-sync and graphshell-admission all-target checks pass separately.
  The handoff's sky-only invocation runs zero tests; the additional
  `legacy-receipts,sky-timeline` invocation executes its fixture.
- [Integration receipt](../receipts/main-integration_20261015.json) records
  the exact merge, log hashes and P4 endpoint source paths. Registration and
  admitted session use `graphshell-host`; `src/admitted.rs` still has its
  feature-gated `graphshell::personal_sync` import. The port also remains in
  the resolved graph; this is the endpoint-path receipt, not L13 acceptance.

## Completed software intake (2026-10-15)

- The coordinated round and subsequent family source convention supersede the
  earlier wait for R2. The receiving locks already resolve Mere `59f88e63`,
  Murm `cba46d85`, Genet `e37762d7`, Netrender `f4f9c0d8`, Knot `11b07360`
  and Dramatis `5c89001a` through one canonical branch source per repository.
- `9aae258` fixes all 15 missing optional assertion-basis initializers without
  inventing evidence or changing the absent-field wire representation.
- Gates pass: 92 default tests; 130 historical-feature tests; portable core
  18/19/20; 5 focused sync fixtures; 1 active sky-timeline fixture; separate
  personal-sync and graphshell-admission all-target checks. The historical
  suite includes both P4 admission/catalog runtime tests.
- All 218 tracked prior receipt files and all 12 original C2 capture hashes
  remain exact. Frozen C2 source blobs and reopen IDs/card bytes also match.
  The [family intake receipt](../receipts/family-intake_20261015.json) records
  current software checks separately from the unchanged native evidence.
- Mere's lift plan and index now record the bounded P4 consumer proof. The
  Graphshell port remains in the dependency graph; L13/P8 is still separate.
  No new foreground run, graphical-unlock or complete shared-wall acceptance
  is claimed by this software intake.
