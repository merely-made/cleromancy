# macOS arm64 candidate packaging

This tool creates an installable `.app` and ZIP from an **existing** arm64
Cleromancy executable. It does not build or launch applications. Output defaults to ad hoc signing; an available Developer ID Application identity
can be selected explicitly. No notarization or upload of the app is performed. It supports local development
candidate handoff; it does not establish a consumer-ready release.

**Before installation/use:** provision the resident and wallet persona through
the owning family workflow. The shared current authority must understand the new
aspect records/portable validator before using a binary that writes them. The
parent owns that authority intake and qualification separately. An app archive
alone does not upgrade a running resident or establish compatible admission.

## Create a candidate

Run from the repository root on macOS with Python 3.9+, Apple command-line tools
(`codesign`, `otool`, `lipo`, `plutil`, `ditto`), and the source dependencies already
available. Cargo is needed only to export the license inventory metadata.

```sh
cargo metadata --locked --offline --filter-platform aarch64-apple-darwin \
  --features analytic-ephemeris,sky-timeline --format-version 1 \
  > /tmp/cleromancy-macos-metadata.json

python3 packaging/macos/package.py \
  --binary /absolute/path/to/qualified/cleromancy \
  --metadata /tmp/cleromancy-macos-metadata.json \
  --source-revision FULL_40_HEX_BINARY_SOURCE_REVISION \
  --source-evidence 'Describe the build receipt and feature set, with no personal data' \
  --profile debug \
  --app-name 'Cleromancy Development Candidate' \
  --bundle-id dev.merely.cleromancy.development.UNIQUE_RUN_ID \
  --output /absolute/path/to/new-output-directory
```

Use the **binary's** build revision, feature set, and profile. `--profile` is an
operator declaration, not inferred from a file's name. Keep its build/qualification
receipt outside the bundle and compare the receipt's SHA-256 with the input hash
recorded in `verification.json`. Debug executables may contain debug information,
build paths and public embedded scenarios. Packaging does not strip or rebuild them.
For a later release-profile build, the parent must supply a newly qualified binary
and its evidence. Never relabel a debug binary as release.

The bundle's name and identifier must be distinct from earlier qualification
bundles, especially the historical bundles sharing the v4 ID. The defaults are
`Cleromancy Packaging Candidate` and
`dev.merely.cleromancy.packaging.candidate20261017`; override both for a new native
qualification. The payload `cleromancy-bin` and launcher `Cleromancy` have distinct
names even on a case-insensitive filesystem.

Output must be a **new directory**. Existing output/receipt paths are never replaced.
Failures leave partial output for diagnosis; choose another output path for retry.
The output contains the app, ZIP, `SHA256SUMS`, and `verification.json`.

## Runtime and first-run boundary

The fonts, CSS, authored card/position content and analytical astronomy code/data
are compiled into the executable. No external asset tree or downloaded kernel is
copied. The packager copies only declared binaries, legal texts and generated
metadata/install instructions. It never visits a persona/vault/reservoir directory,
copies fixture stores, or reads environment secrets.

The pinned source reviewed for this tooling uses
`djinn::identity_ui::SystemNativeIdentityUi`, backed by `light-file-dialog` 3.21.3.
Its macOS provider invokes `osascript` through `sh`. The launcher prepends its own
directory and `/usr/bin:/bin:/usr/sbin:/sbin` to PATH. Both OS providers must exist
in an interactive desktop session. No runtime Swift interpreter is required by
that pinned path. Packaging cannot prove that its dialog can be targeted through
automation, or that graphical unlock succeeds. Parent owns that qualification.

An optional **compiled** helper can be included using
`--helper helper-name=/absolute/path/to/helper`. It must be thin macOS arm64 and
link only Apple system libraries. It is individually signed, hashed, and placed
beside the executable on PATH. This does not change runtime source or select a
new provider: the parent must establish the runtime's helper lookup/selection
contract before claiming the helper is used. Arbitrary scripts, Swift sources,
model weights and library trees are not accepted as helper binaries.

Normal startup attaches to an existing resident when available; the embedded
fallback resolves an existing wallet persona before opening its vault. See the
pinned Mere `ports/djinn/src/embedded_reservoir.rs`, `ReservoirAttachment::open`,
and the app's `src/ui/worker.rs`. Packaging supplies **no persona provisioning**
wizard, persona, passphrase or sample history. A fresh machine without the family
identity setup is not qualified to start successfully. Provision a real identity
through the owning resident's supported workflow before using the app. This guide
does not invent a product CLI for provisioning. A public synthetic identity is
for isolated qualification only and must never ship in this archive.

`MERE_ROOT`, `CLEROMANCY_VAULT_DIR` and `CLEROMANCY_ROOT` retain their existing
application meanings. Finder does not inherit interactive shell exports. Do not
hard-code qualification paths or secrets into the launcher. Optional local
inference still requires a separately configured compatible model; no model ships.

The minimum OS is derived from the executable's Mach-O deployment target (taking
the highest requirement across helpers). It is a loader requirement, not evidence
of testing on older macOS versions. The candidate is arm64 only.

## License inclusion and distribution limits

The app includes Cleromancy's MIT/Apache texts, both embedded font OFL texts,
`THIRD_PARTY_LICENSES.md`, available legal files from Cargo package roots and
inherited Git workspace roots, and `dependency-inventory.json` listing package
versions, source identities, license expressions and copied text locations.
An allowlist restricts copying to legal filenames and files inside LICENSES
directories. No dependency source trees or local manifest paths are shipped.

Cargo metadata may contain development, build, optional or target-inapplicable
packages; this is a conservative inventory, not a linked-code census. Export it
with the same lockfile/features as the binary's build. Reusing newer metadata for
an older qualified binary is only a tooling proof and must be stated explicitly
in `--source-evidence`. Metadata SHA-256 is retained in the manifest.

Some upstream Cargo packages omit full license texts. Missing sets are reported
by name/version in the bundle manifest and `verification.json`; a passing structural
check does **not** waive these gaps. Before public distribution, resolve applicable
missing notices, verify exact dependency closure and source availability (including
MPL corresponding-source obligations), and review embedded content/model terms.
This script does not certify distribution compliance or provide a source archive.

## Verification and reproducibility

```sh
python3 packaging/macos/package.py --verify '/path/to/Cleromancy Development Candidate.app'
shasum -a 256 -c /path/to/output/SHA256SUMS # run in the output directory
python3 -B -m unittest discover -s packaging/macos -p 'test_*.py' -v
```

Packaging refuses a non-arm64 binary or any non-system dynamic dependency. It
checks the copied input hash, derives the OS floor, signs each binary and bundle,
validates the plist, checks strict/deep signatures, and checks every allowlisted
file's SHA-256 and mode. After creating the ZIP it extracts with `ditto` to a
temporary directory and repeats bundle verification, including executable modes.
ZIP entries are sorted and have fixed timestamps/modes, with no resource forks or
host xattrs. Repeated packaging with identical inputs, metadata, source description,
tooling and Apple/zlib versions should produce the same ZIP hash. This is packaging
reproducibility; no reproducible compilation claim is made. Across signing/compression
toolchain versions compare the manifests instead of assuming byte identity.

The standalone verifier establishes self-consistency; it is not a trust signature
against a maliciously replaced manifest. Compare the archive hash with a trusted
`SHA256SUMS` receipt. Bundles default to ad hoc signing, or use an explicitly
selected Developer ID Application identity. They are **not notarized**.
Gatekeeper may block downloaded copies. No quarantine removal, security
policy changes, publishing, committing or pushing is performed.

No app execution occurs in these checks. Graphical unlock, reading behavior,
fresh-process reopening, installation in Applications, fresh-machine provisioning,
and native acceptance of the final binary remain separate parent-owned claims.

## Developer ID candidate

Use `--signing-identity FULL_CERTIFICATE_FINGERPRINT` to select an existing local
Developer ID Application identity. The tool requests Apple's trusted timestamp
and enables hardened runtime for payloads and bundle. It records and rechecks
authorities, team, timestamp and runtime flags, including after extraction.
An Apple Development certificate is refused for this mode. Keychain access must
already permit codesign or be explicitly approved by the user; the tool does not
alter key access or supply a keychain password. No notarization request is made.

Timestamped signatures change archive bytes between runs; the reproducibility
witness above applies to ad hoc timestamp-free packaging. The original ad hoc
qualification and later Developer ID candidate retain separate hashes and native
checks. Signing does not turn a debug-profile binary into a release-profile build.
