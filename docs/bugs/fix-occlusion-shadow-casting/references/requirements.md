# Requirements — Occlusion Shadow Casting Fix

**Source:** Design discussion — 2026-03-21
**Participants:** Developer (Engineer)
**Status:** Implemented

---

## 1. Overview

When a player enters a building, the occlusion system hides voxels above the player's level so the interior is visible from the third-person camera. A depth prepass shader (`occlusion_material_prepass.wgsl`) discards these voxels before they write depth. The same prepass shader pipeline is used for shadow map generation, so the discarded voxels also disappear from the shadow map. This makes the room interior appear fully lit with no shadow from the ceiling — visually incorrect.

The fix must make occluded voxels invisible to the player camera while still casting directional-light shadows onto the floor and walls below.

---

## 2. Data Domains

### 2.1 Camera Depth Prepass vs. Shadow Map Pass

| Domain | Description | Typical Signal |
|--------|-------------|----------------|
| **Camera Depth Prepass** | Prepass run for the player's perspective camera. Occluded voxels must be **discarded** — they should not block the player view. | `view.projection[3][3] == 0.0` (perspective) |
| **Directional Shadow Map Pass** | Prepass run for the directional light's orthographic camera. Occluded voxels must **not** be discarded — they must cast shadows onto the floor. | `view.projection[3][3] == 1.0` (orthographic) |
| **Point / Spot Shadow Map Pass** | Prepass run for point or spot lights (perspective). Out of scope — both default to `shadows_enabled: false`. | Perspective, but shadows disabled |

**Requirement:** The prepass fragment shader must detect the pass type and apply discard only during camera depth prepass passes, not during directional shadow map passes.

---

## 3. Functional Requirements

### 3.1 Shadow Casting

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | Voxels discarded by the occlusion prepass shader must still write depth into the directional-light shadow map. | Phase 1 |
| FR-3.1.2 | Floor, lower walls, and other below-player geometry must receive correct directional-light shadows from ceiling voxels even when those voxels are occluded from the camera. | Phase 1 |
| FR-3.1.3 | Player-camera occlusion behavior must be unchanged — voxels above the player level must remain invisible to the main camera. | Phase 1 |

### 3.2 Light Type Scope

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | The fix must apply to the directional light (sun). This is the primary light source and the cause of the visible artefact. | Phase 1 |
| FR-3.2.2 | Point and spot light shadow casting from occluded voxels is out of scope. Both light types default to `shadows_enabled: false` in all current maps. | Out of scope |

### 3.3 Performance

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | The fix must not add measurable overhead to the directional-light shadow pass. A runtime float comparison per fragment is acceptable. | Phase 1 |
| FR-3.3.2 | The fix must not regress the p95 frame time improvements achieved by the depth prepass optimisation (`perf-depth-prepass`). | Phase 1 |

### 3.4 Correctness

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | Shadow maps must contain depth data for all voxels regardless of occlusion state during directional-light shadow passes. | Phase 1 |
| FR-3.4.2 | The region-based discard (interior AABB check) must also be suppressed during shadow passes, not only the shader-based height discard. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | The fix must be contained to `assets/shaders/occlusion_material_prepass.wgsl`; no Rust changes required. | Phase 1 |
| NFR-4.2 | The approach must be Bevy-version-agnostic — it must not rely on Bevy-internal shader defines that may be renamed across versions. | Phase 1 |
| NFR-4.3 | All existing tests must continue to pass. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- Directional light shadow casting for occluded voxels (FR-3.1.1, FR-3.1.2)
- Camera occlusion unchanged (FR-3.1.3)
- Both region-based and height-based discard blocks suppressed during shadow passes (FR-3.4.2)
- Version-agnostic implementation (NFR-4.2)

### Future Phases

- Point and spot light shadow casting for occluded voxels (requires Option D: NotShadowCaster per chunk)
- Floor-level visibility guarantee (always render the voxel floor the player stands on)
- Hybrid mode shader-based occlusion re-enablement

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The directional light is the only light type with `shadows_enabled: true` in current maps. |
| 2 | Directional light shadows always use orthographic projection in Bevy; player camera always uses perspective. |
| 3 | `view.projection[3][3]` == 1.0 is a reliable signal for orthographic projection across all Bevy versions. |
| 4 | Fix scope is shader-only; no Rust bind group or material changes required. |
| 5 | Point and spot lights having `shadows_enabled: false` is an acceptable constraint for the MVP scope. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | If an orthographic gameplay camera is ever added (e.g., top-down view), discard would be suppressed for it. Is this acceptable, or does it need a separate guard? | Developer |
| 2 | Should point/spot light shadow casting for occluded voxels be tracked as a separate ticket? | Developer |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `perf-depth-prepass` ticket must be complete (AlphaMode::Mask prepass fix) | Done | Developer |
| 2 | Architecture document (`references/architecture.md`) — Option B chosen | Done | Developer |

---

## 9. Key Contacts

| Person | Role | Reach Out For |
|--------|------|---------------|
| Developer | Engineer | All implementation decisions |

---

## 10. Reference: Sample Scenarios

| Type | Example Verification Scenario | Complexity |
|------|-------------------------------|------------|
| Happy path | Walk player inside a building — ceiling invisible, floor lit by sun shadow from ceiling | Medium |
| Shadow cast | At noon sun angle, floor shows shadow stripe from overhead wall/ceiling voxels | Medium |
| Occlusion preserved | Above-player voxels remain invisible to camera after fix | Low |
| No regression | Frame time same as pre-fix baseline within 5% | Low |
| Region discard | Indoor voxels in AABB region still invisible to camera but appear in shadow map | High |

---

*Created: 2026-03-21*
*Source: occlusion system investigation — `docs/bugs/fix-occlusion-shadow-casting/bug.md`*
