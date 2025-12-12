# Map Editor - Getting Started

## Overview

The A Drake's Story Map Editor is a visual tool for creating and editing game maps. It provides an intuitive interface with 3D preview, making it easy to design custom levels without manually editing RON files.

## Launching the Editor

### From Command Line

```bash
# Build and run the map editor
cargo run --bin map_editor --release

# Or if already built
./target/release/map_editor
```

### From IDE

If using VSCode or another IDE, you can configure a launch target for the `map_editor` binary.

## First Steps

When you launch the map editor, it starts with a **blank canvas** - an empty workspace ready for you to create your map from scratch. The canvas has minimal dimensions (1×1×1) that will automatically expand as you place voxels.

### Creating a New Map

1. Launch the map editor - you'll see an empty 3D viewport
2. Start placing voxels immediately, or
3. Click **File → New** or press `Ctrl+N` to set map metadata:
   - **Name**: Your map's name
   - **Author**: Your name
   - **Description**: Brief description
4. Begin building your map!

**Note**: You don't need to worry about map dimensions - they automatically expand as you place voxels.

### Opening an Existing Map

1. Click **File → Open** or press `Ctrl+O`
2. A native file dialog will appear (UI remains responsive)
3. Navigate to `assets/maps/`
4. Select a `.ron` file (only `.ron` files are shown)
5. Click **Open**
6. The map will load and render in the 3D viewport

**Tip**: Use **File → Open Recent** to quickly access maps you've recently worked on. The editor remembers your last 10 opened files.

**Note**: The editor uses a non-blocking file dialog, so the UI stays responsive while you browse for files.

### Saving Your Work

- **Save**: `Ctrl+S` - Saves to current file (or prompts for location if new map)
- **Save As**: `Ctrl+Shift+S` - Saves to new file location
- The editor will warn you about unsaved changes when closing
- **Auto-Expand**: Map dimensions automatically expand to fit all voxels when saving

**Note**: If you place voxels outside the original map dimensions, the editor will automatically expand the map size when saving to ensure all voxels are included. You'll see a log message indicating the new dimensions.

## Interface Overview

```
┌─────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                    [X]        │
├─────────────────────────────────────────────────────────────┤
│ [New] [Open] [Save] | [Undo] [Redo] | [Grid] [Snap]        │
├──────────────────────────────────────┬──────────────────────┤
│                                      │  Properties Panel    │
│                                      │  - Tool settings     │
│         3D Viewport                  │  - Voxel type        │
│                                      │  - Pattern           │
│    (Your map appears here)           │  - Map metadata      │
│                                      │  - Statistics        │
│                                      │                      │
├──────────────────────────────────────┴──────────────────────┤
│ Status: Ready | Voxels: 23 | Entities: 1 | Modified: *     │
└─────────────────────────────────────────────────────────────┘
```

### Main Areas

1. **Menu Bar**: Access all features and commands
2. **Toolbar**: Quick access to common actions
3. **3D Viewport**: Interactive view of your map
4. **Properties Panel**: Edit tool settings and map properties
5. **Status Bar**: Current status and statistics

## Basic Editing

### Placing Voxels

1. Select the **Voxel Place Tool** (click toolbar button or press `B`)
2. Choose voxel type from Properties panel:
   - **Grass**: Green terrain
   - **Dirt**: Brown terrain
   - **Stone**: Gray terrain
3. Choose pattern:
   - **Full**: Solid block
   - **Platform**: Flat surface
   - **Staircase**: Diagonal steps
   - **Pillar**: Vertical column
4. Click in the viewport to place voxels
5. **Drag to place multiple**: Hold left-click and drag to draw lines of voxels

### Removing Voxels

1. Select the **Voxel Remove Tool** (press `X`)
2. Click on a voxel to remove it
3. **Drag to remove multiple**: Hold left-click and drag across voxels to remove them
4. Or select voxels with the Select Tool and press `Delete`

### Selecting Voxels

1. Select the **Select Tool** (press `V`)
2. Click on voxels to select them
3. **Drag to select multiple**: Hold left-click and drag across voxels to select them all
4. Click on a selected voxel to deselect it
5. Use `Delete` or `Backspace` to remove selected voxels

### Placing Entities

1. Select the **Entity Tool** (press `E`)
2. Choose entity type:
   - **PlayerSpawn**: Where the player starts
   - More entity types coming soon
3. Click in the viewport to place

### Camera Controls

- **Orbit**: Right-click and drag
- **Pan**: Multiple options available:
  - Middle-click and drag (traditional)
  - Shift + Right-click and drag
  - **Space + Left-click and drag** (trackpad-friendly)
  - **Cmd/Ctrl + Left-click and drag** (Mac trackpad-friendly)
- **Zoom**: Mouse wheel (smooth, reduced sensitivity)
- **Reset**: Press `Home` or click **View → Reset Camera**

**Tip for Mac Trackpad Users**: Use Space + Left-click or Cmd + Left-click for easy panning without a middle mouse button!

### Grid and Snapping

- **Toggle Grid**: Press `G` or click grid button
- **Toggle Snap**: Press `Shift+G` or click snap button
- Grid helps align voxels precisely

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Map | `Ctrl+N` |
| Open Map | `Ctrl+O` |
| Save | `Ctrl+S` |
| Save As | `Ctrl+Shift+S` |
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Y` |
| Delete | `Delete` or `Backspace` |
| Toggle Grid | `G` |
| Toggle Snap | `Shift+G` |
| Select Tool | `V` |
| Voxel Tool | `B` |
| Entity Tool | `E` |
| Camera Tool | `C` |
| Reset Camera | `Home` |

## Tips for Beginners

1. **Start Small**: Begin with a small map (10×5×10) to learn the tools
2. **Use the Grid**: Keep grid visible while learning
3. **Save Often**: Use `Ctrl+S` frequently - the editor auto-expands dimensions if needed
4. **Experiment**: Try different voxel types and patterns
5. **Test in Game**: Press `F5` to instantly test your map with hot reload support
6. **Don't Worry About Dimensions**: Place voxels freely - the editor will adjust map size automatically when saving
7. **Tool Memory**: When you switch tools, your settings (like voxel type or entity type) are remembered - switch back anytime and your previous selection will be restored

## Testing Your Map

The map editor includes a powerful **Play & Test** feature with hot reload support:

### Quick Testing

1. Press `F5` or click the **▶ Play** button in the toolbar
2. The game launches with your current map loaded
3. Walk around and test your creation!
4. Press `Shift+F5` or click **⏹ Stop** to close the game

### Hot Reload Workflow

While the game is running, you can edit and see changes in real-time:

1. Make changes to your map in the editor
2. Save with `Ctrl+S`
3. The game automatically detects the change and reloads the map
4. Your player position, rotation, and camera are preserved!

**Hot Reload Indicators:**
- **🔄 Hot Reload Active** - Shows in toolbar when game is running
- **"Map reloaded successfully"** - Green notification appears in-game after reload

**In-Game Controls:**
- `F5` or `Ctrl+R` - Manually trigger a reload
- `Ctrl+H` - Toggle hot reload on/off

## Common Tasks

### Creating a Simple Platform

1. Select Voxel Tool (`B`)
2. Choose **Grass** type
3. Choose **Full** pattern
4. Click to place a row of voxels
5. Build up layers to create a platform

### Adding Player Spawn

1. Select Entity Tool (`E`)
2. Choose **PlayerSpawn**
3. Click on your platform where you want the player to start
4. **Important**: Every map needs at least one PlayerSpawn!

### Editing Map Information

1. Look at the Properties panel on the right
2. Find the **Map Info** section
3. Edit:
   - Name
   - Author
   - Description
   - Version

## Next Steps

- Read the [Controls Reference](controls.md) for detailed control information
- Check out [Tips & Tricks](tips-and-tricks.md) for advanced techniques
- See [Troubleshooting](troubleshooting.md) if you encounter issues

## Getting Help

- Press `F1` or click **Help → Keyboard Shortcuts** for quick reference
- Click **Help → About** for version information
- Check the [Troubleshooting Guide](troubleshooting.md) for common issues

---

**Ready to create?** Start with **File → New** and begin building your first map!