# Clippy Warnings Backlog

Tracking warnings after the 2026-04-01 Clippy clean-up passes. All actionable items are either resolved or have a tracking ticket.

---

## Resolved (this session)

### ~~`unused_mut`~~ ✅ RESOLVED
Removed at 6 sites across `interior_detection.rs`, `camera.rs`, `physics.rs`, `player_movement.rs`.

### ~~`map_or(false,…)` / `map_or(true,…)`~~ ✅ RESOLVED
Replaced with `is_some_and` / `is_none_or` at 7 sites.

### ~~`len() > 0`~~ ✅ RESOLVED
Replaced with `!is_empty()` in `editor/camera.rs`.

### ~~`needless_borrow`~~ ✅ RESOLVED
Fixed in `interior_detection.rs`.

### ~~`derivable_impls`~~ ✅ RESOLVED
`#[derive(Default)]` added to `ControllerCameraMode`, `HotbarItem`, `FpsCounterState`.

### ~~`private_interfaces` — occlusion uniform structs~~ ✅ RESOLVED
Widened both structs to `pub(crate)` and added `#[allow(private_interfaces)]` on `update_occlusion_uniforms`.

### ~~`unused_imports` in `settings/mod.rs`~~ ✅ RESOLVED
Added `#[allow(unused_imports)]` with explanatory comment.

### ~~`dead_code` Group A — entity spawning helpers~~ ✅ RESOLVED
Suppressed with `#[allow(dead_code)]` in `entities.rs`.

### ~~`dead_code` Group B — interior detection stubs~~ ✅ RESOLVED
Suppressed with `#[allow(dead_code)]` in `interior_detection.rs`.

### ~~`dead_code` Group C — player movement helpers~~ ✅ RESOLVED (deleted)
`input_to_world_direction`, `calculate_look_direction`, `calculate_target_rotation`, `normalize_movement` and their 16 tests removed from `player_movement.rs`.

### ~~`type_complexity` (6 sites)~~ ✅ RESOLVED
`SubVoxelEntry` type alias in `renderer.rs` and `chunks.rs`; `SelectionBounds` type alias in `selection.rs`; `#[allow]` at the three remaining sites.

### ~~`too_many_arguments` (7 Bevy system functions)~~ ✅ RESOLVED
`#[allow(clippy::too_many_arguments)]` added at all 7 affected functions.

### ~~`dead_code` — `lerp_alpha`~~ ✅ RESOLVED (deleted)
Function and its 4 tests deleted from `camera.rs`. The formula is inlined at both call sites.

### ~~`dead_code` — `cylinder_aabb_intersects`~~ ✅ RESOLVED (deleted)
Function and its 8 tests deleted from `collision.rs`. Logic is already inlined in `check_sub_voxel_collision`.

### ~~`dead_code` — `GreedyMesher::new()`~~ ✅ RESOLVED (deleted)
Deleted; all instantiation uses `or_default()` (calls `Default`).

### ~~`dead_code` — `VoxelMaterialPalette` instance half~~ ✅ RESOLVED (deleted)
Deleted `materials` field, `new()`, `get_material()`, `Resource`/`Clone` derives. `PALETTE_SIZE` and `get_material_index` are kept.

### ~~`dead_code` — `LightSource` fields~~ ✅ RESOLVED (implemented)
`sync_light_sources` system implemented in `input.rs` and registered in `GameSystemSet::Visual`.
Feature ticket: `docs/bugs/runtime-light-mutation/ticket.md`.

---

## Deferred — Tracked by feature ticket

| Item | File | Ticket |
|------|------|--------|
| `Npc::name` field | `components.rs:63` | `docs/bugs/npc-display-names/ticket.md` |
| `GamepadSettings` fields (`trigger_deadzone`, `invert_camera_y`, `camera_sensitivity`) | `gamepad.rs:25` | `docs/bugs/gamepad-settings-apply/ticket.md` |

---

## Pre-existing / Low-value

### `CollisionResult::step_up_height` field (`collision.rs:23`)
Step-up collision was planned but not finished. Field is part of an incomplete feature; suppressed by the struct definition. No separate ticket yet.

### `deprecated` — `enable_multipass_for_primary_context` (`src/bin/map_editor.rs:51`)
Pre-existing. Deferred until a `bevy_egui` upgrade pass. Not part of this clean-up scope.
