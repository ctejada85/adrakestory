# Requirements — Bevy 0.18 API Modernization

## Overview

The codebase was migrated from Bevy 0.15 to Bevy 0.18, but did not adopt several idiomatic API patterns introduced in 0.15–0.18 that simplify system signatures and reduce boilerplate. This feature brings the codebase in line with Bevy 0.18 best practices — particularly the `Single<>` system parameter — reducing error-handling ceremony and making system intent clearer.

**Primary scope:** Replace `Query::single()` / `Query::single_mut()` call patterns with the `Single<>` (or `Option<Single<>>`) system parameter type.  
**Out of scope:** `#[require()]` on co-spawned components — investigated and rejected (see OQ-1).

---

## Analysis Summary

The codebase is already well-aligned with Bevy 0.18 for most patterns:

| Pattern | Status |
|---|---|
| `Camera2d` / `Camera3d` components | ✅ Already modern |
| `OnEnter` / `OnExit` state systems | ✅ Already modern |
| `ChildOf` parent-child API | ✅ Already modern |
| `CascadeShadowConfigBuilder` | ✅ Already modern |
| `SystemParam` derive | ✅ Already modern |
| `MessageReader` / `MessageWriter` events | ✅ Already modern |
| `Query::single()` → `Single<>` | ❌ Not yet adopted — 30+ call sites |
| `#[require()]` on always-co-spawned components | ✅ Investigated — not applicable (see OQ-1) |

---

## Data Domains

### Single-Entity Query Sites
Systems that query for exactly one well-known game entity: player, game camera, primary window. These systems currently use `if let Ok(x) = query.single()` or `let Ok(x) = query.single() else { return; }`, which adds noise without conveying intent.

### Component Co-Spawning Data Domain — Not Applicable
`VoxelChunk` + `ChunkLOD` are always spawned together. `#[require(ChunkLOD)]` was investigated but rejected: every spawn already provides explicit `ChunkLOD` with real mesh handles, so the default would never activate. A co-spawn invariant comment on `VoxelChunk` is sufficient.

---

## Functional Requirements

### FR-1: Convert `Query::single()` to `Single<>` — Core Game Systems
Systems that own the single-entity contract must use `Single<>` or `Option<Single<>>` as a system parameter instead of calling `.single()` at runtime.

Covered files and systems:
- `src/systems/game/physics.rs` — `apply_gravity`, `apply_physics`, `apply_npc_collision`
- `src/systems/game/camera.rs` — `follow_player_camera`, `rotate_camera`
- `src/systems/game/player_movement.rs` — `move_player`
- `src/systems/game/input.rs` — `toggle_flashlight`, `update_flashlight_rotation`
- `src/systems/game/interior_detection.rs` — player query
- `src/systems/game/occlusion/mod.rs` — camera + player queries in update system
- `src/systems/game/map/spawner/mod.rs` — `update_chunk_lods` camera query
- `src/systems/settings/vsync.rs` — `apply_vsync_system` window query
- `src/systems/intro_animation/systems.rs` — text entity query

**Rules:**
- Use `Single<>` when there is always exactly one matching entity (player, game camera, primary window). The system will be skipped automatically if the entity is absent — this replaces the manual early return.
- Use `Option<Single<>>` when the entity may legitimately be absent (e.g., player not yet spawned, occlusion system before map load).
- Do not use `Single<>` in systems that legitimately query zero-or-many entities.

### FR-2: Convert `Query::single()` to `Single<>` — Editor Systems
Same conversion for map editor systems.

Covered files:
- `src/editor/camera.rs`
- `src/editor/cursor/mouse_cursor.rs`
- `src/editor/controller/cursor.rs`
- `src/editor/controller/camera.rs`
- `src/editor/grid/systems.rs`
- `src/editor/tools/voxel_tool/removal.rs`
- `src/editor/tools/voxel_tool/placement.rs`
- `src/editor/tools/selection_tool/selection.rs`

### FR-3: Preserve All Existing Behaviour
Every conversion must produce identical runtime behaviour. No gameplay changes, no visual changes, no performance regressions.

---

## Non-Functional Requirements

### NFR-1: Test Suite
All 355 passing tests must continue to pass after the changes. No new test failures are acceptable.

### NFR-2: Compile Warnings
`cargo clippy` must produce no new warnings after the changes.

### NFR-3: Minimal Scope
Only systems that exclusively query a single entity may be converted. Systems that query zero-or-many entities are out of scope.

### NFR-4: No Logic Changes
This is a mechanical refactor. System logic (movement, collision, camera, etc.) must not be changed.

---

## Phase Scoping

### Phase 1 — Core Game Systems (FR-1 + FR-3)
Convert all `.single()` call sites in `src/systems/game/` and `src/systems/settings/` and `src/systems/intro_animation/`. Verify tests pass.

### Phase 2 — Editor Systems (FR-2 + FR-3)
Convert all `.single()` call sites in `src/editor/`. Verify tests pass.

---

## Assumptions

- The project is on Bevy 0.18, which is confirmed by `Cargo.toml`.
- `Single<>` was stabilised in Bevy 0.15 and is available in 0.18.
- Systems that use `Option<Single<>>` retain early-return semantics automatically (the system runs with `None` when no entity matches, rather than being skipped entirely).

---

## Dependencies

- None. This is a self-contained refactor with no new crate dependencies.

---

## Open Questions

- **OQ-1 — Resolved:** `ChunkLOD` can implement `Default` with null `Handle<Mesh>` values (`Handle::default()` is valid). However, `#[require(ChunkLOD)]` on `VoxelChunk` provides no benefit: every spawn in `spawn_voxels_chunked` already provides an explicit `ChunkLOD` with real mesh handles, so the default is never triggered. Phase 3 should be closed by adding a code comment on `VoxelChunk` explaining the co-spawn invariant, rather than implementing `#[require]`.

- **OQ-2 — Resolved:** All editor `.single()` sites can use plain `Single<>`. The editor launches the game as a separate child process (`std::process::Command`), so the editor's ECS world always has exactly one `EditorCamera` and one primary `Window`. `Option<Single<>>` is not needed for any editor site.
