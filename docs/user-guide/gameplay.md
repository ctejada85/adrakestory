# Gameplay Guide

Learn about the features, mechanics, and gameplay elements of A Drake's Story.

## Overview

A Drake's Story is a 3D voxel-based exploration game featuring physics-driven movement, varied terrain types, and an isometric camera perspective. Navigate through procedurally structured worlds while mastering the physics system.

## Core Gameplay

### Exploration
- **3D Voxel World**: Navigate through a world built from voxels (3D pixels)
- **Sub-Voxel Detail**: Each voxel contains 8×8×8 sub-voxels for high-resolution detail
- **Isometric View**: Dynamic camera provides excellent spatial awareness
- **Physics-Based**: Realistic gravity and collision detection

### Movement Mechanics

#### Walking
- Use **WASD** keys for directional movement
- Movement is relative to camera direction
- Constant speed (no sprint yet)
- Physics applies gravity and collision

#### Grounding
- Player automatically detects ground beneath
- Smooth transitions between surfaces
- Gravity pulls you down when airborne
- Collision prevents falling through solid objects

#### Navigation
- Walk on various terrain types
- Navigate around obstacles
- Climb staircases
- Jump between platforms (when jumping is implemented)

## Terrain Types

### Full Blocks
- **Description**: Solid 8×8×8 voxel cubes
- **Characteristics**: 
  - Most stable surface type
  - Easy to walk on
  - Provides complete collision
- **Use Cases**: Floors, walls, solid structures

### Platforms
- **Description**: Thin 8×8×1 horizontal surfaces
- **Characteristics**:
  - Requires precision to stay on
  - Can walk underneath
  - Good for testing jumping (future)
- **Use Cases**: Bridges, elevated walkways, challenges

### Staircases
- **Description**: Progressive height increase (8 steps)
- **Characteristics**:
  - Smooth climbing experience
  - Each step is one sub-voxel high
  - Natural progression upward
- **Use Cases**: Vertical navigation, ramps, slopes

### Pillars
- **Description**: Small 2×2×2 centered columns
- **Characteristics**:
  - Challenging to navigate
  - Requires precise movement
  - Tests player skill
- **Use Cases**: Obstacles, parkour challenges, decorative

## Camera System

### Isometric Perspective
- **Fixed Distance**: Camera maintains constant distance from player
- **Angled View**: Provides excellent depth perception
- **Smooth Following**: Camera smoothly tracks player movement

### Camera Controls
- **Mouse**: Free-look rotation around player
- **Q/E Keys**: Snap rotation for quick angle changes
- **Automatic**: Camera follows player automatically

### Camera Tips
1. Adjust angle before navigating tight spaces
2. Use Q/E for quick 90-degree rotations
3. Mouse provides fine-tuned control
4. Find comfortable viewing angles for different situations

## Physics System

### Gravity
- Constant downward force
- Pulls player toward ground
- Affects all movement
- Creates realistic falling

### Collision Detection
- **Spatial Grid**: Efficient collision checking
- **Sub-Voxel Precision**: Accurate collision boundaries
- **Real-Time**: Instant collision response
- **Debug Mode**: Press C to visualize collision boxes

### Collision Behavior
- Prevents walking through walls
- Stops at edges (no automatic falling)
- Smooth sliding along surfaces
- Realistic physics response

## Game States

### Intro Animation
- Opening splash screen
- Fade-in effects
- Skip with any key
- Transitions to title screen

### Title Screen
- Main menu interface
- "New Game" option
- Settings (coming soon)
- Exit option

### Loading Map
- Progress bar display
- Real-time loading updates
- Map validation
- World spawning

### In-Game
- Active gameplay
- Full controls enabled
- Physics simulation active
- Camera control available

### Paused
- Pause menu overlay
- Resume option
- Return to title
- Settings (coming soon)

## Debug Features

### Collision Visualization
- **Toggle**: Press **C** key
- **Display**: Green wireframe boxes
- **Purpose**: See collision boundaries
- **Use Cases**:
  - Understanding physics
  - Testing custom maps
  - Learning terrain navigation
  - Debugging issues

## Tips & Strategies

### Movement Tips
1. **Take Your Time**: Physics is realistic, rushing causes mistakes
2. **Camera First**: Adjust view before moving in tight spaces
3. **Edge Awareness**: Be careful near platform edges
4. **Strafing**: Use A/D for precise positioning

### Navigation Tips
1. **Plan Routes**: Look ahead before moving
2. **Use Staircases**: Easier than trying to jump
3. **Test Surfaces**: Try walking on new terrain types
4. **Save Progress**: (Coming soon) Save before risky moves

### Camera Tips
1. **Multiple Angles**: Try different views for different situations
2. **Overhead View**: Good for planning routes
3. **Side View**: Better for judging heights
4. **Experiment**: Find what works for you

## Planned Features

Future gameplay additions:

### Movement
- [ ] Jumping mechanic
- [ ] Sprinting
- [ ] Climbing
- [ ] Swimming

### Interactions
- [ ] Item pickup
- [ ] Inventory system
- [ ] Object interaction
- [ ] NPC dialogue

### World
- [ ] Multiple biomes
- [ ] Procedural generation
- [ ] Dynamic weather
- [ ] Day/night cycle

### Progression
- [ ] Save/load system
- [ ] Achievements
- [ ] Unlockables
- [ ] Story progression

## Performance Tips

### For Better Performance
1. **Use Release Mode**: `cargo run --release`
2. **Close Other Apps**: Free up system resources
3. **Update Drivers**: Keep graphics drivers current
4. **Disable Debug**: Turn off collision visualization (C key)

### If Experiencing Lag
1. Check system requirements
2. Close background applications
3. Reduce map complexity (for custom maps)
4. Use release build instead of debug

## Common Questions

### Q: How do I save my progress?
**A:** Save/load functionality is planned but not yet implemented.

### Q: Can I jump?
**A:** Jumping is planned for a future update.

### Q: How do I interact with objects?
**A:** Interaction system is planned but not yet implemented.

### Q: Are there enemies?
**A:** Enemy system is planned but not yet implemented.

### Q: Can I play multiplayer?
**A:** Multiplayer support is planned for future development.

## Next Steps

- **Create Maps**: Learn [map creation](maps/creating-maps.md)
- **Explore Examples**: Check [example maps](maps/examples.md)
- **Troubleshoot**: See [troubleshooting guide](troubleshooting.md)
- **Contribute**: Read [developer guide](../developer-guide/architecture.md)

---

**Enjoy exploring!** Share your experiences and feedback on GitHub.