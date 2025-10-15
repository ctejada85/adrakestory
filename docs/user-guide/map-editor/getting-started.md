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

### Creating a New Map

1. Launch the map editor
2. Click **File → New** or press `Ctrl+N`
3. Enter map details in the dialog:
   - **Name**: Your map's name
   - **Author**: Your name
   - **Description**: Brief description
   - **Dimensions**: Width, Height, Depth (in voxels)
4. Click **Create**

### Opening an Existing Map

1. Click **File → Open** or press `Ctrl+O`
2. A native file dialog will appear (UI remains responsive)
3. Navigate to `assets/maps/`
4. Select a `.ron` file (only `.ron` files are shown)
5. Click **Open**
6. The map will load and render in the 3D viewport

**Note**: The editor uses a non-blocking file dialog, so the UI stays responsive while you browse for files.

### Saving Your Work

- **Save**: `Ctrl+S` - Saves to current file
- **Save As**: `Ctrl+Shift+S` - Saves to new file
- The editor will warn you about unsaved changes when closing

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

1. Select the **Voxel Tool** (click toolbar button or press `B`)
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

### Removing Voxels

1. Select the **Voxel Tool**
2. Right-click on a voxel to remove it
3. Or select voxels and press `Delete`

### Placing Entities

1. Select the **Entity Tool** (press `E`)
2. Choose entity type:
   - **PlayerSpawn**: Where the player starts
   - More entity types coming soon
3. Click in the viewport to place

### Camera Controls

- **Orbit**: Right-click and drag
- **Pan**: Middle-click and drag (or Shift + drag)
- **Zoom**: Mouse wheel
- **Reset**: Press `Home` or click **View → Reset Camera**

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
3. **Save Often**: Use `Ctrl+S` frequently
4. **Experiment**: Try different voxel types and patterns
5. **Test in Game**: Load your map in the game to see how it plays

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