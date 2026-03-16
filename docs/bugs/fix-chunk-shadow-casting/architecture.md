# Architecture: Fix VoxelChunk Shadow Casting Performance

**Requirements:** `docs/bugs/fix-chunk-shadow-casting/requirements.md`

## Current Architecture

### Chunk Spawn (chunks.rs ~line 232)

```
spawn_voxels_chunked()
  └── for each chunk:
        ├── ChunkMaterial::Occlusion path → commands.spawn((Mesh3d, MeshMaterial3d<OcclusionMaterial>, Transform, VoxelChunk, ChunkLOD, Aabb))
        └── ChunkMaterial::Standard path  → commands.spawn((Mesh3d, MeshMaterial3d<StandardMaterial>, Transform, VoxelChunk, ChunkLOD, Aabb))
```

### Per-Frame Shadow Rendering (Bevy internals)

```
Every frame:
  DirectionalLight (shadows_enabled: true, num_cascades: 4)
    ├── Cascade 0 (0–4 units)   → render all Mesh3d WITHOUT NotShadowCaster
    ├── Cascade 1 (4–~25 units)  → render all Mesh3d WITHOUT NotShadowCaster
    ├── Cascade 2 (~25–~60 units) → render all Mesh3d WITHOUT NotShadowCaster
    └── Cascade 3 (~60–100 units) → render all Mesh3d WITHOUT NotShadowCaster
```

All 10 VoxelChunk meshes fall within at least one cascade every frame. Each cascade submits the full chunk geometry independently. Total shadow cost = 4 × main-camera geometry budget per frame regardless of camera direction.

### Main Camera (works correctly)

```
Camera3d frustum culling → uses Aabb → off-screen chunks skipped ✓
```

## Target Architecture

### Chunk Spawn (chunks.rs ~line 232)

```
spawn_voxels_chunked()
  └── for each chunk:
        ├── ChunkMaterial::Occlusion path → commands.spawn((..., NotShadowCaster))  ← add
        └── ChunkMaterial::Standard path  → commands.spawn((..., NotShadowCaster))  ← add
```

### Per-Frame Shadow Rendering

```
Every frame:
  DirectionalLight (shadows_enabled: true, num_cascades: 4)
    ├── Cascade 0 → all Mesh3d WITHOUT NotShadowCaster → 0 VoxelChunk meshes ✓
    ├── Cascade 1 → ...                                → 0 VoxelChunk meshes ✓
    ├── Cascade 2 → ...                                → 0 VoxelChunk meshes ✓
    └── Cascade 3 → ...                                → 0 VoxelChunk meshes ✓
```

Shadow depth passes for VoxelChunk geometry = 0 draw calls per frame.

## Change Surface

### Files Modified

| File | Change |
|------|--------|
| `src/systems/game/map/spawner/chunks.rs` | Add `bevy::pbr::NotShadowCaster` import; add `NotShadowCaster` to both spawn bundles |

### Files Unchanged

All other files. The directional light, cascade config, LOD system, and occlusion material are unchanged.

## Component Diagram (after fix)

```
VoxelChunk entity components:
  Mesh3d
  MeshMaterial3d<OcclusionMaterial | StandardMaterial>
  Transform (identity — vertices are in world space)
  GlobalTransform (auto-required by Transform)
  Visibility (auto-required by Mesh3d)
  ViewVisibility (auto-required by Mesh3d)
  InheritedVisibility (auto-required by Mesh3d)
  VoxelChunk { chunk_pos, center }
  ChunkLOD { lod_meshes, current_lod }
  Aabb { center, half_extents }          ← enables main-camera frustum culling
  NotShadowCaster                        ← NEW: excludes from shadow depth passes
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Visual regression (no voxel shadows) | Certain | Low | Intended trade-off; shadow casting from greedy-meshed geometry was never art-directed |
| Breakage of other rendering | Very Low | — | `NotShadowCaster` is a leaf component with no side effects on visibility or LOD |
| Test failures | None | — | No tests cover shadow rendering |
