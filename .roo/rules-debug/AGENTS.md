# AGENTS.md - Debug Mode

This file provides guidance for debug mode when working with this repository.

## Debug Commands

```bash
cargo run -- --map assets/maps/simple_test.ron  # Skip title screen, load specific map
```

## In-Game Debug Keys

- `C` - Toggle collision box visualization (shows cylinder collider)
- `F3` - Toggle FPS counter overlay
- `ESC` - Pause menu

## VSCode Debugging

- Use "Debug (CodeLLDB, Debug Build)" launch config for breakpoints
- LLDB configured to hide disassembly - see [`.vscode/settings.json`](../../.vscode/settings.json:11)
- Debug builds have `opt-level = 1` for faster iteration while dependencies use `opt-level = 3`

## Common Issues

- **Physics explosion after alt-tab**: Delta time is clamped to 0.1s max in physics systems. If you see teleporting, check delta clamping
- **Camera order ambiguity warning**: 2D and 3D cameras must not coexist. Check [`cleanup_2d_camera()`](../../src/main.rs:282-287) runs on state transition
- **Collision not working**: Verify SubVoxel entities are in SpatialGrid. Check [`spawn_voxels_chunked()`](../../src/systems/game/map/spawner.rs:1199-1218) for grid insertion
- **Map not loading**: Check RON syntax. Validation errors logged via `warn!()`. See [`validation.rs`](../../src/systems/game/map/validation.rs)

## Hot Reload

- F5 or Ctrl+R reloads map file when running from editor
- Ctrl+H toggles hot reload on/off
- Player position is preserved across reloads via [`restore_player_position()`](../../src/systems/game/hot_reload.rs)
- Debounce prevents rapid reloads - 500ms minimum between reloads

## Logging

- Use `RUST_LOG=info cargo run` for detailed logs
- Map loading progress logged at each stage
- Chunk/quad counts logged after spawn: "Spawned X chunks with Y quads"