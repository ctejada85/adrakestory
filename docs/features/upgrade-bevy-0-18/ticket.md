# Epic — Upgrade Bevy 0.15 → 0.18

**Date:** 2026-03-21
**Type:** Epic / Migration
**Priority:** High
**Status:** Backlog

**References:**
- Requirements: `docs/features/upgrade-bevy-0-18/references/requirements.md`
- Architecture: `docs/features/upgrade-bevy-0-18/references/architecture.md`
- Shadow fix: `docs/bugs/fix-occlusion-shadow-casting/references/architecture.md` (Option B included in this epic)

---

## Story

As a developer, I want to upgrade the game and map editor from Bevy 0.15 to Bevy 0.18 so that the project stays on a supported version, benefits from engine improvements, and resolves the fragile `DEPTH_CLAMP_ORTHO` workaround in the occlusion prepass shader.

---

## Description

The project currently runs Bevy 0.15.3. Bevy 0.18.1 is the current stable release. Three intermediate versions (0.16, 0.17, 0.18) contain breaking changes that must be resolved.

The upgrade also fixes the open shadow-casting bug: the `DEPTH_CLAMP_ORTHO` define used in `occlusion_material_prepass.wgsl` was renamed in Bevy 0.18. This epic replaces it with the version-agnostic **view projection matrix check** (Option B from the shadow bug architecture document).

Key changes:

1. **Dependency bump** — `bevy 0.15` → `0.18`, `bevy_egui 0.31` → `0.39`
2. **Import path updates** — `bevy_render` reorganization (0.16→0.17) moved camera, light, mesh, and shader types to new crates; most are still re-exported via `bevy::pbr` / `bevy::prelude`
3. **Camera spawn update** — `RenderTarget` is now a separate required component (0.17→0.18)
4. **Material API update** — `enable_prepass`/`enable_shadows` moved from `MaterialPlugin` fields to `Material` trait methods (0.17→0.18)
5. **Occlusion prepass shader** — Replace `DEPTH_CLAMP_ORTHO` guard with `view.projection[3][3]` orthographic check; add `#import bevy_render::view::View`
6. **Main occlusion shader** — Verify all `bevy_pbr` import paths still resolve in 0.18
7. **bevy_egui migration** — Update to 0.39 and resolve any API changes across 25+ editor files
8. **Light and shadow config** — Verify `DirectionalLight`, `AmbientLight`, `CascadeShadowConfigBuilder` field names unchanged

---

## Acceptance Criteria

- [ ] `cargo build` compiles with zero errors and zero deprecation warnings on Bevy 0.18
- [ ] `cargo test` — all existing tests pass
- [ ] Game binary starts and all `GameState` transitions work (Intro → Title → Loading → InGame → Paused)
- [ ] Voxels render correctly; occlusion transparency works (ceiling voxels hidden, floor visible)
- [ ] Directional light (sun) shadow appears on the floor inside occluded rooms (**shadow fix included**)
- [ ] Map editor starts; all tools and UI panels render correctly
- [ ] Hot-reload (F5) works from the editor
- [ ] Input systems (keyboard, gamepad) function correctly
- [ ] Frame time within 5% of Bevy 0.15 baseline

---

## Non-Functional Requirements

- No new Rust `unsafe` blocks introduced as part of migration
- All changes committed per phase (one commit per phase minimum) for rollback granularity
- No existing `.ron` map files require changes

---

## Tasks

### Phase 1 — Dependencies
- [ ] Update `bevy = "0.18"` and `bevy_egui = "0.39"` in `Cargo.toml`
- [ ] Run `cargo build`; collect and triage all compile errors

### Phase 2 — Rust Code
- [ ] Fix all import path errors from `bevy_render` reorganization (light, camera, mesh types)
- [ ] Update `NotShadowCaster` import in `shadow_quality.rs` and `chunks.rs`
- [ ] Update camera spawn in `spawner/mod.rs` and `setup.rs` if `RenderTarget` component required
- [ ] Verify `DirectionalLight`, `AmbientLight`, `CascadeShadowConfigBuilder` field names; fix any renames
- [ ] Verify `MaterialPlugin::<OcclusionMaterial>` registration (check `enable_prepass`/`enable_shadows` method migration)
- [ ] Verify `AlphaMode::Mask(0.001)` and `AlphaMode::AlphaToCoverage` still exist
- [ ] Replace any `Handle::weak_from_u128()` calls with `weak_handle!` macro
- [ ] Run `cargo build` — must be error-free before proceeding to shaders

### Phase 3 — Shaders
- [ ] Verify all `bevy_pbr::` import paths in `occlusion_material.wgsl`; update any that moved
- [ ] Implement shadow fix in `occlusion_material_prepass.wgsl`:
  - Add `#import bevy_render::view::View`
  - Add `@group(0) @binding(0) var<uniform> view: View;`
  - Wrap both discard blocks in `if !is_shadow_pass { }` using `view.projection[3][3] >= 0.5`
  - Remove any reference to `DEPTH_CLAMP_ORTHO` or `UNCLIPPED_DEPTH_ORTHO`
- [ ] Run game; verify shaders compile (check Bevy's runtime shader error logs)

### Phase 4 — bevy_egui
- [ ] Consult bevy_egui CHANGELOG for 0.31 → 0.39 breaking changes
- [ ] Update `EguiPlugin` registration in `map_editor/main.rs`
- [ ] Fix any `EguiContexts` API changes across all editor files
- [ ] Run map editor; exercise all tools and panels

### Phase 5 — Validation
- [ ] Run full acceptance criteria checklist
- [ ] Benchmark frame time vs Bevy 0.15 baseline
- [ ] Run `cargo test`; all tests pass
- [ ] Clean up any remaining deprecation warnings

---

## Out of Scope

- Adopting new Bevy 0.16/0.17/0.18 features beyond migration requirements
- Fixing point/spot light shadow casting from occluded voxels (separate bug)
- Re-enabling Hybrid mode shader-based occlusion (separate ticket)
