# Divination mere plan: receipts

Receipts for the archived plan
[2026-09-23_divination_mere_plan.md](../2026-11-01/2026-09-23_divination_mere_plan.md)
(cut into `design_docs/2026-11-01_divination_mere_plan.md` on 2026-11-01). DOC_POLICY §8 puts
run evidence here and not in the plan. This file indexes it; the evidence itself is not in
the repository.

## 2026-11-01, where the evidence lives

- Tracked receipts: `Code/testing/cleromancy/receipts/` (1,281 non-scenario files, 292,442,872
  bytes, sha256-verified against the committed blobs before removal; `Code/testing` is outside
  this repository, so these are path cites). The tabard folder is
  `Code/testing/cleromancy/tabard_20261011/` (175 files, 31,666,395 bytes).
- Scenario files (76) are source and live in `scenarios/` in the repository. The moved
  `verify_*.py`, `run-*.py` and `collect.py` scripts, and the ledgers that name
  `receipts/<name>.scn`, refer to scenarios; read those paths as `scenarios/<name>.scn`.
  Recorded scenario SHA-256 values are unchanged.
- Three replay fixtures that tests read at compile time are also in the repository, under
  `tests/fixtures/natal/` (divination D5). Their copies in the receipt tree are unchanged:
  `c8_natal_hardened-exact-readings-05.json` is `receipts/c8_natal_hardened_20261017/exact-readings-05.json`,
  `natal_houses-exact-readings.json` is `receipts/natal_houses_20261017/exact-readings.json`, and
  `developer_id_native-reopen-01-receipt.json` is `receipts/developer_id_native_20261017/reopen-01/receipt.json`.

## 2026-11-01, ledger index

Each row is a bounded qualification scope; none is a phase acceptance (see the plan's O5).
Paths are under `Code/testing/cleromancy/receipts/`.

| Scope | Ledgers and evidence notes |
| --- | --- |
| C1 resident-held mere | `c1-resident-qualification_20261014.json`, `c1-portable-core_20261014.json` |
| C2 reading narrative | `c2-reading-qualification_20261015.json`, `main-integration_20261015.json`, `family-intake_20261015.json` |
| C3 typed cards and decks | `c3-software-qualification_20261016.json`, `c3-native-qualification_20261016.json`, `c3-publication-intake_20261016.json` |
| C4 ambient card view | `c4-software-qualification_20261016.json`, `c4-native-qualification_20261016.json`, `c4-resident-intake_20261016.json`, `c4-publication_20261016.json`, `c4_20261016/README.md` |
| C5 shuffled deck | `c5-shuffled-qualification_20261016.json`, `c5-publication_20261016.json`, `c5_20261016/README.md` |
| C6 situation | `c6-situation-qualification_20261016.json`, `c6-publication_20261016.json`, `c6_20261016/README.md` |
| C7 generated inferences | `c7-generated-inferences-qualification_20261016.json`, `c7-publication_20261016.json`, `c7_20261016/README.md` |
| C8 first die | `c8-first-die-qualification_20261016.json`, `c8-die-preflight_20261016.json`, `c8-publication_20261016.json`, `c8_20261016/README.md` |
| Dramatis main intake | `dramatis-main-intake_20261016.json`, `dramatis-main-publication_20261016.json` |
| Shared natal | `c8-natal-qualification_20261017.json`, `c8-natal-hardened-intake_20261017.json` |
| Houses and ascendant | `natal-houses-qualification_20261017.json`, `natal_houses_20261017/README.md` |
| Natal cap | `natal-cap-qualification_20261017.json`, `natal_cap_20261017/README.md` |
| Natal aspects | `natal-aspects-qualification_20261017.json`, `natal_aspects_20261017/README.md` |
| Graphical unlock | `graphical-unlock-qualification_20261017.json`, `graphical_unlock_20261017/README.md` |
| Developer ID signed native | `developer-id-native-qualification_20261017.json`, `developer_id_native_20261017/README.md` |
| Resident materialization | `resident_materialization_20261018/`, `verify_resident_materialization.py`, `duplication-reduction_20261018.json` |
| Font roles (tabard) | `Code/testing/cleromancy/tabard_20261011/font-roles/qualification.json` |

## 2026-11-01, old hashes to main hashes

Mark's ruling (divination D4): live docs cite main hashes; archived docs keep the hashes they
were written with. Each row pairs a pre-merge hash found in a document at `f86874e` with the
commit on `main` that has the identical subject (one unique match each; the 37 hashes below
include the 40-character spellings of three of them). Hashes that are already ancestors of
`main`, and hashes of other repositories (Mere, Knot, Turquet, Isocosm, Dramatis, Genet), are
not listed.

| Old | Main | Subject | Cited in |
| --- | --- | --- | --- |
| `052e54f` | `106c02f` | Bound typed spread capture to one immutable field facet and correct capture selectors | 2026-09-23_divination_mere_plan.md |
| `0f775a0` | `adc4edd` | Record portable float repair as superseded by qualified C2 intake | 2026-10-15_cleromancy_codex_handoff.md |
| `0fe3503` | `566c22f` | Qualify the first die with native creation and exact fresh reopening | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `1e2fe7f` | `bc4c3d3` | Merge qualified resident divination and C2 reading narrative | 2026-10-15_cleromancy_codex_handoff.md |
| `22df4c9` | `2997b92` | Lock the branch = main switch; hand the personal-sync source fix to the lane | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `277c6bd` | `9c1d514` | Qualify C3 native cards, crosswalk and fresh-process reopening | 2026-09-23_divination_mere_plan.md |
| `29d4f3b` | `cef2583` | Record C4 software, headed motion, exact reopen and coordinated receiving qualification | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `2e5ef2a` | `2be8699` | Add complete C7 creation and offline reopen qualification on retained context | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `300ebbd` | `bc0b0d6` | Publish C3 originals and completed resident receiving ledger | 2026-09-23_divination_mere_plan.md |
| `3d45fc5` | `6559f15` | Qualify C6 recorded situation through exact offline native reopening | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `3fa5a6d` | `881080f` | Add explicit replayable natal aspect rules and macOS candidate packaging | 2026-10-17_aspects_unlock_packaging_plan.md |
| `420bcc3` | `625c7aa` | Publish resident selection atomically with Mere command completion | 2026-10-18_duplication_reduction_plan.md |
| `5590a2a05f47b79d93d9688506be07710afa0223` | `0fe6513` | Qualify Developer ID signed native launch and reopening | 2026-10-18_duplication_reduction_plan.md |
| `7aeb758` | `0e1ead1` | Align app and portable core locks to published Dramatis main | 2026-09-23_divination_mere_plan.md |
| `7b0c5ef` | `5f85d53` | Check journal card labels against their recorded orientation | 2026-09-23_divination_mere_plan.md |
| `8472bc8` | `ade19d2` | Qualify natal motion, observed orientation and Placidus with preserved receipts | 2026-10-17_aspects_unlock_packaging_plan.md |
| `8472bc8dca53c2646baea0f864db9d1d0578fd61` | `ade19d2` | Qualify natal motion, observed orientation and Placidus with preserved receipts | VERIFICATION-20261017.md |
| `913b667` | `a9144f2` | Support verified Developer ID packaging with explicit local signing failures | 2026-10-17_aspects_unlock_packaging_plan.md |
| `9aae258` | `11c591d` | Adapt selected personal sync to optional assertion basis | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md, DOC_README.md |
| `9d8ebda` | `155a8f3` | Hand Cleromancy's next work to Codex: land the C1 lane, follow the family repin, record P4 | 2026-10-15_cleromancy_codex_handoff.md |
| `a4364de` | `152c411` | Name the family repositories by branch = main (family source convention) | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `a4abd1b` | `0b6078d` | Add keyboard roll and compact control scenarios to the first die qualification | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `a684361` | `4880c5c` | Intake resident profile and contact admission hardening | 2026-09-23_divination_mere_plan.md |
| `adc434d` | `1481e0c` | Take the bounded cold-replay startup fix without widening the supplier intake | 2026-09-23_divination_mere_plan.md |
| `ca562db` | `73a8cc9` | Bind natal house edition to corrected immutable Turquet geometry | 2026-09-23_divination_mere_plan.md |
| `d18590f` | `d498f21` | Drop the weave merge driver from .gitattributes (file removed: weave was its only content) | 2026-10-15_cleromancy_codex_handoff.md |
| `d5c383c` | `8bb0ab0` | Carry canonical kept tags through the signed sync boundary | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `d6ea9f7` | `c034fb4` | Adopt shared ordered grids and reuse exact neighborhood validation within a candidate | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `da495cf` | `1ae2cfe` | Add sealed natal motion, observed Earth orientation and Placidus choices | 2026-09-23_divination_mere_plan.md, 2026-10-17_natal_cap_plan.md |
| `da495cfc75b2ee6bbb9476c8c03386e9057c36d3` | `1ae2cfe` | Add sealed natal motion, observed Earth orientation and Placidus choices | VERIFICATION-20261017.md |
| `dba1c183` | `cdfc632` | Record bounded C7 local inference and offline reopen qualification | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `e51964a` | `69457cd` | Guard native die submission and avoid reflecting on an older saved occasion | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `e94a47c` | `2ae1265` | Qualify C5 shuffled draws, both randomness sources and exact fresh reopening | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
| `ee97133` | `741cb29` | Record typed draw-time sky and opt-in weather with offline situation replay | 2026-09-23_divination_mere_plan.md |
| `ef014d7` | `5a87638` | Use typed deck metadata to identify tarot in the narrative | 2026-09-23_divination_mere_plan.md |
| `f6d2425` | `934f3f8` | Correct the retained receipt toggle construction | 2026-09-23_divination_mere_plan.md |
| `ff0c14d` | `736ee9b` | Qualify published Dramatis main intake without altering prior receipts | 2026-09-23_divination_mere_plan.md, 2026-10-15_cleromancy_codex_handoff.md |
