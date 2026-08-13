# Cleromancy: headed journal depth

**Date:** 2026-08-08
**Scope:** deepen the existing local consultation window without enlarging the
reading model or creating a spread language.

## Product boundary

The headed product already owns a local Redb store, a durable reading session,
and append-only reflections. This slice makes that durable material usable in
the journal:

1. A new context can carry disclosed additional facts as `name: value` lines.
   The existing `question` remains required and cannot be replaced through the
   additional-facts control. Editing creates a new immutable context snapshot;
   selecting a stored context reuses it unchanged.
2. A consultation can make either its existing single reading or the one
   authored A8 three-card cast. The latter is always a cast and uses the saved
   `foundation`, `tension`, and `next_step` placements. It is not a configurable
   spread editor.
3. The journal can compare the receipts of two saved sessions. The comparison
   is a local projection of immutable records: it reports shared or different
   context/field bindings, modes, algorithms, placements, and selected cards.
   It does not infer meaning or overwrite either receipt.
4. Follow-up reflections remain separate immutable records. A second note is
   appended and both notes remain visible after reopening the session.

## Ownership and seams

| Seam | Change | Explicitly not changed |
| --- | --- | --- |
| `src/consultation.rs` | parse and validate additional facts; create three-card sessions; derive receipt comparison | host graph truth, A8 spread identity, reflection identity |
| `src/ui/state.rs` | represent layout, additional-facts text, and comparison selection/action | direct Redb access |
| `src/ui/worker.rs` | execute controller commands and return comparison projection | UI-thread persistence |
| `src/ui/view.rs` | expose facts, fixed layout, all reading cards, append-only notes, and comparison | a generic spread or interpretation UI |
| `src/ui/native.rs` | apply the new worker result | a second host process or probe shortcut |
| `tests/journal_depth.rs` | controller and reopen receipts | network or sync proof |
| `tests/headed_consultation_dom.rs` | retained UI action and rendering proof | OS input proof |

## Acceptance receipts

1. Additional fact lines are canonicalized into a snapshot, reject malformed,
   duplicate, or reserved names, and do not mutate a previously stored context.
2. One three-card command saves exactly three cast readings in A8's fixed
   placements and reopens to identical detail bytes.
3. A comparison of two saved sessions reports only their durable receipt
   differences and survives reopening because it is recomputed from graph
   truth.
4. Two follow-up reflections have distinct identities, remain ordered newest
   first, and are both visible when their session is reopened.
5. The retained DOM exposes the added controls and rendered three-card
   placements without needing a live native window.

## Stop rule

Stop after the local controller, worker, retained UI, and these receipts pass.
Do not add a configurable spread DSL, astrology inputs, generated
interpretation, sync changes, sharing, or a second headed scenario in this
slice.
