# Architecture — Occlusion Shadow Casting Fix

**Date:** 2026-03-21
**Component:** `assets/shaders/occlusion_material_prepass.wgsl`
**Purpose:** Document the root cause, fix options considered, and the chosen implementation.

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| v1 | 2026-03-21 | Initial draft — Option A (`DEPTH_CLAMP_ORTHO` guard) selected |

---

## 1. Current Architecture

### 1.1 Prepass Pipeline Reuse

Bevy 0.15 uses a single `PrepassPipeline<M>` for three distinct GPU passes:

| Pass | Key flags | Fragment shader runs? |
|------|-----------|----------------------|
| Camera depth prepass | `DEPTH_PREPASS` | Yes — `MAY_DISCARD` + custom shader registered |
| Directional-light shadow map | `DEPTH_PREPASS \| DEPTH_CLAMP_ORTHO` | Yes — same custom shader |
| Point/spot-light shadow map | `DEPTH_PREPASS` | Yes — same custom shader |

Because `AlphaMode::Mask` sets `MAY_DISCARD`, and `prepass_fragment_shader()` is overridden to return `occlusion_material_prepass.wgsl`, the same fragment shader runs for **all three passes**.

### 1.2 Current Discard Logic

`occlusion_material_prepass.wgsl` contains two discard blocks that fire unconditionally:

```wgsl
// Block 1 — region-based (mode 2 or 3)
if occlusion.mode == 2u || occlusion.mode == 3u {
    if in_interior_region(world_pos) { discard; }
}

// Block 2 — height-based (mode 1 or 3, dithered)
if (occlusion.mode == 1u || occlusion.mode == 3u) && occlusion.technique == 0u {
    if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
        ...
        discard;
    }
}
```

Both blocks fire during shadow map passes, removing the discarded voxels from the shadow map. The floor inside rooms receives no shadow.

---

## 2. Fix Options Considered

### Option A — `#ifndef DEPTH_CLAMP_ORTHO` guard (chosen)

Wrap all discard logic in `#ifndef DEPTH_CLAMP_ORTHO`. Bevy sets `DEPTH_CLAMP_ORTHO` exclusively for directional-light shadow passes (`light.rs` line 1557: `light_key.set(MeshPipelineKey::DEPTH_CLAMP_ORTHO, is_directional_light)`).

- ✅ One-line change, zero runtime cost (compile-time `#ifdef`)
- ✅ Fixes the directional light (sun) — the primary and default light source
- ✅ No Rust changes required
- ⚠️ Point/spot light shadow passes do not set `DEPTH_CLAMP_ORTHO` — occluded voxels are still discarded from those shadow maps. Acceptable because both default to `shadows_enabled: false`.
- ⚠️ `DEPTH_CLAMP_ORTHO` is not a semantic "shadow pass" flag — it signals orthographic depth clamping. If Bevy introduces an orthographic gameplay camera, the guard would incorrectly suppress discards for that view. Must be documented.

### Option B — Separate uniform flag `is_shadow_pass`

Add a `u32` field to `OcclusionUniforms` and set it per-pass. Not viable: the uniform buffer is a single CPU-uploaded value shared across all passes. There is no per-pass uniform injection mechanism without a second material variant.

### Option C — Remove all prepass discards

Remove discard from the prepass shader entirely — rely only on main-pass discard. Shadow maps would be correct. The early-Z performance gain from `perf-depth-prepass` would be lost entirely and the original player-invisible bug would return. Rejected.

### Option D — Toggle `NotShadowCaster` per chunk (CPU)

A Bevy system marks chunks whose AABB is above the player level with `NotShadowCaster` each frame. Those chunks are excluded from the shadow queue by Bevy before the pass even starts.

- ✅ Correct for all light types (directional, point, spot)
- ✅ No shader changes
- ⚠️ Chunk-level granularity — shadow boundary is blocky
- ⚠️ Requires a per-frame CPU system iterating chunk AABBs
- Suitable as a future complement if point/spot shadows are enabled

---

## 3. Target Architecture — Option A

### 3.1 Change

Single change to `assets/shaders/occlusion_material_prepass.wgsl`: wrap both discard blocks in `#ifndef DEPTH_CLAMP_ORTHO … #endif`.

```wgsl
// NOTE: DEPTH_CLAMP_ORTHO is set by Bevy 0.15 exclusively for directional-light shadow passes
// (bevy_pbr/src/render/light.rs — MeshPipelineKey::DEPTH_CLAMP_ORTHO).
// Skipping discards here lets occluded voxels write depth into the shadow map so the floor
// and walls below receive correct shadows even when ceiling voxels are invisible to the camera.
//
// LIMITATION: Point and spot light shadow passes do NOT set DEPTH_CLAMP_ORTHO, so occluded
// voxels are still discarded from those shadow maps. Both light types default to
// shadows_enabled: false in all current maps, so this is acceptable.
//
// FRAGILITY: If Bevy ever sets DEPTH_CLAMP_ORTHO for a non-shadow orthographic pass, this
// guard will incorrectly suppress discards for that pass. Re-evaluate on Bevy upgrades.
#ifndef DEPTH_CLAMP_ORTHO

    // Block 1 — region-based discard
    if occlusion.mode == 2u || occlusion.mode == 3u {
        if in_interior_region(world_pos) { discard; }
    }

    // Block 2 — height-based discard
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

### 3.2 Pass Behaviour After Fix

| Pass | `DEPTH_CLAMP_ORTHO` set? | Discard fires? | Effect |
|------|--------------------------|----------------|--------|
| Camera depth prepass | No | Yes | Occluded voxels absent from depth buffer — player visible ✅ |
| Directional shadow map | Yes | No | All voxels write shadow depth — correct shadows on floor ✅ |
| Point/spot shadow map | No | Yes | Occluded voxels still absent — unchanged from current (low impact) ⚠️ |
| Main camera pass | N/A | Yes (in `occlusion_material.wgsl`) | Occluded voxels invisible ✅ |

### 3.3 Pipeline Flow

```
Directional shadow pass
  → occlusion_material_prepass.wgsl
  → DEPTH_CLAMP_ORTHO defined → skip all discards
  → all voxels write shadow depth
  → shadow map contains ceiling geometry
  → floor receives shadow ✅

Camera depth prepass
  → occlusion_material_prepass.wgsl
  → DEPTH_CLAMP_ORTHO NOT defined → discards run
  → above-player voxels absent from depth buffer
  → player not occluded ✅
```

---

## Appendix A — Key File Locations

| File | Role |
|------|------|
| `assets/shaders/occlusion_material_prepass.wgsl` | **Change target** — add `#ifndef DEPTH_CLAMP_ORTHO` guard |
| `assets/shaders/occlusion_material.wgsl` | Main pass shader — no change needed |
| `src/systems/game/occlusion/mod.rs` | Rust material definition — no change needed |
| `src/systems/game/map/spawner/mod.rs` | Camera spawn with `DepthPrepass` — no change needed |

---

## Appendix B — Open Questions & Decisions

| # | Question | Decision |
|---|----------|----------|
| Q1 | Should point/spot light shadow casting from occluded voxels be fixed? | Out of scope — both default to `shadows_enabled: false`. Track as Option D if needed. |
| Q2 | Is `DEPTH_CLAMP_ORTHO` reliable across Bevy minor versions? | Yes for 0.15.x. Add comment flagging the fragility for Bevy upgrade reviewers. |
| Q3 | Does the region-based discard also need to be suppressed in shadow passes? | Yes — both discard blocks must be inside the `#ifndef` guard. |
