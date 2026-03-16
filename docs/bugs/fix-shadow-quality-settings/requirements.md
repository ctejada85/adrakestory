# Requirements: Configurable Shadow Quality

**Date:** 2026-03-16  
**Priority:** p2  
**Status:** Draft  
**Component:** Rendering / Settings

---

## Problem Statement

The `DirectionalLight` is configured with `shadows_enabled: true` and 4 cascade shadow maps
covering up to 100 world units. Every `VoxelChunk` entity (10+ meshes) is rendered into all
4 shadow depth passes every frame, regardless of whether the chunks are visible to the camera.
This is the dominant GPU bottleneck: profiling shows **93.2% of frame time is non-CPU**
(GPU + driver + vsync), with p95 frame spikes reaching 37.9ms (~26 fps).

There is currently no way for users or developers to trade shadow visual quality for
performance. A single shadow mode is hard-coded at spawn time.

---

## Goals

- Expose 4 configurable shadow quality levels covering the full range from no shadows
  to the current full-quality cascade setup.
- Persist the chosen level in `settings.ron` alongside existing occlusion settings.
- Apply the setting at map spawn time and dynamically when changed via the settings menu.
- Introduce no new binary size or runtime overhead in release builds.

---

## Functional Requirements

### FR-1: `ShadowQuality` Enum

A new `ShadowQuality` enum with 4 variants:

| Variant | Label | Description |
|---|---|---|
| `None` | "Off" | `shadows_enabled: false`. No shadow passes. |
| `CharactersOnly` | "Characters" | Voxel chunks get `NotShadowCaster`. Only player/NPC meshes cast shadows onto voxels. |
| `Low` | "Low" | 2 cascades, max distance 20 units. Short-range voxel shadows. |
| `High` | "High" | 4 cascades, max distance 100 units. Current default behavior. |

The enum must derive `Serialize`, `Deserialize`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`,
`Default` (default = `Low`).

### FR-2: Settings Field

`OcclusionConfig` gains a new field:

```rust
pub shadow_quality: ShadowQuality,  // default: ShadowQuality::Low
```

This field is serialized to `settings.ron`. If the field is absent in an existing file
(backward compatibility), `ron::from_str` falls back to `Default::default()` → `Low`.

> **Default rationale:** `Low` (2 cascades, 20 units) fixes the p95 spike clusters observed
> in profiling while preserving nearby shadow detail. Users who want the original look can
> select `High`.

### FR-3: Settings Menu Entry

A new row is added to the settings menu:

- **Label:** `"Shadow Quality"`
- **Values cycle:** Off → Characters → Low → High (wrap-around, left/right input)
- **Position:** After `"Hide Shadows"` (existing row), replacing it or adjacent to it.
- `SettingId::ShadowQuality` variant added to the `SettingId` enum.
- `ALL_SETTINGS` array updated with the new entry.
- `SelectedSettingsIndex::total` incremented by 1.

### FR-4: Spawn-Time Application

`spawn_lighting()` in `src/systems/game/map/spawner/mod.rs` receives `OcclusionConfig` and
applies shadow settings at spawn:

- **None / CharactersOnly / Low / High** sets `shadows_enabled` and `CascadeShadowConfigBuilder`
  values as specified in FR-1.
- `spawn_voxels_chunked()` receives `shadow_quality` and conditionally inserts
  `NotShadowCaster` on each `VoxelChunk` entity when `shadow_quality == CharactersOnly`.

### FR-5: Runtime Application on Setting Change

A new system `apply_shadow_quality_system` runs in `GameSystemSet::Visual` when the game is
`InGame` or `Paused`. It:

1. Returns early if `OcclusionConfig` has not changed (`!config.is_changed()`).
2. Queries `DirectionalLight` + `CascadeShadowConfig` components and updates them in-place.
3. Queries all `VoxelChunk` entities and adds or removes `NotShadowCaster` via `Commands`.
4. Also triggers on `Added<VoxelChunk>` to handle hot-reload re-spawns without requiring a
   full settings-menu round-trip.

### FR-6: `hide_shadows` Field Removal (Cleanup)

The existing `hide_shadows: bool` field on `OcclusionConfig` was marked "future use" and never
wired to any real behavior. It is replaced by `shadow_quality` and removed from the struct,
settings file, and menu. `settings.ron` files containing the old field will still deserialize
correctly (RON ignores unknown fields by default with `#[serde(default)]`).

---

## Non-Functional Requirements

### NFR-1: Performance Budget

Expected GPU improvement per level (relative to `High`):

| Level | Est. shadow pass savings | Expected p95 frame time |
|---|---|---|
| `None` | ~100% | <10ms |
| `CharactersOnly` | ~90% (no chunk-mesh shadow draws) | <10ms |
| `Low` | ~60% (2× fewer passes, 20% the distance³ volume) | <15ms |
| `High` | 0% (current) | ~38ms p95 (profiled) |

### NFR-2: Backward Compatibility

Existing `settings.ron` files without `shadow_quality` field must load without error and
default to `Low`.

### NFR-3: Debug Build Only

`apply_shadow_quality_system` uses no debug-only code; it runs in both debug and release.
The `FrameProfiler` param is `Option<Res<FrameProfiler>>` (unchanged pattern).

### NFR-4: No Mesh Recreation

Changing shadow quality at runtime must not despawn or respawn `VoxelChunk` meshes. Only
`NotShadowCaster` component insertion/removal and `DirectionalLight` field mutation are
allowed.

---

## Out of Scope

- Point light or spot light shadow configuration.
- Per-chunk shadow LOD (shadows fade beyond distance threshold).
- PCSS (Percentage Closer Soft Shadows) or other filter modes.
- Ambient occlusion baking.

---

## Cascade Configuration Reference

| Quality | `shadows_enabled` | `num_cascades` | `first_cascade_far_bound` | `maximum_distance` | `NotShadowCaster` on chunks |
|---|---|---|---|---|---|
| None | `false` | — | — | — | No |
| CharactersOnly | `true` | 2 | 4.0 | 20.0 | **Yes** |
| Low | `true` | 2 | 4.0 | 20.0 | No |
| High | `true` | 4 | 4.0 | 100.0 | No |

> `CharactersOnly` uses the same cascade params as `Low` to keep the shadow map alive for
> character shadows, but adds `NotShadowCaster` so chunk meshes are excluded from shadow
> draw calls.

---

## Open Questions

- **Q1:** Should `High` be retained as an option at all, or is `Low` the new "High"?  
  *Current answer: Yes — `High` is kept for users who want the original visual fidelity
  and have capable hardware.*
