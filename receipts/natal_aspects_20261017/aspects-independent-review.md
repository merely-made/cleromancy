# Natal aspects independent review — 2026-10-17

Reviewed working-tree change over HEAD `8472bc8dca53c2646baea0f864db9d1d0578fd61` (same as locally recorded origin/main; no remote refresh). Source review only: no edits to repository files, compiles, test executions, commits, or receipt rewrites. This report is the sole write. Concurrent work was present; hashes below identify inspected files.

## Actionable findings

1. **P2 — Expose the aspect edition selection as control state. Owner: primary/UI.** `src/ui/view/chart/natal.rs:190–210` renders mutually exclusive choices as ordinary buttons with changing “Selected:” text, but no `aria-pressed`, checked state, or labelled choice group. Assistive technology receives names but cannot query which control is selected as a state; a name change alone does not provide reliable selection feedback. Add complementary `aria-pressed` values with stable labels and a named group, or use the existing accessible radio/choice control. Check keyboard activation and selected-state announcements in both directions. Existing product controls use `aria-pressed` (e.g. `src/ui/view/reading_scene.rs:85–91`). Native accessibility behavior was not exercised.

2. **P3 — Blank orb is reported as negative, with no offending field. Owner: primary/UI.** `src/ui/view/chart/natal.rs:753–756`: `chart_input::degrees("", 3)` returns an empty String; the subsequent u32 parse fails and maps this to “Aspect orbs cannot be negative”. Clearing any one of five inputs therefore produces incorrect guidance, and generic parse/range errors do not identify that input. Preserve exact String-to-u32 conversion, but validate blank separately and prefix errors with the aspect name and user-facing degree range/precision. Check blank, negative, excessive precision, and >180° for each field; keep zero valid. The existing input test checks rejection only, so it does not catch misleading messages.

## Domain, receipts, replay, and policy

No correctness or receipt/replay/identity blocker found in the inspected change. Integer shortest-longitude separation is safe after chart validation; inclusive bounds, exact-only zero, multiple overlapping kinds, lexical pair ordering, and angular kind ordering agree with the explicit rules. Results use retained millidegree positions, as disclosed, without motion/house/luminary adjustments.

`NatalReading.aspects` defaults to None and is omitted from serialization when absent (`src/natal.rs:72–73`), preserving the prior serialized identity shape. `with_aspects` reseals the new value; validation compares the complete recalculated extension, including method/schema, rules, chart digest, hits/order, and unavailable reason (`src/natal.rs:124–168`). Host admission and loaded records validate. The resident path creates a fresh nonce/occasion, attaches the selected rules before insertion, and does not update old occasion records (`src/ui/resident.rs:378–403`). No receipt files appear in the reviewed tracked diff.

The UI defaults aspects off (`src/ui/state.rs:201–202`), exposes five explicit inputs when enabled, and correctly parses the degree helper's String with `.parse::<u32>()`. The approved preset is disclosed as chosen geometric rules. Earlier authored prompts still derive from the retained original 1° edition; explicit aspects supply geometric facts only. The narration distinguishes the original edition, so this review does not treat preserved prompts as a replay defect.

Tests inspected cover five inclusive boundaries, wraparound, zero/custom/wide rules, stable ordering, invalid rules/chart inputs, extension roundtrip, resealed tampering, admission, and unavailable results. Existing historical receipt replay test remains at `tests/natal.rs:153–163`. These are source coverage observations, not independently executed passes.

## Acceptance boundary

Native layout, clipping, focus retention, accessibility announcements, and fresh-process reopen remain unverified here. The primary reports graphical unlock blocked by unbundled osascript and user assistance requested; that is reported context, not independently observed evidence. This review does not qualify native release acceptance. No domain fix requested; route the two findings to the primary UI owner.

## Inspected source SHA-256

- `src/natal_aspects.rs`: `5b0d358460012a4cd2aa07acbb8a24a9b9300c0995f087f7cd4a5d0488ede9ee`
- `src/natal.rs`: `48910dbe69cdb77d930797879ed737584630570d1c53a73d7ae5db133335f59e`
- `src/ui/view/chart/natal.rs`: `087bc4638e688a210a5cd6f0cf8d4a0765175f72e35976516e39bc379ae0ab2d`
- `src/ui/action.rs`: `224e3991af2fc1ae1f457851c2202bf63be93cae28a965840b56dd85b82788c1`
- `src/ui/resident.rs`: `2c28853e9148979bac5685e7ba31d174a5f9d8575a9daed592fd2d321cb2c402`
- `src/ui/state.rs`: `ca76c9851b39f3db6f352b16a0524ffde6c0ef689ec6e9014d26b5c5db1d6849`
- `tests/natal.rs`: `485b269fe1d1c5a21348ef546252cfd014b0b358582bd6ed3e040d83dc2ae8e2`

## Focused follow-up — 2026-10-17

Rechecked only the current selection controls and orb-input validation in `src/ui/view/chart/natal.rs`; no broad review repeated. Both original findings are resolved at source level:

- **Finding 1 (P2): closed.** Lines 190–211 use stable button labels, complementary `aria-pressed` values derived from the same boolean, and a `role="group"` container named “Natal aspect edition”. Both activation handlers set the intended state. This addresses the missing programmatic selection state.
- **Finding 2 (P3): closed.** Lines 751–776 check trimmed blank input before conversion, provide a blank-specific message mentioning exact-match zero, prefix decimal-parser errors with the aspect name, identify negative inputs by aspect, and report the 180-degree maximum in user-facing units. Exact String-to-u32 conversion remains intact; zero is accepted. The index-to-label mapping matches the five input fields.

No additional actionable defect found in these exact snippets. Existing tests still assert rejection rather than message text; that observation does not prevent closure by source inspection. No compiles, tests, native interaction, repository edits, or receipt changes performed. Keyboard/accessibility announcements and other native acceptance boundaries remain unverified. The original review and hashes above are retained as historical evidence.

Follow-up source SHA-256 (`src/ui/view/chart/natal.rs`): `b5df1cd9ad43737bdc892e434044547035d2afbcb6da18330d7487e531ae122b`.
