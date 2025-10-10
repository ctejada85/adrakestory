# Example Maps

Walkthrough of the included example maps and what you can learn from each.

## Overview

A Drake's Story includes several example maps that demonstrate different features and techniques. Study these to learn map creation best practices.

## Included Maps

### default.ron - Full-Featured Map

**Location:** `assets/maps/default.ron`

**Purpose:** Demonstrates all major features and serves as the default game map.

#### Features Demonstrated

**1. Floor Layer**
```ron
// Complete 10x10 grass floor
(pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
(pos: (1, 0, 0), voxel_type: Grass, pattern: Some(Full)),
// ... continues for full grid
```

**What to Learn:**
- Creating a solid foundation
- Using Full pattern for stable ground
- Grid-based layout planning

**2. Corner Pillars**
```ron
// Decorative pillars at corners
(pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
(pos: (9, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
(pos: (0, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
(pos: (9, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
```

**What to Learn:**
- Using Pillar pattern for decoration
- Symmetrical placement
- Adding visual interest

**3. Platform Series**
```ron
// Progressive jumping platforms
(pos: (2, 1, 2), voxel_type: Stone, pattern: Some(Platform)),
(pos: (4, 1, 2), voxel_type: Stone, pattern: Some(Platform)),
(pos: (6, 1, 2), voxel_type: Stone, pattern: Some(Platform)),
```

**What to Learn:**
- Creating platform challenges
- Using Platform pattern effectively
- Spacing for gameplay

**4. Staircase**
```ron
// Climbing structure
(pos: (7, 0, 7), voxel_type: Stone, pattern: Some(Staircase)),
```

**What to Learn:**
- Vertical navigation
- Staircase pattern usage
- Height transitions

**5. Lighting Setup**
```ron
lighting: (
    ambient_intensity: 0.3,
    directional_light: Some((
        direction: (-0.5, -1.0, -0.5),
        illuminance: 10000.0,
        color: (1.0, 1.0, 0.9),
    )),
)
```

**What to Learn:**
- Balanced ambient lighting
- Directional light positioning
- Warm color temperature

**6. Camera Positioning**
```ron
camera: (
    position: (8.0, 10.0, 8.0),
    look_at: (5.0, 0.0, 5.0),
    rotation_offset: -1.5707963,
)
```

**What to Learn:**
- Isometric camera setup
- Centering on map
- Optimal viewing angle

**7. Metadata**
```ron
metadata: (
    name: "Default Test Map",
    author: "Kibound",
    description: "A test map showcasing various voxel patterns",
    version: "1.0.0",
    created: "2025-01-10",
)
```

**What to Learn:**
- Proper metadata format
- Descriptive naming
- Version tracking

### simple_test.ron - Minimal Map

**Location:** `assets/maps/simple_test.ron`

**Purpose:** Demonstrates the minimum required elements for a valid map.

#### Features Demonstrated

**1. Minimal Floor**
```ron
world: (
    width: 3,
    height: 2,
    depth: 3,
    voxels: [
        // Simple 3x3 floor
        (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        (pos: (1, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        (pos: (2, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        // ... 9 tiles total
    ],
)
```

**What to Learn:**
- Minimum viable map size
- Essential voxel placement
- Simple grid layout

**2. Basic Player Spawn**
```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (1.5, 0.8, 1.5),
        properties: {},
    ),
]
```

**What to Learn:**
- Required player spawn
- Center positioning
- Height above ground

**3. Simple Lighting**
```ron
lighting: (
    ambient_intensity: 0.4,
    directional_light: Some((
        direction: (-1.0, -1.0, -1.0),
        illuminance: 8000.0,
        color: (1.0, 1.0, 1.0),
    )),
)
```

**What to Learn:**
- Basic lighting setup
- Standard white light
- Moderate intensity

**4. Custom Properties Example**
```ron
custom_properties: {
    "difficulty": "easy",
    "test_mode": "true",
}
```

**What to Learn:**
- Adding custom metadata
- Key-value format
- Game-specific data

## Learning Path

### Beginner: Start with simple_test.ron

1. **Study the Structure**
   - Open `assets/maps/simple_test.ron`
   - Understand each section
   - Note the minimal requirements

2. **Make Small Changes**
   - Add one more floor tile
   - Move the player spawn
   - Adjust lighting intensity

3. **Test Your Changes**
   - Load the map in-game
   - Verify it works
   - Iterate on changes

### Intermediate: Explore default.ron

1. **Analyze Patterns**
   - Study how patterns are used
   - Note the variety of voxel types
   - Understand the layout strategy

2. **Modify Existing Elements**
   - Change platform positions
   - Add more pillars
   - Adjust the staircase location

3. **Experiment with Lighting**
   - Try different light directions
   - Adjust color temperature
   - Change ambient intensity

### Advanced: Create Your Own

1. **Plan Your Design**
   - Sketch on paper
   - Define objectives
   - Plan navigation flow

2. **Build Incrementally**
   - Start with floor
   - Add vertical elements
   - Place entities
   - Configure lighting

3. **Polish and Optimize**
   - Test thoroughly
   - Adjust camera
   - Optimize performance
   - Add custom properties

## Common Techniques

### Creating a Test Arena

Based on default.ron approach:

```ron
// Large open floor
voxels: [
    // 10x10 grass floor
    // ... floor tiles
    
    // Corner markers
    (pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (9, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (0, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (9, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
    
    // Test platforms
    (pos: (2, 1, 2), voxel_type: Stone, pattern: Some(Platform)),
    (pos: (7, 1, 7), voxel_type: Stone, pattern: Some(Platform)),
]
```

### Creating a Parkour Course

Inspired by default.ron platforms:

```ron
voxels: [
    // Starting platform
    (pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Full)),
    
    // Jump sequence
    (pos: (2, 1, 0), voxel_type: Stone, pattern: Some(Platform)),
    (pos: (4, 2, 0), voxel_type: Stone, pattern: Some(Platform)),
    (pos: (6, 3, 0), voxel_type: Stone, pattern: Some(Platform)),
    
    // Staircase down
    (pos: (8, 3, 0), voxel_type: Stone, pattern: Some(Staircase)),
    
    // End platform
    (pos: (9, 4, 0), voxel_type: Stone, pattern: Some(Full)),
]
```

### Creating a Room

Using full blocks like default.ron:

```ron
voxels: [
    // Floor (5x5)
    (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    // ... more floor tiles
    
    // Walls (height 3)
    (pos: (0, 1, 0), voxel_type: Stone, pattern: Some(Full)),
    (pos: (0, 2, 0), voxel_type: Stone, pattern: Some(Full)),
    // ... more wall tiles
    
    // Pillars at corners
    (pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
    // ... corner pillars
]
```

## Comparison Table

| Feature | simple_test.ron | default.ron |
|---------|----------------|-------------|
| **Size** | 3×2×3 | 10×3×10 |
| **Voxels** | 9 | ~100+ |
| **Patterns** | Full only | All patterns |
| **Complexity** | Minimal | Full-featured |
| **Purpose** | Learning | Gameplay |
| **Load Time** | Instant | ~1 second |

## Tips from Examples

### From simple_test.ron

1. **Keep It Simple**: Start with minimal viable map
2. **Center Spawn**: Place player in map center
3. **Test Early**: Small maps load quickly for testing
4. **Use Defaults**: Standard lighting works well

### From default.ron

1. **Add Variety**: Mix different patterns and types
2. **Create Landmarks**: Corner pillars help orientation
3. **Plan Navigation**: Include multiple path types
4. **Balance Lighting**: Warm light looks better than pure white
5. **Think 3D**: Use vertical space effectively

## Exercises

### Exercise 1: Modify simple_test.ron

1. Double the floor size (6×6)
2. Add a pillar in each corner
3. Add a platform at height 1
4. Test and iterate

### Exercise 2: Extend default.ron

1. Add a second staircase
2. Create a platform bridge
3. Add more decorative pillars
4. Adjust lighting for different mood

### Exercise 3: Create Hybrid Map

1. Use simple_test.ron structure
2. Add default.ron features
3. Create your own layout
4. Add custom properties

## Next Steps

- **[Creating Maps Guide](creating-maps.md)** - Detailed creation tutorial
- **[Map Format Reference](map-format.md)** - Complete format spec
- **[Troubleshooting](../troubleshooting.md)** - Solve issues
- **[Developer Guide](../../developer-guide/systems/map-loader.md)** - System internals

---

**Learn by doing!** The best way to understand maps is to create and modify them.