//! Test program to verify coordinate normalization works correctly

use std::fs;
use std::path::Path;

// Simplified structures for testing (matching the actual format)
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MapData {
    metadata: MapMetadata,
    world: WorldData,
    entities: Vec<EntityData>,
    lighting: LightingData,
    camera: CameraData,
    #[serde(default)]
    custom_properties: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MapMetadata {
    name: String,
    author: String,
    description: String,
    version: String,
    created: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WorldData {
    width: i32,
    height: i32,
    depth: i32,
    voxels: Vec<VoxelData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VoxelData {
    pos: (i32, i32, i32),
    voxel_type: String,
    pattern: Option<String>,
    rotation_state: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EntityData {
    entity_type: String,
    position: (f32, f32, f32),
    #[serde(default)]
    properties: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LightingData {
    ambient_intensity: f32,
    directional_light: Option<DirectionalLightData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DirectionalLightData {
    direction: (f32, f32, f32),
    illuminance: f32,
    color: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CameraData {
    position: (f32, f32, f32),
    look_at: (f32, f32, f32),
    rotation_offset: f32,
}

fn calculate_map_bounds(map: &MapData) -> (i32, i32, i32, i32, i32, i32) {
    if map.world.voxels.is_empty() {
        return (0, 0, 0, 0, 0, 0);
    }

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    for voxel in &map.world.voxels {
        let (x, y, z) = voxel.pos;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

fn normalize_map_coordinates(map: &mut MapData) -> bool {
    if map.world.voxels.is_empty() {
        map.world.width = map.world.width.max(1);
        map.world.height = map.world.height.max(1);
        map.world.depth = map.world.depth.max(1);
        return false;
    }

    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_map_bounds(map);

    let offset_x = -min_x.min(0);
    let offset_y = -min_y.min(0);
    let offset_z = -min_z.min(0);

    let needs_normalization = offset_x != 0 || offset_y != 0 || offset_z != 0;

    if needs_normalization {
        println!(
            "Normalizing: bounds=({}, {}, {}, {}, {}, {}), offset=({}, {}, {})",
            min_x, max_x, min_y, max_y, min_z, max_z, offset_x, offset_y, offset_z
        );

        for voxel in &mut map.world.voxels {
            voxel.pos.0 += offset_x;
            voxel.pos.1 += offset_y;
            voxel.pos.2 += offset_z;
        }

        for entity in &mut map.entities {
            entity.position.0 += offset_x as f32;
            entity.position.1 += offset_y as f32;
            entity.position.2 += offset_z as f32;
        }

        map.camera.position.0 += offset_x as f32;
        map.camera.position.1 += offset_y as f32;
        map.camera.position.2 += offset_z as f32;

        map.camera.look_at.0 += offset_x as f32;
        map.camera.look_at.1 += offset_y as f32;
        map.camera.look_at.2 += offset_z as f32;
    }

    let required_width = (max_x - min_x + 1).max(1);
    let required_height = (max_y - min_y + 1).max(1);
    let required_depth = (max_z - min_z + 1).max(1);

    map.world.width = required_width;
    map.world.height = required_height;
    map.world.depth = required_depth;

    if needs_normalization {
        println!(
            "Map normalized: dimensions=({}, {}, {})",
            map.world.width, map.world.height, map.world.depth
        );
    }

    needs_normalization
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing coordinate normalization...\n");

    // Load the problematic map
    let content = fs::read_to_string("assets/maps/default.ron")?;
    let mut map: MapData = ron::from_str(&content)?;

    println!("Original map:");
    println!(
        "  Dimensions: {}x{}x{}",
        map.world.width, map.world.height, map.world.depth
    );
    println!("  Voxel count: {}", map.world.voxels.len());

    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_map_bounds(&map);
    println!(
        "  Bounds: X[{}, {}], Y[{}, {}], Z[{}, {}]",
        min_x, max_x, min_y, max_y, min_z, max_z
    );
    println!(
        "  First entity: {:?}",
        map.entities.first().map(|e| e.position)
    );
    println!("  Camera: {:?}", map.camera.position);
    println!();

    // Normalize
    let was_normalized = normalize_map_coordinates(&mut map);
    println!("\nNormalized: {}", was_normalized);

    println!("\nAfter normalization:");
    println!(
        "  Dimensions: {}x{}x{}",
        map.world.width, map.world.height, map.world.depth
    );

    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_map_bounds(&map);
    println!(
        "  Bounds: X[{}, {}], Y[{}, {}], Z[{}, {}]",
        min_x, max_x, min_y, max_y, min_z, max_z
    );
    println!(
        "  First entity: {:?}",
        map.entities.first().map(|e| e.position)
    );
    println!("  Camera: {:?}", map.camera.position);

    // Verify all coordinates are non-negative
    let mut all_valid = true;
    for voxel in &map.world.voxels {
        let (x, y, z) = voxel.pos;
        if x < 0 || y < 0 || z < 0 {
            println!("ERROR: Found negative coordinate: ({}, {}, {})", x, y, z);
            all_valid = false;
        }
        if x >= map.world.width || y >= map.world.height || z >= map.world.depth {
            println!("ERROR: Coordinate out of bounds: ({}, {}, {})", x, y, z);
            all_valid = false;
        }
    }

    if all_valid {
        println!("\n✓ All voxel coordinates are valid!");
    } else {
        println!("\n✗ Some coordinates are invalid!");
    }

    // Save normalized map
    let ron_string = ron::ser::to_string_pretty(&map, ron::ser::PrettyConfig::default())?;
    fs::write("assets/maps/default_normalized.ron", ron_string)?;
    println!("\nNormalized map saved to: assets/maps/default_normalized.ron");

    Ok(())
}
