# Requirements — Fix Occlusion Shader Fragment Early-Exit

**Source:** Bug report `docs/bugs/2026-03-16-1213-p3-occlusion-fragment-shader-all-visible-chunks.md`  
**Status:** Draft  

---

## 1. Overview

The `OcclusionMaterial` fragment shader (`occlusion_material.wgsl`) runs `calculate_occlusion_alpha()` for every visible fragment in `ShaderBased` mode. Inside that function, a `normalize()` (division + square root) and `point_to_ray_distance_xz()` (dot product + square root) are executed before the radius early-exit check fires. Fragments that are geometrically far from the player in XZ will always exit at the radius check — but only after paying the cost of both expensive operations.

The shader already contains a correct height early-exit at the top of `calculate_occlusion_alpha()` (added in the P1 fix). The remaining gap is an **XZ proximity pre-check** that can exit before the `normalize` and ray-distance calls using only a squared-distance comparison (no square root). On Apple Silicon GPUs — which have fewer execution units than discrete GPUs — `sqrt` and `normalize` per fragment become bottlenecks faster as fragment count scales with voxel density.

---

## 2. Functional Requirements

### 2.1 Shader Early-Exit

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-1 | `calculate_occlusion_alpha()` must return `1.0` immediately when the fragment's squared XZ distance from the player position exceeds `(occlusion_radius * XZ_MARGIN_FACTOR)²`, where `XZ_MARGIN_FACTOR` is a constant `≥ 1.5`. | 1 |
| FR-2 | The XZ pre-check must be evaluated before any call to `normalize()` or `point_to_ray_distance_xz()`. | 1 |
| FR-3 | The check must use squared distance (dot product only) to avoid an additional square root. | 1 |
| FR-4 | The existing height early-exit (line ~88, `fragment_y <= player_y + height_threshold`) must remain first in execution order; the XZ check follows it. | 1 |
| FR-5 | Occlusion visual output must be identical for fragments within the occlusion zone (no change to the visible transparency effect). | 1 |

### 2.2 Correctness Constraint

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-6 | The XZ margin factor must be large enough that no fragment within `occlusion_radius` of the camera-to-player ray is incorrectly returned as fully opaque. Given that the default `occlusion_radius` is `3.0` and camera arm length is typically `≤ 10` world units, a factor of `2.0` provides sufficient margin. | 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Phase |
|----|-------------|-------|
| NFR-1 | The fix is confined to `assets/shaders/occlusion_material.wgsl`; no Rust source files are modified. | 1 |
| NFR-2 | The constant `XZ_MARGIN_FACTOR` must be a `const` in the shader, not a magic number. | 1 |
| NFR-3 | No new shader uniforms or bindings are added; the check uses only `occlusion.player_position` and `occlusion.occlusion_radius`, both already available. | 1 |

---

## 4. Out of Scope

- **Option C — Material split by chunk Y:** Assigning `StandardMaterial` to chunks entirely below the player so they never run the shader is a valid long-term optimisation but requires changes to the spawner and a runtime material-swap mechanism on player movement. Not in scope for this fix.
- **Prepass pipeline:** The shader has a `PREPASS_PIPELINE` branch; the early-exit already fires before `pbr_input_from_standard_material` in the main pass, so no prepass changes are needed.
- **Mode 0 (None) / Mode 2 (RegionBased):** The `calculate_occlusion_alpha` call is already guarded by `occlusion.mode == 1u`. This fix is inside that function; modes 0, 2, and 3 are unaffected.

---

## 5. Open Questions

| ID | Question | Status |
|----|----------|--------|
| Q1 | Is `XZ_MARGIN_FACTOR = 2.0` the right constant, or should it be configurable via a uniform? | ✅ Hardcoded constant — the margin is geometry-driven, not user-tunable |
| Q2 | Should the XZ pre-check also account for camera XZ position (to handle extreme low-angle cameras)? | ✅ No — a 2× radius margin already covers the default camera arm length; the check is conservative |
