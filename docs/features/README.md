# Open Feature Tickets

All open tickets are listed below. Each entry links to its `ticket.md`.
Tickets are grouped by category and sorted by severity within each group.

To close a ticket: implement it, verify all tests pass, then delete the entire feature directory.

---

## Bugs

| Slug | Title | Severity |
|------|-------|----------|
| [bug-npc-push-arbitrary-direction](bug-npc-push-arbitrary-direction/ticket.md) | NPC overlap fallback always pushes player in the +X direction | Medium |
| [bug-rename-index-shift-after-delete](bug-rename-index-shift-after-delete/ticket.md) | Outliner rename mode targets the wrong entity after a lower-index deletion | Medium |
| [bug-redo-ignores-history-cap](bug-redo-ignores-history-cap/ticket.md) | `EditorHistory::redo()` does not enforce the maximum history cap | Low |
| [bug-save-as-dirty-flag-desync](bug-save-as-dirty-flag-desync/ticket.md) | Save As clears the dirty flag for edits made while the dialog was open | Low |

## Performance

| Slug | Title | Severity |
|------|-------|----------|
| [perf-outliner-per-frame-allocations](perf-outliner-per-frame-allocations/ticket.md) | Eliminate per-frame allocations in the outliner panel | Medium |

## Tests

| Slug | Title | Severity |
|------|-------|----------|
| [test-editor-history-coverage](test-editor-history-coverage/ticket.md) | Improve `EditorHistory` unit test coverage | Medium |
| [test-collision-physics-coverage](test-collision-physics-coverage/ticket.md) | Expand collision and physics unit test coverage | Medium |
| [test-map-validation-coverage](test-map-validation-coverage/ticket.md) | Expand map validation unit test coverage | Low |

## Code Quality

| Slug | Title | Severity |
|------|-------|----------|
| [cq-dead-code-api-cleanup](cq-dead-code-api-cleanup/ticket.md) | Remove dead code and tighten the public API surface | Low |
| [cq-duplicate-map-path-resource](cq-duplicate-map-path-resource/ticket.md) | Eliminate duplicate map-path Bevy resources | Low |
| [cq-document-magic-constants](cq-document-magic-constants/ticket.md) | Replace magic numbers with named constants and semantic types | Low |

## Architecture

| Slug | Title | Severity |
|------|-------|----------|
| [arch-map-mutation-centralization](arch-map-mutation-centralization/ticket.md) | Centralize all map mutations through a single edit path | Low |
