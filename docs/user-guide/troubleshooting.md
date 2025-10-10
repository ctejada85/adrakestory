# Troubleshooting Guide

Solutions to common issues and problems in A Drake's Story.

## Installation Issues

### Rust/Cargo Not Found

**Problem:** `cargo: command not found` or `rustc: command not found`

**Solution:**
1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Restart your terminal
3. Verify installation:
   ```bash
   cargo --version
   rustc --version
   ```

### Build Fails on macOS

**Problem:** Missing Xcode Command Line Tools

**Solution:**
```bash
xcode-select --install
```

### Build Fails on Linux

**Problem:** Missing development libraries

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libudev-dev libasound2-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install alsa-lib-devel systemd-devel

# Arch
sudo pacman -S alsa-lib systemd
```

### Build Fails on Windows

**Problem:** Missing Visual Studio Build Tools

**Solution:**
1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. Install "Desktop development with C++"
3. Restart terminal
4. Try building again

## Compilation Issues

### First Build Takes Forever

**Problem:** Initial compilation is very slow

**Explanation:** This is normal! Cargo is downloading and compiling all dependencies (especially Bevy, which is large).

**Solutions:**
- Be patient (first build can take 5-15 minutes)
- Subsequent builds will be much faster
- Use `cargo build` (debug) for faster compilation during development
- Use `cargo build --release` only when you need performance

### Out of Memory During Build

**Problem:** Build fails with memory errors

**Solutions:**
1. Close other applications
2. Increase swap space (Linux)
3. Build with fewer parallel jobs:
   ```bash
   cargo build -j 2
   ```

### Linker Errors

**Problem:** Linking fails with undefined symbols

**Solutions:**
1. Clean and rebuild:
   ```bash
   cargo clean
   cargo build
   ```
2. Update Rust:
   ```bash
   rustup update
   ```
3. Check platform-specific dependencies are installed

## Runtime Issues

### Game Won't Start

**Problem:** Game crashes immediately or won't launch

**Checklist:**
1. ✅ Built successfully? Check for compilation errors
2. ✅ Using correct command? Try `cargo run --release`
3. ✅ Graphics drivers updated?
4. ✅ Check console for error messages

**Solutions:**
- Update graphics drivers
- Try debug build: `cargo run`
- Check system requirements
- Look for error messages in terminal

### Black Screen

**Problem:** Game launches but shows black screen

**Solutions:**
1. Wait a few seconds (loading)
2. Check if intro animation is playing
3. Try pressing any key to skip intro
4. Check graphics drivers
5. Try windowed mode (if available)

### Poor Performance / Low FPS

**Problem:** Game runs slowly or stutters

**Solutions:**

**1. Use Release Mode**
```bash
cargo run --release
```
Debug builds are much slower!

**2. Close Background Apps**
- Close web browsers
- Close other games
- Free up system resources

**3. Update Graphics Drivers**
- NVIDIA: GeForce Experience
- AMD: Radeon Software
- Intel: Intel Driver & Support Assistant

**4. Check System Resources**
- Monitor CPU/GPU usage
- Check available RAM
- Close unnecessary processes

**5. Disable Debug Features**
- Press `C` to turn off collision boxes
- Reduce map complexity (for custom maps)

### Game Freezes

**Problem:** Game stops responding

**Solutions:**
1. Wait 10-15 seconds (might be loading)
2. Check console for errors
3. Force quit and restart
4. Try debug build for better error messages
5. Check system resources

## Map Loading Issues

### Map Won't Load

**Problem:** Map fails to load or shows error

**Common Causes:**

**1. File Not Found**
```
Error: Failed to read map file
```
**Solution:** Check file path and name
```bash
ls assets/maps/
```

**2. Invalid RON Syntax**
```
Error: Failed to parse map data
```
**Solution:** 
- Check for missing commas
- Verify parentheses match
- Check quotes around strings
- Use a RON validator

**3. Validation Failed**
```
Error: Map validation failed
```
**Solution:** Check validation requirements:
- At least one PlayerSpawn entity
- Voxel positions within world bounds
- Positive world dimensions
- Valid lighting values (0.0-1.0)
- Version starts with "1."

### Map Loads But Looks Wrong

**Problem:** Map loads but appears incorrect

**Checklist:**

**1. Player Spawns in Wrong Place**
- Check PlayerSpawn position coordinates
- Ensure Y coordinate is above ground
- Verify position is within world bounds

**2. Voxels Missing**
- Check voxel positions are within bounds
- Verify voxel_type is valid
- Check pattern is specified correctly

**3. Too Dark/Bright**
- Adjust `ambient_intensity` (0.2-0.4 recommended)
- Check directional light `illuminance`
- Verify light color values (0.0-1.0)

**4. Camera Angle Wrong**
- Adjust camera `position`
- Change `look_at` target
- Try `rotation_offset: -1.5707963` for isometric

### Map Loads Slowly

**Problem:** Loading takes a long time

**Causes:**
- Very large map (many voxels)
- Complex sub-voxel patterns
- Debug build (slower)

**Solutions:**
1. Use release build: `cargo run --release`
2. Reduce map size
3. Use simpler patterns (Platform instead of Full)
4. Optimize voxel count

## Control Issues

### Mouse Not Working

**Problem:** Mouse doesn't control camera

**Solutions:**
1. Click in game window to focus
2. Check mouse is connected
3. Try keyboard controls (Q/E)
4. Restart game
5. Check system mouse settings

### Keys Not Responding

**Problem:** Keyboard controls don't work

**Solutions:**
1. Click in game window to focus
2. Check keyboard layout (QWERTY assumed)
3. Try different keys
4. Check for conflicting system shortcuts
5. Restart game

### Camera Feels Wrong

**Problem:** Camera movement is too fast/slow/jerky

**Solutions:**
1. Adjust system mouse sensitivity
2. Try keyboard controls (Q/E) instead
3. Experiment with different viewing angles
4. Check if mouse acceleration is enabled
5. Try different mouse surface

### Can't Pause Game

**Problem:** ESC key doesn't pause

**Solutions:**
1. Ensure game window has focus
2. Try clicking in window first
3. Check if already in pause menu
4. Restart game if stuck

## Graphics Issues

### Flickering or Artifacts

**Problem:** Visual glitches or flickering

**Solutions:**
1. Update graphics drivers
2. Try different graphics settings (when available)
3. Check GPU temperature
4. Reduce map complexity
5. Try different display mode

### Collision Boxes Always Visible

**Problem:** Can't turn off green wireframes

**Solution:** Press `C` key to toggle collision visualization

### Textures Look Wrong

**Problem:** Voxels have incorrect colors or appearance

**Solutions:**
1. Check voxel_type in map file
2. Verify assets are present
3. Rebuild: `cargo clean && cargo build --release`
4. Check for asset loading errors in console

## Debug Mode Issues

### Breakpoints Not Hit

**Problem:** Debugger doesn't stop at breakpoints

**Solutions:**
1. Ensure using debug build (not release)
2. Check breakpoint is in executed code
3. Verify CodeLLDB extension is installed
4. See [Debugging Guide](../developer-guide/debugging.md)

### Can't See Variables

**Problem:** Variable values not visible in debugger

**Solutions:**
1. Use debug build
2. Check optimization level
3. Try different debug configuration
4. See [Debugging Guide](../developer-guide/debugging.md)

## Platform-Specific Issues

### macOS: "App is damaged"

**Problem:** macOS blocks the app

**Solution:**
```bash
xattr -cr target/release/adrakestory
```

### Linux: Permission Denied

**Problem:** Can't execute binary

**Solution:**
```bash
chmod +x target/release/adrakestory
```

### Windows: Antivirus Blocks

**Problem:** Antivirus flags the game

**Solution:**
1. Add exception for project directory
2. Rebuild after adding exception
3. This is a false positive (common with Rust games)

## Getting Help

### Before Asking for Help

1. ✅ Check this troubleshooting guide
2. ✅ Search existing GitHub issues
3. ✅ Try the solutions above
4. ✅ Collect error messages
5. ✅ Note your system info

### When Reporting Issues

Include:
- **Operating System:** (macOS/Linux/Windows + version)
- **Rust Version:** Output of `rustc --version`
- **Build Command:** What command you ran
- **Error Messages:** Full error output
- **Steps to Reproduce:** What you did before the error
- **Expected vs Actual:** What should happen vs what happened

### Where to Get Help

1. **GitHub Issues:** Report bugs and problems
2. **Documentation:** Check other docs pages
3. **Community:** (if available) Discord/forum
4. **Code:** Read the source for understanding

## Common Error Messages

### "Failed to read map file"
→ Check file path and permissions

### "Failed to parse map data"
→ Check RON syntax (commas, parentheses, quotes)

### "Map validation failed"
→ Check validation requirements (see [Map Format](maps/map-format.md))

### "No player spawn entity found"
→ Add at least one PlayerSpawn entity

### "Invalid voxel position"
→ Voxel position exceeds world dimensions

### "Version must start with '1.'"
→ Use version format like "1.0.0"

### "Lighting values must be between 0.0 and 1.0"
→ Check ambient_intensity and color values

## Still Having Issues?

If you've tried everything and still have problems:

1. **Open a GitHub Issue** with:
   - Detailed description
   - System information
   - Error messages
   - Steps to reproduce

2. **Check Documentation**:
   - [Installation Guide](../getting-started/installation.md)
   - [Quick Start](../getting-started/quick-start.md)
   - [Developer Guide](../developer-guide/architecture.md)

3. **Review Examples**:
   - [Example Maps](maps/examples.md)
   - [Creating Maps](maps/creating-maps.md)

---

**Most issues have simple solutions!** Don't hesitate to ask for help if you're stuck.