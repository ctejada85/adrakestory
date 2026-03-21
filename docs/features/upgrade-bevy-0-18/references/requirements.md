# Requirements — Upgrade Bevy 0.15 → 0.18

**Date:** 2026-03-21
**Status:** Draft
**Feature:** `upgrade-bevy-0-18`

---

## 1. Overview

Migrate the game and map editor from Bevy 0.15.3 to Bevy 0.18.1. The upgrade spans three intermediate versions (0.16, 0.17, 0.18) each with breaking changes. The migration must preserve all existing functionality with no user-visible regressions: gameplay, rendering, occlusion, physics, input, map loading, and the map editor.

The upgrade also resolves an open bug: the occlusion shadow-casting fix (`docs/bugs/fix-occlusion-shadow-casting/`) depends on shader behavior that changed in 0.18. The migration is the right time to implement the correct fix (Option B — view projection check) rather than patching 0.15.

---

## 2. Functional Requirements

### FR-1 — Dependency Updates

**FR-1.1** — `bevy` in `Cargo.toml` must be updated to `0.18`.
**FR-1.2** — `bevy_egui` must be updated to `0.39` (the version compatible with Bevy 0.18).
**FR-1.3** — After dependency update, `cargo build` must compile without errors or warnings about deprecated API usage.

---

### FR-2 — Rust Code Compatibility

**FR-2.1** — All camera spawning code must remain compatible.
- `Camera3d`, `Camera2d` component patterns are already in use; verify no new required components.
- `Camera { target: ... }` → `RenderTarget` is now a separate component (0.17→0.18 breaking change); update all camera spawns.

**FR-2.2** — All `StandardMaterial` usages must compile and render correctly.
- Field names (`base_color`, `metallic`, `roughness`, `alpha_mode`, etc.) must be verified against 0.18.
- `AlphaMode::Mask(0.001)` must still function as expected for the occlusion material.
- `AlphaMode::AlphaToCoverage` must still be supported.

**FR-2.3** — The `ExtendedMaterial` / `MaterialExtension` API must be verified and updated.
- `OcclusionMaterial = ExtendedMaterial<StandardMaterial, OcclusionExtension>` must remain valid.
- `impl MaterialExtension for OcclusionExtension` must implement all required methods for 0.18.
- Shader bind group indices (currently group 2, binding 100) must be verified.

**FR-2.4** — `MaterialPlugin::<OcclusionMaterial>` registration must use 0.18 API.
- `enable_prepass` and `enable_shadows` are now `Material`/`MaterialExtension` methods, not `MaterialPlugin` fields (0.17→0.18 change).
- If `MaterialPlugin` fields were in use, convert to trait methods.

**FR-2.5** — All light types must render correctly.
- `DirectionalLight`, `AmbientLight` field names must be verified (`illuminance`, `brightness`, `shadows_enabled`, etc.).
- `CascadeShadowConfigBuilder` API must be verified (currently used in 4 locations).
- `NotShadowCaster` component must still be available (types moved to `bevy_light` in 0.16→0.17; import via `bevy::pbr` or `bevy::light`).

**FR-2.6** — All import paths must be updated for the `bevy_render` reorganization (0.16→0.17).
- Camera types moved to `bevy_camera` (accessible via `bevy::camera`).
- Light types moved to `bevy_light` (accessible via `bevy::light` or `bevy::pbr`).
- `NotShadowCaster`, `NotShadowReceiver` moved to `bevy_light`.
- Mesh types moved to `bevy_mesh` (accessible via `bevy::mesh`).
- All affected imports must be updated to compile.

**FR-2.7** — `Mesh::new(topology, usage)` API must be verified; update if signature changed.

**FR-2.8** — `Handle::weak_from_u128()` is deprecated in 0.16. Replace any usages with the `weak_handle!` macro.

---

### FR-3 — WGSL Shader Compatibility

**FR-3.1** — `occlusion_material.wgsl` must compile without errors in Bevy 0.18.
- All `#import bevy_pbr::...` paths must exist in 0.18 (`pbr_fragment`, `pbr_functions`, `prepass_io`, `forward_io`, `pbr_deferred_functions`).
- `VertexOutput` and `FragmentOutput` struct layouts must be verified.
- Update any renamed or moved shader imports.

**FR-3.2** — `occlusion_material_prepass.wgsl` must compile and implement the shadow fix.
- `#import bevy_pbr::prepass_io::VertexOutput` must still be valid.
- `#import bevy_render::view::View` must be added.
- The shadow-pass detection guard (view projection matrix check) from `docs/bugs/fix-occlusion-shadow-casting/references/architecture.md` Option B must be implemented.
- The old `DEPTH_CLAMP_ORTHO` define (renamed to `UNCLIPPED_DEPTH_ORTHO_EMULATION` in 0.18) must be removed.

**FR-3.3** — Shader behavior must be functionally identical to the current 0.15 behavior for all non-shadow cases.

---

### FR-4 — Map Editor Compatibility

**FR-4.1** — The map editor binary (`cargo run --bin map_editor`) must start and render without errors.
**FR-4.2** — All `bevy_egui` usages must be updated for bevy_egui 0.39.
- `EguiPlugin` registration pattern.
- `EguiContexts` system parameter.
- Any UI context API calls used across the 25+ editor UI files.
**FR-4.3** — All editor tools (paint, erase, select, fill, etc.) must function correctly after migration.
**FR-4.4** — Map saving and loading from the editor must work correctly.
**FR-4.5** — Hot-reload (F5 / Ctrl+R) from the editor must work correctly.

---

### FR-5 — Game Binary Compatibility

**FR-5.1** — The game binary must start, display the intro animation, title screen, and load a map.
**FR-5.2** — Voxel rendering must be visually correct (greedy meshing, LOD switching, chunk culling).
**FR-5.3** — Occlusion transparency must work correctly: ceiling voxels invisible, floor voxels visible.
**FR-5.4** — Shadows from the directional light (sun) must appear correctly inside occluded rooms.
**FR-5.5** — Physics, collision, and player movement must function without regression.
**FR-5.6** — Input systems (keyboard, gamepad) must function correctly.
**FR-5.7** — All `GameState` transitions (Intro → Title → Loading → InGame → Paused) must work.
**FR-5.8** — FPS counter and debug overlays must render without errors.

---

## 3. Non-Functional Requirements

**NFR-1 — No performance regression.** Frame time at the standard map benchmark scene must not increase by more than 5% compared to the 0.15 baseline.

**NFR-2 — Incremental migration.** The migration must be verifiable as a compile/run step at each intermediate milestone (FR-1, FR-2, FR-3, FR-4, FR-5) before proceeding.

**NFR-3 — No new deprecation warnings.** The final build must produce zero `deprecated` warnings from the `bevy` or `bevy_egui` crates.

**NFR-4 — Preserved test suite.** All existing `cargo test` tests must pass after migration.

---

## 4. Out of Scope

- Adopting any new Bevy 0.16/0.17/0.18 features beyond what is required for migration.
- Migrating to the new `bevy_light` module paths as first-class imports (using `bevy::pbr` re-exports is acceptable if they still exist).
- Point and spot light shadow casting from occluded voxels (tracked separately in the shadow casting bug; currently `shadows_enabled: false`).
- Hybrid mode 3 shader-based occlusion re-enablement (separate ticket).

---

## 5. Assumptions & Dependencies

- **Bevy 0.18.1** is the target version (latest stable as of 2026-03-21).
- **bevy_egui 0.39.x** is the compatible version for Bevy 0.18.
- All three intermediate migration guides must be consulted:
  - `https://bevyengine.org/learn/migration-guides/0-15-to-0-16/`
  - `https://bevyengine.org/learn/migration-guides/0-16-to-0-17/`
  - `https://bevyengine.org/learn/migration-guides/0-17-to-0-18/`
- The shadow casting fix (Option B) is included in this migration as part of FR-3.2.

---

## 6. Open Questions

| # | Question | Owner |
|---|----------|-------|
| Q1 | Are `bevy::pbr` re-export paths for light types (`NotShadowCaster`, `DirectionalLight`, etc.) still available in 0.18, or must imports switch to `bevy::light`? | Verify during FR-2.6 |
| Q2 | Does `AlphaMode::AlphaToCoverage` still exist in 0.18, or was it renamed? | Verify during FR-2.2 |
| Q3 | Does `bevy_pbr::pbr_fragment::pbr_input_from_standard_material` still exist in 0.18 shaders? | Verify during FR-3.1 |
| Q4 | Does bevy_egui 0.39 require any changes to `EguiPlugin` registration or `EguiContexts` usage? | Verify during FR-4.2 |
