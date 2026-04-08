# Coding Guardrails

Hard rules that prevent known classes of bugs in this codebase. Each guardrail has a real incident behind it.

---

## 1. Never call `Assets::get_mut()` unconditionally

**Why:** `Assets::get_mut()` stamps the asset's change-detection tick *before* any write. The render world re-prepares GPU bind groups for every asset marked changed that frame — even if the value is identical. At 60 fps this causes a full GPU re-upload every frame.

**Rule:** Only call `get_mut()` after confirming a value actually changed.

```rust
// ✗ Wrong — marks asset dirty every frame regardless of change
fn update_uniforms(mut materials: ResMut<Assets<M>>, config: Res<OcclusionConfig>) {
    if let Some(mat) = materials.get_mut(&handle) {
        mat.extension.uniforms.radius = config.occlusion_radius; // always written
    }
}

// ✓ Correct — guard with dirty check before get_mut
fn update_uniforms(
    mut materials: ResMut<Assets<M>>,
    config: Res<OcclusionConfig>,
    mut cache: Local<Option<f32>>,
) {
    let new_radius = config.occlusion_radius;
    if cache.as_ref() != Some(&new_radius) {
        if let Some(mat) = materials.get_mut(&handle) {
            mat.extension.uniforms.radius = new_radius;
            *cache = Some(new_radius);
        }
    }
}
```

---

## 2. Always use `SpatialGrid` for collision queries

**Why:** Iterating all `SubVoxel` entities is O(n²) and causes severe frame drops on larger maps. `SpatialGrid` provides O(n) spatial partitioning.

**Rule:** Never write a collision or proximity check that directly iterates `Query<&SubVoxel>` over the whole world.

```rust
// ✗ Wrong — O(n²), kills performance on large maps
for sub_voxel in sub_voxel_query.iter() {
    if overlaps(player_aabb, sub_voxel.bounds) { ... }
}

// ✓ Correct — O(n) with spatial partitioning
let nearby = spatial_grid.get_entities_in_aabb(player_min, player_max);
for entity in nearby {
    if let Ok(sub_voxel) = sub_voxel_query.get(entity) { ... }
}
```

`SpatialGrid` is inserted at spawn time in `spawn_voxels_chunked()`. Every `SubVoxel` entity must be registered there at spawn.

---

## 3. Clamp delta time in physics systems

**Why:** After a window loses focus, the next frame's delta time can be seconds instead of milliseconds. Without clamping, velocity integration explodes and the player teleports.

**Rule:** Every system that integrates velocity or position must clamp delta time to `0.1` seconds maximum.

```rust
// ✗ Wrong — unbounded delta
let delta = time.delta_secs();
player.velocity.y += GRAVITY * delta;

// ✓ Correct — clamped delta
let delta = time.delta_secs().min(0.1);
player.velocity.y += GRAVITY * delta;
```

---

## 4. All map mutations in the editor must go through `EditorHistory`

**Why:** Direct `MapData` mutation bypasses the undo/redo stack. The user loses their history and the renderer's dirty flag may not be set correctly.

**Rule:** In any editor system, always call `history.apply(...)` — never mutate `MapData` or `EditorState.map` directly.

```rust
// ✗ Wrong — breaks undo stack
editor_state.map.voxels[pos] = new_voxel;

// ✓ Correct — goes through history
history.apply(EditorCommand::PlaceVoxel { pos, voxel: new_voxel }, &mut editor_state);
```

---

## 5. Never have 2D and 3D cameras active simultaneously

**Why:** Bevy will log an ambiguity warning and one camera's output may be composited incorrectly. The 2D camera is for UI states; the 3D camera is for `InGame`.

**Rule:** State transition systems must clean up the outgoing camera before spawning the incoming one. Use `cleanup_2d_camera()` on exit from UI states.

```rust
// In state teardown
app.add_systems(OnExit(GameState::TitleScreen), cleanup_2d_camera);
app.add_systems(OnEnter(GameState::InGame), spawn_3d_camera);
```

---

## 6. Use `SubVoxel.bounds` — never recompute bounds at runtime

**Why:** Sub-voxel collision bounds are pre-calculated at spawn time in `spawn_voxels_chunked()`. Recomputing them every frame from geometry is expensive and redundant.

**Rule:** Always read `sub_voxel.bounds` directly. Do not call geometry helpers in hot-path collision code.

```rust
// ✗ Wrong — expensive per-frame recompute
let bounds = compute_sub_voxel_bounds(&sub_voxel.geometry);

// ✓ Correct — cached at spawn time
let bounds = sub_voxel.bounds;
```

---

## 7. Gate systems to the correct `GameState`

**Why:** Systems without a state condition run in every state — including title screen, loading, and pause. A physics system running during `TitleScreen` can panic because player entities don't exist.

**Rule:** Every game system must be gated with `.run_if(in_state(GameState::InGame))` (or the appropriate state), except systems explicitly designed to run across states.

```rust
// ✗ Wrong — runs in all states
app.add_systems(Update, apply_gravity.in_set(GameSystemSet::Physics));

// ✓ Correct — gated to InGame
app.add_systems(
    Update,
    apply_gravity
        .in_set(GameSystemSet::Physics)
        .run_if(in_state(GameState::InGame)),
);
```

---

## 8. Assign systems to the correct `GameSystemSet`

**Why:** `GameSystemSet` enforces the `Input → Movement → Physics → Visual → Camera` ordering within a frame. A physics system placed in `Visual` or left without a set can read stale transforms or trigger race conditions.

| Set | Systems that belong here |
|-----|--------------------------|
| `Input` | Keyboard/gamepad/mouse readers, `PlayerInput` population |
| `Movement` | Velocity application, character rotation |
| `Physics` | Gravity, collision resolution, position updates |
| `Visual` | Material updates, LOD, occlusion uniforms |
| `Camera` | Camera follow, smooth rotation — always last |

---

## 9. Only derive `PartialEq` (not `Eq`) on structs with float fields

**Why:** `Eq` requires reflexivity (`a == a` always true). `f32` violates this for NaN. Rust won't compile `#[derive(Eq)]` on structs with `f32` fields, but manually implementing it would be a subtle bug.

**Rule:** For structs with `f32`, `Vec2`, `Vec3`, or `Vec4` fields, derive `PartialEq` only. If NaN appears (physics explosion), `PartialEq` returns false → re-upload occurs → safe conservative behavior.

```rust
// ✓ Correct
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DynamicOcclusionUniforms {
    pub player_position: Vec3,  // f32 fields — cannot derive Eq
    pub camera_position: Vec3,
}
```

---

## 10. `Local<T>` is per-system state, never shared

**Why:** `Local<T>` parameters give each system instance its own private state. Two systems cannot share a `Local` — it is not a `Resource`. Using `Local` as a poor-man's global produces silent bugs where each system sees different values.

**Rule:** Use `Local<T>` only for state that belongs exclusively to one system (frame counters, per-system caches). For shared mutable state, use `ResMut<T>`.

```rust
// ✓ Correct — frame counter private to this system
pub fn update_occlusion_uniforms(mut frame_counter: Local<u32>, ...) {
    *frame_counter += 1;
}

// ✗ Wrong — trying to share a Local across two systems won't work;
// each system gets its own independent copy
```

---

## 11. Never access the egui context in `Startup`

**Why:** `bevy_egui` initialises its `PrimaryEguiContext` entity lazily — it is not present during `Startup` on all platforms (confirmed panic on macOS with Metal backend). Calling `contexts.ctx_mut().expect(...)` in a `Startup` system crashes the app before the first frame renders.

**Rule:** Any one-time egui initialisation (font setup, style overrides) must run in `Update`, guarded by a `Local<bool>` flag. Use `let Ok(ctx) = contexts.ctx_mut() else { return; }` so the system silently skips frames where the context is not yet ready.

```rust
// ✗ Wrong — panics on macOS (egui context not ready at Startup)
pub fn setup_fonts(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut().expect("egui context"); // PANIC
    ctx.set_fonts(/* ... */);
}
// registered as: .add_systems(Startup, setup_fonts)

// ✓ Correct — deferred to Update, skips until context is available
pub fn setup_fonts(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done { return; }
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    ctx.set_fonts(/* ... */);
    *done = true;
}
// registered as: .add_systems(Update, setup_fonts)
```

---

## 12. Use write-through for egui text fields — never rebuild the buffer each frame

**Why:** `ui.text_edit_singleline(&mut name)` modifies the passed `&mut String` in-place for the current frame only. If `name` is a local that is re-initialized from stored state every frame (`let mut name = stored_value.clone()`), the modifications are silently discarded at end-of-frame. The field appears to accept input (the cursor moves, the caret blinks) but typed characters vanish immediately.

**Rule:** For any egui text field that commits on focus-lost, write the updated value back to the stored state on every `response.changed()` call. Capture a pre-edit snapshot for undo on `response.gained_focus()`.

```rust
// ✗ Wrong — typed characters reset each frame
fn render_name_field(ui: &mut egui::Ui, state: &mut MyState) {
    let current = state.name.clone();          // re-initialized every frame
    let mut name = current.clone();
    let response = ui.text_edit_singleline(&mut name);
    if response.lost_focus() && name != current {
        state.name = name;                     // only written on focus-lost
    }
    // `name` is dropped here — next frame starts over from `state.name`
}

// ✓ Correct — write-through on change + snapshot on gained_focus for undo
fn render_name_field(
    ui: &mut egui::Ui,
    state: &mut MyState,
    history: &mut EditorHistory,
    index: usize,
) {
    let current = state.name.clone();
    let mut name = current.clone();
    let snapshot_id = egui::Id::new("name_snapshot").with(index);
    let response = ui.text_edit_singleline(&mut name);

    if response.gained_focus() {
        // Save pre-edit snapshot for undo (one entry per session, not per keystroke)
        ui.data_mut(|d| d.insert_temp(snapshot_id, state.clone()));
    }
    if response.changed() {
        // Write through so the next frame reads the updated value
        state.name = name.clone();
        state.mark_modified();
    }
    if response.lost_focus() {
        let old = ui.data_mut(|d| d.get_temp::<MyState>(snapshot_id));
        ui.data_mut(|d| d.remove::<MyState>(snapshot_id));
        if let Some(old_state) = old {
            if state.name != old_state.name {
                history.push(Action::Modify { old: old_state, new: state.clone() });
            }
        }
    }
}
```

The snapshot stored in egui's temp data (`ui.data_mut`) is keyed by a stable `Id` and persists across frames for the lifetime of the focus. It is automatically cleared when the focus leaves.

See: `src/editor/ui/properties/entity_props.rs` — `render_entity_name_field()`
