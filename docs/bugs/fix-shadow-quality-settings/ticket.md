# Configurable Shadow Quality Setting

**Date:** 2026-03-16  
**Severity:** High  
**Component:** Rendering / Settings

---

## Story

As a player, I want to choose my shadow quality level in the settings menu so that I can trade visual fidelity for performance on my hardware.

---

## Description

All VoxelChunk meshes are rendered into 4 directional-light shadow cascade passes every frame, consuming ~93% of frame time (GPU-bound). There is no user-facing control — shadow quality is hard-coded at map spawn. This feature introduces a `ShadowQuality` enum with four levels (Off, Characters, Low, High), adds it to `OcclusionConfig` (persisted to `settings.ron`), exposes it in the settings menu, and applies it both at spawn time and dynamically at runtime. The existing `hide_shadows` stub field is removed as part of this work. Shadow filter modes, point/spot light shadows, and ambient occlusion are out of scope.

---

## Acceptance Criteria

1. The settings menu displays a "Shadow Quality" row whose value cycles Off → Characters → Low → High (and back) when the player presses left/right.
2. Selecting **Off** disables all shadows: `DirectionalLight.shadows_enabled` is `false` and no shadow depth passes are submitted to the GPU.
3. Selecting **Characters** adds `NotShadowCaster` to every `VoxelChunk` entity; player and NPC meshes continue to cast shadows onto the voxel world.
4. Selecting **Low** configures 2 shadow cascades with a 20-unit maximum distance; no `NotShadowCaster` is added to chunks.
5. Selecting **High** configures 4 shadow cascades with a 100-unit maximum distance (original behavior); no `NotShadowCaster` is added to chunks.
6. Changing shadow quality at runtime (while in-game or paused) takes effect within one frame without despawning or re-spawning any VoxelChunk mesh entity.
7. The chosen level is saved to `settings.ron` when the player exits the settings menu and restored correctly on the next game launch.
8. An existing `settings.ron` that contains `hide_shadows` but no `shadow_quality` loads without error and defaults to `Low`.
9. After a hot-reload, newly spawned VoxelChunk entities reflect the current shadow quality without requiring a settings-menu round-trip.
10. The default shadow quality for a fresh install (no `settings.ron`) is `Low`.
11. All existing tests continue to pass.

---

## Non-Functional Requirements

- Changing shadow quality must not cause a visible frame hitch longer than one frame (no mesh rebuilds, no asset reloads).
- `apply_shadow_quality_system` must early-return (O(1)) when neither `OcclusionConfig` has changed nor any `VoxelChunk` has been added that frame.
- The `ShadowQuality` enum and all shadow-related logic must compile and run in both debug and release builds.
- The `SelectedSettingsIndex::total` count must remain accurate after the `HideShadows` → `ShadowQuality` replacement (net change: zero new rows).
- No new `unsafe` code is introduced.

---

## Tasks

1. Add `ShadowQuality` enum (derive `Default = Low`, `Serialize`, `Deserialize`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`) to `src/systems/game/occlusion/mod.rs`.
2. Replace `hide_shadows: bool` with `shadow_quality: ShadowQuality` in `OcclusionConfig` struct and its `Default` impl.
3. Add `shadow_params_for_quality(ShadowQuality) -> (bool, CascadeShadowConfig)` helper function in `src/systems/game/map/spawner/mod.rs`.
4. Update `spawn_lighting()` to accept `&OcclusionConfig` and call `shadow_params_for_quality` to set `shadows_enabled` and `CascadeShadowConfig` at spawn time.
5. Add `shadow_quality: ShadowQuality` to `SpawnContext` in `src/systems/game/map/spawner/chunks.rs`; conditionally insert `NotShadowCaster` on each `VoxelChunk` entity when quality is `CharactersOnly`.
6. Create `src/systems/game/map/spawner/shadow_quality.rs` with `apply_shadow_quality_system` (change-detected on `OcclusionConfig` + `Added<VoxelChunk>`; mutates `DirectionalLight`, `CascadeShadowConfig`, and adds/removes `NotShadowCaster` on all chunks).
7. Register `apply_shadow_quality_system` in `GameSystemSet::Visual` gated to `InGame | Paused`.
8. Replace `SettingId::HideShadows` with `SettingId::ShadowQuality` in `src/systems/settings/components.rs`.
9. Update `ALL_SETTINGS`, `format_value`, and `adjust_value` in `src/systems/settings/systems.rs` for the new `ShadowQuality` variant.
10. Remove all references to `hide_shadows` (struct field, `Default` impl, settings menu arms, `settings.ron`); update `settings.ron` to use `shadow_quality: Low`.
11. Run `cargo test` and `cargo clippy` — confirm zero errors and all tests pass.
12. Manual verification: launch the game, open settings, cycle through all four shadow quality levels, observe shadow changes in-game, quit and relaunch to confirm persistence.
