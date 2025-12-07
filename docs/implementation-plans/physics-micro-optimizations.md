# Physics Micro-Optimizations Implementation Plan

## Overview

This document outlines potential micro-optimizations for the physics system. These are low-priority improvements that would provide marginal performance gains (~5% or less). The main physics optimizations have already been implemented.

**Status**: 📋 Planned  
**Priority**: Low  
**Estimated Impact**: <5% performance improvement  
**Last Updated**: 2025-12-07

---

## Prerequisites

Before implementing these optimizations, consider:

1. Profile the game to confirm physics is a bottleneck
2. Test on target hardware (especially lower-end devices)
3. Measure baseline performance metrics

---

## Optimization 1: SpatialGrid AABB Query - Reduce Allocations

### Status: ✅ IMPLEMENTED (2025-12-07)

### Problem

In `src/systems/game/resources.rs`, the `get_entities_in_aabb` function creates a new `Vec` without capacity hints, causing potential reallocations during vector growth.

### Current Implementation (Lines 27-42)

```rust
pub fn get_entities_in_aabb(&self, min_world: Vec3, max_world: Vec3) -> Vec<Entity> {
    let min_grid = Self::world_to_grid_coords(min_world);
    let max_grid = Self::world_to_grid_coords(max_world);

    let mut entities = Vec::new();  // ← No capacity hint
    for x in min_grid.x..=max_grid.x {
        for y in min_grid.y..=max_grid.y {
            for z in min_grid.z..=max_grid.z {
                if let Some(cell_entities) = self.get_entities_in_cell(IVec3::new(x, y, z)) {
                    entities.extend(cell_entities.iter().copied());
                }
            }
        }
    }
    entities
}
```

### Proposed Solution

```rust
pub fn get_entities_in_aabb(&self, min_world: Vec3, max_world: Vec3) -> Vec<Entity> {
    let min_grid = Self::world_to_grid_coords(min_world);
    let max_grid = Self::world_to_grid_coords(max_world);

    // Pre-allocate capacity based on expected entity count
    // Estimate: number of cells × average entities per cell (~8 sub-voxels per voxel)
    let num_cells = ((max_grid.x - min_grid.x + 1)
                   * (max_grid.y - min_grid.y + 1)
                   * (max_grid.z - min_grid.z + 1)) as usize;
    let mut entities = Vec::with_capacity(num_cells * 8);

    for x in min_grid.x..=max_grid.x {
        for y in min_grid.y..=max_grid.y {
            for z in min_grid.z..=max_grid.z {
                if let Some(cell_entities) = self.get_entities_in_cell(IVec3::new(x, y, z)) {
                    entities.extend_from_slice(cell_entities);
                }
            }
        }
    }
    entities
}
```

### Changes Summary

| Change | Description |
|--------|-------------|
| Add capacity estimation | Calculate expected number of entities based on grid cells |
| Use `with_capacity` | Pre-allocate vector to avoid reallocations |
| Use `extend_from_slice` | Slightly more efficient than `extend(iter().copied())` |

### Impact

- **Performance**: Reduces heap allocations during collision checks
- **Memory**: Slightly higher initial allocation, but fewer total allocations
- **Complexity**: Minimal change, easy to implement

---

## Optimization 2: Diagonal Movement Optimization

### Status: ✅ IMPLEMENTED (2025-12-07)

### Problem

In `src/systems/game/player_movement.rs`, the movement system performs separate collision checks for X and Z axes, even when the player is moving diagonally without obstacles.

### Current Implementation (Lines 93-136)

```rust
// Try moving on X axis
let new_x = current_pos.x + move_delta.x;
let x_collision = check_sub_voxel_collision(
    &spatial_grid, &sub_voxel_query,
    new_x, current_pos.y, current_pos.z,
    player.radius, current_floor_y,
);
// ... handle X collision ...

// Try moving on Z axis
let new_z = current_pos.z + move_delta.z;
let z_collision = check_sub_voxel_collision(
    &spatial_grid, &sub_voxel_query,
    transform.translation.x, transform.translation.y, new_z,
    player.radius, current_floor_y,
);
// ... handle Z collision ...
```

### Proposed Solution

```rust
// Optimize: Try diagonal movement first when moving in both axes
if move_delta.x != 0.0 && move_delta.z != 0.0 {
    let new_x = current_pos.x + move_delta.x;
    let new_z = current_pos.z + move_delta.z;
    
    let diagonal_collision = check_sub_voxel_collision(
        &spatial_grid, &sub_voxel_query,
        new_x, current_pos.y, new_z,
        player.radius, current_floor_y,
    );
    
    if !diagonal_collision.has_collision {
        // Fast path: Can move diagonally without obstacles
        transform.translation.x = new_x;
        transform.translation.z = new_z;
    } else if diagonal_collision.can_step_up && player.is_grounded {
        // Diagonal step-up
        transform.translation.x = new_x;
        transform.translation.z = new_z;
        transform.translation.y = current_floor_y + diagonal_collision.step_up_height + player.radius;
        player.velocity.y = 0.0;
    } else {
        // Fall back to individual axis checks for wall sliding behavior
        // ... existing X and Z axis code ...
    }
} else {
    // Moving in only one axis - use existing logic
    // ... existing X and Z axis code ...
}
```

### Changes Summary

| Change | Description |
|--------|-------------|
| Add diagonal fast path | Check diagonal movement first when moving in both X and Z |
| Early exit on success | Skip individual axis checks if diagonal succeeds |
| Preserve wall sliding | Fall back to axis-by-axis for proper wall collision behavior |

### Impact

- **Performance**: Reduces collision checks by ~33% when moving diagonally in open areas
- **Behavior**: No change to gameplay - wall sliding still works correctly
- **Complexity**: Moderate - requires careful testing to ensure wall sliding behavior is preserved

### Considerations

- Wall sliding behavior must be preserved when colliding with walls
- Step-up on diagonal movement needs testing on stairs
- May require additional edge case handling

---

## Optimization 3: Fused Multiply-Add for Velocity

### Problem

In `src/systems/game/physics.rs`, velocity calculations use separate multiply and add operations.

### Current Implementation (Line 28)

```rust
player.velocity.y += GRAVITY * delta;
```

### Proposed Solution

```rust
player.velocity.y = GRAVITY.mul_add(delta, player.velocity.y);
```

### Impact

- **Performance**: Negligible - modern CPUs often auto-optimize this
- **Precision**: Slightly better floating-point precision with FMA
- **Complexity**: Trivial change

### Recommendation

**Skip this optimization** - the benefit is negligible and the current code is more readable.

---

## Implementation Order

1. **Optimization 1** (SpatialGrid) - Low risk, easy to implement
2. **Optimization 2** (Diagonal Movement) - Medium risk, requires testing
3. ~~Optimization 3~~ (FMA) - Skip unless profiling shows benefit

---

## Testing Plan

### Unit Tests

- [ ] Verify `get_entities_in_aabb` returns same results with optimization
- [ ] Test diagonal movement in open areas
- [ ] Test diagonal movement against walls (wall sliding)
- [ ] Test diagonal step-up on stairs

### Performance Tests

- [ ] Benchmark spatial grid queries before/after
- [ ] Measure frame time with many entities
- [ ] Profile memory allocations

### Integration Tests

- [ ] Play through all maps to ensure no regressions
- [ ] Test edge cases (corners, narrow passages)
- [ ] Verify step-up behavior on stairs

---

## Success Metrics

| Metric | Current | Target | Method |
|--------|---------|--------|--------|
| Spatial grid query time | Baseline | -10% | Profiler |
| Memory allocations/frame | Baseline | -20% | Allocator tracking |
| Collision checks (diagonal) | 2 per axis | 1 (fast path) | Counter |

---

## Rollback Plan

If any optimization causes issues:

1. Revert the specific commit
2. Document the issue in this plan
3. Re-evaluate approach before retrying

---

## References

- [Physics Analysis Documentation](../developer-guide/systems/physics-analysis.md)
- [Spatial Grid Implementation](../../src/systems/game/resources.rs)
- [Player Movement System](../../src/systems/game/player_movement.rs)
- [Physics System](../../src/systems/game/physics.rs)
