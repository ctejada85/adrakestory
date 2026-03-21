# Bug: Occluded voxels discarded from shadow map pass — no shadows cast inside rooms

**Date:** 2026-03-21
**Priority:** p3
**Severity:** Medium
**Status:** Open
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

Bevy sets `MeshPipelineKey::DEPTH_CLAMP_ORTHO` (and therefore the `DEPTH_CLAMP_ORTHO` shader def) **only** for directional-light shadow passes. Point and spot light shadow passes do not receive this flag. The camera depth prepass also never sets it. This makes `#ifdef DEPTH_CLAMP_ORTHO` a reliable signal to distinguish directional-light shadow passes from everything else.

```wgsl
// Current: discard fires for all invocations, including shadow map passes
if (occlusion.mode == 1u || occlusion.mode == 3u) && occlusion.technique == 0u {
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

## Suggested Fix

Wrap all discard logic in the prepass shader in `#ifndef DEPTH_CLAMP_ORTHO`, so directional-light shadow passes skip discarding and all voxels write depth into the shadow map:

```wgsl
#ifndef DEPTH_CLAMP_ORTHO
// Only discard for camera depth prepass — not for shadow map passes.
// DEPTH_CLAMP_ORTHO is set exclusively for directional-light shadow passes in Bevy 0.15.
if occlusion.mode == 2u || occlusion.mode == 3u {
    if in_interior_region(world_pos) { discard; }
}
if (occlusion.mode == 1u || occlusion.mode == 3u) && occlusion.technique == 0u {
    if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
        let xz_offset = world_pos.xz - occlusion.player_position.xz;
        let xz_dist_sq = dot(xz_offset, xz_offset);
        let xz_margin = occlusion.occlusion_radius * XZ_MARGIN_FACTOR;
        if xz_dist_sq <= xz_margin * xz_margin { discard; }
    }
}
#endif // DEPTH_CLAMP_ORTHO
```

**Known limitation:** Point and spot light shadow passes do not set `DEPTH_CLAMP_ORTHO`, so they still discard occluded voxels. In practice, map-placed point/spot lights default to `shadows_enabled: false`, so this is low impact. If point/spot shadow casting is enabled in future, toggling `NotShadowCaster` per chunk (Option D) would be the correct complement.

See `references/architecture.md` for full analysis of fix options.

---

## Related

- `docs/bugs/perf-depth-prepass/` — introduced the depth prepass and prepass shader
- `docs/developer-guide/systems/occlusion-rendering-pipeline.md` — pipeline diagram
- `assets/shaders/occlusion_material_prepass.wgsl` — file to change
