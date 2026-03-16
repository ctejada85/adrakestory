# Depth Prepass — Reduce Fragment Overdraw on OcclusionMaterial Chunks

**Date:** 2026-03-16
**Severity:** Medium
**Component:** Rendering / OcclusionMaterial / Game Camera

---

## Story

As a player, I want the game to maintain smooth frame rates when many voxels are visible so that gameplay feels responsive in dense areas of the map.

---

## Description

The `OcclusionMaterial` WGSL fragment shader uses `discard` for dithered transparency. This
disables the GPU's hardware early-Z optimisation, causing the expensive PBR + occlusion shader to
run for every overlapping fragment — not just the nearest visible one. Benchmark data shows frame
time grows ~2× when moving into dense areas even with shadows off, consistent with 3–5× overdraw.

Adding Bevy's built-in `DepthPrepass` marker component to the 3D game camera activates a
depth-only pass before the main forward pass. The depth buffer is pre-populated, so occluded
fragments fail the hardware depth test before the fragment shader executes. The WGSL shader already
contains a `#ifdef PREPASS_PIPELINE` branch; no shader changes are required for Phase 1.

Out of scope: custom prepass fragment shader for interior-region discard (Phase 2), shader LOD
reduction, and draw-call batching.

---

## Acceptance Criteria

1. `DepthPrepass` is present in the game camera's spawn bundle in `spawn_camera()`.
2. The visual output with `DepthPrepass` enabled is indistinguishable from the current output: dithered fade, region-hide occlusion, and standard opaque rendering all appear unchanged.
3. No z-fighting artefacts appear on chunk surfaces when viewed at normal gameplay distances.
4. When the map is hot-reloaded, the new camera entity retains `DepthPrepass` automatically (it is part of the spawn tuple, not inserted by a separate system).
5. The editor camera (`EditorCamera` entity in the map editor binary) is not affected by this change.
6. The game compiles without warnings on macOS.

---

## Non-Functional Requirements

- Must not introduce a new Bevy plugin or crate dependency; `DepthPrepass` is from `bevy::core_pipeline::prepass`.
- Must not alter the existing shadow quality behaviour or `ShadowQuality` setting.
- The change is limited to the game binary's `spawn_camera()` function; the editor binary (`src/bin/map_editor/`) must remain untouched.
- Average FPS in a simple open-sky scene must not regress (depth prepass overhead must be lower than the fragment culling gain for typical overdraw).

---

## Tasks

1. Add `use bevy::core_pipeline::prepass::DepthPrepass;` import to `src/systems/game/map/spawner/mod.rs`.
2. Insert `DepthPrepass` into the `commands.spawn(...)` tuple in `spawn_camera()` alongside `Camera3d` and `GameCamera`.
3. Add `prepass_fragment_shader()` override to `OcclusionExtension` in `src/systems/game/occlusion/mod.rs`, returning `"shaders/occlusion_material.wgsl"` — the same WGSL used by `fragment_shader()` and `deferred_fragment_shader()`.
4. Build and run the game; visually verify dithered occlusion, region-hide, and opaque rendering are unchanged — specifically that geometry is visible through dither holes (not culled).
5. Confirm no z-fighting on chunk surfaces at close range and at LOD transition distances.
6. Hot-reload the map (Ctrl+R) and confirm the new camera also has correct rendering.
7. Run `cargo build --release` and confirm zero warnings.
8. Run `cargo test --lib` and confirm all existing tests pass.
9. Capture a benchmark session in a dense area with the `FrameProfiler` (debug build) and compare `frame_interval_us` p95 against the pre-change baseline in `profile_1773690261.csv`.
