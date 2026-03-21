# Epic — Upgrade Bevy 0.15 → 0.18

**Date:** 2026-03-21
**Type:** Epic
**Component:** All — `Cargo.toml`, `src/`, `assets/shaders/`, `src/editor/`

---

## Overview

The project currently runs Bevy 0.15.3. Bevy 0.18.1 is the current stable release. Three intermediate versions introduced breaking changes in import paths, material trait API, and the `bevy_egui` crate. The upgrade also implements the occlusion shadow-casting fix: the prepass shader currently has unconditional discards that must be guarded with the view projection matrix check (Option B). This epic migrates the game, map editor, and custom shaders in four phased stories and validates the result.

---

## Epic Story

As a developer, I want to upgrade the game and map editor from Bevy 0.15 to Bevy 0.18 so that the project stays on a supported engine version, benefits from upstream improvements, and eliminates version-fragile workarounds in the occlusion prepass shader.

---

## Child Stories

| # | Title | Status |
|---|-------|--------|
| 1 | Bump dependencies and triage compile errors | Backlog |
| 2 | Migrate Rust code to Bevy 0.18 API | Backlog |
| 3 | Migrate shaders and apply shadow fix | Backlog |
| 4 | Migrate bevy_egui and validate map editor | Backlog |

---

## Story 1 — Bump dependencies and triage compile errors

As a developer, I want to update `Cargo.toml` to Bevy 0.18 and `bevy_egui` 0.39 so that the full set of breaking changes is visible and can be triaged before any code is changed.

### Description

Update `bevy` and `bevy_egui` version pins in `Cargo.toml` and run `cargo build` to surface the first wave of compile errors. Produce a triage list grouping errors by category (import paths, camera, material, editor UI). This story is complete when errors are fully catalogued — no code changes yet.

### Acceptance Criteria

1. `Cargo.toml` specifies `bevy = "0.18"` and `bevy_egui = "0.39"`.
2. `cargo build` output is captured and every error is assigned to a story (Story 2, 3, or 4).
3. A brief triage comment is added in the PR / commit body listing error counts per category.
4. No functional code is changed in this story — only `Cargo.toml`.

### Non-Functional Requirements

- Triage must distinguish `bevy_render` reorganization errors from other errors to scope Story 2.

### Tasks

1. Update `bevy = "0.18.1"` and `bevy_egui = "0.39"` in `Cargo.toml`.
2. Run `cargo build 2>&1 | tee /tmp/bevy018-errors.txt`.
3. Review errors and label each as: import-path / camera-spawn / material-api / egui / shader.
4. Post error counts in commit message.

---

## Story 2 — Migrate Rust code to Bevy 0.18 API

As a developer, I want all Rust source files to compile against Bevy 0.18 so that the game binary builds without errors.

### Description

Apply the breaking changes identified in Story 1 to `src/`. Three sets of changes are expected: (a) import path updates from the `bevy_render` 0.16→0.17 reorganization; (b) camera spawn update — `RenderTarget` is now a separate required component in 0.17→0.18; (c) material API update — `enable_prepass`/`enable_shadows` move from `MaterialPlugin` fields to `Material` trait methods. Editor files are out of scope for this story (covered in Story 4).

### Acceptance Criteria

1. `cargo build --bin adrakestory` produces zero errors.
2. All `NotShadowCaster` import paths resolve correctly.
3. Camera spawning compiles with the `RenderTarget` component pattern.
4. `MaterialPlugin::<OcclusionMaterial>` compiles with the new trait-method API.
5. `AlphaMode::Mask` and `AlphaMode::AlphaToCoverage` variants still compile.
6. All existing tests pass (`cargo test`).

### Non-Functional Requirements

- No new `unsafe` blocks introduced.
- `weak_from_u128()` / `weak_handle!`: verified not present in the codebase — no action needed for existing code.
- Changes limited to `src/` and `Cargo.toml` — no editor files.

### Tasks

1. Fix all import path errors from the `bevy_render` reorganization (light, camera, mesh, shader types).
2. Update `NotShadowCaster` import in `shadow_quality.rs` and `chunks.rs`.
3. Update camera spawn in `spawner/mod.rs` and `setup.rs` for the `RenderTarget` component.
4. Verify `DirectionalLight`, `AmbientLight`, `CascadeShadowConfigBuilder` field names; fix any renames.
5. Migrate `MaterialPlugin::<OcclusionMaterial>` to the `Material` trait method pattern.
6. `weak_from_u128()` / `weak_handle!`: verified not used — no action needed.
7. Run `cargo build --bin adrakestory` — must be error-free.
8. Run `cargo test` — all tests must pass.

---

## Story 3 — Migrate shaders and apply shadow fix

As a developer, I want the occlusion shaders to compile on Bevy 0.18 and the shadow-casting bug to be fixed so that ceiling voxels cast correct shadows onto the floor when the player is inside a building.

### Description

Verify all `bevy_pbr::` import paths in `occlusion_material.wgsl`. In `occlusion_material_prepass.wgsl`, implement the version-agnostic shadow fix: wrap both discard blocks in a `view.projection[3][3]` orthographic check (Option B from `docs/bugs/fix-occlusion-shadow-casting/references/architecture.md`). Note: `DEPTH_CLAMP_ORTHO` is not present in the current shader — no removal needed.

### Acceptance Criteria

1. `cargo build` (including asset shader compilation) produces zero shader errors.
2. At runtime, `occlusion_material.wgsl` compiles with no Bevy shader error logs.
3. With occlusion active, ceiling voxels are invisible to the camera but cast a visible directional shadow on the floor.
4. Occlusion discard (height-based and region-based) still works correctly for the player camera.
5. No stray `DEPTH_CLAMP_ORTHO` or `UNCLIPPED_DEPTH_ORTHO` references added during migration (neither exists in the current shader — do not introduce them).

### Non-Functional Requirements

- No Rust changes required; changes limited to `.wgsl` files under `assets/shaders/`.
- The projection check comment in the shader must explain the orthographic = shadow pass reasoning and the point/spot light caveat.

### Tasks

1. Audit `occlusion_material.wgsl` for any `bevy_pbr::` import paths that moved in 0.17–0.18.
2. In `occlusion_material_prepass.wgsl`: add `#import bevy_render::view::View` and `@group(0) @binding(0) var<uniform> view: View;`.
3. Add `let is_shadow_pass = view.projection[3][3] >= 0.5;` and wrap both discard blocks in `if !is_shadow_pass { }`.
4. Add shadow fix imports and guard (see architecture doc) — no existing define references need removal.
5. Run game in a map with directional light; verify floor shadow from ceiling voxels appears.
6. Verify occlusion still hides ceiling voxels from the camera.

---

## Story 4 — Migrate bevy_egui and validate map editor

As a map editor user, I want the map editor to start and all tools and UI panels to work correctly on Bevy 0.18 so that I can continue editing maps without regression.

### Description

Update all editor files under `src/editor/` for the `bevy_egui` 0.31 → 0.39 API. Changes may include `EguiPlugin` registration, `EguiContexts` API, and any other breaking changes listed in the bevy_egui CHANGELOG. Validate the full editor flow: start, open map, use all tools, hot-reload, save.

### Acceptance Criteria

1. `cargo build --bin map_editor` produces zero errors.
2. Map editor starts and main window renders.
3. All editor tool panels render without visual glitches.
4. Voxel painting, selection, and placement tools work.
5. Hot-reload (F5) triggers map reload and player position is preserved.
6. Editor save writes a valid `.ron` map that the game can load.
7. All existing tests pass.

### Non-Functional Requirements

- Changes limited to `src/editor/` and `Cargo.toml`.
- No changes to the editor's `EditorHistory` logic — undo/redo must still work.

### Tasks

1. Read bevy_egui CHANGELOG for 0.31 → 0.39 breaking changes.
2. Update `EguiPlugin` registration in `src/bin/map_editor.rs`.
3. Fix any `EguiContexts` / `EguiContext` API changes across all editor files.
4. Fix any other bevy_egui API changes (widget helpers, input handling, etc.).
5. Run `cargo build --bin map_editor` — must be error-free.
6. Manually exercise all editor tools and panels.
7. Test hot-reload with F5.
8. Run `cargo test`.

---

## Epic Acceptance Criteria

1. All four child stories are complete and individually verified.
2. `cargo build` compiles with zero errors and zero new deprecation warnings.
3. `cargo test` — all existing tests pass.
4. Game binary completes the full state machine: Intro → Title → Loading → InGame → Paused.
5. Voxel occlusion works: ceiling voxels hidden from camera, floor lit by correct directional shadow.
6. Map editor starts, all tools function, and hot-reload works.
7. Frame time is within 5% of the Bevy 0.15 baseline.
8. No `.ron` map files require changes.

---

## Epic Non-Functional Requirements

- No new `unsafe` blocks introduced during migration.
- At least one commit per story for rollback granularity.
- All changes must stay within the scope of migration — no new Bevy features adopted.

---

## Dependencies & Risks

| # | Item | Type | Status | Notes |
|---|------|------|--------|-------|
| 1 | Bevy 0.16, 0.17, 0.18 migration guides reviewed | Dependency | Done | Guides consulted during architecture phase |
| 2 | `fix-occlusion-shadow-casting` architecture (Option B) | Dependency | Done | Implemented in Story 3 |
| 3 | bevy_egui 0.31 → 0.39 CHANGELOG | Dependency | Pending | Must be read before Story 4 |
| 4 | `bevy_render` crate reorganization may move more types than documented | Risk | Open | Mitigation: triage step in Story 1 catches all errors before code changes |
| 5 | `bevy_egui` 0.39 may have large API surface changes across 25+ editor files | Risk | Open | Mitigation: Story 4 scoped separately; editor binary builds independently |
