# Coding Style Guide

Concrete conventions used throughout this codebase. All new code must follow these patterns.

---

## 1. Naming Conventions

| Kind | Convention | Example |
|------|-----------|---------|
| Types, structs, enums, traits | `PascalCase` | `SubVoxel`, `OcclusionConfig`, `GameState` |
| Functions, methods, variables, fields | `snake_case` | `apply_gravity`, `player_position`, `occlusion_radius` |
| Constants | `SCREAMING_SNAKE_CASE` | `GRAVITY`, `MAX_REGION_SIZE`, `GROUND_DETECTION_EPSILON` |
| Modules / files | `snake_case` | `occlusion`, `interior_detection`, `player_movement` |

### Bevy-specific naming

- **Components**: noun describing the entity role — `Player`, `SubVoxel`, `GameCamera`, `LightSource`
- **Resources**: noun describing the data — `SpatialGrid`, `OcclusionConfig`, `GameInitialized`
- **Events**: noun + past/present event — `MapReloadEvent`, `MapReloadedEvent`, `AppExitEvent`
- **Systems**: verb phrase describing the action — `apply_gravity`, `move_player`, `update_occlusion_uniforms`
- **Plugins**: PascalCase + `Plugin` suffix — `OcclusionPlugin`, `GamePlugin`

---

## 2. Derive Macros

Only derive what is needed. Common patterns:

```rust
// Components — minimal; add Debug/Clone/Copy only if genuinely needed
#[derive(Component)]
pub struct Player { ... }

// Marker components — no fields
#[derive(Component)]
pub struct CollisionBox;

// Resources — Default almost always needed for App::init_resource
#[derive(Resource, Default)]
pub struct SpatialGrid { ... }

// Events — no derives required beyond Event
#[derive(Event)]
pub struct MapReloadEvent { ... }

// Data/config types shared between game and editor
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransparencyTechnique { ... }

// Private helper structs used in tests — must have Debug for assert_eq!
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StaticOcclusionUniforms { ... }
```

**Rules:**
- Never derive `Eq` on structs containing `f32`, `Vec3`, or `Vec4` — use `PartialEq` only.
- Add `Debug` to any struct used in `assert_eq!` / `assert_ne!`.
- Add `Serialize, Deserialize` only for map format types and config that must persist to disk.

---

## 3. System Function Signatures

Systems are free functions (not methods). Declare parameters in this order:

1. Time / input resources (`Res<Time>`, `Res<PlayerInput>`)
2. Read-only resources (`Res<SpatialGrid>`, `Res<OcclusionConfig>`)
3. Mutable resources (`ResMut<Assets<T>>`)
4. Local state (`Local<T>`) — Bevy per-system state, never shared
5. Queries — read-only before mutable, simple before complex
6. Event readers / writers

```rust
// Simple system
pub fn apply_gravity(time: Res<Time>, mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.get_single_mut() {
        let delta = time.delta_secs().min(0.1);
        player.velocity.y += GRAVITY * delta;
    }
}

// System with multiple parameter kinds
pub fn move_player(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) { ... }

// System with Local caches (per-system state, not shared)
pub fn update_occlusion_uniforms(
    config: Res<OcclusionConfig>,
    mut materials: ResMut<Assets<OcclusionMaterial>>,
    mut frame_counter: Local<u32>,
    mut static_cache: Local<Option<StaticOcclusionUniforms>>,
    mut dynamic_cache: Local<Option<DynamicOcclusionUniforms>>,
    player_query: Query<Ref<Transform>, With<Player>>,
) { ... }
```

---

## 4. System Set Registration

All gameplay systems belong to a `GameSystemSet`. Always specify the correct set — omitting it causes race conditions.

```
Input → Movement → Physics → Visual → Camera
```

```rust
// In main.rs
app.add_systems(Update, gather_input.in_set(GameSystemSet::Input));
app.add_systems(Update, move_player.in_set(GameSystemSet::Movement));
app.add_systems(Update, (apply_gravity, check_collisions).in_set(GameSystemSet::Physics));
app.add_systems(Update, update_occlusion_uniforms.in_set(GameSystemSet::Visual));
app.add_systems(Update, follow_player_camera.in_set(GameSystemSet::Camera));
```

Systems added without a set run in an undefined order relative to all other systems.

---

## 5. Module Organization

```
some_feature/
├── mod.rs          # Public API, plugin registration, system scheduling
├── components.rs   # Component structs for this feature
├── resources.rs    # Resource structs
├── systems.rs      # System functions (split further if > ~300 lines)
└── tests.rs        # Unit tests (see §7)
```

**Target file size:** 200–400 lines. Split when a file exceeds ~400 lines or contains multiple unrelated concerns.

The `mod.rs` re-exports public items:

```rust
pub use components::{Player, GameCamera};
pub use resources::SpatialGrid;
```

---

## 6. Comments and Documentation

Use `///` (rustdoc) for public items and non-obvious private items. Use `//` for inline logic explanations.

```rust
//! Module-level doc: what this module does and why it exists.

/// Uniform buffer for occlusion parameters.
///
/// Uploaded to the GPU once per frame. All fields map 1:1 to the
/// WGSL `OcclusionUniforms` struct in `occlusion.wgsl`.
#[derive(ShaderType, Clone)]
pub struct OcclusionUniforms { ... }

pub fn apply_gravity(time: Res<Time>, mut query: Query<&mut Player>) {
    // Clamp delta to prevent physics explosion after window focus loss.
    let delta = time.delta_secs().min(0.1);
    ...
}
```

**Rules:**
- Comment the **why**, not the **what**.
- Avoid comments that restate the code: `// increment counter` above `counter += 1` adds no value.
- Add `///` docs on any public function whose purpose is not obvious from its name.

---

## 7. Test Organization

Tests live in a sibling `tests.rs` file, declared in the parent module:

```rust
// in mod.rs or the_file.rs
#[cfg(test)]
mod tests;
```

```rust
// in tests.rs
#![cfg(test)]

use super::*;

#[test]
fn cache_hit_same_config_is_not_dirty() {
    let a = StaticOcclusionUniforms::from_config(&OcclusionConfig::default());
    let b = StaticOcclusionUniforms::from_config(&OcclusionConfig::default());
    assert_eq!(a, b);
}
```

**Rules:**
- One logical claim per `assert_*`. Prefer descriptive values over magic numbers.
- Test pure helper functions — avoid full `bevy::App` setup unless strictly necessary.
- Test names describe the scenario, not the function: `cache_hit_same_config_is_not_dirty`, not `test_from_config`.
- Cover: happy path, change path, independence of concerns, cold start, assembly correctness.

---

## 8. Logging

Use Bevy's tracing macros. Always prefix with the subsystem name in brackets.

```rust
info!("[Occlusion] Mode: {:?}, Player: ({:.1}, {:.1}, {:.1})", mode, x, y, z);
warn!("[Occlusion] Material asset not found in Assets<OcclusionMaterial>");
warn!("[Map] Validation failed: {}", reason);
error!("[Physics] Spatial grid query returned invalid entity {:?}", entity);
```

**When to use each level:**

| Level | When |
|-------|------|
| `info!` | Periodic heartbeat data, major state transitions, spawn counts |
| `warn!` | Recoverable unexpected states (missing asset, invalid config) |
| `error!` | Unrecoverable or data-corrupting failures |

Throttle noisy per-frame logs with a frame counter:

```rust
*frame_counter += 1;
if frame_counter.is_multiple_of(120) {
    info!("[Occlusion] ...");
}
```

---

## 9. Formatting

Run before every commit:

```bash
cargo fmt
cargo clippy
```

Clippy is zero-tolerance — no `#[allow(...)]` without an explanatory comment. If Clippy flags a private type in a public interface, make it `pub(super)`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StaticOcclusionUniforms { ... }
```
