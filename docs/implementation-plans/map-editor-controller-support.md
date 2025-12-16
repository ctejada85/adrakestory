# Map Editor Controller Support Implementation Plan

## Status: 📋 PLANNED

## Overview

This document outlines the implementation plan for adding Xbox controller support to the Map Editor, enabling a Minecraft Creative Mode-style editing experience. The goal is to allow users to fly around the map, place/remove voxels, and switch between tools and materials using a gamepad.

## Goals

1. **First-person camera control** - Fly through the map like Minecraft Creative mode
2. **Direct voxel manipulation** - Place/remove voxels with triggers
3. **Hotbar system** - Quick-switch between voxel types, patterns, and entities
4. **Tool cycling** - Switch between editor tools with shoulder buttons
5. **Seamless input switching** - Support both controller and keyboard/mouse simultaneously

---

## Minecraft Creative Mode Reference

### Controls We're Emulating

| Minecraft Action | Minecraft Control | Our Mapping |
|------------------|-------------------|-------------|
| Move/Fly | Left Stick | Left Stick |
| Look | Right Stick | Right Stick |
| Place Block | Right Trigger | RT (Right Trigger) |
| Break Block | Left Trigger | LT (Left Trigger) |
| Fly Up | A Button | A Button |
| Fly Down | B Button | B Button |
| Open Inventory | Y Button | Y Button (open palette) |
| Hotbar Slot 1-9 | 1-9 keys | D-Pad + bumpers |
| Scroll Hotbar | Mouse Wheel | LB/RB (bumpers) |
| Pick Block | Middle Mouse | X Button |

---

## Architecture

### New Resources

```rust
/// Controller editing mode state
#[derive(Resource)]
pub struct ControllerEditMode {
    /// Whether controller edit mode is active
    pub enabled: bool,
    /// Current hotbar slot (0-8)
    pub hotbar_slot: usize,
    /// Hotbar contents - what's in each slot
    pub hotbar: [HotbarItem; 9],
    /// Current cursor position in world space
    pub cursor_position: Option<IVec3>,
    /// Direction the cursor is facing (for placement)
    pub cursor_face: Option<Vec3>,
    /// Whether the item palette is open
    pub palette_open: bool,
    /// Current palette category
    pub palette_category: PaletteCategory,
    /// Selected item in palette (row, column)
    pub palette_selection: (usize, usize),
}

/// What can be placed from hotbar
#[derive(Clone, Debug, PartialEq)]
pub enum HotbarItem {
    Empty,
    Voxel { voxel_type: VoxelType, pattern: SubVoxelPattern },
    Entity { entity_type: EntityType },
    Tool(EditorTool),
}

/// Categories in the item palette
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PaletteCategory {
    #[default]
    Voxels,
    Patterns,
    Entities,
    Tools,
}

/// First-person camera for controller mode
#[derive(Component)]
pub struct ControllerCamera {
    /// Position in world space
    pub position: Vec3,
    /// Yaw angle (horizontal rotation)
    pub yaw: f32,
    /// Pitch angle (vertical rotation)
    pub pitch: f32,
    /// Movement speed (units per second)
    pub speed: f32,
    /// Look sensitivity
    pub sensitivity: f32,
    /// Whether flying up
    pub fly_up: bool,
    /// Whether flying down
    pub fly_down: bool,
}
```

### File Structure

```
src/editor/
├── controller/
│   ├── mod.rs              # Module exports
│   ├── camera.rs           # First-person camera system
│   ├── cursor.rs           # 3D cursor/crosshair targeting
│   ├── input.rs            # Controller input processing
│   ├── hotbar.rs           # Hotbar state and cycling
│   └── palette.rs          # Item palette UI
├── ui/
│   ├── controller_hud.rs   # HUD overlay for controller mode
│   └── ...
```

---

## Implementation Phases

### Phase 1: First-Person Camera Mode (4-6 hours)

**Goal:** Fly through the map with controller like Minecraft Creative

#### Tasks

1. **Create ControllerCamera Component**
   ```rust
   impl ControllerCamera {
       pub fn new(position: Vec3) -> Self {
           Self {
               position,
               yaw: 0.0,
               pitch: 0.0,
               speed: 10.0,
               sensitivity: 2.0,
               fly_up: false,
               fly_down: false,
           }
       }
       
       pub fn forward(&self) -> Vec3 {
           Vec3::new(
               self.yaw.cos() * self.pitch.cos(),
               self.pitch.sin(),
               self.yaw.sin() * self.pitch.cos(),
           ).normalize()
       }
       
       pub fn right(&self) -> Vec3 {
           Vec3::new(self.yaw.cos() + PI/2.0, 0.0, (self.yaw + PI/2.0).sin())
               .normalize()
       }
   }
   ```

2. **Controller Camera Movement System**
   - Left stick: Forward/backward/strafe movement
   - Right stick: Look around (yaw/pitch)
   - A button: Fly up (hold)
   - B button: Fly down (hold)
   - Left stick click (L3): Sprint/fast mode

3. **Mode Toggle**
   - Toggle between orbit camera (existing) and first-person camera
   - Keyboard shortcut: `Tab` or `F1` to toggle
   - Controller: Start button or both stick clicks

4. **Camera Collision (Optional)**
   - Prevent camera from clipping through voxels
   - Or allow noclip for editor convenience

#### Files to Create/Modify
- `src/editor/controller/mod.rs` (NEW)
- `src/editor/controller/camera.rs` (NEW)
- `src/editor/mod.rs` (add module)
- `src/bin/map_editor.rs` (add systems)

---

### Phase 2: 3D Cursor and Targeting (3-4 hours)

**Goal:** Show where voxels will be placed/removed

#### Tasks

1. **Raycast Cursor System**
   ```rust
   pub fn update_controller_cursor(
       camera: Query<&ControllerCamera>,
       voxels: Query<&SubVoxel>,
       spatial_grid: Res<SpatialGrid>,
       mut cursor_state: ResMut<ControllerEditMode>,
   ) {
       // Cast ray from camera position in look direction
       // Find first voxel intersection
       // Store hit position and face normal
   }
   ```

2. **Cursor Visualization**
   - Wire-frame cube at target position (like Minecraft's block highlight)
   - Different colors for place vs remove mode
   - Show placement preview (ghost block)

3. **Reach Distance**
   - Maximum distance for placement/removal (e.g., 5-7 voxels)
   - Visual indicator when target is out of range

#### Files to Create/Modify
- `src/editor/controller/cursor.rs` (NEW)
- May reuse existing raycasting from `src/editor/cursor/raycasting.rs`

---

### Phase 3: Hotbar System (4-5 hours)

**Goal:** Quick access to 9 items like Minecraft hotbar

#### Tasks

1. **Hotbar Data Structure**
   ```rust
   impl ControllerEditMode {
       pub fn default_hotbar() -> [HotbarItem; 9] {
           [
               HotbarItem::Voxel { voxel_type: VoxelType::Grass, pattern: SubVoxelPattern::Full },
               HotbarItem::Voxel { voxel_type: VoxelType::Dirt, pattern: SubVoxelPattern::Full },
               HotbarItem::Voxel { voxel_type: VoxelType::Stone, pattern: SubVoxelPattern::Full },
               HotbarItem::Voxel { voxel_type: VoxelType::Grass, pattern: SubVoxelPattern::StaircaseX },
               HotbarItem::Voxel { voxel_type: VoxelType::Stone, pattern: SubVoxelPattern::PlatformXZ },
               HotbarItem::Entity { entity_type: EntityType::PlayerSpawn },
               HotbarItem::Entity { entity_type: EntityType::LightSource },
               HotbarItem::Tool(EditorTool::Select),
               HotbarItem::Empty,
           ]
       }
   }
   ```

2. **Hotbar Cycling**
   - LB (Left Bumper): Previous slot
   - RB (Right Bumper): Next slot
   - D-Pad Left/Right: Also cycle slots
   - D-Pad Up: Use tool associated with slot
   - D-Pad Down: Drop item from slot / clear slot

3. **Hotbar HUD**
   - Horizontal bar at bottom of screen
   - 9 slots with icons showing contents
   - Current slot highlighted
   - Shows voxel color/pattern preview
   - Shows entity icon for entities

4. **Quick Slot Access**
   - Hold LB + D-Pad directions for slots 1-4
   - Hold RB + D-Pad directions for slots 5-8
   - Or: D-Pad left/right cycles, Y opens full palette

#### Files to Create/Modify
- `src/editor/controller/hotbar.rs` (NEW)
- `src/editor/ui/controller_hud.rs` (NEW)

---

### Phase 4: Place/Remove Actions (3-4 hours)

**Goal:** Actually place and remove voxels with controller

#### Tasks

1. **Place Voxel Action**
   - RT (Right Trigger): Place voxel/entity at cursor position
   - Use adjacent-to-face placement (like Minecraft)
   - Respect current hotbar slot item
   - Integrate with EditorHistory for undo/redo

2. **Remove Voxel Action**
   - LT (Left Trigger): Remove voxel at cursor position
   - Hold for continuous removal (with repeat rate)
   - Visual feedback (break animation optional)

3. **Pick Block**
   - X Button: Pick voxel under cursor into current hotbar slot
   - Copies both voxel type AND pattern

4. **Entity Placement**
   - When hotbar item is Entity, RT places entity
   - Show entity preview at cursor location
   - Snap to voxel grid or allow free placement

#### Files to Create/Modify
- `src/editor/controller/input.rs` (NEW)
- Integrate with existing `src/editor/tools/voxel_tool/`

---

### Phase 5: Item Palette UI (4-5 hours)

**Goal:** Full inventory screen like Minecraft Creative

#### Tasks

1. **Palette Structure**
   ```
   ┌─────────────────────────────────────────┐
   │ [Voxels] [Patterns] [Entities] [Tools]  │  <- Category tabs
   ├─────────────────────────────────────────┤
   │  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐   │
   │  │🟩 │ │🟫 │ │⬜ │ │   │ │   │ │   │   │  <- Grid of items
   │  │   │ │   │ │   │ │   │ │   │ │   │   │
   │  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘   │
   │  Grass  Dirt  Stone                     │
   │                                         │
   │  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐   │
   │  │█  │ │▬  │ │⟋  │ ││  │ │┼  │ │   │   │
   │  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘   │
   │  Full  Platform Stairs Pillar Fence    │
   ├─────────────────────────────────────────┤
   │  Hotbar: [1][2][3][4][5][6][7][8][9]    │  <- Current hotbar
   └─────────────────────────────────────────┘
   ```

2. **Palette Navigation**
   - Y Button: Open/close palette
   - Left Stick / D-Pad: Navigate grid
   - LB/RB: Switch category tabs
   - A Button: Place selected item in current hotbar slot
   - B Button: Close palette
   - X Button: Clear hotbar slot

3. **Palette Contents by Category**
   - **Voxels**: All VoxelType variants (Grass, Dirt, Stone)
   - **Patterns**: All SubVoxelPattern variants with previews
   - **Entities**: All EntityType variants with icons
   - **Tools**: Select, Camera, VoxelRemove

4. **Voxel + Pattern Combination**
   - In Voxels tab, show type + current pattern
   - In Patterns tab, apply pattern to current voxel type
   - Or: Separate "Create Item" mode to combine type + pattern

#### Files to Create/Modify
- `src/editor/controller/palette.rs` (NEW)
- `src/editor/ui/controller_hud.rs` (extend)

---

### Phase 6: HUD and Visual Feedback (3-4 hours)

**Goal:** Complete controller mode UI

#### Tasks

1. **Controller HUD Elements**
   - Crosshair in center of screen
   - Hotbar at bottom
   - Current tool/item name
   - Coordinates display
   - Controller button hints

2. **Mode Indicator**
   - Show "Controller Mode" badge
   - Different UI style than orbit camera mode
   - Hide traditional editor panels (or minimize)

3. **Visual Feedback**
   - Placement preview (translucent block)
   - Invalid placement indicator (red tint)
   - Reach limit indicator
   - Selection highlight for palette

4. **Button Hints**
   ```
   [LT] Remove  [RT] Place  [LB][RB] Hotbar  [Y] Palette
   ```

#### Files to Create/Modify
- `src/editor/ui/controller_hud.rs` (extend)

---

### Phase 7: Integration and Polish (2-3 hours)

**Goal:** Seamless experience between modes

#### Tasks

1. **Mode Switching**
   - Remember camera position when switching modes
   - Preserve hotbar when switching
   - Save/load hotbar configuration

2. **Input Coexistence**
   - Controller and keyboard work simultaneously
   - Auto-switch HUD hints based on last input
   - Mouse click disables controller camera temporarily

3. **Undo/Redo Support**
   - All controller actions go through EditorHistory
   - LB+RB held + A = Undo
   - LB+RB held + B = Redo
   - Or: Back/Select button for undo

4. **Settings**
   - Sensitivity adjustment
   - Invert Y axis
   - Button remapping (future)

#### Files to Modify
- `src/editor/state.rs`
- `src/editor/shortcuts.rs`
- Various integration points

---

## Control Mapping Summary

### Movement & Camera
| Action | Controller | Keyboard Equivalent |
|--------|------------|---------------------|
| Move | Left Stick | WASD |
| Look | Right Stick | Mouse |
| Fly Up | A (hold) | Space |
| Fly Down | B (hold) | Shift |
| Sprint | L3 (Left Stick Click) | Ctrl |

### Editing Actions
| Action | Controller | Keyboard Equivalent |
|--------|------------|---------------------|
| Place | RT (Right Trigger) | Left Click |
| Remove | LT (Left Trigger) | Right Click |
| Pick Block | X Button | Middle Click |
| Open Palette | Y Button | E |

### Hotbar & Navigation
| Action | Controller | Keyboard Equivalent |
|--------|------------|---------------------|
| Previous Slot | LB (Left Bumper) | Mouse Wheel Up |
| Next Slot | RB (Right Bumper) | Mouse Wheel Down |
| Slot 1-9 | D-Pad + Bumper combos | Number Keys 1-9 |

### System
| Action | Controller | Keyboard Equivalent |
|--------|------------|---------------------|
| Toggle Mode | Start / L3+R3 | Tab / F1 |
| Undo | LB+RB+A | Ctrl+Z |
| Redo | LB+RB+B | Ctrl+Y |
| Pause/Menu | Start | Escape |

---

## Technical Considerations

### Deadzone Handling
Reuse existing deadzone logic from `src/systems/game/gamepad.rs`:
```rust
fn apply_deadzone(value: Vec2, deadzone: f32) -> Vec2 {
    let magnitude = value.length();
    if magnitude < deadzone {
        Vec2::ZERO
    } else {
        let normalized = value / magnitude;
        let rescaled = (magnitude - deadzone) / (1.0 - deadzone);
        normalized * rescaled
    }
}
```

### Raycast Performance
- Use SpatialGrid for efficient voxel lookup
- Cache raycast result until camera moves significantly
- Limit raycast distance to ~10 voxels

### UI Rendering
- Use bevy_egui for palette and HUD
- Overlay rendering on top of 3D viewport
- Handle UI focus to prevent camera movement when in palette

### State Persistence
- Save hotbar to editor config file
- Remember last controller mode state
- Per-map hotbar presets (optional)

---

## Testing Checklist

### Camera
- [ ] Left stick moves in look direction
- [ ] Right stick rotates camera smoothly
- [ ] A button flies up, B button flies down
- [ ] Sprint (L3) increases speed
- [ ] Camera doesn't clip through geometry (if collision enabled)
- [ ] Mode toggle works (Tab / Start button)

### Cursor
- [ ] Crosshair appears in center
- [ ] Target block highlights correctly
- [ ] Face normal detected for placement direction
- [ ] Out-of-range indicator works

### Hotbar
- [ ] 9 slots display correctly
- [ ] LB/RB cycles slots
- [ ] Current slot highlighted
- [ ] Item icons show correctly
- [ ] Pick block (X) copies voxel to slot

### Placement
- [ ] RT places current hotbar item
- [ ] Placement uses face normal for adjacent placement
- [ ] LT removes targeted voxel
- [ ] Hold for continuous action works
- [ ] Undo/redo records actions

### Palette
- [ ] Y opens/closes palette
- [ ] Navigate with D-pad/stick
- [ ] LB/RB switches categories
- [ ] A selects item to hotbar
- [ ] B closes palette
- [ ] All voxel types shown
- [ ] All patterns shown
- [ ] All entity types shown

### Integration
- [ ] Works alongside keyboard/mouse
- [ ] HUD hints update based on input
- [ ] No input conflicts with existing systems
- [ ] EditorHistory integration works

---

## Timeline Estimate

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1: First-Person Camera | 4-6 hours | None |
| Phase 2: 3D Cursor | 3-4 hours | Phase 1 |
| Phase 3: Hotbar System | 4-5 hours | Phase 1 |
| Phase 4: Place/Remove | 3-4 hours | Phase 2, 3 |
| Phase 5: Item Palette | 4-5 hours | Phase 3 |
| Phase 6: HUD/Visual | 3-4 hours | Phase 3-5 |
| Phase 7: Integration | 2-3 hours | All phases |

**Total Estimated Effort**: 23-31 hours

---

## Future Enhancements (Out of Scope)

- Custom hotbar layouts/profiles
- Controller vibration feedback
- Multi-block placement (drag to create line/plane)
- Copy/paste with controller
- Blueprint system for prefabs
- Split-screen co-op editing
- PlayStation/Switch controller prompts
- Radial menu alternative to linear hotbar

---

## References

- [Minecraft Controls](https://minecraft.wiki/w/Controls)
- [Bevy Gamepad Input](https://docs.rs/bevy/latest/bevy/input/gamepad/index.html)
- Existing game controller implementation: `docs/implementation-plans/xbox-controller-support.md`
- Editor keyboard mode: `src/editor/cursor/keyboard_mode.rs`
- Editor camera: `src/editor/camera.rs`
