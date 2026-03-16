# Bug: Occlusion Fragment Shader Runs on All Visible Chunk Fragments Every Frame

**Date:** 2026-03-16  
**Severity:** Medium  
**Status:** Open  
**Component:** Occlusion system / Shader  
**Platform:** macOS worse; cross-platform GPU cost  

---

## Description

The `OcclusionMaterial` fragment shader (`occlusion_material.wgsl`) runs for every visible fragment of every voxel chunk mesh every frame, regardless of whether occlusion is relevant to that fragment. The shader performs a ray-distance calculation (normalize, dot product, `smoothstep`) per fragment. On macOS with Apple Silicon's GPU execution model, this per-fragment overhead is more costly than on Windows' discrete GPUs, and it compounds the MSAA cost from the `AlphaToCoverage` issue. With many voxels on screen, total fragment count is large and shader execution time scales accordingly.

---

## Actual Behavior

- All `VoxelChunk` meshes use `OcclusionMaterial` when occlusion is enabled
- The fragment shader runs unconditionally for every visible chunk fragment
- Shader work per fragment: ray normalization, XZ point-to-ray distance, two `smoothstep` calls, Bayer dither check
- No early-exit mechanism for fragments that are geometrically far from the occlusion zone
- Total GPU shader cost scales with on-screen fragment count (i.e., with voxel density and zoom level)

---

## Expected Behavior

- Fragments that are clearly outside the occlusion zone (far from the camera-to-player ray, far below the player's height threshold) should exit the shader early with minimal work
- Shader overhead should not grow proportionally with total on-screen fragment count when most fragments are unaffected by occlusion

---

## Root Cause Analysis

**File:** `assets/shaders/occlusion_material.wgsl`  
**Function:** `fragment()`  
**Approximate lines:** 157–216

```wgsl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Runs for every fragment, every frame
    var base_color = textureSample(base_color_texture, ...);

    // No distance pre-check — always calls into occlusion logic
    let alpha = calculate_occlusion_alpha(in.world_position.xyz, uniforms);

    // calculate_occlusion_alpha (~lines 84–126):
    //   normalize(camera_pos - player_pos)        — division
    //   point_to_ray_distance_xz(...)             — dot + sqrt
    //   smoothstep(height_threshold - softness, ...) — 2× smoothstep
    //   smoothstep(radius - edge_softness, ...)
}
```

The existing `region_active` early-exit (line 167) only triggers when in `RegionBased` or `Hybrid` mode with an active interior region. For `ShaderBased` mode (the common case), there is no early exit based on the fragment's proximity to the occlusion zone.

**Why worse on macOS:**  
Apple Silicon GPUs have fewer execution units than high-end discrete GPUs. Non-trivial per-fragment math (`sqrt` in `point_to_ray_distance_xz`, `smoothstep` calls) becomes a bottleneck faster as fragment count rises. Combined with the MSAA cost (Finding 1), each fragment does more total work on macOS.

---

## Steps to Reproduce

1. Build and run on macOS: `cargo run --release`
2. Load a map with many voxels
3. Enable FPS counter: press `F3`
4. Zoom the camera out (more chunks, more fragments on screen)
5. Observe FPS drop proportional to screen coverage
6. Compare: temporarily disable occlusion via `OcclusionConfig::mode = OcclusionMode::None` at startup; FPS should increase measurably

---

## Suggested Fix

**Option A — Early Exit on Height (recommended, low risk):**  
Add a fast height check at the top of `calculate_occlusion_alpha()`. If the fragment is at or below `player_position.y + height_threshold`, it cannot be occluded. Return `1.0` immediately.

```wgsl
fn calculate_occlusion_alpha(world_pos: vec3<f32>, u: OcclusionUniforms) -> f32 {
    // Early exit: fragment below the height threshold cannot be occluded
    if world_pos.y < u.player_position.y + u.height_threshold {
        return 1.0;
    }
    // ... existing logic ...
}
```

This eliminates shader work for the majority of fragments in a typical scene (floor, lower walls, ground-level chunks).

**Option B — Early Exit on XZ Distance:**  
Before computing the ray projection, check if the fragment's XZ distance from the player exceeds `occlusion_radius` by a large margin (e.g., 2×). If so, return `1.0` early. This is cheaper than the full `point_to_ray_distance_xz()` calculation.

```wgsl
let xz_dist_from_player = length(world_pos.xz - u.player_position.xz);
if xz_dist_from_player > u.occlusion_radius * 2.0 {
    return 1.0;
}
```

**Option C — Disable OcclusionMaterial for Chunks Below Player:**  
At spawn/LOD time, assign `StandardMaterial` to chunks whose Y coordinate is entirely below the player's starting height. These chunks can never occlude the player. This reduces the total number of fragments processed by the expensive shader.

---

## Related

- Investigation: `docs/investigations/2026-03-16-0809-macos-fps-drop-many-voxels.md` — Finding 3
- Related: `docs/bugs/2026-03-16-1213-p1-alphatocoverage-msaa-macos-tbdr.md` — MSAA cost compounds this shader cost on macOS
