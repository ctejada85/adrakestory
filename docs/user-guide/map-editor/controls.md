# Map Editor - Controls Reference

Complete reference for all map editor controls and shortcuts.

## Mouse Controls

### 3D Viewport

| Action | Control |
|--------|---------|
| **Orbit Camera** | Right-click + Drag |
| **Pan Camera** | Middle-click + Drag |
| **Pan Camera (Alt 1)** | Shift + Right-click + Drag |
| **Pan Camera (Alt 2)** | Space + Left-click + Drag |
| **Pan Camera (Trackpad)** | Cmd/Ctrl + Left-click + Drag |
| **Zoom In/Out** | Mouse Wheel |
| **Place Voxel** | Left-click (Voxel Tool active) |
| **Remove Voxel** | Right-click on voxel (Voxel Tool active) |
| **Place Entity** | Left-click (Entity Tool active) |
| **Select Item** | Left-click (Select Tool active) |
| **Multi-Select** | Shift + Left-click |

### UI Interactions

| Action | Control |
|--------|---------|
| **Click Button** | Left-click |
| **Open Dropdown** | Left-click on dropdown |
| **Adjust Slider** | Left-click + Drag |
| **Edit Text Field** | Left-click to focus, type to edit |

## Keyboard Shortcuts

### File Operations

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| **New Map** | `Ctrl+N` | File → New |
| **Open Map** | `Ctrl+O` | File → Open |
| **Save** | `Ctrl+S` | File → Save |
| **Save As** | `Ctrl+Shift+S` | File → Save As |
| **Exit** | `Ctrl+Q` | File → Exit |

### Edit Operations

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| **Undo** | `Ctrl+Z` | Edit → Undo |
| **Redo** | `Ctrl+Y` | Edit → Redo |
| **Redo (Alt)** | `Ctrl+Shift+Z` | Edit → Redo |
| **Delete** | `Delete` | Edit → Delete |
| **Delete (Alt)** | `Backspace` | Edit → Delete |
| **Select All** | `Ctrl+A` | Edit → Select All |
| **Deselect All** | `Ctrl+D` | Edit → Deselect All |

### View Controls

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| **Toggle Grid** | `G` | View → Toggle Grid |
| **Toggle Snap** | `Shift+G` | View → Toggle Snap |
| **Reset Camera** | `Home` | View → Reset Camera |
| **Top View** | `Numpad 7` | View → Top View |
| **Front View** | `Numpad 1` | View → Front View |
| **Side View** | `Numpad 3` | View → Side View |
| **Isometric View** | `Numpad 5` | View → Isometric View |

### Tool Selection

| Action | Shortcut | Toolbar Button |
|--------|----------|----------------|
| **Select Tool** | `V` | First button |
| **Voxel Tool** | `B` | Second button |
| **Entity Tool** | `E` | Third button |
| **Camera Tool** | `C` | Fourth button |

### Help

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| **Keyboard Shortcuts** | `F1` | Help → Keyboard Shortcuts |
| **About** | - | Help → About |

## Tool-Specific Controls

### Voxel Tool (`B`)

When the Voxel Tool is active:

| Action | Control |
|--------|---------|
| **Place Voxel** | Left-click in viewport |
| **Remove Voxel** | Right-click on voxel |
| **Change Type** | Use dropdown in Properties panel |
| **Change Pattern** | Use dropdown in Properties panel |
| **Quick Delete** | Hold `Shift` + Left-click |

**Available Voxel Types:**
- Grass (Green terrain)
- Dirt (Brown terrain)
- Stone (Gray terrain)
- Air (Empty space)

**Available Patterns:**
- Full (Solid block)
- Platform (Flat surface)
- Staircase (Diagonal steps)
- Pillar (Vertical column)

### Entity Tool (`E`)

When the Entity Tool is active:

| Action | Control |
|--------|---------|
| **Place Entity** | Left-click in viewport |
| **Select Entity** | Left-click on entity |
| **Move Entity** | Drag selected entity |
| **Delete Entity** | Select + `Delete` key |
| **Change Type** | Use dropdown in Properties panel |

**Available Entity Types:**
- PlayerSpawn (Required - where player starts)
- More types coming soon

### Select Tool (`V`)

When the Select Tool is active:

| Action | Control |
|--------|---------|
| **Select Single** | Left-click on item |
| **Multi-Select** | `Shift` + Left-click |
| **Box Select** | Left-click + Drag (coming soon) |
| **Deselect** | Click empty space |
| **Delete Selected** | `Delete` key |

### Camera Tool (`C`)

When the Camera Tool is active:

| Action | Control |
|--------|---------|
| **Free Look** | Left-click + Drag |
| **Pan** | Middle-click + Drag |
| **Zoom** | Mouse Wheel |
| **Reset** | `Home` key |

## Camera Navigation

### Orbit Mode (Default)

- **Rotate**: Right-click + Drag
- **Pan**: Middle-click + Drag, or Shift + Right-click + Drag
- **Pan (Trackpad-Friendly)**: Space + Left-click + Drag, or Cmd/Ctrl + Left-click + Drag
- **Zoom**: Mouse Wheel
- **Focus**: Double-click on voxel (coming soon)

### Pan Mode

- **Pan**: Left-click + Drag
- **Zoom**: Mouse Wheel

### Zoom Behavior

- **Zoom In**: Scroll wheel up
- **Zoom Out**: Scroll wheel down
- **Zoom Speed**: Adjustable in settings (coming soon)

## Grid Controls

### Grid Visibility

- **Toggle**: Press `G` or click grid button in toolbar
- **Opacity**: Adjust slider in Properties panel
- **Color**: Configurable in settings (coming soon)

### Snap to Grid

- **Toggle**: Press `Shift+G` or click snap button
- **When Enabled**: All placements snap to grid intersections
- **When Disabled**: Free placement (sub-grid precision)

## Properties Panel

### Navigation

- **Scroll**: Mouse wheel while hovering over panel
- **Collapse Section**: Click section header
- **Expand Section**: Click collapsed section header

### Editing Fields

- **Text Input**: Click field, type, press `Enter` to confirm
- **Number Input**: Click field, type number, or use arrow keys
- **Dropdown**: Click to open, click option to select
- **Checkbox**: Click to toggle
- **Slider**: Click and drag, or click track to jump

## Status Bar

The status bar shows:
- Current tool
- Voxel count
- Entity count
- Undo/Redo available actions
- Modified indicator (*)
- Current file name

## Tips

### Efficient Navigation

1. Use `Home` to reset camera when lost
2. Use numpad keys for quick view changes
3. Right-click drag for most navigation
4. Middle-click for precise panning
5. **Mac Trackpad Users**: Use Space + Left-click or Cmd + Left-click to pan

### Efficient Editing

1. Learn tool shortcuts (`V`, `B`, `E`, `C`)
2. Use `Ctrl+Z` liberally - undo is your friend
3. Enable snap (`Shift+G`) for precise placement
4. Use `Shift+Click` for multi-select

### Workflow Tips

1. Save frequently with `Ctrl+S`
2. Use grid for alignment
3. Test in game regularly
4. Keep Properties panel visible for quick adjustments

## Customization

### Keyboard Shortcuts

Currently, keyboard shortcuts are fixed. Custom key bindings will be available in a future update.

### Mouse Sensitivity

Camera sensitivity settings will be available in a future update.

## Accessibility

### Alternative Controls

- All mouse actions have keyboard alternatives
- All menu items are accessible via keyboard
- Tab navigation through UI elements

### Visual Aids

- Grid for alignment
- Cursor indicator for placement
- Selection highlighting
- Status bar feedback

## Troubleshooting

### Camera Issues

- **Camera stuck**: Press `Home` to reset
- **Can't rotate**: Ensure you're right-clicking in viewport
- **Zoom too fast/slow**: Adjust zoom speed in settings (coming soon)

### Input Not Working

- **Shortcuts not working**: Ensure viewport has focus
- **Can't place voxels**: Check that correct tool is selected
- **Can't select**: Ensure Select Tool (`V`) is active

## See Also

- [Getting Started Guide](getting-started.md) - Learn the basics
- [Tips & Tricks](tips-and-tricks.md) - Advanced techniques
- [Troubleshooting](troubleshooting.md) - Common issues

---

**Quick Reference Card**: Press `F1` in the editor for a quick reference overlay.