# Architecture: Remove Orphaned `Voxel` Marker Entities

**Requirements:** `docs/bugs/fix-orphaned-voxel-entities/requirements.md`

## Current Architecture

### Voxel Entity Lifecycle

```
spawn_voxels_chunked() — for each voxel_data in chunk:
  ├── ctx.commands.spawn(Voxel)          ← creates entity with single Voxel component
  ├── [sub-voxel geometry loop]          ← spawns SubVoxel collision entities
  └── [chunk mesh logic]                 ← accumulated into VoxelChunk mesh
```

```
handle_map_reload() — hot reload:
  ├── despawn all With<VoxelChunk>       ← despawns chunk mesh entities
  ├── despawn all With<SubVoxel>         ← despawns 230k collision entities
  └── (Voxel entities despawned via despawn_all_map_entities somehow OR left orphaned)
```

### ECS Archetype Table

```
Archetype "Voxel" {
  component columns: [Voxel]
  entity count:      ~1000 (scales with map voxel count)
  change tick cols:  added_tick[], changed_tick[]   ← maintained every frame
  purpose:           none
}
```

### Component Definition

```rust
// src/systems/game/components.rs:36
#[derive(Component)]
pub struct Voxel;

// src/systems/game/map/spawner/chunks.rs:5
use super::super::super::components::{SubVoxel, Voxel};

// src/systems/game/map/spawner/chunks.rs:96
ctx.commands.spawn(Voxel);
```

No system contains `Query<..., With<Voxel>>` or `RemovedComponents<Voxel>`.

## Target Architecture

### Voxel Entity Lifecycle

```
spawn_voxels_chunked() — for each voxel_data in chunk:
  ├── (Voxel spawn removed)               ← DELETED
  ├── [sub-voxel geometry loop]
  └── [chunk mesh logic]
```

### ECS Archetype Table

```
Archetype "Voxel" — REMOVED (no entities, archetype is vacuous)
```

### Component Definition

```rust
// src/systems/game/components.rs:36
// pub struct Voxel;  ← DELETED

// src/systems/game/map/spawner/chunks.rs:5
use super::super::super::components::SubVoxel;  // ← Voxel import removed
```

## Change Surface

### Files Modified

| File | Change |
|------|--------|
| `src/systems/game/map/spawner/chunks.rs` | Remove `Voxel` from import; remove `ctx.commands.spawn(Voxel)` |
| `src/systems/game/components.rs` | Remove `pub struct Voxel;` |

### Files to Check for Voxel Usage

```
src/systems/game/components.rs     — struct definition
src/systems/game/map/spawner/chunks.rs  — only import + spawn call
src/lib.rs                         — re-export check
```

No other files reference `Voxel`.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| `Voxel` used in code not found by grep | Very Low | — | Full grep confirms zero query usage |
| Hot-reload despawn misses `Voxel` entities | N/A | — | Entities are removed; nothing to miss |
| Public API breakage (Voxel exported) | Low | Low | Check `src/lib.rs` re-exports before removing |
