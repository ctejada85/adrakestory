# User Story — LOD Chunk Iteration Throttle

**Ticket ID:** fix-lod-chunk-iteration  
**Type:** Bug Fix  
**Priority:** P4  
**Status:** Ready for implementation

---

## Story

As a developer running the game on large maps, I want the LOD system to skip its per-chunk distance calculations when the camera hasn't moved, so that CPU overhead from `update_chunk_lods` is near-zero during steady-state gameplay.

---

## Description

`update_chunk_lods()` iterates every `VoxelChunk` entity every frame even when the camera is completely stationary. Since LOD levels only change when the camera moves tens of world units, running the O(N) loop at 60 fps is unnecessary. This ticket delivers three phases: a distance-threshold guard (Phase 1), a new-chunk bypass so spawned chunks always get their correct LOD immediately (Phase 2), and a `LodConfig` resource making the threshold runtime-configurable (Phase 3).

**Root cause:** `src/systems/game/map/spawner/mod.rs`, `update_chunk_lods()`, ~line 339.

**Reference:** `docs/bugs/2026-03-15-2141-p4-lod-all-chunks-iterated-every-frame.md`

---

## Acceptance Criteria

### Phase 1 — Distance guard

| # | Criterion |
|---|-----------|
| AC-1 | A `LOD_MOVEMENT_THRESHOLD: f32 = 0.5` constant is added near `LOD_DISTANCES` in `spawner/mod.rs` |
| AC-2 | `update_chunk_lods` has a `mut last_camera_pos: Local<Vec3>` parameter |
| AC-3 | If `camera_pos.distance(*last_camera_pos) < lod_config.movement_threshold`, the system returns early without iterating any chunks |
| AC-4 | When the guard passes, `last_camera_pos` is updated to the current camera position before iterating |
| AC-5 | On the first frame (cold start at `Vec3::ZERO`), a non-origin camera position naturally exceeds the threshold and the full pass runs |

### Phase 2 — New-chunk bypass

| # | Criterion |
|---|-----------|
| AC-6 | `update_chunk_lods` has a `new_chunks: Query<(), Added<VoxelChunk>>` parameter |
| AC-7 | When `!new_chunks.is_empty()`, the distance guard is bypassed and a full pass runs even if the camera is stationary |
| AC-8 | `last_camera_pos` is updated after a new-chunk bypass pass |

### Phase 3 — Configurable threshold

| # | Criterion |
|---|-----------|
| AC-9 | A `LodConfig` resource exists with `movement_threshold: f32` |
| AC-10 | `LodConfig::default().movement_threshold == LOD_MOVEMENT_THRESHOLD` |
| AC-11 | `app.init_resource::<LodConfig>()` is called in `main.rs` |
| AC-12 | `update_chunk_lods` reads `lod_config.movement_threshold` via `Res<LodConfig>` |

### Validation (all phases)

| # | Criterion |
|---|-----------|
| AC-13 | All existing LOD switch behaviour (4 levels, `LOD_DISTANCES`, mesh swaps) is functionally unchanged |
| AC-14 | `VoxelChunk`, `ChunkLOD`, `LOD_LEVELS`, `LOD_DISTANCES` are unmodified |
| AC-15 | Unit tests cover all three phases (see Tasks) |
| AC-16 | All existing tests pass |
| AC-17 | `cargo clippy` produces no new warnings |
| AC-18 | Release build succeeds |

---

## Non-Functional Requirements

| # | Requirement |
|---|-------------|
| NFR-1 | Stationary camera cost must be O(1) — one `distance()` call, no chunk iteration |
| NFR-2 | No new heap allocations: `Local<Vec3>` and `Added<VoxelChunk>` are stack-allocated |
| NFR-3 | `LodConfig` is a plain `Resource` with a single `f32` — negligible memory cost |
| NFR-4 | Changes are scoped to `spawner/mod.rs` and `main.rs` only |

---

## Tasks

### Phase 1 — Distance guard
1. **Add `LOD_MOVEMENT_THRESHOLD` constant** — `pub const LOD_MOVEMENT_THRESHOLD: f32 = 0.5;` near `LOD_DISTANCES` with doc comment
2. **Add `Local<Vec3>` parameter** — `mut last_camera_pos: Local<Vec3>` as the third parameter of `update_chunk_lods`
3. **Add early-exit guard** — distance check + `last_camera_pos` update before the chunk loop; update function doc comment

### Phase 2 — New-chunk bypass
4. **Add `Added<VoxelChunk>` query** — `new_chunks: Query<(), Added<VoxelChunk>>` parameter
5. **Bypass guard when new chunks present** — `!camera_moved && !new_chunks_present` as the combined skip condition

### Phase 3 — Configurable threshold
6. **Add `LodConfig` resource** — `movement_threshold: f32`, `Default` impl uses `LOD_MOVEMENT_THRESHOLD`, registered in `main.rs`
7. **Wire `Res<LodConfig>` into `update_chunk_lods`** — replace constant reference with `lod_config.movement_threshold`

### Tests & validation
8. **Write unit tests** — 7 tests total in the existing `#[cfg(test)] mod tests` block in `spawner/mod.rs`:
   - Phase 1: `lod_threshold_constant_is_well_below_lod_distances`, `lod_threshold_guard_skips_when_camera_stationary`, `lod_threshold_guard_runs_when_camera_moves_beyond_threshold`, `lod_threshold_exact_boundary_skips`, `lod_threshold_cold_start_passes_at_non_origin_position`
   - Phase 2: `lod_new_chunk_bypasses_distance_guard`
   - Phase 3: `lod_config_default_matches_constant`
9. **Validate** — `cargo test --lib`, `cargo clippy --lib`, `cargo build --release`
10. **Update `docs/developer-guide/architecture.md`** — add LOD throttle and config notes
11. **Commit** — single `fix(lod)` commit with all changes

---

## Dependencies / Blockers

None. Independent of P1, P2, and P3 fixes.

---

## Pre-existing Failures to Document (not fix)

`editor::state::tests::test_get_display_name_with_path` — fails before this change; unrelated.

---

*Created: 2026-03-16*  
*Documents: [Requirements](./requirements.md) · [Architecture](./architecture.md)*
