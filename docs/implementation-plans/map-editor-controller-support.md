# Map Editor Controller Support Implementation Plan

## Status: ✅ IMPLEMENTED

## Overview

This document outlines the implementation plan for adding Xbox controller support to the Map Editor, enabling a Minecraft Creative Mode-style editing experience. The goal is to allow users to fly around the map, place/remove voxels, and switch between tools and materials using a gamepad.

## Implementation Summary

The controller support has been fully implemented with the following features:

### Core Features Implemented
- **First-person flying camera**: Minecraft Creative mode-style movement with left stick
- **Camera look control**: Right stick for looking around (yaw/pitch)
- **Vertical movement**: A button to fly up, B button to fly down
- **Raycast-based cursor**: Cursor appears on the voxel face you're looking at
- **Tool actions via triggers**: RT executes current tool, LT removes voxels
- **Automatic input switching**: Seamlessly switch between controller and mouse/keyboard

### Files Modified
- `src/editor/camera.rs` - Added GamepadCameraState, flying camera controls, and trigger actions
- `src/editor/cursor/mod.rs` - Made raycasting module public for controller use
- `src/bin/map_editor/main.rs` - Added controller systems to the app

## Controller Mapping

### Movement & Camera
| Action | Controller | Description |
|--------|------------|-------------|
| Move/Fly | Left Stick | Fly in the direction you push (forward/back/strafe) |
| Look | Right Stick | Rotate camera view (yaw and pitch) |
| Fly Up | A Button (hold) | Ascend vertically |
| Fly Down | B Button (hold) | Descend vertically |
| Reset Camera | Y Button | Reset camera to default position |

### Editing Actions
| Action | Controller | Description |
|--------|------------|-------------|
| Primary Action | RT (Right Trigger) | Execute current tool's action |
| Remove Voxel | LT (Left Trigger) | Always removes voxel (secondary action) |

### Tool-Specific RT Behavior
| Tool | RT Action |
|------|-----------|
| Voxel Place | Places voxel at cursor position |
| Voxel Remove | Removes voxel you're looking at |
| Entity Place | Places entity at cursor position |
| Select | Toggles selection on voxel you're looking at |
| Camera | No action |

## Technical Details

### Input Detection
The system supports both axis-based triggers (`RightZ`/`LeftZ`) and button-based triggers (`RightTrigger2`/`LeftTrigger2`) to work with different controller types.

### Automatic Input Switching
- Moving sticks activates gamepad mode (hides mouse cursor)
- Moving mouse or clicking deactivates gamepad mode (shows cursor)
- Both input methods work seamlessly without manual switching

### Raycast Cursor System
When gamepad is active:
1. A ray is cast from camera position in the look direction
2. If it hits a voxel, cursor appears on the hit face for placement
3. If no voxel hit, falls back to ground plane (y=0)
4. If neither, uses fixed distance fallback

### Deadzone Handling
- 15% deadzone on analog sticks
- Smooth rescaling for precise control above deadzone
