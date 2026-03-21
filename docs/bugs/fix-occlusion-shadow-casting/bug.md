# Bug: Occluded voxels discarded from shadow map pass — no shadows cast inside rooms

**Date:** 2026-03-21
**Priority:** p3
**Severity:** Medium
**Status:** Fixed
**Component:** Occlusion system / depth prepass shader

---

## Description

When the player enters a building, ceiling and upper-wall voxels are correctly discarded from the camera view. However the same prepass shader runs during directional-light shadow map generation, so those voxels are also discarded from the shadow map. The result is that the interior of a room loses all shadow from the ceiling and upper walls — the floor appears in full sunlight as if there were no roof.

---

## Actual Behavior

- Player walks into a building.
- Ceiling voxels disappear (correct occlusion behaviour).
- The floor and lower walls inside the room receive no shadow from the ceiling — the room interior is lit as if outdoors.
- Directional-light shadow map contains no depth data for the discarded voxels.

---

## Expected Behavior

- Ceiling voxels remain invisible to the player camera (occlusion unchanged).
- Ceiling voxels still appear in the directional-light shadow map.
- Floor and lower walls inside the room receive correct shadows from the ceiling.

---

## Root Cause Analysis

**File:** `assets/shaders/occlusion_material_prepass.wgsl`
**Function:** `fn fragment(in: VertexOutput)`

The shadow map pass uses `DrawPrepass<M>` — the same pipeline as the camera depth prepass. Because the material is `AlphaMode::Mask`, Bevy sets `MAY_DISCARD` and the prepass fragment shader runs for both passes. There is no mechanism inside the shader to distinguish a shadow map invocation from a camera depth prepass invocation, so the discard logic fires unconditionally for all passes.

Directional-light shadow passes use **orthographic** projection; the player camera uses **perspective**. This difference is detectable via `view.projection[3][3]`: `1.0` for orthographic, `0.0` for perspective.

```wgsl
// Before fix: discard fires for all invocations, including shadow map passes
if occlusion.mode == 1u || occlusion.mode == 3u {
    if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
        ...
        discard;   // ← also runs during shadow map pass
    }
}
```

---

## Steps to Reproduce

1. `cargo run --release`
2. Load any map with a building (enclosed room with ceiling).
3. Walk the player character inside the building.
4. Observe that the floor inside receives no directional-light shadow — it is as bright as open sky.

---

## Fix Applied

`assets/shaders/occlusion_material_prepass.wgsl` — wrap all discard logic in a shadow-pass guard using the projection matrix:

```wgsl
// projection[3][3]: 1.0 = orthographic (shadow pass) → skip discards
//                  0.0 = perspective  (camera prepass) → apply discards
let is_shadow_pass = view.projection[3][3] >= 0.5;

if !is_shadow_pass {
    // Region-based discard
    if occlusion.mode == 2u || occlusion.mode == 3u {
        if in_interior_region(world_pos) { discard; }
    }
    // Height-based discard (any technique)
    if occlusion.mode == 1u || occlusion.mode == 3u {
        if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
            let xz_offset = world_pos.xz - occlusion.player_position.xz;
            let xz_dist_sq = dot(xz_offset, xz_offset);
            let xz_margin = occlusion.occlusion_radius * XZ_MARGIN_FACTOR;
            if xz_dist_sq <= xz_margin * xz_margin { discard; }
        }
    }
}
```

The projection matrix check is Bevy-version-agnostic — it relies on pure WGSL math rather than shader defines, making it more robust across upgrades.

**Known limitation:** Point and spot light shadow passes also use perspective projection, so discards still fire for those. In practice, map-placed point/spot lights default to `shadows_enabled: false`, making this low impact.

---

## Suggested Fix (Superseded)

~~Wrap all discard logic in `#ifndef DEPTH_CLAMP_ORTHO`.~~

This approach was considered but not used. `DEPTH_CLAMP_ORTHO` is only set for directional-light shadow passes in specific Bevy versions and is not available in Bevy 0.18 as a reliable shader define. The projection matrix check above is the implemented solution.

See `references/architecture.md` for full analysis of fix options.

---

## Related

- `docs/bugs/perf-depth-prepass/` — introduced the depth prepass and prepass shader
- `docs/developer-guide/systems/occlusion-rendering-pipeline.md` — pipeline diagram
- `assets/shaders/occlusion_material_prepass.wgsl` — file to change
