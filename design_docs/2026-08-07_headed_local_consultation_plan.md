# Cleromancy: headed local consultation plan

**Date:** 2026-08-07
**Status:** active; H0-H1 complete
**First proof:** one saved-and-reopened single-card tarot consultation

## Decision

The A0-A25 protocol ladder stops here. Cleromancy's next phase is an ordinary
local application over the graph truth already proved.

The first vertical slice must let a person:

1. launch Cleromancy against a private local data root;
2. create or select a context;
3. choose the stored Rider-Waite-Smith Major Arcana field;
4. choose `Calculated` or `Cast` and make one reading;
5. inspect the selected card, prompt, and complete workings;
6. save a reflection;
7. close the process, reopen the same Redb store, and recover the exact
   session, receipt, and reflection.

That is the phase gate. Three-card spreads, astrology, Pattern projections,
personal sync, resident routing, and game clients remain behind it.

## Current boundary map

| Concern | Existing authority | Missing product seam |
| --- | --- | --- |
| Context and field truth | `ContextSnapshot`, `Field`, and digest-addressed facets in `src/host.rs` | Typed catalog queries and a manual-context command |
| Reading | `ReadingEngine::{calculate, cast}` with a replayable `Receipt` | One local application command that records a session and persists it atomically from the user's perspective |
| Tarot | `TarotPack::rws_major_arcana()` and its contextual or uniform fields | A visible stored-field choice; the headed slice uses the contextual field so both selection modes are valid |
| Session and reflection | `ReadingSession`, `Reflection`, and `CleromancyHost` insertion/replay methods | History/detail read models and reflection submission |
| Persistence | `CleromancyHost::open` and `persist` over `RedbBackend` | Save-before-success behavior and recovery in the product controller |
| Portable projection | `CleromancyApp` plus Graphshell snapshots and intents | Remains a remote/portable adapter; it is not the local UI state model |
| Remote writes | Servitor scopes and admitted Graphshell subjects | User-facing grant policy, deferred until a remote headed path is resumed |
| Native UI | Cambium controls, `GenetAppRunner`, Genet layout/render, and `genet-winit-host::SurfaceHost` | A Cleromancy-owned view, state reducer, and small winit shell |

The live native host name matters. Current Genet exposes
`genet-winit-host::SurfaceHost` and Cambium's `GenetAppRunner`.
`ServalAppRunner` survives only as a deprecated alias. This plan uses the live
Genet names. A later Serval/Mere product catalog route is a separate embedding
proof.

## Product shape

The first window has three concrete regions:

- **Consultation:** context picker/editor, stored field, selection mode, and
  the `Read` action.
- **Reading:** card title and prompt, followed by a collapsed `Workings`
  section containing the algorithm, context digest, field digest, qualified
  weights, total weight, bounded sample when present, and selected index.
- **Journal:** reflection editor plus recent sessions. Selecting a session
  restores its context label, field, reading, workings, and reflections.

`Calculated` is described as the highest disclosed qualified weight. `Cast`
is described as a fresh choice from OS cryptographic randomness. The UI calls
neither source "pure entropy." A future `Derived` mode will be described as
reproducible public pseudorandomness and gets its own receipt schema.

The first context editor is deliberately bounded:

- label;
- question, stored as the `question` fact;
- comma-separated tags, normalized into the existing ordered tag set.

Arbitrary fact editing follows after the vertical proof. This keeps the first
form ordinary without creating a second context schema.

## Ownership and runtime

### Cleromancy owns

- the context draft and validation language;
- stored tarot-pack installation and labels;
- the distinction among selection mode, field qualification, and layout;
- reading and journal view models;
- conversion of one user command into graph writes;
- persistence confirmation and user-visible errors;
- receipt explanation;
- the Cambium view and its semantic labels.

### Mere and Muniment own

- graph identity, facets, relations, and snapshots;
- Redb storage primitives;
- portable Graphshell projection when another host requests it.

### Cambium and Genet own

- retained controls and event routing;
- DOM semantics, focus, and accessibility projection;
- layout, paint, wgpu presentation, and the native window bridge.

### Servitor and Graphshell own

- authenticated remote intent admission and capability checks.

The local process is the direct owner of its private Redb store. Its UI calls
the same Cleromancy domain operations directly and does not manufacture a
Servitor grant for itself. Remote Graphshell actions continue through the
existing admitted-subject path.

## Proposed seams

### `src/host.rs`: typed reads over graph truth

Add public, replay-validating queries rather than letting UI code walk raw
facets:

```rust
pub fn contexts(&self) -> Result<Vec<ContextSnapshot>, HostError>;
pub fn fields(&self) -> Result<Vec<Field>, HostError>;
pub fn sessions(&self) -> Result<Vec<ReadingSession>, HostError>;
pub fn context_for_digest(&self, digest: &str) -> Result<ContextSnapshot, HostError>;
pub fn field_for_digest(&self, digest: &str) -> Result<Field, HostError>;
pub fn reflections_for_session(&self, id: &str) -> Result<Vec<Reflection>, HostError>;
```

Results use stable product ordering: contexts and fields by visible label then
digest, sessions and reflections newest first with digest as the final tie
break. Every returned session passes `replay_session`; malformed stored facets
surface as errors instead of disappearing from the journal.

### `src/consultation.rs`: local application controller

Introduce a thin `Consultation<B>` over `CleromancyHost<B>`. It owns product
transactions, not rendering.

Its first commands are:

```rust
pub async fn install_builtin_tarot(&mut self) -> Result<String, ConsultationError>;
pub async fn save_context(&mut self, draft: ContextDraft) -> Result<String, ConsultationError>;
pub async fn read(
    &mut self,
    context_digest: &str,
    field_digest: &str,
    mode: SelectionMode,
) -> Result<ConsultationDetail, ConsultationError>;
pub async fn reflect(
    &mut self,
    session_id: &str,
    body: String,
) -> Result<ConsultationDetail, ConsultationError>;
pub fn catalog(&self) -> Result<ConsultationCatalog, ConsultationError>;
pub fn detail(&self, session_id: &str) -> Result<ConsultationDetail, ConsultationError>;
```

The exact signatures may change to fit `Backend`; the boundary must remain.
The controller resolves stored values by digest, calls `ReadingEngine`, records
one `ReadingSession`, and persists before returning success. Reflection writes
follow the same rule. A persistence failure remains visible as a failed
command even though the in-memory host became dirty. The native worker reopens
the Redb authority before accepting another write or displaying a recovered
catalog. This behavior needs an explicit test rather than a success message
based only on graph mutation.

`ConsultationCatalog` and `ConsultationDetail` are product read models. They
may carry complete domain values, but they never become a parallel store.

### `src/ui/`: retained product surface

Use four files with narrow roles:

- `state.rs`: `ConsultationUi`, text inputs, selected digests, current screen,
  status, and `ConsultationAction`;
- `view.rs`: pure Cambium view construction from `ConsultationUi`;
- `worker.rs`: the Redb-backed `Consultation` owner, command channel, and
  result delivery through a winit event-loop proxy;
- `native.rs`: winit events, Genet layout/paint, accessibility, and
  `genet_probe::Automatable` wiring.

Use existing Cambium controls: `text_field`/`textarea`, `select` or
`radio_group`, `button`, `detail_panel`, `disclosure`, and `sectioned_list`.
The product sheet lives in Cleromancy and preserves semantic roles and visible
focus. Cambium dispatch produces product actions; the native shell sends
storage commands to the worker and rebuilds from returned read models. Redb
work therefore stays outside view construction and the window thread.

### `src/main.rs`: the product executable

At the headed gate, `cleromancy` becomes the native application. Preserve the
static A0 receipt writer as a named proof binary, such as
`cleromancy-receipt`, so old receipts stay reproducible without making the
ordinary launch write fixture readings.

`CLEROMANCY_ROOT` remains the testable local-data override. A normal first
launch installs only the built-in contextual Major Arcana field. It does not
create fixture contexts, readings, sessions, or reflections.

## First vertical slice

### H0. Query and transaction proof

**Files:** `src/host.rs`, `src/consultation.rs`, `src/lib.rs`,
`tests/headed_consultation.rs`.

**Implemented:** 2026-08-08. The controller exposes only stored-digest
selection, persists each successful command, and faults after a storage error.
The isolated Redb receipt passes 3/3 under default and all features. The full
default test suite also passes. Parallel all-features compilation reproduced
the checkout's missing-rlib failure; `-j 1` removes that failure for the H0
receipt.

Implement the typed graph queries and controller. Use injected entropy and
timestamps in tests, matching the existing domain proof style. The Redb test
authors one manual context, installs the contextual Major Arcana field, makes
a cast, saves a reflection, persists, drops the host, reopens the same file,
and compares every recovered domain value byte-for-byte after serialization.
It also replays the recovered reading.

**Done when:**

- the controller never accepts an inline field for this flow;
- both `Calculated` and `Cast` work against the stored contextual tarot field;
- failed context, mode, field, session, and reflection inputs leave no
  persisted success receipt;
- close/reopen restores the same session ID, reading ID, receipt, and
  reflection ID;
- `cargo test --test headed_consultation --offline` passes with a
  Cleromancy-specific target directory.

### H1. Retained consultation surface

**Files:** `src/ui/state.rs`, `src/ui/view.rs`, `src/ui/mod.rs`, `Cargo.toml`,
`tests/headed_consultation_dom.rs`.

**Implemented:** 2026-08-08. `ConsultationUi` retains the bounded form,
catalog selections, current detail, disclosure, status, and visible error while
emitting storage-neutral `ConsultationAction`s for the H2 worker. The pure
Cambium view exposes the three semantic regions and controller-derived reading,
receipt, reflection, and session values. The headless real-dispatch receipt
passes 1/1 under default and all features, including Tab traversal, validation,
typing, stored field and mode selection, Workings disclosure, reflection save,
and stable session/reflection keys. The full default test suite also passes.
Native window, GPU, and worker wiring remain H2.

Build the three-region surface over `GenetAppRunner`. A DOM-level test uses the
real Cambium dispatch path to enter a context, choose the stored field and
mode, activate `Read`, open `Workings`, enter a reflection, and activate
`Save reflection`.

The test asserts semantic roles, focus traversal, visible validation errors,
and stable selectors such as `data-key=session:<digest>`. It also asserts that
the result title and receipt labels come from the controller response rather
than fixture-only view state.

**Done when:** the complete interaction passes without a native window or GPU,
and all controls have visible labels plus accessible names.

### H2. Native Genet host

**Files:** `src/ui/worker.rs`, `src/ui/native.rs`, `src/main.rs`,
`src/bin/cleromancy_receipt.rs`, `Cargo.toml`.

Adopt the current Hocket/Woodshed host path:

```text
Cambium view -> GenetAppRunner -> ScriptedDom -> IncrementalLayout
             -> paint list -> netrender scene -> SurfaceHost -> wgpu window
```

Keep this as a small Cleromancy-owned shell. A generic app-host extraction
needs a separate proof converting an existing consumer; it is not a
prerequisite for this slice.

The window starts hidden, installs its initial accessibility tree, then shows.
Mouse, keyboard, IME, wheel, resize, scale-factor, and accessibility actions
route through the same retained DOM paths used by the test.

**Done when:** a Windows run completes the ordinary consultation with real
keyboard and pointer input, the window remains responsive during saves, and a
captured frame shows Consultation, Reading, Workings, and Journal states.

### H3. Headed close/reopen receipt

**Files:** `src/ui/scenario.rs`, `receipts/headed-tarot.scn`, and a small
PowerShell harness under `receipts/` if process relaunch is required.

Wire `genet_probe` to the one Cleromancy surface. Run the headed application
twice against a temporary `CLEROMANCY_ROOT`:

1. the first process authors and saves the consultation and reflection;
2. the second process selects the recovered session and captures its detail;
3. the harness compares the typed IDs reported by both runs and checks the
   persisted Redb reopening receipt.

The scenario receipt proves semantic headed interaction and presented pixels.
The H0 Redb test proves persistence and replay. The Windows keyboard pass
proves real OS text input. Keep these evidence claims separate.

**Phase done when:** all three receipts are present and the normal
`cleromancy` launch reaches the local consultation instead of the static A0
HTML writer.

## Work after the first proof

Proceed in this order, one consumer-backed gate at a time:

1. **Journal depth:** three-card sessions, context reuse, arbitrary fact
   editing, receipt comparisons, and immutable follow-up reflections.
2. **Selection contract:** add `Derived` with a disclosed seed, domain, and
   replay vectors before publishing a game-facing API.
3. **Second consumer:** integrate Isometry through the smallest request/result
   contract that survives a real game call and remove any redundant adapter.
4. **Astrology:** select an ephemeris and license, then prove golden chart
   vectors with source/version/time/location metadata.
5. **Pattern:** project lived sessions, facts, reflections, and concurrence
   through Scenograph. Statistical views disclose method, sample size, and
   missing data; concurrence retains its narrower meaning.
6. **Private sync controls:** render the existing Cleromancy sync selection
   after the shared `SettingsProvider` renderer has two converted consumers.
7. **Resident catalog:** integrate the installed product route generically
   with Knot and Cleromancy, retaining Personae and Graphshell authority.

## Gates and stop rules

- The first phase remains single-card, upright Major Arcana with the existing
  original prompts.
- The contextual tarot field is stored as graph truth before use; each reading
  retains its exact field digest.
- UI state contains drafts and selections. Durable readings, sessions, and
  reflections live only in the Mere graph.
- Graphshell remains the portable projection adapter. The local window does
  not resnapshot itself through client tokens.
- Servitor continues to gate remote writes. Local product ownership is not
  represented by an artificial grant.
- The default data root is private and local. Personal sync stays opt-in and
  off until its later UI gate.
- Derived selection, game APIs, ephemeris work, Pattern statistics, and the
  installed-host route begin only after H3.
- Generated screenshots, scenario output, Redb stores, and target directories
  remain on disk and outside Git. Curated textual receipts may be committed.
- Run aggregate features from an isolated target directory before claiming
  the full wall green; compatibility-path results count as regression
  evidence, not headed implementation proof.

## Verification wall

Use one Cleromancy-owned target directory to avoid the shared-target failures
seen during A24/A25:

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cleromancy-headed-target'
cargo test --test headed_consultation --offline
cargo test --test headed_consultation_dom --offline
cargo test --all-features --offline
cargo run --bin cleromancy --offline
```

The final command is an interactive Windows receipt. Record the exact headed
scenario command and generated artifact paths when H3 lands rather than
pretending the current static HTML executable satisfies this wall.
