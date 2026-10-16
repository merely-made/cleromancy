# Published Dramatis main intake, 2026-10-16

Cleromancy's app and portable-core manifests already track the canonical
Dramatis main URL. Cargo aligns both lockfiles to published main
2f27f1a939aa2e046b78f7db18dcd8f81e0d7c59. The app formerly locked 5c89001;
the portable core formerly locked 814ffa0. Both resolved graphs now contain
one Dramatis revision. No import or product source changes were necessary.

Cargo update changes no package versions, package set, other Git revisions
or source selectors. In the app lock it also re-resolves existing registry
edges (syn, constant_time_eq and windows-sys); these are retained as Cargo
generated them and covered by the scoped gates. The core lock changes only
the two Dramatis source revisions. Original lockfiles and update logs remain
in this pack. Metadata's initial offline unfiltered attempt lacked an already
locked antithesis_sdk cache entry; subsequent locked platform-filtered metadata
succeeds. That cache diagnostic is preserved; no dependency version changed.

Checks bind committed source in source-inputs.json: default and expanded
product suites, four all-target variants, three portable-core suites and the
desktop build. See checks.json for exact commands/counts. Cargo 1.99 runs
explicit rustc/rustdoc 1.97.1 with four build jobs. Metadata summaries bind the
full root-platform and core graphs. All prior receipt paths in prior-artifacts.json
remain byte-identical, and the existing C7/C8 frozen-evidence verifiers pass.

This is source/software/build compatibility, not renewed native qualification.
No public fixture was opened by a desktop run for this intake. C7 and C8 native
captures, fresh-process reopenings and exact binaries retain their earlier
sources. The separate local Dramatis subject-reference proposal is unpublished
and excluded from this supplier revision; no natal API, birth-fact storage or
natal reading is adopted by this dependency update.
