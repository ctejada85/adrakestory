# Map Editor UI Improvements Plan

## Implementation Status

**Last Updated**: 2025-10-24

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Horizontal Toolbar | ✅ Complete | Tool buttons, dropdowns implemented |
| Phase 2: Status Bar | ✅ Complete | Tool display, counts, modified indicator |
| Phase 3: Viewport Overlays | ✅ Complete | Dynamic positioning relative to panels |
| Phase 4: Outliner Panel | ✅ Complete | Left panel with entity/voxel lists |
| Phase 5: Properties Panel | ✅ Complete | Tool-specific properties |

### Additional Completed Items (2025-10-24)
- ✅ **Recent Files**: File → Open Recent submenu with persistence
- ✅ **All Tool Shortcuts**: V, B, X, E, C shortcuts for tool switching
- ✅ **Dynamic Panel Positioning**: Overlays adjust to panel resize
- ✅ **Click-through Prevention**: Resize bars don't trigger tool actions
- ✅ **Entity Grid Alignment**: Entities snap to integer grid positions
- ✅ **Entity Movement Fix**: Proper system ordering for responsiveness

---

## Overview

This document outlines improvements to the map editor's user interface to enhance usability, discoverability, and workflow efficiency.

## Current UI Analysis

### Current Layout
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                                               │
├─────────────────────────────────────────────────────────────────────────┬───┤
│                                                                         │   │
│                                                                         │ P │
│                                                                         │ r │
│                                                                         │ o │
│                         3D VIEWPORT                                     │ p │
│                                                                         │ e │
│                                                                         │ r │
│                                                                         │ t │
│                                                                         │ i │
│                                                                         │ e │
│                                                                         │ s │
│                                                                         │   │
│ ┌──────────────────┐                                                    │ 3 │
│ │ Camera Controls  │                                                    │ 0 │
│ │ • Right-drag     │                                                    │ 0 │
│ │ • Middle-drag    │                                                    │ p │
│ │ • Scroll         │                                                    │ x │
│ └──────────────────┘                                                    │   │
├─────────────────────────────────────────────────────────────────────────┴───┤
│ Status: Ready                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Pain Points

1. **Tool Selection**: No visual toolbar for quick tool switching
2. **Tool Feedback**: Active tool not prominently displayed
3. **Properties Panel**: Mixes too many concerns (tool settings, map info, cursor)
4. **Entity Management**: Entity list not visible; hard to manage multiple entities
5. **Layer/Organization**: No way to organize or filter voxels/entities
6. **Status Feedback**: Limited feedback on operations and state
7. **Keyboard Mode**: No visual indicator when in keyboard edit mode
8. **Selection**: No visual count of selected items in viewport

---

## Proposed UI Redesign

### New Layout
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                                [Untitled*] ▼  │
├───────────────────────────────────────────────────────────────────────────┬─┤
│ [🔲][✏️][📍][🎯][📷]  │  Grass ▼  │  Full ▼  │        │ Grid: On │ Snap: On │
├────────────────────────┴────────────┴──────────┴────────┴──────────┴────────┤
│ ┌───────────┐ ┌─────────────────────────────────────────────────────┐ ┌────┐│
│ │ OUTLINER  │ │                                                     │ │TOOL││
│ │           │ │                                                     │ │    ││
│ │ ▼ Voxels  │ │                                                     │ │Plac││
│ │   (127)   │ │                                                     │ │    ││
│ │           │ │                                                     │ │Type││
│ │ ▼ Entities│ │              3D VIEWPORT                            │ │Gras││
│ │   Player  │ │                                                     │ │    ││
│ │   NPC (2) │ │                                                     │ │Patt││
│ │   Enemy(1)│ │                                                     │ │Full││
│ │           │ │                                                     │ │    ││
│ │           │ │                                                     │ ├────┤│
│ │           │ │                                             [I]     │ │SEL ││
│ │           │ │                              Selected: 3 voxels     │ │    ││
│ │           │ │                                                     │ │3 vx││
│ └───────────┘ └─────────────────────────────────────────────────────┘ └────┘│
├─────────────────────────────────────────────────────────────────────────────┤
│ 🔲 Select Tool │ Cursor: (5, 1, 3) │ Voxels: 127 │ Entities: 4 │ Modified * │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Component Specifications

### 1. Horizontal Toolbar (New)

**Purpose**: Quick access to tools and settings without menus

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [🔲][✏️][🗑️][📍][📷] │ Grass ▼ │ Full ▼ │ ││ │ [Grid][Snap] │ [⬚][⬛][🔳] │
│  ▲                     ▲         ▲        ▲    ▲              ▲             │
│  │                     │         │        │    │              │             │
│  Tools                 Type      Pattern  Sep  View Toggles   View Presets  │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Tool Buttons**:
| Icon | Tool | Shortcut | Tooltip |
|------|------|----------|---------|
| 🔲 | Select | V / 2 | Select and transform voxels/entities |
| ✏️ | Voxel Place | B / 1 | Place voxels |
| 🗑️ | Voxel Remove | X | Remove voxels |
| 📍 | Entity Place | E | Place entities |
| 📷 | Camera | C | Camera control mode |

**Type Dropdown** (context-sensitive):
- Voxel Tool: Grass, Dirt, Stone, Water, etc.
- Entity Tool: Player Spawn, NPC, Enemy, Item, Trigger

**Pattern Dropdown** (Voxel Tool only):
- Full, Platform (XZ), Platform (XY), Platform (YZ)
- Staircase (+X), Staircase (-X), Staircase (+Z), Staircase (-Z)
- Pillar

**View Toggles**:
- Grid: Toggle grid visibility
- Snap: Toggle snap-to-grid

**View Presets** (camera views):
- ⬚ Top (Numpad 7)
- ⬛ Front (Numpad 1)
- 🔳 Iso (Numpad 5)

---

### 2. Outliner Panel (New - Left Side)

**Purpose**: Hierarchical view of all map contents for easy selection and organization

```
┌─────────────────────────┐
│ OUTLINER          [🔍]  │
├─────────────────────────┤
│ ▼ Map: Village          │
│   ├─ ▼ Voxels (127)     │
│   │    Filter: [______] │
│   │    ├─ Grass (45)    │
│   │    ├─ Dirt (32)     │
│   │    └─ Stone (50)    │
│   │                     │
│   └─ ▼ Entities (4)     │
│        ├─ 🟢 PlayerSpawn│
│        ├─ 🔵 NPC: Elder │
│        ├─ 🔵 NPC: Guard │
│        └─ 🔴 Enemy: Slime│
│                         │
│ [+ Add Entity]          │
└─────────────────────────┘
```

**Features**:
- Collapsible sections for voxels and entities
- Entity icons color-coded by type
- Click to select, double-click to focus camera
- Filter/search box for large maps
- Count badges showing totals
- Right-click context menu for delete/duplicate

**Interactions**:
| Action | Result |
|--------|--------|
| Click entity | Select entity, show in properties |
| Double-click entity | Focus camera on entity |
| Right-click | Context menu (Delete, Duplicate, Rename) |
| Drag entity | Reorder (future: grouping) |

---

### 3. Tool Properties Panel (Right Side - Simplified)

**Purpose**: Context-sensitive properties for current tool or selection

```
┌─────────────────────────┐
│ TOOL PROPERTIES         │
├─────────────────────────┤
│ ✏️ Voxel Place          │
│ ─────────────────────── │
│ Type:    [Grass     ▼]  │
│ Pattern: [Full      ▼]  │
│                         │
│ Preview:                │
│ ┌─────────────────────┐ │
│ │    ███████████      │ │
│ │    ███████████      │ │
│ │    ███████████      │ │
│ └─────────────────────┘ │
│                         │
│ Shortcuts:              │
│ • Click to place        │
│ • R to rotate pattern   │
│ • Scroll for height     │
└─────────────────────────┘
```

**For Select Tool with Selection**:
```
┌─────────────────────────┐
│ SELECTION               │
├─────────────────────────┤
│ 🔲 3 voxels selected    │
│ ─────────────────────── │
│ Actions:                │
│ [🔄 Move] [↻ Rotate]   │
│ [📋 Copy] [🗑️ Delete]  │
│                         │
│ Transform:              │
│ X: [  0  ] ← offset →   │
│ Y: [  0  ]              │
│ Z: [  0  ]              │
│                         │
│ Bounds:                 │
│ Min: (2, 0, 1)          │
│ Max: (4, 2, 3)          │
│                         │
│ [Clear Selection]       │
└─────────────────────────┘
```

**For Entity Selected**:
```
┌─────────────────────────┐
│ ENTITY PROPERTIES       │
├─────────────────────────┤
│ 🔵 NPC                  │
│ ─────────────────────── │
│ Name: [Village Elder__] │
│                         │
│ Position:               │
│ X: [ 5.5 ] Y: [ 1.0 ]   │
│ Z: [ 2.5 ]              │
│                         │
│ Properties:             │
│ Radius: [===●===] 0.5   │
│ Dialog: [Edit...]       │
│                         │
│ [🔄 Move] [📋 Duplicate]│
│ [🗑️ Delete Entity]     │
└─────────────────────────┘
```

---

### 4. Enhanced Status Bar

**Purpose**: Persistent feedback on editor state and current operation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ✏️ Voxel Place │ Cursor: (5, 1, 3) │ Voxels: 127 │ Entities: 4 │ Modified * │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Sections**:
| Section | Content |
|---------|---------|
| Tool Icon + Name | Current active tool with icon |
| Cursor Position | Grid coordinates under cursor |
| Voxel Count | Total voxels in map |
| Entity Count | Total entities in map |
| Modified Indicator | * if unsaved changes |

**During Operations**:
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 🔄 MOVING 3 voxels │ Offset: (2, 0, -1) │ Press ENTER to confirm, ESC cancel │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 5. Viewport Overlays

**Purpose**: In-viewport feedback without UI panels

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                              [KEYBOARD MODE]│
│                                                                             │
│                                                                             │
│                                                                             │
│                              ┌───────────┐                                  │
│                              │  VOXEL    │                                  │
│                              │  PREVIEW  │ ← Ghost preview of voxel        │
│                              └───────────┘                                  │
│                                                                             │
│                                                                             │
│                                          ┌──────────────────┐               │
│                                          │ Selected: 3      │               │
│                                          │ G:Move R:Rotate  │               │
│                                          │ Del:Remove       │               │
│                                          └──────────────────┘               │
│                                                                             │
│ Cursor: (5, 1, 3)                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Overlay Elements**:
1. **Keyboard Mode Indicator**: Top-right badge when in keyboard edit mode
2. **Ghost Preview**: Semi-transparent voxel at placement position
3. **Selection Info**: Bottom-right tooltip with count and shortcuts
4. **Cursor Coords**: Bottom-left position display (optional, also in status bar)

---

### 6. Keyboard Mode Indicator (New)

**Purpose**: Clear visual feedback when in keyboard navigation mode

```
Normal Mode:                    Keyboard Mode:
┌────────────────┐              ┌────────────────────────────┐
│  (no indicator)│              │ [I] KEYBOARD MODE          │
└────────────────┘              │     HJKL: Move cursor      │
                                │     Space: Place           │
                                │     X: Remove              │
                                │     ESC: Exit              │
                                └────────────────────────────┘
```

**Features**:
- Appears in top-right of viewport when keyboard mode active
- Shows relevant shortcuts for current tool
- Pulses or has distinct color to draw attention
- Press I to toggle on, ESC to exit

---

## Implementation Phases

### Phase 1: Horizontal Toolbar (High Impact, Medium Effort)
**Files to modify**: `src/editor/ui/toolbar.rs`, `src/editor/ui/mod.rs`

1. Create horizontal toolbar below menu bar
2. Add tool buttons with icons (using Unicode or egui icons)
3. Add context-sensitive dropdowns for type/pattern
4. Add view toggles (Grid, Snap)
5. Show active tool with highlight

**Wireframe**:
```
┌─────────────────────────────────────────────────────────────┐
│ [🔲][✏️][🗑️][📍][📷] │ [Grass ▼][Full ▼] │ [Grid ☑][Snap ☑] │
│   ▲                                                         │
│   └─ Active tool highlighted                                │
└─────────────────────────────────────────────────────────────┘
```

### Phase 2: Enhanced Status Bar (High Impact, Low Effort)
**Files to modify**: `src/bin/map_editor.rs`, new `src/editor/ui/status_bar.rs`

1. Create dedicated status bar component
2. Show current tool icon and name
3. Display cursor position
4. Show voxel/entity counts
5. Show modified indicator
6. Add operation-specific messages

### Phase 3: Viewport Overlays (Medium Impact, Medium Effort)
**Files to modify**: `src/editor/ui/viewport.rs`

1. Add keyboard mode indicator overlay
2. Add selection count tooltip
3. Improve ghost preview visibility
4. Add contextual shortcut hints

### Phase 4: Outliner Panel (Medium Impact, High Effort)
**Files to modify**: New `src/editor/ui/outliner.rs`, `src/editor/ui/mod.rs`

1. Create left panel with collapsible tree
2. List voxels by type with counts
3. List entities with icons and names
4. Implement click-to-select
5. Implement double-click-to-focus
6. Add filter/search functionality
7. Add right-click context menu

### Phase 5: Simplified Properties Panel (Medium Impact, Medium Effort)
**Files to modify**: `src/editor/ui/properties.rs`

1. Reorganize into tool-specific views
2. Add visual pattern preview
3. Improve entity property editing
4. Add quick action buttons
5. Remove redundant information (move to outliner/status bar)

---

## Keyboard Shortcut Additions

| Shortcut | Action | Notes |
|----------|--------|-------|
| `Tab` | Cycle through panels | Outliner → Viewport → Properties |
| `Ctrl+L` | Toggle Outliner | Show/hide left panel |
| `Ctrl+P` | Toggle Properties | Show/hide right panel |
| `F2` | Rename selected entity | Quick rename |
| `Ctrl+D` | Duplicate selection | Clone voxels/entities |
| `Ctrl+G` | Group selection | Future: grouping feature |
| `[` / `]` | Previous/Next voxel type | Quick type switching |
| `Shift+[` / `]` | Previous/Next pattern | Quick pattern switching |

---

## Color Scheme

**Entity Type Colors** (consistent across UI):
| Entity Type | Color | Hex |
|-------------|-------|-----|
| Player Spawn | Green | #00FF00 |
| NPC | Blue | #0080FF |
| Enemy | Red | #FF0000 |
| Item | Yellow | #FFFF00 |
| Trigger | Magenta | #FF00FF |

**UI States**:
| State | Style |
|-------|-------|
| Active Tool | Highlighted background, bold icon |
| Hovered | Subtle highlight |
| Selected Item | Blue outline/background |
| Modified | Asterisk (*) indicator |
| Error | Red text/border |

---

## Accessibility Considerations

1. **Tooltips**: All buttons have descriptive tooltips with shortcuts
2. **Keyboard Navigation**: Full keyboard support for all operations
3. **Color + Icons**: Don't rely on color alone; use icons and text labels
4. **Focus Indicators**: Clear visual focus for keyboard navigation
5. **Scalable UI**: Support for UI scaling/font size changes

---

## Success Metrics

1. **Discoverability**: New users can find tools without documentation
2. **Efficiency**: Common operations require fewer clicks
3. **Feedback**: Users always know the current tool and state
4. **Navigation**: Large maps are manageable via outliner
5. **Keyboard Users**: Full workflow possible without mouse

---

## File Structure After Implementation

```
src/editor/ui/
├── mod.rs              # Module exports
├── toolbar.rs          # Menu bar (existing) + horizontal toolbar (new)
├── status_bar.rs       # New: enhanced status bar
├── outliner.rs         # New: left panel with map contents tree
├── properties.rs       # Simplified: tool/selection properties only
├── viewport.rs         # Enhanced: overlays and indicators
└── dialogs.rs          # Existing: modal dialogs
```

---

## Timeline Estimate

| Phase | Effort | Priority | Dependencies |
|-------|--------|----------|--------------|
| Phase 1: Horizontal Toolbar | 4-6 hours | High | None |
| Phase 2: Status Bar | 2-3 hours | High | None |
| Phase 3: Viewport Overlays | 3-4 hours | Medium | None |
| Phase 4: Outliner Panel | 6-8 hours | Medium | None |
| Phase 5: Properties Cleanup | 3-4 hours | Medium | Phase 4 |

**Total Estimated Effort**: 18-25 hours

---

## Future Enhancements (Out of Scope)

- Dockable/rearrangeable panels
- Custom themes/dark mode toggle
- Minimap for large maps
- Asset browser for textures/models
- Undo history panel
- Viewport split views
