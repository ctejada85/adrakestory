# Epic — Bevy 0.18 API Modernization: `Single<>` Query Parameters

## Epic Story

As a developer working on A Drake's Story, I want the codebase to use Bevy 0.18 idiomatic `Single<>` system parameters so that system signatures express intent clearly, error-handling boilerplate is eliminated, and systems skip gracefully when expected entities are absent rather than failing silently.

---

## Description

The codebase was migrated to Bevy 0.18 but did not adopt the `Single<>` system parameter type, which replaces the `Query<T>` + `.single()` / `.single_mut()` runtime call pattern. There are 30+ call sites across game systems, editor systems, and settings where `.single()` is called inside `if let Ok(...)` or `let Ok(...) else { return }` blocks.

Converting these to `Single<>` (or `Option<Single<>>` where the entity is conditionally present) makes the single-entity contract visible in the system signature, removes boilerplate, and lets Bevy skip systems automatically when the entity count doesn't match — rather than silently doing nothing.

A secondary scope (Phase 3) investigates whether `VoxelChunk` and `ChunkLOD` can declare a `#[require()]` relationship.

**Out of scope:** Event/observer migration, Cargo feature collections, shader changes, any logic changes.

---

## Acceptance Criteria

1. All `Query::single()` and `Query::single_mut()` call sites in `src/systems/game/` and `src/systems/settings/` and `src/systems/intro_animation/` are replaced with `Single<>` or `Option<Single<>>` system parameters.
2. All `Query::single()` and `Query::single_mut()` call sites in `src/editor/` are replaced with `Single<>` or `Option<Single<>>` system parameters.
3. Systems that always have exactly one matching entity use `Single<>` (not `Option<Single<>>`).
4. Systems where the entity may legitimately be absent (occlusion, hot-reload gap, editor context) use `Option<Single<>>`.
5. No system logic is changed — only the parameter and the associated binding/destructuring code.
6. All 355 existing tests pass after the changes.
7. `cargo clippy` produces no new warnings.
8. The game loads, the player can move, collision works, the camera follows the player, and VSync settings apply correctly at runtime.
9. The map editor opens, the 3D viewport renders, voxel placement/removal tools work, and the camera pans correctly.
10. `VoxelChunk` in `src/systems/game/map/spawner/mod.rs` has a `// Invariant: always spawned with ChunkLOD` comment added.

---

## Non-Functional Requirements

- Must not introduce any new runtime panics — `Single<>` skips the system instead of panicking when the entity is absent.
- Must not change system execution order or scheduling.
- Must not add any new crate dependencies.
- Changes must be limited to system parameter declarations and the corresponding destructuring / dereference code in system bodies.
- The hot-reload gap (1–2 frames where player/camera entities are absent) must remain handled correctly — use `Option<Single<>>` for any system that runs during or after `handle_map_reload`.

---

## Tasks

### Phase 1 — Core Game Systems

- [ ] **T1.1** Convert `src/systems/game/physics.rs`
  - `apply_gravity`: `Query<&mut Player>` → `Single<&mut Player>`
  - `apply_physics`: `Query<(&mut Player, &mut Transform)>` → `Single<(&mut Player, &mut Transform)>`
  - `apply_npc_collision`: player query → `Option<Single<>>` (runs during reload gap)

- [ ] **T1.2** Convert `src/systems/game/camera.rs`
  - `follow_player_camera`: nested `if let Ok` for player + camera → `Single<>` for both
  - `rotate_camera`: `camera_query.single_mut()` → `Single<>`

- [ ] **T1.3** Convert `src/systems/game/player_movement.rs`
  - `move_player`: `player_query.single_mut()` inside `if let Ok` → `Single<>` (already gated by `run_if(in_state(InGame))`)

- [ ] **T1.4** Convert `src/systems/game/input.rs`
  - `toggle_flashlight`: player query `.single()` → `Option<Single<>>`
  - `update_flashlight_rotation`: player + flashlight queries → `Single<>` / `Option<Single<>>`
  - Window query → `Single<&mut Window, With<PrimaryWindow>>`

- [ ] **T1.5** Convert `src/systems/game/interior_detection.rs`
  - Player query `.single()` → `Option<Single<&Transform, With<Player>>>`

- [ ] **T1.6** Convert `src/systems/game/occlusion/mod.rs`
  - Camera + player queries in update system → `Option<Single<>>` (may be absent during hot-reload)

- [ ] **T1.7** Convert `src/systems/game/map/spawner/mod.rs`
  - Camera query in `update_chunk_lods` → `Single<&Transform, With<GameCamera>>`

- [ ] **T1.8** Convert `src/systems/settings/vsync.rs`
  - Window query → `Single<&mut Window, With<PrimaryWindow>>`

- [ ] **T1.9** Convert `src/systems/intro_animation/systems.rs`
  - Text query `.single_mut()` → `Single<&mut TextColor, ...>`

- [ ] **T1.10** Run `cargo test` — verify all 355 tests pass
- [ ] **T1.11** Run `cargo clippy` — verify no new warnings
- [ ] **T1.12** Manual smoke test: launch game, verify player movement, camera, VSync toggle

### Phase 2 — Editor Systems

- [ ] **T2.1** Convert `src/editor/camera.rs` (2 sites)
- [ ] **T2.2** Convert `src/editor/cursor/mouse_cursor.rs` (1 site)
- [ ] **T2.3** Convert `src/editor/controller/cursor.rs` (1 site)
- [ ] **T2.4** Convert `src/editor/controller/camera.rs` (1 site)
- [ ] **T2.5** Convert `src/editor/grid/systems.rs` (1 site)
- [ ] **T2.6** Convert `src/editor/tools/voxel_tool/removal.rs` (2 sites)
- [ ] **T2.7** Convert `src/editor/tools/voxel_tool/placement.rs` (2 sites)
- [ ] **T2.8** Convert `src/editor/tools/selection_tool/selection.rs` (2 sites)

- [ ] **T2.9** Run `cargo test` — verify tests still pass
- [ ] **T2.10** Manual smoke test: launch map editor, verify viewport, voxel tools, camera pan

### Phase 3 — Invariant Comment (1 task)

- [ ] **T3.1** Add `// Invariant: always spawned with ChunkLOD — see spawn_voxels_chunked` comment to the `VoxelChunk` struct. (`#[require(ChunkLOD)]` was rejected — see OQ-1 in requirements.)

---

## References

- [Requirements](references/requirements.md)
- [Architecture](references/architecture.md)
- [Bevy `Single<>` docs](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Single.html)
- [Bevy 0.17 → 0.18 Migration Guide](https://bevyengine.org/learn/migration-guides/0-17-to-0-18/)
