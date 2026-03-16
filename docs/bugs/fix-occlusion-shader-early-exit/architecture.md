# Architecture — Fix Occlusion Shader Fragment Early-Exit

---

## 1. Current Architecture

### 1.1 Shader Execution Path (Mode 1 — ShaderBased)

For every visible fragment, `fragment()` in `occlusion_material.wgsl` executes:

```
fragment()
│
├─ [Mode 2/3] in_interior_region() check → discard if inside region
│
├─ pbr_input_from_standard_material()          ← PBR setup (always)
│
└─ [Mode 1] calculate_occlusion_alpha(world_pos)
    │
    ├─ height check: fragment_y <= player_y + height_threshold
    │   └─ YES → return 1.0 early ✅ (already implemented)
    │
    ├─ normalize(player_pos - camera_pos)       ← sqrt + 3 divisions  ← COSTLY
    │
    ├─ point_to_ray_distance_xz()               ← dot + sqrt          ← COSTLY
    │
    ├─ radius check: dist >= occlusion_radius
    │   └─ YES → return 1.0 early ✅ (already implemented, but TOO LATE)
    │
    └─ 2× smoothstep + mix → return alpha
```

### 1.2 Problem

The radius early-exit fires *after* both expensive operations. For the majority of fragments in a typical scene (ground, distant walls, any chunk more than `occlusion_radius` away from the player in XZ), `normalize` and `point_to_ray_distance_xz` run unnecessarily. On Apple Silicon (TBDR), these fragment shader costs compound across the total on-screen fragment count.

The height early-exit (added in the P1 fix) already handles fragments below the player's height threshold. The remaining gap is fragments that are **above** the height threshold but **far in XZ** from the player — they still pay the normalize + ray distance cost before the radius check discards them.

---

## 2. Target Architecture

### 2.1 Design Principles

1. **No-op for in-zone fragments.** Fragments within the occlusion zone must receive exactly the same visual output as before.
2. **Shader-only change.** No Rust changes, no new uniforms, no material rebuilds.
3. **No sqrt for the pre-check.** Use squared distance (`dot(delta, delta)`) compared against a squared threshold — same branch cost with no square root.
4. **Height check remains first.** The height check covers the most fragments cheapest (single float compare). The XZ check is second.

### 2.2 Updated Execution Path

```
calculate_occlusion_alpha(world_pos)
│
├─ ① height check: fragment_y <= player_y + height_threshold
│   └─ YES → return 1.0  (unchanged, covers most floor/wall fragments)
│
├─ ② XZ squared-distance pre-check  🆕
│   let xz = world_pos.xz - player_pos.xz
│   let xz_sq = dot(xz, xz)
│   let margin_sq = (occlusion_radius * XZ_MARGIN_FACTOR)²
│   xz_sq > margin_sq?
│   └─ YES → return 1.0  (skip normalize + ray calculation entirely)
│
├─ normalize(player_pos - camera_pos)       ← only reached by near-player fragments
│
├─ point_to_ray_distance_xz()
│
├─ radius check: dist >= occlusion_radius
│
└─ 2× smoothstep + mix → return alpha
```

### 2.3 Correctness Analysis

The XZ pre-check uses the player's XZ position as the centre, not the ray itself. This is conservative: a fragment can be within `occlusion_radius` of the ray even if it is more than `occlusion_radius` from the player in XZ (when the camera is far away). The `XZ_MARGIN_FACTOR = 2.0` accounts for this:

- Default `occlusion_radius`: 3.0 world units → margin = 6.0 units
- Default camera arm length: ~5–8 world units at typical angles  
- A fragment that is > 6.0 units from the player in XZ cannot be within 3.0 units of a ray that starts at the camera (~5–8 units away) and ends at the player (0 units away) — the ray's maximum XZ deviation from the player equals the camera's XZ offset, which is bounded by the arm length.
- Even at extreme low angles, the check yields false positives (fragment incorrectly returned as opaque) only for fragments far outside the zone, which is the desired behaviour.

### 2.4 Code Change (Appendix A — Exact Diff)

**File:** `assets/shaders/occlusion_material.wgsl`

```wgsl
// BEFORE
fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    let player_y = occlusion.player_position.y;
    let fragment_y = world_pos.y;

    // Only apply occlusion to fragments above the player
    if fragment_y <= player_y + occlusion.height_threshold {
        return 1.0;
    }

    // Calculate ray from camera to player
    let ray_direction = normalize(occlusion.player_position - occlusion.camera_position);
    // ...
}

// AFTER
const XZ_MARGIN_FACTOR: f32 = 2.0;

fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    let player_y = occlusion.player_position.y;
    let fragment_y = world_pos.y;

    // ① Height early-exit: fragment below occlusion height cannot be affected
    if fragment_y <= player_y + occlusion.height_threshold {
        return 1.0;
    }

    // ② XZ distance early-exit: fragment far from player cannot be on the occlusion ray.
    // Uses squared distance to avoid sqrt — compared against (radius * margin)².
    let xz_offset = world_pos.xz - occlusion.player_position.xz;
    let xz_dist_sq = dot(xz_offset, xz_offset);
    let xz_margin = occlusion.occlusion_radius * XZ_MARGIN_FACTOR;
    if xz_dist_sq > xz_margin * xz_margin {
        return 1.0;
    }

    // Calculate ray from camera to player (only reached for near-player fragments)
    let ray_direction = normalize(occlusion.player_position - occlusion.camera_position);
    // ...
}
```

---

## 3. Fragment Coverage Estimate

For a typical scene with the player standing on a map:

| Fragments | Early-exit path | Cost avoided |
|-----------|-----------------|--------------|
| Floor / ground chunks (below `height_threshold`) | ① Height check | normalize + ray dist + smoothstep |
| Distant wall chunks (> 6 units from player in XZ) | ② XZ check | normalize + ray dist + smoothstep |
| Near-player upper fragments (within zone) | Full path | — (correct output required) |

In practice, ① and ② together handle the large majority of fragments on a typical outdoor map. Only fragments near the player's XZ column and above the height threshold reach the `normalize` call.

---

## 4. Test Plan

| Test | Verifies |
|------|---------|
| `xz_pre_check_skips_expensive_ops_for_far_fragment` | Fragment at XZ distance > 2× radius returns 1.0 without reaching the smoothstep path (pure WGSL logic test using constant inputs) |
| `xz_pre_check_does_not_discard_near_fragment` | Fragment within radius passes the XZ check and reaches the smoothstep calculation |
| `height_check_still_fires_first` | Fragment below height threshold returns 1.0 even if XZ distance is within margin |
| `occlusion_output_unchanged_for_in_zone_fragment` | A fragment within both height and XZ bounds produces the same alpha as before the change |

> **Note:** WGSL shaders cannot be unit-tested via `cargo test`. The test plan refers to visual regression testing (run the game, verify occlusion appearance is unchanged) and the acceptance criteria in `ticket.md`.

---

## 5. Invariants

| Invariant | How Maintained |
|-----------|---------------|
| In-zone fragments visually unchanged | XZ_MARGIN_FACTOR ensures near-player fragments are never early-exited |
| No new uniforms | Pre-check reuses `occlusion.player_position` and `occlusion.occlusion_radius` |
| Height check executes before XZ check | Code order — height check is line 1, XZ check is lines 2–5 |
| No false opaque fragments in normal gameplay | 2× radius margin covers the default camera arm length |
