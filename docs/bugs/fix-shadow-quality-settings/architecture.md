# Architecture: Configurable Shadow Quality

**Date:** 2026-03-16  
**Status:** Draft  
**Component:** Rendering / Settings  
**Related:** `docs/bugs/fix-shadow-quality-settings/requirements.md`

---

## Current Architecture

```
OcclusionConfig (Resource)
  ├── enabled: bool
  ├── hide_shadows: bool   ← stub, never wired
  └── ... (occlusion fields)

spawn_lighting()  ← called once from spawn_map_system
  └── commands.spawn(DirectionalLight { shadows_enabled: true })
      shadows hard-coded: 4 cascades, 100 units

spawn_voxels_chunked()
  └── commands.spawn(VoxelChunk + Mesh3d)
      no NotShadowCaster → all chunks drawn into all 4 shadow passes every frame
```

**Problem:** `DirectionalLight` shadow config and `NotShadowCaster` presence on `VoxelChunk`
entities are set once at spawn and never revisited. There is no runtime mechanism to
change shadow quality.

---

## Target Architecture

```
OcclusionConfig (Resource)
  ├── shadow_quality: ShadowQuality   ← NEW (replaces hide_shadows)
  └── ... (all other fields unchanged)

ShadowQuality (Enum) ← NEW, defined in occlusion/mod.rs
  ├── None
  ├── CharactersOnly
  ├── Low
  └── High

spawn_lighting(config: &OcclusionConfig)   ← receives config, applies quality at spawn
spawn_voxels_chunked(ctx: &SpawnContext)   ← conditionally inserts NotShadowCaster

apply_shadow_quality_system()              ← NEW: runs in Visual set, handles runtime changes
  ├── Watches OcclusionConfig::is_changed()
  ├── Watches Added<VoxelChunk> (hot reload)
  ├── Mutates DirectionalLight.shadows_enabled
  ├── Replaces CascadeShadowConfig component
  └── Adds/removes NotShadowCaster on VoxelChunk entities

Settings Menu
  └── SettingId::ShadowQuality ← NEW variant in SettingId enum
```

---

## Component Changes

### 1. `src/systems/game/occlusion/mod.rs`

**Add `ShadowQuality` enum** (alongside `OcclusionMode`, `TransparencyTechnique`):

```rust
#[derive(
    Default, Clone, Copy, Debug, PartialEq, Eq,
    Serialize, Deserialize,
)]
pub enum ShadowQuality {
    None,
    CharactersOnly,
    #[default]
    Low,
    High,
}
```

**Modify `OcclusionConfig`** — remove `hide_shadows`, add `shadow_quality`:

```rust
// Remove:
pub hide_shadows: bool,

// Add:
pub shadow_quality: ShadowQuality,
```

**Update `OcclusionConfig::default()`**:

```rust
// Remove:
hide_shadows: true,

// Add:
shadow_quality: ShadowQuality::Low,
```

---

### 2. `src/systems/game/map/spawner/mod.rs`

**Modify `spawn_map_system`** — pass `occlusion_config` into `spawn_lighting` and
`spawn_voxels_chunked` already receives context through `SpawnContext`:

```rust
pub fn spawn_map_system(
    ...,
    occlusion_config: Res<OcclusionConfig>,
    ...
) {
    ...
    spawn_lighting(&mut commands, &map, &occlusion_config);
    ...
}
```

**Modify `spawn_lighting`** — apply shadow quality at spawn:

```rust
fn spawn_lighting(
    commands: &mut Commands,
    map: &MapData,
    config: &OcclusionConfig,   // NEW param
) {
    if let Some(dir_light) = &map.lighting.directional_light {
        let (shadows_enabled, cascade_config) =
            shadow_params_for_quality(config.shadow_quality);

        commands.spawn((
            DirectionalLight {
                shadows_enabled,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
                ...
            },
            cascade_config,
            ...
        ));
    }
}

/// Returns (shadows_enabled, CascadeShadowConfig) for each quality level.
fn shadow_params_for_quality(quality: ShadowQuality) -> (bool, CascadeShadowConfig) {
    match quality {
        ShadowQuality::None => (
            false,
            CascadeShadowConfigBuilder::default().build(), // unused, shadows_enabled=false
        ),
        ShadowQuality::CharactersOnly | ShadowQuality::Low => (
            true,
            CascadeShadowConfigBuilder {
                num_cascades: 2,
                first_cascade_far_bound: 4.0,
                maximum_distance: 20.0,
                ..default()
            }
            .build(),
        ),
        ShadowQuality::High => (
            true,
            CascadeShadowConfigBuilder {
                num_cascades: 4,
                first_cascade_far_bound: 4.0,
                maximum_distance: 100.0,
                ..default()
            }
            .build(),
        ),
    }
}
```

**Modify `SpawnContext`** — add `shadow_quality` field so `spawn_voxels_chunked` can
conditionally attach `NotShadowCaster`:

```rust
pub struct SpawnContext<'a> {
    ...
    pub shadow_quality: ShadowQuality,   // NEW
}
```

In `spawn_voxels_chunked`, after spawning the `VoxelChunk` entity:

```rust
if ctx.shadow_quality == ShadowQuality::CharactersOnly {
    entity_commands.insert(NotShadowCaster);
}
```

---

### 3. `src/systems/game/map/spawner/chunks.rs` (new system)

**New file: `src/systems/game/map/spawner/shadow_quality.rs`** (or inline in `mod.rs`):

```rust
/// Applies shadow quality from OcclusionConfig to the live scene.
///
/// Runs every frame in Visual set but early-returns unless:
/// - OcclusionConfig changed (settings menu interaction), OR
/// - New VoxelChunk entities were added (hot-reload re-spawn)
pub fn apply_shadow_quality_system(
    config: Res<OcclusionConfig>,
    new_chunks: Query<Entity, Added<VoxelChunk>>,
    all_chunks: Query<Entity, With<VoxelChunk>>,
    mut dir_lights: Query<(&mut DirectionalLight, &mut CascadeShadowConfig)>,
    mut commands: Commands,
) {
    let setting_changed = config.is_changed();
    let has_new_chunks = !new_chunks.is_empty();

    if !setting_changed && !has_new_chunks {
        return;
    }

    // Update DirectionalLight (only when setting changed; light persists across hot reload)
    if setting_changed {
        let (shadows_on, cascade_cfg) = shadow_params_for_quality(config.shadow_quality);
        for (mut light, mut cascade) in dir_lights.iter_mut() {
            light.shadows_enabled = shadows_on;
            *cascade = cascade_cfg.clone();
        }
    }

    // Update NotShadowCaster on chunks
    let chunks_to_update: Vec<Entity> = if setting_changed {
        all_chunks.iter().collect()   // full sweep on setting change
    } else {
        new_chunks.iter().collect()   // only new chunks on hot-reload
    };

    let needs_no_cast = config.shadow_quality == ShadowQuality::CharactersOnly;
    for entity in chunks_to_update {
        if needs_no_cast {
            commands.entity(entity).insert(NotShadowCaster);
        } else {
            commands.entity(entity).remove::<NotShadowCaster>();
        }
    }
}
```

---

### 4. `src/systems/settings/components.rs`

Add variant to `SettingId`:

```rust
pub enum SettingId {
    ...
    ShadowQuality,   // NEW (replaces HideShadows)
}
```

---

### 5. `src/systems/settings/systems.rs`

**Update `ALL_SETTINGS`** — replace `HideShadows` with `ShadowQuality`:

```rust
(SettingId::ShadowQuality, "Shadow Quality"),   // replaces HideShadows
```

**Add `format_value` arm:**

```rust
SettingId::ShadowQuality => match config.shadow_quality {
    ShadowQuality::None          => "Off".to_string(),
    ShadowQuality::CharactersOnly => "Characters".to_string(),
    ShadowQuality::Low           => "Low".to_string(),
    ShadowQuality::High          => "High".to_string(),
},
```

**Add `adjust_value` arm:**

```rust
SettingId::ShadowQuality => {
    let variants = [
        ShadowQuality::None,
        ShadowQuality::CharactersOnly,
        ShadowQuality::Low,
        ShadowQuality::High,
    ];
    let cur = variants.iter().position(|v| *v == config.shadow_quality).unwrap_or(2);
    config.shadow_quality =
        variants[(cur as i32 + delta).rem_euclid(variants.len() as i32) as usize];
}
```

**Update `SelectedSettingsIndex::total`** in `resources.rs`: `11` → `11` (count stays the
same because `HideShadows` is replaced by `ShadowQuality`, not added).

---

### 6. `src/systems/game/mod.rs` (or plugin registration)

Register `apply_shadow_quality_system` in `GameSystemSet::Visual`, gated to
`InGame` and `Paused` states (same pattern as other occlusion systems):

```rust
.add_systems(
    Update,
    apply_shadow_quality_system
        .in_set(GameSystemSet::Visual)
        .run_if(in_state(GameState::InGame).or(in_state(GameState::Paused))),
)
```

---

## Data Flow

### Startup / Map Load

```
App::Startup
  └── load_settings()
        └── OcclusionConfig.shadow_quality = (from file or default Low)

spawn_map_system (Update, first frame, GameInitialized guard)
  ├── spawn_lighting(config)
  │     └── shadow_params_for_quality(config.shadow_quality)
  │           → DirectionalLight { shadows_enabled, cascade_config }
  │
  └── spawn_voxels_chunked(ctx with shadow_quality)
        └── if CharactersOnly → insert NotShadowCaster on each VoxelChunk
```

### Runtime Change (Settings Menu)

```
User presses ←/→ on "Shadow Quality" row
  └── adjust_value(ShadowQuality, config, delta)
        └── config.shadow_quality = new_value   ← marks OcclusionConfig as changed

Next frame Visual set:
  └── apply_shadow_quality_system
        ├── config.is_changed() == true
        ├── Update DirectionalLight.shadows_enabled + CascadeShadowConfig
        └── Add/remove NotShadowCaster on all VoxelChunk entities

OnExit(GameState::Settings):
  └── save_settings()
        └── write shadow_quality to settings.ron
```

### Hot Reload

```
Map file saved in editor
  └── handle_map_reload despawns all VoxelChunk entities
  └── spawn_map_system re-runs (GameInitialized reset)
        └── spawn_voxels_chunked applies shadow_quality from current config
              (apply_shadow_quality_system also fires on Added<VoxelChunk> as safety net)
```

---

## Sequence Diagram: Runtime Shadow Quality Change

```
User Input           Settings System         OcclusionConfig         Visual System
    │                      │                       │                       │
    │──press →─────────────▶                       │                       │
    │               adjust_value()                 │                       │
    │               ───────────────────────────────▶                       │
    │                      │               shadow_quality = Low             │
    │                      │                       │                       │
    │                    (next frame)              │                       │
    │                      │                       │──is_changed()==true──▶│
    │                      │                       │               update DirectionalLight
    │                      │                       │               update CascadeShadowConfig
    │                      │                       │               add/remove NotShadowCaster
    │                      │                       │                       │
```

---

## Migration: `hide_shadows` Field Removal

`settings.ron` files containing `hide_shadows: true/false` will still parse correctly
because `ron::from_str` with `#[serde(deny_unknown_fields)]` would reject them — but
`OcclusionConfig` does **not** use `deny_unknown_fields`. Unknown fields are silently
ignored, so existing save files load cleanly with `shadow_quality` defaulting to `Low`.

---

## Files to Modify

| File | Change |
|---|---|
| `src/systems/game/occlusion/mod.rs` | Add `ShadowQuality` enum; replace `hide_shadows` with `shadow_quality` in `OcclusionConfig` |
| `src/systems/game/map/spawner/mod.rs` | Pass config to `spawn_lighting`; add `shadow_params_for_quality` helper; pass `shadow_quality` into `SpawnContext` |
| `src/systems/game/map/spawner/chunks.rs` | Conditionally insert `NotShadowCaster` at spawn time |
| `src/systems/game/map/spawner/shadow_quality.rs` | **New file** — `apply_shadow_quality_system` |
| `src/systems/game/mod.rs` or plugin | Register new system |
| `src/systems/settings/components.rs` | Add `ShadowQuality` to `SettingId` enum; remove `HideShadows` |
| `src/systems/settings/systems.rs` | Update `ALL_SETTINGS`, `format_value`, `adjust_value` |
| `settings.ron` | Replace `hide_shadows` field with `shadow_quality: Low` |

---

## Risks & Mitigations

| Risk | Mitigation |
|---|---|
| `NotShadowCaster` causes flat/wrong lighting | This only affects shadow *casting*, not light reception. Voxels are still lit by the directional and ambient lights. The "CharactersOnly" label sets correct expectations. |
| `CascadeShadowConfig` mutation may have a one-frame lag | Bevy applies component mutations the same frame via change detection; no lag expected. |
| Hot-reload leaves stale `NotShadowCaster` on old entities | Entities are fully despawned on hot-reload; new entities get the correct state from spawn + the Added<VoxelChunk> system. |
| Existing `settings.ron` breaks | RON ignores unknown fields by default; `shadow_quality` defaults to `Low` when absent. |
