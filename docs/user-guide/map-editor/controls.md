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
| **Place Voxel** | Left-click (Voxel Place Tool) |
| **Drag Place Voxels** | Left-click + Drag (Voxel Place Tool) |
| **Remove Voxel** | Left-click (Voxel Remove Tool) |
| **Drag Remove Voxels** | Left-click + Drag (Voxel Remove Tool) |
| **Place Entity** | Left-click (Entity Tool active) |
| **Select Item** | Left-click (Select Tool active) |
| **Drag Select** | Left-click + Drag (Select Tool) |
| **Toggle Selection** | Click on already selected voxel |

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
| **Open Recent** | - | File → Open Recent → [file] |
| **Save** | `Ctrl+S` | File → Save |
| **Save As** | `Ctrl+Shift+S` | File → Save As |
| **Exit** | `Ctrl+Q` | File → Exit |

> **Tip:** The **Open Recent** submenu shows up to 10 recently opened map files for quick access. Files are automatically added when you open or save maps, and the list persists between editor sessions.

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

| Action | Shortcut | Alternative | Toolbar Button |
|--------|----------|-------------|----------------|
| **Select Tool** | `V` | `2` | First button |
| **Voxel Place Tool** | `B` | `1` | Second button |
| **Voxel Remove Tool** | `X` | - | - |
| **Entity Tool** | `E` | - | Third button |
| **Camera Tool** | `C` | - | Fourth button |

**Quick Tool Switching:**
- Press `V` or `2` to switch to Select tool (for editing and moving)
- Press `B` or `1` to switch to Voxel Place tool (most common for building)
- Press `X` to switch to Voxel Remove tool
- Press `E` to switch to Entity Place tool
- Press `C` to switch to Camera tool
- Number keys work from anywhere (except when typing in text fields)
- **Tool parameters are remembered** - when you switch back to a tool, it restores your previous settings (e.g., voxel type, pattern, entity type)

### Help

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| **Keyboard Shortcuts** | `F1` | Help → Keyboard Shortcuts |
| **About** | - | Help → About |

## Tool-Specific Controls

### Keyboard Cursor Navigation

The map editor features a **vim-like keyboard editing mode** that allows you to navigate the 3D grid without using the mouse.

#### Entering/Exiting Keyboard Mode

| Action | Control |
|--------|---------|
| **Enter Keyboard Mode** | `I` (like vim's insert mode) |
| **Exit Keyboard Mode** | `Escape` (when no selections) |
| **Clear Selections** | `Escape` (in Select tool with selections) |

When keyboard edit mode is active, you'll see a green **⌨ KEYBOARD MODE** indicator in the status bar.

**Note:** When using the Select tool with active selections, pressing `Escape` will first clear the selections and keep you in keyboard mode. Press `Escape` again (with no selections) to exit keyboard mode.

#### Movement Controls (When in Keyboard Mode)

| Action | Control | Alternative |
|--------|---------|-------------|
| **Move Forward** | `Arrow Up` | - |
| **Move Backward** | `Arrow Down` | - |
| **Move Left** | `Arrow Left` | - |
| **Move Right** | `Arrow Right` | - |
| **Move Up** | `Space` | - |
| **Move Down** | `C` | - |
| **Fast Movement** | Hold `Shift` + any direction | Moves 5 units instead of 1 |
| **Select Voxel** | `Enter` (Select Tool only) | Toggles selection at cursor position |

#### Behavior Notes

- **Mouse Override Prevention**: When in keyboard mode, mouse movements won't affect the cursor position
- **Tool Compatibility**: Works with all tools except Camera tool
- **Transform Operations**: Cursor movement is **disabled** during Move/Rotate operations to keep the cursor stationary while transforming selections
- **UI Focus**: Won't interfere when typing in text fields (UI has focus)
- **Visual Feedback**: The cursor indicator updates in real-time as you navigate

#### Workflow Example

1. Press `I` to enter keyboard edit mode (status bar shows **⌨ KEYBOARD MODE**)
2. Use arrow keys to navigate horizontally (X/Z plane)
3. Press `Space` to move up or `C` to move down (Y axis)
4. Hold `Shift` + arrow keys for faster navigation (5 units at a time)
5. Use tool-specific controls:
   - With Select Tool: Press `Enter` to select/deselect voxel at cursor
   - With Voxel Tool: Left-click to place voxel
   - With Entity Tool: Left-click to place entity
6. Press `Escape` to return to normal mouse control when done

### Voxel Place Tool (`B`)

When the Voxel Place Tool is active:

| Action | Control |
|--------|---------|
| **Place Voxel** | Left-click in viewport |
| **Drag Place** | Left-click + Drag to place multiple voxels |
| **Change Type** | Use dropdown in Properties panel |
| **Change Pattern** | Use dropdown in Properties panel |

> **Tip:** When dragging to place voxels, they are placed in the direction of your cursor movement, extending from the last placed voxel. This makes it easy to draw lines and walls.

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

### Voxel Remove Tool (`X`)

When the Voxel Remove Tool is active:

| Action | Control |
|--------|---------|
| **Remove Voxel** | Left-click on voxel |
| **Drag Remove** | Left-click + Drag to remove multiple voxels |
| **Quick Delete** | `Delete` or `Backspace` key |

> **Tip:** Drag across voxels to quickly clear areas. Each voxel under the cursor as you drag will be removed.

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
| **Select Voxel** | Left-click on voxel (works in full 3D space) |
| **Drag Select** | Left-click + Drag to select multiple voxels |
| **Select Voxel (Keyboard)** | `Enter` (when in keyboard mode) |
| **Toggle Selection** | Click on already selected voxel (without dragging) |
| **Delete Selected** | `Delete` or `Backspace` key |
| **Delete via UI** | Click "🗑 Delete Selected" button in Properties panel |
| **Clear Selection** | Click "Clear Selection" button in Properties panel |

> **Tip:** Drag across voxels to quickly select multiple voxels at once. Clicking on an already-selected voxel will deselect it (if you don't drag).

**Visual Feedback:**
- Selected voxels are highlighted with a bright yellow wireframe outline
- Properties panel shows count and positions of selected voxels
- Selection persists until cleared or tool is changed
- **3D Selection**: Can select voxels at any height in the 3D space, not just ground level

### Move Mode (`G` in Select Tool)

When in Move mode (after pressing `G` with selected voxels):

| Action | Control | Alternative |
|--------|---------|-------------|
| **Move Forward/Back** | `Arrow Up/Down` | - |
| **Move Left/Right** | `Arrow Left/Right` | - |
| **Move Up (Jump)** | `Space` | `Page Up` |
| **Move Down (Crouch)** | `C` | `Page Down` |
| **Fast Movement** | Hold `Shift` + any direction | Moves 5 units instead of 1 |
| **Confirm Move** | `Enter` | - |
| **Cancel Move** | `Escape` | - |

**Visual Feedback:**
- Preview of moved voxels shown in real-time
- Invalid positions (collisions) shown in red
- Valid positions shown in green
- Current offset displayed in Properties panel


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
- **Selection highlighting** - Yellow wireframe outlines around selected voxels
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