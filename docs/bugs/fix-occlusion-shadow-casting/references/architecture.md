# Architecture — Occlusion Shadow Casting Fix

**Date:** 2026-03-21
**Component:** `assets/shaders/occlusion_material_prepass.wgsl`
**Purpose:** Document the root cause, fix options considered, and the chosen implementation.

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| v1 | 2026-03-21 | Initial draft — Option A (`DEPTH_CLAMP_ORTHO` guard) selected |
| v2 | 2026-03-21 | Option B (view projection check) promoted to chosen approach; Option A superseded — `DEPTH_CLAMP_ORTHO` renamed in Bevy 0.18 |

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

### Option A — Bevy-internal define guard (superseded)

**Bevy 0.15:** Wrap discards in `#ifndef DEPTH_CLAMP_ORTHO`. Set exclusively for directional shadow passes.
**Bevy 0.18:** `DEPTH_CLAMP_ORTHO` was **renamed** to `UNCLIPPED_DEPTH_ORTHO_EMULATION` and its semantics changed — it is now also set during the regular depth prepass on hardware that lacks native depth unclipping. The new `UNCLIPPED_DEPTH_ORTHO` define covers directional-shadow-only, but remains a semantically repurposed depth-clamping signal.

Superseded by Option B which makes no assumptions about Bevy internals.

### Option B — View projection matrix check (chosen)

Detect the shadow pass geometrically: directional-light shadow maps use an **orthographic** projection; the player camera uses **perspective**. The standard `view` uniform (bound at `@group(0) @binding(0)` in all Bevy render passes) exposes `view.projection`, a 4×4 matrix where `[3][3]` is `1.0` for orthographic and `0.0` for perspective.

```wgsl
#import bevy_render::view::View
@group(0) @binding(0) var<uniform> view: View;

@fragment
fn fragment(in: VertexOutput) {
    // projection[3][3] == 1.0  →  orthographic  →  directional shadow pass  →  skip discards
    // projection[3][3] == 0.0  →  perspective   →  player camera prepass   →  apply discards
    let is_shadow_pass = view.projection[3][3] >= 0.5;

    if !is_shadow_pass {
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
    }
}
```

- ✅ Zero dependency on Bevy-internal defines — immune to Bevy-version renames
- ✅ Works across all Bevy versions (view uniform has been stable since 0.11)
- ✅ Covers directional light shadows (always orthographic)
- ✅ No Rust changes required — one shader edit only
- ✅ Zero runtime cost — single float comparison per fragment
- ⚠️ Point/spot light shadow passes use **perspective** projection → discards still fire → unchanged from current (acceptable: both default to `shadows_enabled: false`)
- ⚠️ If a future orthographic gameplay camera is added, discards would be suppressed for it. Add a comment.

### Option C — Remove all prepass discards

Rejected. Loses the `perf-depth-prepass` early-Z gain and restores the player-invisible bug.

### Option D — Toggle `NotShadowCaster` per chunk (CPU)

Valid complement if point/spot lights with shadows are added. Chunk-level granularity; requires a per-frame CPU system. Deferred.

---

## 3. Target Architecture — Option B

### 3.1 Change

Single change to `assets/shaders/occlusion_material_prepass.wgsl`:
1. Add `#import bevy_render::view::View` and `@group(0) @binding(0) var<uniform> view: View;`
2. Wrap both discard blocks in `if !is_shadow_pass { … }` using the projection matrix check.

Full implementation:

```wgsl
#import bevy_pbr::prepass_io::VertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(2) @binding(100) var<uniform> occlusion: OcclusionUniforms;

@fragment
fn fragment(in: VertexOutput) {
    let world_pos = in.world_position.xyz;

    // projection[3][3] == 1.0 → orthographic → directional light shadow pass.
    // Directional shadow maps must include occluded voxels so the floor receives
    // shadows from ceiling geometry even when that geometry is invisible to the camera.
    //
    // projection[3][3] == 0.0 → perspective → player camera depth prepass.
    // Occluded voxels must be absent from the player's depth buffer.
    //
    // NOTE: Point and spot light shadow passes also use perspective projection, so
    // discards still fire for those. Both default to shadows_enabled: false in all
    // current maps, so this is acceptable. If that changes, use Option D
    // (NotShadowCaster per chunk) as a complement.
    let is_shadow_pass = view.projection[3][3] >= 0.5;

    if !is_shadow_pass {
        if occlusion.mode == 2u || occlusion.mode == 3u {
            // region-based discard (interior flood-fill AABB)
            let rel = world_pos - occlusion.region_min;
            let size = occlusion.region_max - occlusion.region_min;
            if all(rel >= vec3<f32>(0.0)) && all(rel <= size) { discard; }
        }

        if (occlusion.mode == 1u || occlusion.mode == 3u) && occlusion.technique == 0u {
            if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
                let xz_offset = world_pos.xz - occlusion.player_position.xz;
                let xz_dist_sq = dot(xz_offset, xz_offset);
                let xz_margin = occlusion.occlusion_radius * 1.2;
                if xz_dist_sq <= xz_margin * xz_margin { discard; }
            }
        }
    }
}
```

### 3.2 Pass Behaviour After Fix

| Pass | Projection | `is_shadow_pass` | Discard fires? | Effect |
|------|-----------|-----------------|----------------|--------|
| Camera depth prepass | Perspective | false | Yes | Occluded voxels absent — player visible ✅ |
| Directional shadow map | Orthographic | true | No | All voxels write shadow depth — correct shadows ✅ |
| Point/spot shadow map | Perspective | false | Yes | Occluded voxels still absent — unchanged ⚠️ |
| Main camera pass | N/A | N/A | Yes (main shader) | Occluded voxels invisible ✅ |

### 3.3 Pipeline Flow

```
Directional shadow pass
  → prepass shader fragment runs
  → view.projection[3][3] == 1.0  →  is_shadow_pass = true
  → discards skipped
  → all voxels write shadow depth
  → floor receives correct shadow ✅

Camera depth prepass
  → prepass shader fragment runs
  → view.projection[3][3] == 0.0  →  is_shadow_pass = false
  → discards run
  → above-player voxels absent from depth buffer
  → player visible ✅
```

---

## Appendix A — Key File Locations

| File | Role |
|------|------|
| `assets/shaders/occlusion_material_prepass.wgsl` | **Change target** — add `view` import + projection check guard |
| `assets/shaders/occlusion_material.wgsl` | Main pass shader — no change needed |
| `src/systems/game/occlusion/mod.rs` | Rust material definition — no change needed |
| `src/systems/game/map/spawner/mod.rs` | Camera spawn with `DepthPrepass` — no change needed |

---

## Appendix B — Open Questions & Decisions

| # | Question | Decision |
|---|----------|----------|
| Q1 | Should point/spot light shadow casting from occluded voxels be fixed? | Out of scope — both default to `shadows_enabled: false`. Track as Option D if needed. |
| Q2 | Is the projection matrix check reliable across Bevy versions? | Yes — the view uniform and projection matrix layout are stable across all Bevy versions since 0.11. No version-specific flags required. |
| Q3 | Does the region-based discard also need to be suppressed in shadow passes? | Yes — both discard blocks must be inside the `if !is_shadow_pass { }` guard. |
| Q4 | What happens if an orthographic gameplay camera is added (e.g., for top-down view)? | Discards would be suppressed for that camera's depth prepass too. Add a code comment; revisit with a view-tag uniform if that feature is ever implemented. |
