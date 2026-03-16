# Ticket: Fix VoxelChunk Shadow Casting Performance

**Type:** Bug Fix
**Priority:** p2 High
**Requirements:** `docs/bugs/fix-chunk-shadow-casting/requirements.md`
**Architecture:** `docs/bugs/fix-chunk-shadow-casting/architecture.md`
**Bug report:** `docs/bugs/2026-03-16-1310-p2-chunk-mesh-missing-not-shadow-caster.md`

## Story

As a player on macOS, I want the game to maintain consistent frame rate as the map grows, so that adding more voxels to the world doesn't degrade performance even when I'm not looking at them.

## Description

VoxelChunk meshes participate in 4 shadow depth passes every frame because `NotShadowCaster` is missing from their component bundles. This causes GPU work to scale with total map voxel count rather than visible voxel count. The fix is a one-line addition to both chunk spawn paths.

## Acceptance Criteria

- [ ] Both `ChunkMaterial::Occlusion` and `ChunkMaterial::Standard` spawn bundles include `NotShadowCaster`
- [ ] `NotShadowCaster` is imported from `bevy::pbr`
- [ ] `cargo clippy` — zero warnings
- [ ] `cargo test` — all tests pass
- [ ] `cargo build --release` — succeeds

## Tasks

### 1. Add `NotShadowCaster` import to `chunks.rs`

In `src/systems/game/map/spawner/chunks.rs`, add to the Bevy import:

```rust
use bevy::pbr::NotShadowCaster;
```

### 2. Add `NotShadowCaster` to the Occlusion material spawn bundle (~line 233)

```rust
ChunkMaterial::Occlusion(mat) => {
    ctx.commands.spawn((
        Mesh3d(lod_meshes[0].clone()),
        MeshMaterial3d(mat.clone()),
        Transform::default(),
        VoxelChunk { chunk_pos, center: chunk_center },
        ChunkLOD { lod_meshes: lod_meshes.clone(), current_lod: 0 },
        Aabb { center: Vec3A::from(chunk_center), half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0) },
        NotShadowCaster,
    ));
}
```

### 3. Add `NotShadowCaster` to the Standard material spawn bundle (~line 252)

```rust
ChunkMaterial::Standard(mat) => {
    ctx.commands.spawn((
        Mesh3d(lod_meshes[0].clone()),
        MeshMaterial3d(mat.clone()),
        Transform::default(),
        VoxelChunk { chunk_pos, center: chunk_center },
        ChunkLOD { lod_meshes, current_lod: 0 },
        Aabb { center: Vec3A::from(chunk_center), half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0) },
        NotShadowCaster,
    ));
}
```

## Non-Goals

- Do not modify the directional light or cascade configuration
- Do not add shadow receiver components
- Do not add `NotShadowReceiver` (let chunks still receive shadows from other casters)
