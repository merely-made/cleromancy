# Packaging tooling verification, 2026-10-17

Tooling ready for parent handoff. No application/source logic was changed by
this work, no app was opened, no expensive build was run, and nothing was committed
or pushed. New repository paths are confined to `packaging/macos/`. Parent's
concurrent source changes are outside this worker's ownership.

## Inputs and scope

Repository inspection began at `8472bc8dca53c2646baea0f864db9d1d0578fd61`, equal
to the local `origin/main` reference. No remote freshness claim is made. No
`AGENTS.md` was found in the repository/ancestor paths searched.

The bounded packaging input is the previously qualified native executable in
`/Users/markik/Code/testing/cleromancy/natal-cap-20261017/Cleromancy Natal Cap v1.app/Contents/MacOS/Cleromancy`.
Its SHA-256 is
`9abef3dc97cd831a6f7f1b7223d5152e637223821e5d37da151bdf522c3b1fad`, matching both
the existing `native-build.json` and `native-qualification.json` in that directory.
Those receipts identify source `da495cfc75b2ee6bbb9476c8c03386e9057c36d3`.
The input remained unchanged after packaging. This is a historical **debug-profile**
binary, not the parent's new aspect-feature binary or a release-profile build.

License inventory came from offline metadata of the packaging checkout's current
lockfile, with `--filter-platform aarch64-apple-darwin --features
analytic-ephemeris,sky-timeline`. It does not certify the historical executable's
exact dependency closure. The unfiltered offline metadata attempt failed because
`antithesis_sdk 0.3.0` was uncached; the target-filtered export succeeded without
downloads or compilation.

## Results

- Seven focused tests passed: clean allowlist, changed bytes, lost mode, extra
  store, symlink, wrong architecture, and non-system dynamic library rejection.
- `sh -n` passed on the emitted launcher.
- Actual thin arm64 payload and system-library dependency inspection passed.
- The executable's Mach-O minimum is macOS 11.0; older macOS was not tested.
- Ad hoc payload/bundle signing, strict/deep verification and plist lint passed.
- ZIP extraction with `ditto` preserved the verified bundle, hashes and modes.
- Standalone `--verify` passed on candidate-03 after packaging.
- Two independent runs, candidate-03 and candidate-04, produced byte-identical
  archives with SHA-256:
  `b1694a14736d7ae550af8bfb13568e56765aaccb0c1fed0910ed8f928d2de01e`.

The candidate name is `Cleromancy Packaging Candidate`, with fresh identifier
`dev.merely.cleromancy.packaging.candidate20261017`. Actual outputs live under
`/Users/markik/Code/testing/cleromancy/macos-packaging-20261017/`, including
candidate-03's `.app`, `Cleromancy-0.1.0-macos-arm64-debug.zip`, `SHA256SUMS`,
`verification.json`, and the parent directory's `tooling-verification.json`.

The archive includes 1,749 allowlisted files before the generated manifest/signature:
the payload/launcher, plist/install instructions, legal texts and dependency
inventory. It copies no vault, wallet, fixture store, reading history, model,
downloaded kernel, or source tree. Metadata inventories 1,164 packages, with
**138 missing full upstream license-text sets** explicitly listed in the manifest
and receipt. This conservative inventory includes packages beyond a linked-code
census. Applicable legal-text/source-availability gaps and exact binary dependency
closure remain distribution blockers.

The first candidate attempt is preserved: its outer signing failed because a
case-insensitive filesystem treated launcher `Cleromancy` and payload `cleromancy`
as the same file. Tooling now uses `cleromancy-bin` for the payload. Candidate-02
passed after that correction; candidate-03/04 include final provisioning and
authority warnings plus improved inherited LICENSES-directory collection. Earlier
outputs are preserved and are not the final tooling evidence.

## Parent-owned acceptance still required

Feed a newly built/qualified binary, exact source revision/build features, matching
Cargo metadata and an honest profile into the script. Pick a new candidate name
and identifier for graphical qualification. Provision the resident/persona first;
the shared current authority must understand the new aspect records and portable
validator. This bundle neither provisions identity nor upgrades a running resident.

The pinned macOS dialog provider needs `/bin/sh` and `/usr/bin/osascript` on PATH.
Compiled helper inclusion is supported explicitly, but no additional helper was
required or bundled for the frozen input. Parent must establish any new helper's
runtime lookup contract. No private runtime paths or secret environment values
are written by the packager. Developer tool build paths may remain in the debug
executable itself.

The deliverable is an installable local development candidate: **ad hoc signed,
without Developer ID, not notarized, and not a qualified public release**.
Gatekeeper acceptance, graphical unlock, installation into Applications, fresh
machine startup, final reading behavior, fresh-process reopening and public
redistribution compliance were not claimed or tested here.
