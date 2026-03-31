# Ticket: Merge preview systems to fix editor rotate double-despawn

**Bug:** `docs/bugs/editor-rotate-double-despawn/bug.md`
**Requirements:** `docs/bugs/editor-rotate-double-despawn/references/requirements.md`
**Architecture:** `docs/bugs/editor-rotate-double-despawn/references/architecture.md`

---

## Story

As a map editor user, I want to rotate voxels without seeing "entity already despawned"
warnings in the terminal, so that the editor is clean and the preview visuals work
correctly.

---

## Description

Two systems — `render_transform_preview` and `render_rotation_preview` — both clean up
`TransformPreview` entities and run unordered in the same frame during rotate mode.
Merging them into a single system eliminates the shared-query race at its root.

---

## Acceptance Criteria

- **AC-1:** No `WARN bevy_ecs::error::handler: Entity despawned` messages appear when
  rotating voxels in the editor.
- **AC-2:** Move preview (coarse cubes, green/red) is visually unchanged.
- **AC-3:** Rotate preview (sub-voxel geometry, blue/red) is visually unchanged.
- **AC-4:** `render_rotation_preview` no longer exists as a public export or registered
  system.
- **AC-5:** `cargo test` passes with no new failures.
- **AC-6:** `cargo clippy` reports no new warnings or errors.

---

## Non-Functional Requirements

- The merged function body must not exceed ~120 lines (per file-size guidelines).
- No new Bevy `App` test infrastructure; tests target pure helper functions only.

---

## Tasks

### Task 1 — Rewrite `preview.rs`

In `src/editor/tools/selection_tool/preview.rs`:

- Replace the two functions (`render_transform_preview` and `render_rotation_preview`)
  with a single `render_transform_preview` that follows the template in
  `architecture.md` Appendix D.
- The function must have:
  - One `mode == None` early-return + cleanup path at the top.
  - One `is_changed()` guard.
  - One `existing_previews` cleanup loop before any spawn.
  - A `match active_transform.mode` with `Move` and `Rotate` arms.
- `rotate_position()` is unchanged; keep it in the same file.
- Remove the `render_rotation_preview` function entirely.

### Task 2 — Update `selection_tool/mod.rs`

Remove `render_rotation_preview` from the `pub use preview::{...}` line.

### Task 3 — Update `tools/mod.rs`

Remove `render_rotation_preview` from the `pub use selection_tool::{...}` block.

### Task 4 — Update `main.rs`

Remove the line:
```rust
.add_systems(Update, tools::render_rotation_preview)
```

### Task 5 — Validate

Run:
```bash
cargo test
cargo clippy
```

Fix all failures and new warnings before committing.
