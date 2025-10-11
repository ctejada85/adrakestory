# Map Editor - Troubleshooting

Common issues and solutions for the A Drake's Story Map Editor.

## Installation & Launch Issues

### Editor Won't Launch

**Problem**: Running `cargo run --bin map_editor` fails

**Solutions**:
1. Ensure you're in the project root directory
2. Check that Rust is installed: `rustc --version`
3. Try rebuilding: `cargo clean && cargo build --bin map_editor --release`
4. Check for compilation errors in the output

**Common Errors**:
```bash
# Error: binary not found
Solution: Run `cargo build --bin map_editor` first

# Error: dependency issues
Solution: Run `cargo update` then rebuild
```

### Window Doesn't Appear

**Problem**: Editor launches but no window appears

**Solutions**:
1. Check if window is behind other windows
2. Try different display if using multiple monitors
3. Check graphics drivers are up to date
4. Look for error messages in terminal

### Slow Performance

**Problem**: Editor is laggy or unresponsive

**Solutions**:
1. Use release build: `cargo run --bin map_editor --release`
2. Close other applications
3. Reduce map size if working with large maps
4. Disable grid if not needed (press `G`)

## UI Issues

### UI Elements Not Visible

**Problem**: Panels or buttons are missing

**Solutions**:
1. Try resizing the window
2. Check View menu for hidden panels
3. Reset window layout (coming soon)
4. Restart the editor

### Text Too Small/Large

**Problem**: UI text is hard to read

**Solutions**:
1. Adjust system display scaling
2. UI scaling options coming in future update
3. Try different window size

### Properties Panel Empty

**Problem**: Properties panel shows no content

**Solutions**:
1. Select a tool from the toolbar
2. Click in the viewport to give it focus
3. Check that a map is loaded
4. Restart the editor if issue persists

## Editing Issues

### Can't Place Voxels

**Problem**: Clicking in viewport doesn't place voxels

**Solutions**:
1. Ensure Voxel Tool is selected (press `B`)
2. Check that viewport has focus (click in it)
3. Verify you're clicking within map bounds
4. Check that snap is not preventing placement

### Voxels Appear in Wrong Location

**Problem**: Voxels don't appear where you click

**Solutions**:
1. Enable snap to grid (press `Shift+G`)
2. Check camera angle - try top view (`Numpad 7`)
3. Zoom in for more precision
4. Verify cursor indicator position

### Can't Remove Voxels

**Problem**: Right-clicking doesn't remove voxels

**Solutions**:
1. Ensure you're right-clicking directly on a voxel
2. Check that Voxel Tool is active (press `B`)
3. Try selecting the voxel first, then press `Delete`
4. Verify the voxel isn't protected (future feature)

### Undo/Redo Not Working

**Problem**: `Ctrl+Z` doesn't undo changes

**Solutions**:
1. Check that changes were actually made
2. Verify keyboard focus is on editor window
3. Look at status bar for undo count
4. Some operations may not be undoable yet (see implementation status)

### Entity Placement Issues

**Problem**: Can't place entities

**Solutions**:
1. Select Entity Tool (press `E`)
2. Choose entity type from Properties panel
3. Click on a voxel surface, not empty space
4. Check that entity limit isn't reached (future feature)

## Camera Issues

### Camera Controls Not Working

**Problem**: Mouse doesn't move camera

**Solutions**:
1. Ensure viewport has focus (click in it)
2. Try different mouse button (right-click for orbit)
3. Check that Camera Tool isn't active
4. Reset camera with `Home` key

### Camera Stuck or Inverted

**Problem**: Camera behaves strangely

**Solutions**:
1. Press `Home` to reset camera
2. Try switching to different view (numpad keys)
3. Restart editor if issue persists
4. Check mouse sensitivity settings (coming soon)

### Can't See Map

**Problem**: Viewport is black or empty

**Solutions**:
1. Check that a map is loaded
2. Try zooming out (scroll wheel)
3. Reset camera (`Home` key)
4. Verify map has voxels placed
5. Check lighting settings (coming soon)

## File Operations Issues

### Can't Save Map

**Problem**: Save operation fails

**Solutions**:
1. Check file permissions in target directory
2. Ensure filename is valid (no special characters)
3. Try "Save As" to different location
4. Check disk space
5. Look for error messages in terminal

### Can't Open Map

**Problem**: Open dialog doesn't show files or fails to load

**Solutions**:
1. Verify file is valid RON format
2. Check file isn't corrupted
3. Try opening in text editor to verify format
4. Look for error messages in terminal
5. Try loading a known-good map first

### File Dialog Doesn't Appear

**Problem**: Open/Save dialog doesn't show

**Solutions**:
1. Check if dialog is behind main window
2. Try clicking in main window then trying again
3. Check system file dialog permissions
4. Restart editor

### Unsaved Changes Warning

**Problem**: Warning appears even after saving

**Solutions**:
1. This is a known issue - will be fixed
2. Check status bar for modified indicator (*)
3. Try saving again
4. Restart editor if issue persists

## Performance Issues

### Slow Rendering

**Problem**: 3D viewport is laggy

**Solutions**:
1. Use release build
2. Reduce map size
3. Disable grid (press `G`)
4. Close other applications
5. Update graphics drivers

### High Memory Usage

**Problem**: Editor uses too much RAM

**Solutions**:
1. Work with smaller maps
2. Restart editor periodically
3. Close unused applications
4. This will be optimized in future updates

### Slow File Operations

**Problem**: Saving/loading takes too long

**Solutions**:
1. This is normal for large maps
2. Use SSD if available
3. Reduce map complexity
4. Optimization coming in future updates

## Data Issues

### Map Validation Errors

**Problem**: Editor shows validation errors

**Solutions**:
1. Read error messages carefully
2. Common issues:
   - Missing PlayerSpawn entity
   - Voxels outside world bounds
   - Invalid voxel types
3. Fix issues one at a time
4. Save after each fix

### Lost Work

**Problem**: Changes disappeared

**Solutions**:
1. Check if file was saved (look for * in status bar)
2. Look for auto-save files (coming soon)
3. Check recent files list
4. Prevention: Save frequently with `Ctrl+S`

### Corrupted Map File

**Problem**: Map file won't load or is corrupted

**Solutions**:
1. Try opening in text editor
2. Check RON syntax
3. Restore from backup if available
4. Start with a working map and rebuild

## Platform-Specific Issues

### macOS

**Problem**: Permission errors

**Solutions**:
1. Grant file access permissions in System Preferences
2. Run from terminal with proper permissions
3. Check Gatekeeper settings

### Linux

**Problem**: Missing dependencies

**Solutions**:
1. Install required libraries (see installation guide)
2. Update graphics drivers
3. Check Wayland vs X11 compatibility

### Windows

**Problem**: Antivirus blocking

**Solutions**:
1. Add exception for editor executable
2. Run as administrator if needed
3. Check Windows Defender settings

## Getting More Help

### Check Logs

Look for error messages in:
- Terminal output where you launched the editor
- System logs (platform-specific)

### Report Issues

If you encounter a bug:
1. Note the exact steps to reproduce
2. Check implementation status document
3. Report on GitHub with:
   - Your OS and version
   - Rust version (`rustc --version`)
   - Error messages
   - Steps to reproduce

### Known Limitations

See [Implementation Status](../../developer-guide/systems/map-editor/implementation-status.md) for:
- Features not yet implemented
- Known bugs
- Planned improvements

## Workarounds

### Temporary Solutions

While features are being implemented:

**File I/O**: Currently UI only - actual save/load coming soon
**Voxel Rendering**: Voxels don't appear in 3D yet - coming soon
**Keyboard Shortcuts**: Some shortcuts defined but not wired yet

### Manual Editing

If editor fails, you can:
1. Edit RON files directly in text editor
2. Use existing map as template
3. Validate with game's map loader

## Prevention Tips

### Best Practices

1. **Save frequently**: Use `Ctrl+S` often
2. **Start small**: Test with small maps first
3. **Use version control**: Keep map files in git
4. **Test in game**: Load maps in game regularly
5. **Keep backups**: Copy important maps before editing

### Avoiding Issues

1. Don't edit very large maps in debug mode
2. Close editor when not in use
3. Keep software updated
4. Use valid filenames (no special characters)
5. Stay within map dimension limits

## See Also

- [Getting Started Guide](getting-started.md) - Basic usage
- [Controls Reference](controls.md) - All controls
- [Implementation Status](../../developer-guide/systems/map-editor/implementation-status.md) - Current state
- [GitHub Issues](https://github.com/yourusername/adrakestory/issues) - Report bugs

---

**Still having issues?** Check the implementation status document to see if your issue is a known limitation, or report it on GitHub.