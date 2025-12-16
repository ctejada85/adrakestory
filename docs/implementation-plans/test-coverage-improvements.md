# Test Coverage Improvements Plan

## Current State (Updated 2025-12-16)

- **Total .rs files**: 121
- **Files with tests**: 21 (17.4%)
- **Total tests**: 166
- **Files tested**:
  - `editor/camera.rs` - 4 tests (camera position, zoom, limits, interpolation)
  - `editor/history.rs` - 3 tests (push/undo, redo, clear redo on action)
  - `editor/shortcuts.rs` - 3 tests (apply place/remove voxel, undo via inverse)
  - `editor/grid/bounds.rs` - 1 test (grid bounds calculation)
  - `editor/grid/cursor_indicator.rs` - 1 test (cursor mesh creation)
  - `editor/grid/mesh.rs` - 1 test (infinite grid mesh creation)
  - `editor/grid/systems.rs` - 1 test (should_regenerate_grid)
  - `systems/game/hot_reload/state.rs` - 2 tests (default state, debounce)
  - `systems/game/map/validation.rs` - 4 tests (default map, invalid dimensions, invalid position, missing spawn)
  - `systems/game/map/loader.rs` - 3 tests (progress percentage, map load progress, default map)
  - `systems/game/map/geometry/tests.rs` - 19 tests (comprehensive geometry tests)
  - `systems/game/player_movement.rs` - 19 tests (input conversion, normalization, look direction, rotation) ✅ Phase 1
  - `systems/game/collision.rs` - 12 tests (CollisionResult, cylinder-AABB intersection) ✅ Phase 1
  - `systems/game/resources.rs` - 9 tests (spatial grid queries, world-to-grid conversion) ✅ Phase 1
  - `systems/game/map/format/patterns.rs` - 13 tests (pattern geometry, serialization, fence detection) ✅ Phase 2
  - `systems/game/map/format/rotation.rs` - 7 tests (angle normalization, composition, serialization) ✅ Phase 2
  - `systems/game/map/spawner/mod.rs` - 18 tests (Face normals/offsets, constants) ✅ Phase 2
  - `systems/game/map/spawner/chunks.rs` - 9 tests (sub-voxel positions, chunk coordinates) ✅ Phase 2
  - `systems/game/map/spawner/entities.rs` - 22 tests (property parsing for lights, NPCs) ✅ Phase 2
  - `systems/game/map/spawner/meshing/occupancy.rs` - 11 tests (neighbor detection, global coords) ✅ Phase 2
  - `systems/game/map/spawner/meshing/palette.rs` - 6 tests (hash distribution, index bounds) ✅ Phase 2

## Prioritized Testing Improvements

### Priority 1: Critical Core Systems (High Impact) ✅ COMPLETED

These modules are fundamental to the game's functionality and have the highest risk if broken.

#### 1.1 Player Movement (`systems/game/player_movement.rs` - 218 lines)
- **Why**: Core gameplay mechanic, physics-based collision
- **Tests needed**:
  - `test_horizontal_movement_basic` - Movement in each direction
  - `test_movement_speed_normalization` - Diagonal movement shouldn't be faster
  - `test_sprint_speed_multiplier` - Sprint mode increases speed correctly
  - `test_wall_collision_stops_movement` - Movement stops at walls
  - `test_stair_step_detection` - Can walk up stairs within step height

#### 1.2 Collision System (`systems/game/collision.rs` - exists in collision module)
- **Why**: Prevents falling through world, critical for gameplay
- **Tests needed**:
  - `test_cylinder_aabb_intersection` - Basic collision detection
  - `test_collision_response_ground` - Standing on ground works
  - `test_collision_response_wall` - Wall sliding works
  - `test_sub_voxel_collision_bounds` - Sub-voxel boundaries accurate

#### 1.3 Spatial Grid (`systems/game/spatial_grid.rs` - if exists)
- **Why**: O(n) collision optimization, performance critical
- **Tests needed**:
  - `test_insert_and_query` - Basic insert/query
  - `test_aabb_query_returns_correct_entities` - Spatial queries work
  - `test_entity_removal` - Cleanup works correctly
  - `test_cell_boundary_handling` - Entities near cell boundaries found

### Priority 2: Map System (Medium-High Impact)

#### 2.1 Map Format (`systems/game/map/format/` - various files)
- **Why**: Data integrity for saved maps
- **Tests needed**:
  - `test_ron_serialization_roundtrip` - Save/load preserves data
  - `test_pattern_deserialization` - All SubVoxelPattern variants deserialize
  - `test_entity_type_serialization` - EntityType enum handled
  - `test_lighting_config_defaults` - Default values sensible

#### 2.2 Spawner System (`systems/game/map/spawner/` - multiple files ~1000+ lines total)
- **Why**: Complex meshing and entity spawning
- **Tests needed**:
  - `test_chunk_coordinate_calculation` - Voxel→chunk mapping correct
  - `test_lod_level_selection` - Distance-based LOD works
  - `test_greedy_mesher_single_voxel` - Single voxel mesh correct
  - `test_greedy_mesher_merges_adjacent` - Adjacent same-color voxels merge
  - `test_entity_spawn_position` - Entities spawn at correct positions

### Priority 3: Editor Core (Medium Impact)

#### 3.1 Editor State (`editor/state.rs` - 201 lines)
- **Why**: Central editor state management
- **Tests needed**:
  - `test_state_initialization` - Default state is valid
  - `test_tool_switching` - Tool changes work correctly
  - `test_selection_state` - Selection add/remove/clear
  - `test_modified_flag` - Dirty flag set on changes

#### 3.2 Editor File I/O (`editor/file_io.rs` - 255 lines)
- **Why**: Save/load integrity
- **Tests needed**:
  - `test_save_creates_valid_ron` - Saved file is valid RON
  - `test_load_saved_map` - Saved map loads correctly
  - `test_save_preserves_all_data` - No data loss on save

#### 3.3 Voxel Tool (`editor/tools/voxel_tool/` - multiple files)
- **Why**: Core editing functionality
- **Tests needed**:
  - `test_placement_position_calculation` - Placement adjacent to face
  - `test_voxel_removal` - Removal works correctly
  - `test_pattern_application` - SubVoxel patterns applied correctly

### Priority 4: Input Systems (Lower Impact)

#### 4.1 Gamepad (`systems/game/gamepad.rs` - 339 lines)
- **Why**: Alternative input method
- **Tests needed**:
  - `test_axis_deadzone` - Small inputs filtered
  - `test_button_mapping` - Buttons map to correct actions

#### 4.2 Editor Input (`editor/tools/input/` - multiple files)
- **Why**: Editor usability
- **Tests needed**:
  - `test_transform_calculations` - Rotation/scaling math correct
  - `test_raycast_helper` - Mouse picking works

### Priority 5: UI Components (Lower Impact)

These are harder to unit test and may need integration tests.

- `editor/ui/outliner.rs` - 330 lines
- `editor/ui/viewport.rs` - 321 lines
- `editor/ui/toolbar/menus.rs` - 283 lines

**Recommendation**: Focus on logic extraction for testable functions rather than UI testing.

---

## Implementation Order

### Phase 1: Foundation (Week 1) ✅ COMPLETED
1. [x] Add tests to `player_movement.rs` - 19 tests added (input conversion, normalization, look direction, rotation)
2. [x] Add tests to collision system - 12 tests added (CollisionResult, cylinder-AABB intersection)
3. [x] Add tests to spatial grid - 8 tests added (world-to-grid conversion, AABB queries, cell handling)

### Phase 2: Map System (Week 2) ✅ COMPLETED
4. [x] Add tests to `format/` modules (serialization) - 20 tests added (patterns, rotation)
5. [x] Add tests to `spawner/` modules (meshing) - 44 tests added (chunks, Face, occupancy, palette)
6. [x] Add tests to `spawner/entities.rs` - 22 tests added (property parsing)

### Phase 3: Editor Core (Week 3)
7. [ ] Add tests to `editor/state.rs`
8. [ ] Add tests to `editor/file_io.rs`
9. [ ] Add tests to voxel tool modules

### Phase 4: Input & Cleanup (Week 4)
10. [ ] Add tests to `gamepad.rs`
11. [ ] Add tests to editor input transforms
12. [ ] Extract testable logic from UI components

---

## Test Guidelines

### Unit Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test pure functions directly
    #[test]
    fn test_function_basic_case() {
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }

    // Test edge cases
    #[test]
    fn test_function_edge_case() {
        let result = function_under_test(edge_input);
        assert!(result.is_valid());
    }
}
```

### What NOT to Test
- Bevy systems directly (require App setup)
- UI rendering code
- Simple getters/setters
- Trivial constructors

### What TO Test
- Pure functions with clear inputs/outputs
- State transitions
- Math calculations
- Serialization/deserialization
- Validation logic
- Business rules

---

## Metrics Goals

| Metric | Current | Target |
|--------|---------|--------|
| Files with tests | 21 (17.4%) | 30 (25%) |
| Total tests | 166 | 100+ ✅ |
| Core systems covered | ~60% | 80%+ |

---

## Notes

- Tests should follow existing patterns in `geometry/tests.rs`
- Use `#[cfg(test)]` modules at bottom of files per AGENTS.md
- Focus on testing pure logic, not Bevy ECS integration
- Consider extracting pure functions from system functions to enable testing
