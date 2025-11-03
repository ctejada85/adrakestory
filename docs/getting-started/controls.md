# Controls Reference

Complete keyboard and mouse controls for A Drake's Story.

## Movement Controls

### Basic Movement
| Key | Action |
|-----|--------|
| **W** | Move forward |
| **A** | Move left (strafe) |
| **S** | Move backward |
| **D** | Move right (strafe) |

**Notes:**
- Movement is relative to camera direction
- Character automatically rotates to face movement direction
- Smooth rotation with 0.2 second duration (snappy, arcade-like)
- Physics system applies gravity and collision
- Speed is constant (no sprint/walk toggle yet)

### Character Rotation
The character model automatically rotates to face the direction you're moving:
- **Instant Response:** Rotation begins immediately when direction changes
- **Smooth Animation:** Uses ease-in-out cubic easing for natural movement
- **Fixed Duration:** All rotations (45°, 90°, 180°) take 0.2 seconds
- **Shortest Path:** Character always rotates the shortest way around
- **8 Directions:** Supports all cardinal and diagonal movement directions

## Camera Controls

### Mouse Control
| Input | Action |
|-------|--------|
| **Mouse Movement** | Rotate camera view |
| **Move Left/Right** | Pan camera horizontally |
| **Move Up/Down** | Tilt camera vertically |

### Keyboard Control
| Key | Action |
|-----|--------|
| **Q** | Rotate camera counter-clockwise around player |
| **E** | Rotate camera clockwise around player |

**Camera Features:**
- Isometric perspective for better spatial awareness
- Smooth rotation and movement
- Maintains fixed distance from player
- Automatically follows player movement

## System Controls

### Game Management
| Key | Action |
|-----|--------|
| **ESC** | Pause game / Open pause menu |
| **ESC** (in menu) | Resume game |

### Debug Controls
| Key | Action |
|-----|--------|
| **C** | Toggle collision box visualization |

**Debug Features:**
- Green wireframe boxes show collision boundaries
- Helps understand physics and collision detection
- Can be toggled on/off at any time
- Useful for map creation and testing

## Menu Navigation

### Title Screen
| Input | Action |
|-------|--------|
| **Mouse Click** | Select menu button |
| **Enter** | Start new game (when "New Game" is highlighted) |
| **Space** | Start new game (when "New Game" is highlighted) |
| **Arrow Keys** | Navigate menu options |
| **Tab** | Navigate menu options |

### Pause Menu
| Input | Action |
|-------|--------|
| **Mouse Click** | Select menu option |
| **Enter** | Confirm selection |
| **Space** | Confirm selection |
| **Arrow Keys** | Navigate options |
| **Tab** | Navigate options |
| **ESC** | Resume game |

## Control Tips

### Movement Tips
1. **Strafing**: Use A/D while moving forward (W) for diagonal movement
2. **Precision**: Release keys early when approaching edges
3. **Camera First**: Adjust camera before moving in tight spaces
4. **Physics**: Let gravity settle before making precise movements
5. **Direction Changes**: Character smoothly rotates when you change direction
6. **Visual Feedback**: Watch the character model to see which way you're facing

### Camera Tips
1. **Find Your Angle**: Experiment with Q/E to find comfortable viewing angles
2. **Mouse Sensitivity**: Adjust system mouse settings if camera feels too fast/slow
3. **Reset View**: Use Q/E to quickly reorient if you get disoriented
4. **Isometric Advantage**: The angled view helps judge distances and heights

### Debug Tips
1. **Learning Tool**: Enable collision boxes (C) when learning the game
2. **Map Testing**: Essential for testing custom maps
3. **Physics Understanding**: See exactly where collision boundaries are
4. **Performance**: Disable in normal play for slightly better performance

## Planned Controls

Future updates may include:

- **Jump** - Space bar (not yet implemented)
- **Sprint** - Shift key (not yet implemented)
- **Interact** - E key (not yet implemented)
- **Inventory** - I key (not yet implemented)
- **Map** - M key (not yet implemented)
- **Custom Keybindings** - Rebindable controls (planned)

## Accessibility

### Current Limitations
- No controller support yet
- No customizable keybindings
- No accessibility options (colorblind modes, etc.)

### Planned Improvements
- Controller/gamepad support
- Customizable key bindings
- Accessibility options
- Alternative control schemes

## Troubleshooting Controls

### Mouse Not Working
- Ensure the game window has focus
- Check system mouse settings
- Try clicking in the game window

### Keys Not Responding
- Verify keyboard layout (QWERTY assumed)
- Check for conflicting system shortcuts
- Ensure game window has focus
- Try restarting the game

### Camera Feels Wrong
- Adjust system mouse sensitivity
- Try using Q/E keyboard controls instead
- Experiment with different viewing angles
- Check if mouse acceleration is affecting control

## Quick Reference Card

```
┌─────────────────────────────────────┐
│         MOVEMENT                    │
│  W/A/S/D - Move (auto-rotate)       │
│  Mouse   - Look around              │
│  Q/E     - Rotate camera            │
├─────────────────────────────────────┤
│         SYSTEM                      │
│  ESC - Pause/Resume                 │
│  C   - Toggle collision boxes       │
├─────────────────────────────────────┤
│         MENU                        │
│  Mouse/Enter - Select               │
│  Arrows/Tab  - Navigate             │
└─────────────────────────────────────┘
```

---

**Next Steps:**
- [Quick Start Guide](quick-start.md) - Learn to play
- [Gameplay Guide](../user-guide/gameplay.md) - Master the mechanics
- [Troubleshooting](../user-guide/troubleshooting.md) - Solve issues