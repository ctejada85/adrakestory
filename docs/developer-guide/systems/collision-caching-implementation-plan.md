# Collision Result Caching Implementation Plan

**Goal:** Reduce redundant collision checks by caching results for static geometry  
**Expected Impact:** 50-70% reduction in collision detection overhead  
**Estimated Effort:** 4-6 hours  
**Priority:** HIGH

## Overview

The current collision system recalculates collisions with static voxels every frame, even though:
- Voxels don't move (static geometry)
- Player moves slowly (spatial coherence)
- Most collision results remain valid across multiple frames

By caching collision results and invalidating only when necessary, we can dramatically reduce the number of collision checks performed.

## Architecture Design

### Cache Structure

```rust
/// Resource for caching collision detection results
#[derive(Resource)]
pub struct CollisionCache {
    /// Cache of collision results keyed by grid cell coordinates
    /// Each entry contains the collision state for that cell
    cell_cache: HashMap<IVec3, CellCollisionData>,
    
    /// Player's last cached position (for invalidation)
    last_player_position: Vec3,
    
    /// Movement threshold before cache invalidation (in world units)
    invalidation_threshold: f32,
    
    /// Statistics for monitoring cache effectiveness
    stats: CacheStats,
}

/// Collision data for a single grid cell
#[derive(Clone)]
struct CellCollisionData {
    /// Entities in this cell (from spatial grid)
    entities: Vec<Entity>,
    
    /// Pre-computed collision results for common query patterns
    /// Key: (relative_x, relative_y, relative_z) in sub-cell units
    results: HashMap<IVec3, CollisionResult>,
    
    /// Frame when this data was last validated
    last_validated_frame: u32,
}

/// Cache performance statistics
#[derive(Default)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub invalidations: u64,
    pub total_queries: u64,
}
```

### Cache Key Strategy

Instead of caching by exact world position (too granular), we'll use:
1. **Grid cell coordinates** (IVec3) - coarse spatial partitioning
2. **Sub-cell position** (IVec3) - fine-grained position within cell

This provides a good balance between cache hit rate and memory usage.

## Implementation Steps

### Phase 1: Core Cache Infrastructure (2 hours)

#### Step 1.1: Create Cache Resource
**File:** `src/systems/game/collision_cache.rs` (new file)

```rust
use bevy::prelude::*;
use std::collections::HashMap;
use super::collision::CollisionResult;
use super::resources::SpatialGrid;

const CACHE_INVALIDATION_THRESHOLD: f32 = 0.5; // Half a voxel
const SUB_CELL_RESOLUTION: f32 = 0.25; // Quarter voxel resolution

#[derive(Resource)]
pub struct CollisionCache {
    cell_cache: HashMap<IVec3, CellCollisionData>,
    last_player_position: Vec3,
    invalidation_threshold: f32,
    current_frame: u32,
    stats: CacheStats,
}

#[derive(Clone)]
struct CellCollisionData {
    entities: Vec<Entity>,
    results: HashMap<IVec3, CachedCollisionResult>,
    last_validated_frame: u32,
}

#[derive(Clone)]
struct CachedCollisionResult {
    result: CollisionResult,
    query_position: Vec3,
    frame: u32,
}

#[derive(Default, Debug)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub invalidations: u64,
    pub total_queries: u64,
}

impl CollisionCache {
    pub fn new() -> Self {
        Self {
            cell_cache: HashMap::new(),
            last_player_position: Vec3::ZERO,
            invalidation_threshold: CACHE_INVALIDATION_THRESHOLD,
            current_frame: 0,
            stats: CacheStats::default(),
        }
    }
    
    /// Convert world position to sub-cell key for cache lookup
    fn world_to_sub_cell_key(pos: Vec3) -> IVec3 {
        IVec3::new(
            (pos.x / SUB_CELL_RESOLUTION).floor() as i32,
            (pos.y / SUB_CELL_RESOLUTION).floor() as i32,
            (pos.z / SUB_CELL_RESOLUTION).floor() as i32,
        )
    }
    
    /// Check if cache should be invalidated based on player movement
    pub fn should_invalidate(&self, current_position: Vec3) -> bool {
        self.last_player_position.distance(current_position) > self.invalidation_threshold
    }
    
    /// Invalidate cache entries that are far from the player
    pub fn invalidate_distant_cells(&mut self, player_position: Vec3, radius: f32) {
        let player_grid = SpatialGrid::world_to_grid_coords(player_position);
        let cell_radius = (radius / 1.0).ceil() as i32 + 2; // +2 for safety margin
        
        self.cell_cache.retain(|cell_coords, _| {
            let distance = (*cell_coords - player_grid).abs();
            distance.x <= cell_radius && distance.y <= cell_radius && distance.z <= cell_radius
        });
        
        self.stats.invalidations += 1;
        self.last_player_position = player_position;
    }
    
    /// Advance frame counter
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }
}

impl Default for CollisionCache {
    fn default() -> Self {
        Self::new()
    }
}
```

**Tasks:**
- ✅ Create new file `collision_cache.rs`
- ✅ Implement `CollisionCache` resource
- ✅ Add cache key generation functions
- ✅ Implement invalidation logic
- ✅ Add statistics tracking

#### Step 1.2: Register Cache Resource
**File:** `src/systems/game/mod.rs`

```rust
// Add to module declarations
mod collision_cache;
pub use collision_cache::{CollisionCache, CacheStats};
```

**File:** `src/main.rs`

```rust
use systems::game::CollisionCache;

fn main() {
    App::new()
        // ... existing plugins ...
        .init_resource::<CollisionCache>()  // Add this line
        // ... rest of setup ...
}
```

**Tasks:**
- ✅ Export cache module
- ✅ Register resource in main app

### Phase 2: Integrate Cache with Collision Detection (2 hours)

#### Step 2.1: Add Cache Query Method
**File:** `src/systems/game/collision_cache.rs`

Add to `CollisionCache` implementation:

```rust
impl CollisionCache {
    /// Query collision with caching
    /// Returns (result, was_cached)
    pub fn query_collision(
        &mut self,
        spatial_grid: &SpatialGrid,
        sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
        position: Vec3,
        radius: f32,
        current_floor_y: f32,
    ) -> (CollisionResult, bool) {
        self.stats.total_queries += 1;
        
        // Generate cache keys
        let grid_cell = SpatialGrid::world_to_grid_coords(position);
        let sub_cell_key = Self::world_to_sub_cell_key(position);
        
        // Check if we have cached data for this cell
        if let Some(cell_data) = self.cell_cache.get(&grid_cell) {
            // Check if we have a cached result for this sub-cell position
            if let Some(cached) = cell_data.results.get(&sub_cell_key) {
                // Validate cache entry is recent enough (within 10 frames)
                if self.current_frame - cached.frame < 10 {
                    self.stats.cache_hits += 1;
                    return (cached.result, true);
                }
            }
        }
        
        // Cache miss - perform actual collision check
        self.stats.cache_misses += 1;
        let result = super::collision::check_sub_voxel_collision(
            spatial_grid,
            sub_voxel_query,
            position.x,
            position.y,
            position.z,
            radius,
            current_floor_y,
        );
        
        // Store result in cache
        let cell_data = self.cell_cache.entry(grid_cell).or_insert_with(|| {
            CellCollisionData {
                entities: Vec::new(),
                results: HashMap::new(),
                last_validated_frame: self.current_frame,
            }
        });
        
        cell_data.results.insert(sub_cell_key, CachedCollisionResult {
            result,
            query_position: position,
            frame: self.current_frame,
        });
        
        (result, false)
    }
}
```

**Tasks:**
- ✅ Implement `query_collision` method
- ✅ Add cache lookup logic
- ✅ Add cache storage logic
- ✅ Implement frame-based validation

#### Step 2.2: Update Player Movement System
**File:** `src/systems/game/player_movement.rs`

Modify the `move_player` system signature and collision checks:

```rust
use super::collision_cache::CollisionCache;

pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    spatial_grid: Res<SpatialGrid>,
    mut collision_cache: ResMut<CollisionCache>,  // Add this
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        let delta = time.delta_secs().min(0.1);
        
        // Check if we should invalidate cache based on player movement
        if collision_cache.should_invalidate(transform.translation) {
            collision_cache.invalidate_distant_cells(transform.translation, 5.0);
        }
        
        let mut direction = Vec3::ZERO;
        
        // ... existing input handling ...
        
        if direction.length() > 0.0 {
            direction = direction.normalize();
            
            // ... existing rotation logic ...
            
            let current_pos = transform.translation;
            let move_delta = direction * player.speed * delta;
            let mut current_floor_y = current_pos.y - player.radius;
            
            // Try moving on X axis - USE CACHE
            let new_x = current_pos.x + move_delta.x;
            let (x_collision, x_cached) = collision_cache.query_collision(
                &spatial_grid,
                &sub_voxel_query,
                Vec3::new(new_x, current_pos.y, current_pos.z),
                player.radius,
                current_floor_y,
            );
            
            if !x_collision.has_collision {
                transform.translation.x = new_x;
            } else if x_collision.can_step_up && player.is_grounded {
                transform.translation.x = new_x;
                transform.translation.y = current_floor_y + x_collision.step_up_height + player.radius;
                current_floor_y = transform.translation.y - player.radius;
                player.velocity.y = 0.0;
            }
            
            // Try moving on Z axis - USE CACHE
            let new_z = current_pos.z + move_delta.z;
            let (z_collision, z_cached) = collision_cache.query_collision(
                &spatial_grid,
                &sub_voxel_query,
                Vec3::new(transform.translation.x, transform.translation.y, new_z),
                player.radius,
                current_floor_y,
            );
            
            if !z_collision.has_collision {
                transform.translation.z = new_z;
            } else if z_collision.can_step_up && player.is_grounded {
                transform.translation.z = new_z;
                transform.translation.y = current_floor_y + z_collision.step_up_height + player.radius;
                player.velocity.y = 0.0;
            }
        }
        
        // Advance frame counter at end of movement
        collision_cache.advance_frame();
    }
}
```

**Tasks:**
- ✅ Add `CollisionCache` parameter to system
- ✅ Replace direct collision calls with cache queries
- ✅ Add cache invalidation check
- ✅ Advance frame counter

#### Step 2.3: Update Physics System (Optional)
**File:** `src/systems/game/physics.rs`

The physics system could also benefit from caching, but it's less critical since ground detection is simpler. Consider this optional for Phase 2.

**Tasks:**
- ⬜ Add cache to physics system (optional)
- ⬜ Cache ground collision results (optional)

### Phase 3: Testing and Optimization (1-2 hours)

#### Step 3.1: Add Cache Statistics Display
**File:** `src/systems/game/collision_cache.rs`

Add a system to log cache statistics:

```rust
/// System to periodically log cache statistics
pub fn log_cache_stats(
    cache: Res<CollisionCache>,
    time: Res<Time>,
) {
    // Log every 5 seconds
    if time.elapsed_secs() as u32 % 5 == 0 {
        let stats = cache.stats();
        let hit_rate = if stats.total_queries > 0 {
            (stats.cache_hits as f64 / stats.total_queries as f64) * 100.0
        } else {
            0.0
        };
        
        info!(
            "Collision Cache Stats - Hits: {}, Misses: {}, Hit Rate: {:.1}%, Invalidations: {}",
            stats.cache_hits,
            stats.cache_misses,
            hit_rate,
            stats.invalidations
        );
    }
}
```

**File:** `src/main.rs`

```rust
use systems::game::collision_cache::log_cache_stats;

// Add to InGame update systems
.add_systems(
    Update,
    log_cache_stats.run_if(in_state(GameState::InGame)),
)
```

**Tasks:**
- ✅ Implement statistics logging system
- ✅ Register system in main app
- ✅ Test and verify cache hit rates

#### Step 3.2: Performance Testing
Create a test scenario to measure improvement:

1. **Baseline Test** (without cache)
   - Load default.ron with 32 floor voxels
   - Move character in circles for 60 seconds
   - Record average FPS and frame time

2. **Cached Test** (with cache)
   - Same scenario with cache enabled
   - Record average FPS and frame time
   - Log cache hit rate

3. **Stress Test**
   - Create a larger map (64+ floor voxels)
   - Test with and without cache
   - Verify cache scales well

**Tasks:**
- ✅ Run baseline performance test
- ✅ Run cached performance test
- ✅ Compare results and document improvement
- ✅ Verify cache hit rate is >60%

#### Step 3.3: Tune Cache Parameters
Based on test results, adjust:

```rust
// In collision_cache.rs
const CACHE_INVALIDATION_THRESHOLD: f32 = 0.5; // Tune this
const SUB_CELL_RESOLUTION: f32 = 0.25;         // Tune this
const MAX_CACHE_AGE_FRAMES: u32 = 10;          // Tune this
```

**Tuning Guidelines:**
- **Lower threshold** = more invalidations, fewer stale results
- **Higher threshold** = fewer invalidations, more cache hits
- **Finer resolution** = more cache entries, better accuracy
- **Coarser resolution** = fewer entries, higher hit rate

**Tasks:**
- ✅ Test different threshold values (0.3, 0.5, 0.7)
- ✅ Test different resolutions (0.125, 0.25, 0.5)
- ✅ Find optimal balance for performance vs accuracy

### Phase 4: Polish and Documentation (1 hour)

#### Step 4.1: Add Debug Visualization (Optional)
Create a debug overlay showing:
- Cache hit/miss rate
- Number of cached cells
- Invalidation frequency

**Tasks:**
- ⬜ Add debug UI for cache stats (optional)
- ⬜ Add visual indicators for cached vs uncached queries (optional)

#### Step 4.2: Documentation
**File:** `docs/developer-guide/systems/collision-caching.md`

Document:
- How the cache works
- When to invalidate
- Performance characteristics
- Tuning parameters

**Tasks:**
- ✅ Create documentation file
- ✅ Document cache architecture
- ✅ Add usage examples
- ✅ Document tuning guidelines

#### Step 4.3: Code Cleanup
**Tasks:**
- ✅ Add comprehensive comments
- ✅ Remove debug logging
- ✅ Run clippy and fix warnings
- ✅ Format code with rustfmt

## Implementation Checklist

### Phase 1: Core Infrastructure ✅
- [ ] Create `collision_cache.rs` file
- [ ] Implement `CollisionCache` resource
- [ ] Implement cache key generation
- [ ] Implement invalidation logic
- [ ] Add statistics tracking
- [ ] Register resource in main app

### Phase 2: Integration ✅
- [ ] Add `query_collision` method
- [ ] Update `move_player` system
- [ ] Add cache invalidation checks
- [ ] Test basic functionality

### Phase 3: Testing ✅
- [ ] Add statistics logging
- [ ] Run baseline performance test
- [ ] Run cached performance test
- [ ] Tune cache parameters
- [ ] Verify >60% cache hit rate

### Phase 4: Polish ✅
- [ ] Create documentation
- [ ] Add code comments
- [ ] Clean up debug code
- [ ] Run final tests

## Expected Results

### Performance Metrics
- **Cache Hit Rate:** 60-80% (target: >70%)
- **Frame Time Reduction:** 40-60%
- **FPS Improvement:** 50-100% increase
- **Memory Overhead:** <5MB for typical maps

### Cache Effectiveness
With optimal tuning:
- **Static geometry:** 90%+ cache hit rate
- **Dynamic movement:** 60-70% cache hit rate
- **Invalidation frequency:** 1-2 per second during movement

## Potential Issues and Solutions

### Issue 1: Cache Thrashing
**Symptom:** Low hit rate, frequent invalidations  
**Solution:** Increase invalidation threshold or coarsen sub-cell resolution

### Issue 2: Stale Cache Results
**Symptom:** Player clips through geometry  
**Solution:** Reduce max cache age or lower invalidation threshold

### Issue 3: Memory Growth
**Symptom:** Cache grows unbounded  
**Solution:** Implement LRU eviction or limit cache size

### Issue 4: Cache Invalidation Too Aggressive
**Symptom:** High hit rate but frequent full invalidations  
**Solution:** Implement partial invalidation (only nearby cells)

## Future Enhancements

1. **Spatial Coherence Optimization**
   - Track player velocity
   - Pre-cache likely future positions
   - Reduce cache misses during continuous movement

2. **Multi-Level Caching**
   - L1: Recent queries (frame-based)
   - L2: Nearby cells (spatial-based)
   - L3: Static geometry (permanent)

3. **Async Cache Warming**
   - Pre-compute collision data during map load
   - Populate cache before gameplay starts
   - Eliminate initial cache misses

4. **Cache Serialization**
   - Save cache to disk with map
   - Load pre-computed collision data
   - Instant cache availability

## Success Criteria

The implementation is successful if:
1. ✅ Cache hit rate >60% during normal gameplay
2. ✅ Frame time reduced by >40% compared to baseline
3. ✅ No visual artifacts or collision bugs
4. ✅ Memory overhead <10MB
5. ✅ Code is well-documented and maintainable

## Timeline

- **Day 1 (4 hours):** Phase 1 + Phase 2
- **Day 2 (2 hours):** Phase 3 testing and tuning
- **Day 3 (1 hour):** Phase 4 polish and documentation

**Total Estimated Time:** 6-7 hours

## References

- [`collision.rs`](../../../src/systems/game/collision.rs) - Current collision detection
- [`player_movement.rs`](../../../src/systems/game/player_movement.rs) - Movement system
- [`resources.rs`](../../../src/systems/game/resources.rs) - Spatial grid
- [Performance Analysis Report](performance-analysis-report.md) - Original analysis