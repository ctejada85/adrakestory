# Fix Occlusion Shader Fragment Early-Exit

**Date:** 2026-03-16  
**Severity:** Medium (P3)  
**Component:** Occlusion system / Shader  

---

## Story

As a player on macOS, I want the occlusion shader to skip expensive GPU math for fragments that cannot be affected by occlusion so that FPS does not drop when many voxel chunks are on screen.

---

## Description

`calculate_occlusion_alpha()` in `occlusion_material.wgsl` performs a `normalize()` (sqrt + divisions) and `point_to_ray_distance_xz()` (dot + sqrt) for every visible fragment before the radius early-exit fires. Fragments far from the player in XZ always exit at the radius check, but only after paying the full cost of both operations. The fix adds a squared-XZ-distance pre-check immediately after the existing height early-exit. Fragments whose squared XZ distance from the player exceeds `(occlusion_radius × 2.0)²` return `1.0` without reaching `normalize` or the ray distance calculation. This is a shader-only change; no Rust files are modified.

---

## Acceptance Criteria

1. After the change, the occlusion visual effect is identical to before for all fragments within the occlusion zone.
2. A fragment at XZ distance greater than `occlusion_radius × 2.0` from the player returns `1.0` from `calculate_occlusion_alpha()` without executing `normalize` or `point_to_ray_distance_xz`.
3. A fragment within `occlusion_radius` of the player in XZ and above `height_threshold` reaches the full `smoothstep` calculation path.
4. The height early-exit still fires before the XZ check (verified by code order).
5. The XZ check uses squared distance (`dot(xz_offset, xz_offset)`) — no additional `sqrt` or `length` call.
6. `XZ_MARGIN_FACTOR` is declared as a `const f32` at module scope, not as an inline magic number.
7. `cargo build --release` succeeds (shader is compiled as an asset; Rust build validates asset presence).

---

## Non-Functional Requirements

- The fix is confined to `assets/shaders/occlusion_material.wgsl`; no Rust source files are modified.
- No new shader uniforms, bindings, or material fields are added.
- The change must not affect fragment output for `ShaderBased` mode in any case where the fragment is within the occlusion zone.

---

## Tasks

1. Add `const XZ_MARGIN_FACTOR: f32 = 2.0;` at module scope in `occlusion_material.wgsl`, after the Bayer matrix constant.
2. In `calculate_occlusion_alpha()`, after the height early-exit, insert the XZ squared-distance pre-check (see `architecture.md` Appendix A for the exact code block).
3. Run `cargo build --release` to confirm the shader asset compiles cleanly.
4. Manually verify in-game that occlusion transparency appears unchanged: load a map, walk under a covered area, confirm voxels above the player become transparent as before.
5. Manually verify the optimisation path: stand in open terrain (no ceiling), observe that no occlusion effect appears (unchanged), and confirm using F3 FPS overlay that frame rate has not regressed.
