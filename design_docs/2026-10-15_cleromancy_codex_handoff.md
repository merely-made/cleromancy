# Cleromancy handoff for Codex, 2026-10-15

**Status (2026-10-15 continuation):** C1/C2 lane published and merged on main;
integration CPU checks pass. The coordinated R2 intake
remains pending. The historical handoff below records its original inputs.

**Original status:** handoff, written for the Codex lane to execute in order. It covers
what the Mere stack lift changed under Cleromancy on 2026-10-15 and what
Cleromancy still owes. Every ruling quoted here is Mark's and is settled. If
a task needs a decision that isn't here, stop and ask Mark, giving concrete
options.

## 1. Where Cleromancy stands

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
itself. Keep using it. Knot and Turnstone hit the same Cargo limit today.

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
- **Platform family:** pin one immutable Mere, Genet, Netrender and Murm
  family, and update those pins together.
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

Task 1 has landed (`main` is `39d65f7`, the lane merged). The repin round is
done and Mark ruled the family onto branches (Mere's family composition brief,
"Family source convention"): first-party rows name their repository as
`git = "https://github.com/merely-made/<repo>.git", branch = "main"`, with no
`www.` spelling, no double slash and no `rev`; the committed `Cargo.lock` pins
the tested revision; a repin is `cargo update -p <package>`. Current tips at
the time of writing: Mere `4a74b1cf2` (plus a lock-only bump), Murm
`a3621dc8`, Genet `7af8de32`, netrender `f4f9c0d8`, Knot `3fc16df2`.

1. Run the helper that rewrites those rows and drops the same-repository patch
   tables:
   `python3 "/Users/markik/Library/Application Support/Claude/side-session-notes/local_3d62af74-7793-41a6-ad04-66e8c1de0af1/to_main.py" <repo-root> mere,murm,netrender,genet,knot-editor`.
   Leave `dramatis`, the vello fork and any other pinned fork on their `rev`
   or `tag`. The double-slash workaround goes away: nothing Cleromancy pulls
   names another spelling once every supplier is on the convention, and Cargo
   cannot bridge two spellings of one repository.
2. Make the source changes:
   - `graphshell_client::line` and `graphshell_host::line`, if named
     directly, become `chirograph::line`;
   - `edit_history` becomes `backstep`;
   - any `match` on `MereViewRequest` or `SessionStep` gets a wildcard arm;
   - anything that named a moved Graphshell module (`view`, `sessions`,
     `canary`, `browser_carrier`, `identity_*`, `resident_mere_proxy`) takes
     its new path.
3. Run `cargo update -p <a Mere package> -p mere-transport -p layout-dom-api`,
   then confirm `cargo metadata` resolves exactly one source for each family
   repository. If a second source remains, find the dependent (a supplier that
   still names a repository by `rev`) and report it rather than adding a
   bridge.

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
