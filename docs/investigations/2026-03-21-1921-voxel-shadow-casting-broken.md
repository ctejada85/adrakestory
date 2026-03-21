# Investigation: Voxels Not Casting Shadows

**Date:** 2026-03-21 19:21
**Status:** Complete — Fixed
**Component:** Occlusion material prepass shader

## Summary

Voxels rendered via `OcclusionMaterial` were not casting any shadows despite
the directional light having shadows enabled and `ShadowQuality::High` being
configured. The root cause was a WGSL field name rename introduced in Bevy
0.18 that caused the custom prepass shader to fail pipeline compilation
silently, suppressing shadow rendering for all voxel entities.

## Environment

- Platform: macOS (also reproduced on Windows)
- Bevy: 0.18
- Shader: `assets/shaders/occlusion_material_prepass.wgsl`
- Config: `settings.ron` — `shadow_quality: High`, `technique: Dithered`

## Investigation Method

1. Verified `DirectionalLight` configuration: `shadows_enabled: true`, 4
   cascades, 100-unit range — correct.
2. Checked `NotShadowCaster` insertion: only inserted for
   `ShadowQuality::CharactersOnly`, not `High` — correct.
3. Inspected `OcclusionMaterial` alpha mode: `AlphaMode::Mask(0.001)` for
   Dithered — shadow-compatible, correct.
4. Read `occlusion_material_prepass.wgsl` line 79 — found `view.projection[3][3]`.
5. Cross-referenced Bevy 0.18 `crates/bevy_render/src/view/view.wgsl`: the
   `View` struct field was renamed `projection` → `clip_from_view` and the
   Bevy source even includes a comment:
   > `clip_from_view[3][3] == 1.0 is the standard way to check if a projection is orthographic`
6. Confirmed: accessing a non-existent field in WGSL causes the shader
   module compilation to fail. Bevy's pipeline cache silently skips shadow
   pass pipeline creation for the material, resulting in no shadows.

## Findings

### Finding 1 — Invalid WGSL field `view.projection` (p1 Critical)

**File:** `assets/shaders/occlusion_material_prepass.wgsl`, line 79
**Previous code:**
```wgsl
let is_shadow_pass = view.projection[3][3] >= 0.5;
```
**Fixed code:**
```wgsl
// clip_from_view[3][3] >= 0.5 detects orthographic projection (shadow map pass).
// Field renamed projection -> clip_from_view in Bevy 0.18's View struct.
let is_shadow_pass = view.clip_from_view[3][3] >= 0.5;
```

**Why it breaks silently:** Bevy's `PipelineCache::process_pipeline_queue_system`
catches pipeline compilation errors and marks the pipeline as failed, but does
not panic or log a user-visible error for material prepass pipelines. The main
forward pass pipeline is separate and unaffected — voxels render normally but
cast no shadows.

**Why the value works:** Orthographic projection matrices (used for directional
light shadow maps) have `M[3][3] = 1.0`. Perspective projection matrices (used
for the player camera) have `M[3][3] = 0.0`. The threshold `>= 0.5` reliably
distinguishes them across all projection parameters.

## Root Cause Summary

| # | Root Cause | Location | Priority | Severity | Notes |
|---|-----------|----------|----------|----------|-------|
| 1 | `view.projection` renamed to `view.clip_from_view` in Bevy 0.18 | `occlusion_material_prepass.wgsl:79` | p1 | Critical | Silent pipeline failure; all voxel shadows suppressed |

## Recommended Fix

Replace `view.projection[3][3]` with `view.clip_from_view[3][3]` in
`occlusion_material_prepass.wgsl`. *(Applied in commit `4b47a33`.)*

## Related

- Commit `742a521` — Bevy 0.18 migration (introduced the regression)
- Commit `4b47a33` — fix applied
- `docs/developer-guide/systems/occlusion-rendering-pipeline.md` — shadow
  pass detection design decision documented
