# Quick Start Guide

Get started with A Drake's Story in just a few minutes!

## Your First Game

### 1. Launch the Game

```bash
cargo run --release
```

### 2. Navigate the Intro

- Watch the intro animation (or press any key to skip)
- You'll arrive at the **Title Screen**

### 3. Start a New Game

- Click **"New Game"** or press **Enter**
- The game will load the default map
- Watch the loading progress bar

### 4. Explore the World

You'll spawn in a 3D voxel world with:
- **Floor tiles** - Solid ground to walk on
- **Platforms** - Thin platforms for jumping
- **Staircases** - Progressive steps to climb
- **Pillars** - Small columns for navigation

## Basic Controls

### Movement
- **W** - Move forward
- **A** - Move left  
- **S** - Move backward
- **D** - Move right

### Camera
- **Mouse** - Look around
- **Q/E** - Rotate camera around player

### System
- **ESC** - Pause game
- **C** - Toggle collision boxes (debug)

For complete controls, see the [Controls Reference](controls.md).

## What to Try First

### 1. Walk Around
Use **WASD** to explore the environment. The physics system will keep you grounded.

### 2. Test the Camera
Move your **mouse** to look around. The isometric view gives you a great perspective of the world.

### 3. Navigate Terrain
Try walking on:
- **Full blocks** - Solid, stable ground
- **Platforms** - Thin surfaces that require precision
- **Staircases** - Progressive height changes
- **Pillars** - Small centered columns

### 4. Experiment with Physics
- Walk off edges to test gravity
- Try jumping between platforms
- Observe collision detection

### 5. Pause and Resume
Press **ESC** to open the pause menu. You can:
- Resume the game
- Return to the title screen
- Adjust settings (coming soon)

## Understanding the Interface

### Loading Screen
Shows real-time progress:
- Loading file (0-20%)
- Parsing data (20-40%)
- Validating map (40-60%)
- Spawning voxels (60-90%)
- Spawning entities (90-95%)
- Finalizing (95-100%)

### Debug Mode
Press **C** to toggle collision box visualization:
- **Green boxes** - Show collision boundaries
- Useful for understanding physics
- Can be toggled on/off anytime

## Common First-Time Questions

### Q: Why is the first build so slow?
**A:** Cargo is downloading and compiling all dependencies. Subsequent builds will be much faster.

### Q: The game runs slowly in debug mode
**A:** Use `cargo run --release` for better performance. Debug builds prioritize compilation speed over runtime performance.

### Q: How do I create my own maps?
**A:** See the [Creating Maps Guide](../user-guide/maps/creating-maps.md) for step-by-step instructions.

### Q: Can I change the controls?
**A:** Control customization is planned for a future update. Current controls are fixed.

### Q: Where are the save files?
**A:** Save/load functionality is planned but not yet implemented.

## Next Steps

Now that you're familiar with the basics:

1. **Master the Controls** - Read the [Controls Reference](controls.md)
2. **Learn Gameplay Mechanics** - Check the [Gameplay Guide](../user-guide/gameplay.md)
3. **Create Custom Maps** - Try [Creating Maps](../user-guide/maps/creating-maps.md)
4. **Explore the Code** - See [Architecture Overview](../developer-guide/architecture.md)

## Troubleshooting

Having issues? Check the [Troubleshooting Guide](../user-guide/troubleshooting.md) for solutions to common problems.

---

**Enjoy the game!** If you have feedback or find bugs, please open an issue on GitHub.