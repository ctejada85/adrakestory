# Requirements — Occlusion Shadow Casting Fix

**Source:** Design discussion — 2026-03-21
**Status:** Draft

---

## 1. Overview

When a player enters a building, the occlusion system hides voxels above the player's level so the interior is visible from the third-person camera. A depth prepass shader (`occlusion_material_prepass.wgsl`) discards these voxels before they write depth. The same prepass shader pipeline is used for shadow map generation, so the discarded voxels also disappear from the shadow map. This makes the room interior appear fully lit with no shadow from the ceiling — visually incorrect.

The fix must make occluded voxels invisible to the player camera while still casting directional-light shadows onto the floor and walls below.

---

## 2. Functional Requirements

### 2.1 Shadow Casting

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | Voxels discarded by the occlusion prepass shader must still write depth into the directional-light shadow map. | Phase 1 |
| FR-2.1.2 | Floor, lower walls, and other below-player geometry must receive correct directional-light shadows from ceiling voxels even when those voxels are occluded. | Phase 1 |
| FR-2.1.3 | Player-camera occlusion behaviour must be unchanged — voxels above the player level must remain invisible to the main camera. | Phase 1 |

### 2.2 Light Type Scope

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The fix must apply to the directional light (sun). This is the primary light source and the cause of the visible artefact. | Phase 1 |
| FR-2.2.2 | Point and spot light shadow casting from occluded voxels is out of scope. Both light types default to `shadows_enabled: false` in all current maps. | Out of scope |

### 2.3 Performance

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | The fix must not add measurable overhead to the directional-light shadow pass. A shader branch on a compile-time `#ifdef` is acceptable. | Phase 1 |
| FR-2.3.2 | The fix must not regress the p95 frame time improvements achieved by the depth prepass optimisation (`perf-depth-prepass`). | Phase 1 |

### 2.4 Correctness

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | Shadow maps must contain depth data for all voxels regardless of occlusion state during directional-light shadow passes. | Phase 1 |
| FR-2.4.2 | The region-based discard (interior AABB) must also be suppressed during shadow passes, not only the shader-based height discard. | Phase 1 |

---

## 3. Non-Functional Requirements

- The fix must be contained to `assets/shaders/occlusion_material_prepass.wgsl`; no Rust changes required.
- The approach must remain correct against Bevy 0.15.x. The use of `DEPTH_CLAMP_ORTHO` as a shadow-pass signal must be documented with a comment noting its Bevy-version dependency.
- All 325 passing tests must continue to pass.

---

## 4. Out of Scope

- Fixing point/spot light shadow casting for occluded voxels.
- Changing the visual appearance of occlusion (dither pattern, radius, height threshold).
- Floor-level guarantee (always render the floor the player stands on) — tracked separately.
- Hybrid mode shader-based fallback re-enablement — tracked separately.
