# Requirements: Fix VoxelChunk Shadow Casting Performance

**Bug report:** `docs/bugs/2026-03-16-1310-p2-chunk-mesh-missing-not-shadow-caster.md`
**Priority:** p2 High
**Investigation:** `docs/investigations/2026-03-16-1310-off-screen-voxel-performance.md`

## Problem Statement

VoxelChunk mesh entities are spawned without `NotShadowCaster`. Bevy's directional light renders every chunk mesh into 4 cascaded shadow depth maps per frame using the light's frustum — bypassing main-camera frustum culling entirely. GPU work scales with total map voxel count, causing frame-rate to degrade even when all chunks are off-screen.

## Functional Requirements

### FR-1 — VoxelChunk entities must not cast shadows
VoxelChunk entities must have the `NotShadowCaster` component added at spawn time, for both the `ChunkMaterial::Occlusion` and `ChunkMaterial::Standard` spawn paths.

### FR-2 — No regression to existing chunk rendering
Main-camera frustum culling (via `Aabb`), LOD switching, and occlusion material behaviour must be unaffected.

### FR-3 — No regression to lighting appearance
Ambient light and point light appearance must be unchanged. Only directional shadow casting is removed from chunk meshes.

## Non-Functional Requirements

### NFR-1 — Zero per-frame overhead added
The fix is a single component addition at spawn time. No per-frame cost is introduced.

### NFR-2 — Both chunk material paths covered
The fix applies identically to the occlusion material path and the standard material path.

## Out of Scope

- Disabling or modifying the directional light itself
- Changing shadow cascade configuration
- Any LOD, frustum culling, or occlusion changes

## Acceptance Criteria

- [ ] `cargo clippy` reports zero new warnings
- [ ] `cargo test` passes (320 tests)
- [ ] `cargo build --release` succeeds
- [ ] A map with many voxels runs at the same frame rate whether the player is facing the voxels or away from them
