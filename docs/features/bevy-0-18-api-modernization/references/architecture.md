# Architecture — Bevy 0.18 API Modernization

## Current Architecture

### Affected System Parameter Pattern

Every system that needs to access a single well-known entity (player, camera, window) currently declares a `Query<T>` and calls `.single()` or `.single_mut()` at runtime inside the system body. This produces two styles of boilerplate:

**Style A — Optional early return:**
```rust
pub fn follow_player_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform), Without<Player>>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let Ok((mut game_camera, mut camera_transform)) = camera_query.single_mut() else { return; };
    // logic ...
}
```

**Style B — Nested `if let Ok`:**
```rust
pub fn apply_gravity(
    time: Res<Time>,
    mut player_query: Query<&mut Player>,
) {
    if let Ok(mut player) = player_query.single_mut() {
        // logic ...
    }
}
```

**Inventory of call sites:**

| File | System | Style |
|---|---|---|
| `systems/game/physics.rs` | `apply_gravity` | B |
| `systems/game/physics.rs` | `apply_physics` | B |
| `systems/game/physics.rs` | `apply_npc_collision` | A |
| `systems/game/camera.rs` | `follow_player_camera` | A + B |
| `systems/game/camera.rs` | `rotate_camera` | B |
| `systems/game/player_movement.rs` | `move_player` | B |
| `systems/game/input.rs` | `toggle_flashlight` | A |
| `systems/game/input.rs` | `update_flashlight_rotation` | A |
| `systems/game/interior_detection.rs` | interior check | A |
| `systems/game/occlusion/mod.rs` | occlusion update | A |
| `systems/game/map/spawner/mod.rs` | `update_chunk_lods` | A |
| `systems/settings/vsync.rs` | `apply_vsync_system` | B |
| `systems/intro_animation/systems.rs` | `animate_intro` | B |
| `editor/camera.rs` | editor camera | A |
| `editor/cursor/mouse_cursor.rs` | cursor | A |
| `editor/controller/cursor.rs` | controller cursor | A |
| `editor/controller/camera.rs` | controller camera | A |
| `editor/grid/systems.rs` | grid update | A |
| `editor/tools/voxel_tool/removal.rs` | voxel removal | A |
| `editor/tools/voxel_tool/placement.rs` | voxel placement | A |
| `editor/tools/selection_tool/selection.rs` | selection | A |

---

### Current Data Flow (Query resolution)

```
Schedule runs system
  → Bevy resolves Query<T> parameter (validates world access)
  → System body calls .single() / .single_mut()
  → Bevy checks archetype count at runtime
    → count == 1: returns Ok(item)
    → count != 1: returns Err(QuerySingleError)
  → System body handles Ok / Err
    → Ok: runs logic
    → Err: silent early return (no log, no panic)
```

The problem: the `.single()` failure path is invisible. If a player entity is inadvertently despawned, the system silently does nothing, which can make bugs hard to detect.

---

## Target Architecture

### `Single<>` System Parameter

Bevy's `Single<D, F = ()>` system parameter is a first-class alternative to `Query` + `.single()`. It declares the single-entity contract at the parameter level, shifting validation to system-param validation before the system body runs.

```rust
pub fn follow_player_camera(
    player_transform: Single<&Transform, With<Player>>,
    mut camera: Single<(&mut GameCamera, &mut Transform), Without<Player>>,
) {
    // player_transform and camera are guaranteed to exist here.
    // No error handling needed.
}
```

**Bevy behaviour:**
- If exactly one matching entity exists: system runs, parameter resolves to the entity's data.
- If zero or more-than-one matching entities exist: system is **skipped** this frame (validation failure). Bevy logs a warning in debug builds.

### `Option<Single<>>` for Conditionally-Present Entities

For entities that may legitimately be absent (e.g., player not yet spawned during map load, occlusion system running before game state), use `Option<Single<>>`. The system always runs; the body handles the `None` case explicitly.

```rust
pub fn update_occlusion(
    camera: Option<Single<(&GameCamera, &Transform)>>,
    player: Option<Single<(&Player, &Transform)>>,
) {
    let (Some(camera), Some(player)) = (camera, player) else { return; };
    // logic ...
}
```

### Decision Rule: `Single<>` vs `Option<Single<>>`

| Condition | Use |
|---|---|
| Entity always exists when the system is registered to run (player in `InGame`, camera after `spawn_map_system`) | `Single<>` |
| Entity may or may not exist (occlusion system, hot-reload gap frames, editor context) | `Option<Single<>>` |
| System is gated by `run_if(in_state(GameState::InGame))` and the entity is spawned in `OnEnter(InGame)` | `Single<>` (safe) |

---

### Target Data Flow (system-param validation)

```
Schedule runs
  → Bevy resolves Single<T> parameter
    → count == 1: resolves to entity data, system runs
    → count != 1: SystemParamValidationError, system is SKIPPED (no body runs)
  → System body receives resolved data directly
    → No runtime .single() call
    → No Ok/Err handling needed
```

---

### `#[require()]` — Not Applicable

`VoxelChunk` and `ChunkLOD` are always co-spawned. `#[require(ChunkLOD)]` was considered but rejected: every spawn in `spawn_voxels_chunked` already provides an explicit `ChunkLOD` with real mesh handles, so `ChunkLOD::default()` (which would produce null handles) would never be triggered. The co-spawn relationship is instead documented with a comment on `VoxelChunk`.

---

## Modified Components

### No New Types

This refactor introduces no new types, traits, crates, or resources. It is a mechanical substitution of one system parameter form for another.

### Modified Files (Phase 1 — Core Game)

| File | Change |
|---|---|
| `src/systems/game/physics.rs` | 3 systems: `Query<&mut Player>` → `Single<&mut Player>` (or `Option`) |
| `src/systems/game/camera.rs` | 2 systems: nested `.single()` → `Single<>` parameters |
| `src/systems/game/player_movement.rs` | 1 system: `Query::single_mut()` → `Single<>` |
| `src/systems/game/input.rs` | 2 systems: window + player `.single()` → `Single<>` |
| `src/systems/game/interior_detection.rs` | 1 site: player `.single()` → `Option<Single<>>` |
| `src/systems/game/occlusion/mod.rs` | 2 sites: camera + player `.single()` → `Option<Single<>>` |
| `src/systems/game/map/spawner/mod.rs` | 1 site: camera `.single()` → `Single<>` |
| `src/systems/settings/vsync.rs` | 1 site: window `.single_mut()` → `Single<&mut Window, With<PrimaryWindow>>` |
| `src/systems/intro_animation/systems.rs` | 1 site: text `.single_mut()` → `Single<>` |

### Modified Files (Phase 2 — Editor)

The editor always has exactly one `EditorCamera` and one primary `Window` (the game launches as a separate child process, never in-process). All editor sites use plain `Single<>`.

| File | Change |
|---|---|
| `src/editor/camera.rs` | 2 sites: `EditorCamera` + `CursorOptions` → `Single<>` |
| `src/editor/cursor/mouse_cursor.rs` | 2 sites: `EditorCamera` + `PrimaryWindow` → `Single<>` |
| `src/editor/controller/cursor.rs` | 1 site: `EditorCamera` → `Single<>` |
| `src/editor/controller/camera.rs` | 1 site: `EditorCamera` → `Single<>` |
| `src/editor/grid/systems.rs` | 1 site: `EditorCamera` → `Single<>` |
| `src/editor/tools/voxel_tool/removal.rs` | 2 sites: `PrimaryWindow` → `Single<>` |
| `src/editor/tools/voxel_tool/placement.rs` | 2 sites: `PrimaryWindow` → `Single<>` |
| `src/editor/tools/selection_tool/selection.rs` | 2 sites: camera + window via `SystemParam` → `Single<>` |

---

## Migration Pattern Reference

### Before → After (system-level)

**Physics (apply_gravity):**
```rust
// Before
pub fn apply_gravity(time: Res<Time>, mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.single_mut() {
        player.velocity.y -= 20.0 * delta;
    }
}

// After
pub fn apply_gravity(time: Res<Time>, mut player: Single<&mut Player>) {
    player.velocity.y -= 20.0 * delta;
}
```

**Occlusion (may be absent during hot-reload gap):**
```rust
// Before
pub fn update_occlusion_uniforms(
    camera_query: Query<(&GameCamera, &Transform)>,
    player_query: Query<(&Player, &Transform)>,
) {
    let camera_ref = camera_query.single().ok();
    let player_ref = player_query.single().ok();
    let (Some(camera), Some(player)) = (camera_ref, player_ref) else { return; };
    // ...
}

// After
pub fn update_occlusion_uniforms(
    camera: Option<Single<(&GameCamera, &Transform)>>,
    player: Option<Single<(&Player, &Transform)>>,
) {
    let (Some(camera), Some(player)) = (camera, player) else { return; };
    // ...
}
```

**VSync window:**
```rust
// Before
pub fn apply_vsync_system(
    mut window_query: Query<&mut Window>,
    // ...
) {
    if let Ok(mut window) = window_query.single_mut() { /* ... */ }
}

// After
pub fn apply_vsync_system(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    // ...
) { /* window is directly accessible */ }
```

---

## Sequence Diagram — System Execution (Before vs After)

```
Before:
  Bevy → resolves Query<&mut Player> → system runs
    system body → player_query.single_mut() → Ok or Err
      Ok → logic runs
      Err → silent return

After:
  Bevy → resolves Single<&mut Player>
    exactly 1 match → system runs, player is the data directly
    0 or 2+ matches → system SKIPPED (bevy warns in debug)
```

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Converting a legitimately multi-entity query to `Single<>` causes a system to always be skipped | Low — all identified sites have exactly one entity by design | Use `Single<>` only for entities spawned as singletons; use `Query` for all others |
| Hot-reload gap: player/camera despawned but system still registered | Medium — happens 1–2 frames during reload | Use `Option<Single<>>` for all queries in systems that run during map reload |
| Editor context: multiple editor camera entities | Low — editor has exactly one camera | Confirm before converting; use `Option<Single<>>` if uncertain |
| `#[require()]` with empty `ChunkLOD` default causes rendering glitch | Low — handles are always set immediately after spawn | Verify in-game by loading a map and checking chunk rendering |
