# Lighting Update Issue - Analysis & Solution Architecture

## Problem Statement

The map editor is repeatedly updating ambient and directional lighting every frame, causing excessive logging and unnecessary system overhead. The logs show:

```
2025-10-30T12:16:18.269787Z  INFO map_editor: Updated ambient light brightness to: 300
2025-10-30T12:16:18.270079Z  INFO map_editor: Updated directional light with high-quality shadows...
```

This occurs multiple times per second, indicating the lighting update system is running when it shouldn't.

## Root Cause Analysis

### The Change Detection Chain

1. **Cursor Position Updates** ([`cursor.rs:49-112`](../../src/editor/cursor.rs:49))
   - The `update_cursor_position` system runs every frame
   - It mutates `EditorState` by updating `cursor_position` and `cursor_grid_pos` fields
   - This happens whenever the mouse moves or keyboard cursor navigation occurs

2. **Bevy's Change Detection** 
   - When `EditorState` is accessed as `ResMut`, Bevy marks it as "changed"
   - Any mutation to the resource triggers the change flag
   - The flag persists until the next frame

3. **Lighting Update Trigger** ([`map_editor.rs:296-352`](../../src/bin/map_editor.rs:296))
   - The `update_lighting_on_map_change` system checks `editor_state.is_changed()` (line 303)
   - Because cursor updates mutate `EditorState`, this check returns `true` every frame
   - The system then unnecessarily updates lighting configuration

### Why This Is Problematic

- **Performance**: Despawning and respawning directional lights every frame is wasteful
- **Log Spam**: Excessive INFO logs make debugging difficult
- **Incorrect Semantics**: Cursor movement is not a "map change" that should trigger lighting updates
- **Change Detection Pollution**: Cursor updates pollute the change detection system

## Solution Architecture

### Option 1: Separate Cursor State Resource (Recommended)

**Approach**: Extract cursor-related state into a dedicated `CursorState` resource.

**Changes Required**:
```rust
// New resource in cursor.rs
#[derive(Resource, Default)]
pub struct CursorState {
    pub position: Option<Vec3>,
    pub grid_pos: Option<(i32, i32, i32)>,
}

// Remove from EditorState:
// - cursor_position: Option<Vec3>
// - cursor_grid_pos: Option<(i32, i32, i32)>
```

**Impact**:
- ✅ Clean separation of concerns
- ✅ Cursor updates don't trigger EditorState changes
- ✅ Lighting system only responds to actual map changes
- ⚠️ Requires updating ~10 systems that reference cursor state
- ⚠️ Breaking change to EditorState structure

**Files to Modify**:
- `src/editor/state.rs` - Remove cursor fields from EditorState
- `src/editor/cursor.rs` - Add CursorState resource, update systems
- `src/editor/tools/*.rs` - Update cursor position references
- `src/editor/grid.rs` - Update cursor indicator system
- `src/bin/map_editor.rs` - Initialize CursorState resource

### Option 2: Track Map-Specific Changes

**Approach**: Add a separate flag to track when the actual map data changes.

**Changes Required**:
```rust
// Add to EditorState
pub struct EditorState {
    // ... existing fields ...
    
    /// Flag to track when map data (voxels/entities/lighting) changes
    /// This is separate from cursor/UI state changes
    map_data_changed: bool,
}

impl EditorState {
    pub fn mark_map_changed(&mut self) {
        self.map_data_changed = true;
    }
    
    pub fn clear_map_changed(&mut self) {
        self.map_data_changed = false;
    }
    
    pub fn has_map_changed(&self) -> bool {
        self.map_data_changed
    }
}
```

**Impact**:
- ✅ Minimal code changes
- ✅ No breaking changes to existing structure
- ✅ Easy to implement
- ⚠️ Requires manual flag management in all map-modifying systems
- ⚠️ Prone to human error (forgetting to set flag)
- ⚠️ Doesn't solve the semantic issue of cursor state in EditorState

**Files to Modify**:
- `src/editor/state.rs` - Add flag and methods
- `src/bin/map_editor.rs` - Check flag instead of is_changed()
- `src/editor/tools/*.rs` - Call mark_map_changed() after modifications
- `src/editor/file_io.rs` - Call mark_map_changed() after loading

### Option 3: Use Bevy Events for Map Changes

**Approach**: Emit a `MapDataChanged` event whenever map data is modified.

**Changes Required**:
```rust
// New event
#[derive(Event)]
pub struct MapDataChanged;

// In map_editor.rs
fn update_lighting_on_map_change(
    mut commands: Commands,
    mut events: EventReader<MapDataChanged>,
    editor_state: Res<EditorState>,
    // ... other params
) {
    // Only update if we received an event
    if events.read().count() == 0 {
        return;
    }
    
    // ... update lighting
}
```

**Impact**:
- ✅ Event-driven architecture (Bevy best practice)
- ✅ Explicit about when map changes occur
- ✅ No change detection pollution
- ✅ Easy to debug (can log events)
- ⚠️ Requires emitting events in all map-modifying systems
- ⚠️ More boilerplate code

**Files to Modify**:
- `src/editor/mod.rs` - Add MapDataChanged event
- `src/bin/map_editor.rs` - Add event, update lighting system
- `src/editor/tools/*.rs` - Emit event after modifications
- `src/editor/file_io.rs` - Emit event after loading

### Option 4: Bypass Change Detection with Interior Mutability

**Approach**: Use `Res<EditorState>` with interior mutability for cursor updates.

**Changes Required**:
```rust
// Wrap cursor fields in Cell/RefCell
pub struct EditorState {
    // ... other fields ...
    cursor_position: Cell<Option<Vec3>>,
    cursor_grid_pos: Cell<Option<(i32, i32, i32)>>,
}

// Update without triggering change detection
fn update_cursor_position(
    editor_state: Res<EditorState>,  // Note: Res, not ResMut
    // ...
) {
    editor_state.cursor_position.set(Some(pos));
}
```

**Impact**:
- ✅ Minimal structural changes
- ✅ Cursor updates don't trigger change detection
- ⚠️ Requires Cell/RefCell (runtime borrow checking)
- ⚠️ Less idiomatic Bevy code
- ⚠️ Potential for runtime panics if misused
- ❌ Doesn't solve the semantic issue

## Comparison Matrix

| Criterion | Option 1: Separate Resource | Option 2: Flag | Option 3: Events | Option 4: Interior Mutability |
|-----------|---------------------------|----------------|------------------|------------------------------|
| **Correctness** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Maintainability** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| **Implementation Effort** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Bevy Best Practices** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Debuggability** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |

## Recommended Solution

**Option 1: Separate Cursor State Resource** is the recommended approach because:

1. **Semantic Correctness**: Cursor state is fundamentally different from editor/map state
2. **Clean Architecture**: Follows single responsibility principle
3. **Bevy Best Practices**: Uses resources appropriately for different concerns
4. **Future-Proof**: Makes it easier to add cursor-related features without polluting EditorState
5. **No Manual Tracking**: Relies on Bevy's change detection working correctly

While it requires more initial work, it provides the best long-term solution.

### Alternative: Hybrid Approach (Option 1 + Option 3)

For maximum robustness, combine:
- **Option 1** for cursor state separation
- **Option 3** for explicit map change events

This provides:
- Clean separation of concerns
- Explicit event-driven architecture for map changes
- Easy to extend with additional map-related events (MapSaved, MapLoaded, etc.)

## Implementation Plan

### Phase 1: Create CursorState Resource
1. Define `CursorState` resource in `cursor.rs`
2. Add initialization in `map_editor.rs`
3. Update `update_cursor_position` to use `CursorState`

### Phase 2: Update Cursor Consumers
1. Update `handle_keyboard_cursor_movement` 
2. Update `handle_keyboard_selection`
3. Update tool systems (voxel_tool, entity_tool, selection_tool)
4. Update grid cursor indicator system

### Phase 3: Fix Lighting System
1. Verify `update_lighting_on_map_change` only triggers on actual map changes
2. Test with cursor movement, keyboard navigation, and actual edits
3. Remove or reduce logging verbosity

### Phase 4: Optional Event System
1. Add `MapDataChanged` event
2. Emit from all map-modifying operations
3. Update lighting system to use events
4. Add similar events for other map lifecycle events

## Testing Strategy

1. **Cursor Movement Test**: Move mouse around viewport, verify no lighting updates
2. **Keyboard Navigation Test**: Use arrow keys to move cursor, verify no lighting updates
3. **Voxel Placement Test**: Place a voxel, verify lighting updates once
4. **Map Load Test**: Load a new map, verify lighting updates once
5. **Lighting Change Test**: Modify lighting in properties panel, verify update occurs

## Migration Notes

If implementing Option 1, existing code that accesses `editor_state.cursor_position` or `editor_state.cursor_grid_pos` will need to be updated to use the new `CursorState` resource instead.

Search pattern: `editor_state\.cursor_(position|grid_pos)`

Estimated affected systems: ~10 systems across 5 files.