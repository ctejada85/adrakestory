# Ticket: Remove Orphaned `Voxel` Marker Entities

**Type:** Bug Fix / Cleanup
**Priority:** p3 Medium
**Requirements:** `docs/bugs/fix-orphaned-voxel-entities/requirements.md`
**Architecture:** `docs/bugs/fix-orphaned-voxel-entities/architecture.md`
**Bug report:** `docs/bugs/2026-03-16-1310-p3-orphaned-voxel-marker-entities.md`

## Story

As a developer, I want the ECS world to only contain entities that serve a purpose, so that map loading, hot reload, and Bevy's internal bookkeeping stay lean as maps grow.

## Description

One `Voxel` marker entity is spawned per voxel during map load. No system queries or uses these entities. They are pure dead weight. Removing the spawn call and the component struct cleans up the ECS world proportionally to map size.

## Acceptance Criteria

- [ ] `grep -rn "spawn(Voxel"` finds no matches in `src/`
- [ ] `grep -rn "struct Voxel"` finds no matches in `src/`
- [ ] `cargo clippy` — zero warnings
- [ ] `cargo test` — all tests pass
- [ ] `cargo build --release` — succeeds

## Tasks

### 1. Remove `Voxel` import from `chunks.rs`

In `src/systems/game/map/spawner/chunks.rs` line 5:

```rust
// Before
use super::super::super::components::{SubVoxel, Voxel};

// After
use super::super::super::components::SubVoxel;
```

### 2. Remove the orphaned spawn call from `chunks.rs` (~line 96)

```rust
// Delete this line:
ctx.commands.spawn(Voxel);
```

### 3. Remove `Voxel` struct from `components.rs`

In `src/systems/game/components.rs`, remove:

```rust
/// Marker component for individual voxel entities (collision-related parent of sub-voxels)
#[derive(Component)]
pub struct Voxel;
```

### 4. Remove `Voxel` re-export from `lib.rs` if present

Check `src/lib.rs` for any `pub use ... Voxel` and remove if found.

## Non-Goals

- Do not modify `SubVoxel` entities
- Do not modify `VoxelChunk` entities
- Do not add any replacement component or entity
