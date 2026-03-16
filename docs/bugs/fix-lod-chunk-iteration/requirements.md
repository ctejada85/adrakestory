# Requirements — LOD Chunk Iteration Throttle

**Source:** Bug report `2026-03-15-2141-p4-lod-all-chunks-iterated-every-frame.md` — 2026-03-15  
**Status:** Draft

---

## 1. Overview

`update_chunk_lods()` iterates every `VoxelChunk` entity every frame, computing a `camera_pos.distance(chunk.center)` for each one. While the mesh swap itself is guarded by a `new_lod != current_lod` check, the distance computation and query iteration still run unconditionally at 60 fps regardless of camera movement. On large maps this is wasteful CPU work that accumulates every frame.

The fix adds a `Local<Vec3>` tracking the last camera position at which a LOD pass ran. If the camera has not moved more than `LOD_MOVEMENT_THRESHOLD` (0.5 world units) since the last pass, the system returns early without iterating any chunks. Because LOD transitions happen over tens of world units, a 0.5-unit dead zone is imperceptible to players and eliminates the overhead during all steady-state camera conditions.

---

## 2. Functional Requirements

### 2.1 Distance-Threshold Early Exit

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `update_chunk_lods` must store the last camera position in a `Local<Vec3>` system-local field. | Phase 1 |
| FR-2.1.2 | At the start of each frame, if the camera has not moved more than `LOD_MOVEMENT_THRESHOLD` from the last recorded position, the system must return early without iterating any chunks. | Phase 1 |
| FR-2.1.3 | When the camera has moved beyond the threshold, `last_camera_pos` must be updated to the current camera position before iterating chunks. | Phase 1 |
| FR-2.1.4 | On the first frame (cold start), `last_camera_pos` defaults to `Vec3::ZERO`. If the camera spawn position is not at the origin, the distance check naturally exceeds the threshold and the full pass runs. No special first-frame handling is needed. | Phase 1 |

### 2.2 Threshold Constant

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | A named constant `LOD_MOVEMENT_THRESHOLD: f32 = 0.5` must be added near `LOD_DISTANCES` in `spawner/mod.rs`. The value 0.5 is smaller than the smallest LOD transition distance (50.0 units) by two orders of magnitude, making it invisible to players. | Phase 1 |
| FR-2.2.2 | The constant must be used in the guard — not an inline literal — so it is easy to tune. | Phase 1 |

### 2.3 New-Chunk Bypass (Phase 2)

When chunks are spawned (e.g. on map load or hot-reload), they must receive their correct LOD level immediately — even if the camera hasn't moved since the last pass.

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | `update_chunk_lods` must accept an `Added<VoxelChunk>` query parameter to detect newly spawned chunks. | Phase 2 |
| FR-2.3.2 | If `!new_chunks.is_empty()`, the system must bypass the distance guard and run a full LOD pass on that frame regardless of camera movement. | Phase 2 |
| FR-2.3.3 | `last_camera_pos` must still be updated on a new-chunk bypass pass, so subsequent frames behave correctly. | Phase 2 |

### 2.4 Configurable Threshold (Phase 3)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | A `LodConfig` resource must be added with a `movement_threshold: f32` field, replacing the compile-time constant as the runtime guard value. | Phase 3 |
| FR-2.4.2 | `LodConfig::default().movement_threshold` must equal `LOD_MOVEMENT_THRESHOLD` — the constant becomes the default source, not the direct guard value. | Phase 3 |
| FR-2.4.3 | `LodConfig` must be registered via `app.init_resource::<LodConfig>()` in `main.rs`. | Phase 3 |
| FR-2.4.4 | `update_chunk_lods` must read `lod_config.movement_threshold` via `Res<LodConfig>` instead of referencing the constant directly. | Phase 3 |

### 2.5 Existing LOD Behaviour Preservation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | When the camera is moving, all existing LOD switch behaviour (4 levels, `LOD_DISTANCES` thresholds, mesh handle swaps) must remain functionally unchanged. | Phase 1 |
| FR-2.5.2 | `VoxelChunk`, `ChunkLOD`, `LOD_LEVELS`, and `LOD_DISTANCES` must not be modified. | Phase 1 |
| FR-2.5.3 | The system signature must remain compatible with the `GameSystemSet::Visual` registration. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | When the camera is stationary, CPU cost of `update_chunk_lods` must be reduced to a single `distance()` call — not proportional to chunk count. | Phase 1 |
| NFR-3.2 | No new allocations are introduced. `Local<Vec3>` is stack-allocated Bevy per-system state. | Phase 1 |
| NFR-3.3 | `Added<VoxelChunk>` is a Bevy query filter — O(1), no allocation. | Phase 2 |
| NFR-3.4 | `LodConfig` is a plain `Resource` with a single `f32` field — negligible memory cost. | Phase 3 |
| NFR-3.5 | All existing tests must pass without modification. | Phase 1 |
| NFR-3.6 | The fix must not introduce any `get_mut()` calls that would dirty chunk assets unconditionally (see `coding-guardrails.md` §1). The existing `new_lod != current_lod` guard already satisfies this for mesh swaps. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — Distance-threshold guard

- Add `LOD_MOVEMENT_THRESHOLD` constant
- Add `Local<Vec3> last_camera_pos` parameter to `update_chunk_lods`
- Add early-exit guard using the threshold
- Update doc comment on `update_chunk_lods`
- Unit tests for guard logic (5 tests)

### Phase 2 — New-chunk bypass

- Add `new_chunks: Query<(), Added<VoxelChunk>>` parameter
- Bypass distance guard when `!new_chunks.is_empty()`
- Unit test: `lod_new_chunk_bypasses_distance_guard`

### Phase 3 — Configurable threshold

- Add `LodConfig` resource with `movement_threshold: f32`
- Register `app.init_resource::<LodConfig>()`
- Replace constant reference with `lod_config.movement_threshold` in the guard
- Unit test: `lod_config_default_matches_constant`

### Out of Scope (not planned)

- Option B (frame-counter throttle) — superseded by distance-threshold which is more correct
- Streaming LOD / chunk unloading — separate feature, not related to this bug

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `LOD_MOVEMENT_THRESHOLD = 0.5` is a safe default: LOD transitions span 50–200 world units, so 0.5 units dead zone causes no perceptible visual difference. |
| 2 | The camera is a single entity (enforced by `get_single()`). No multi-camera scenario exists in the game binary. |
| 3 | `Local<Vec3>` is zero-initialized by Bevy; `Vec3::ZERO` is the correct cold-start default. |
| 4 | The guard uses `distance()` (not `distance_squared()`) to keep the threshold unit intuitive and consistent with the existing code style. |

---

## 6. Open Questions

No open questions.

---

## 7. Dependencies & Blockers

No blockers. This fix is independent of P1, P2, and P3 fixes.

---

*Created: 2026-03-16*  
*Source: Bug report `docs/bugs/2026-03-15-2141-p4-lod-all-chunks-iterated-every-frame.md`*  
*Companion document: [Architecture](./architecture.md)*
