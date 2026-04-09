//! Entity spawning functions for players, NPCs, and light sources.

use super::super::super::character::CharacterModel;
use super::super::super::components::{
    CollisionBox, FlickerLight, LightSource, Npc, Player, PlayerFlashlight,
};
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use std::collections::HashMap;

/// Context for spawning entities.
pub struct EntitySpawnContext<'w, 's, 'a> {
    pub commands: Commands<'w, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub asset_server: &'a AssetServer,
}

/// Spawn the player entity with a 3D character model.
///
/// This function creates:
/// 1. A player entity with physics components (no visible mesh)
/// 2. A GLB character model as a child entity for visuals
/// 3. An invisible collision box for debugging
///
/// The physics collision uses a cylinder collider (radius: 0.2, half_height: 0.4)
/// which is kept separate from the visual model for flexibility and performance.
pub fn spawn_player(ctx: &mut EntitySpawnContext, position: Vec3) {
    let player_radius = 0.2;
    let player_half_height = 0.4; // Total height = 0.8 units

    // Load the character model (GLB file) with explicit scene specification
    // Using GltfAssetLabel::Scene(0) to load the first (default) scene from the GLB file
    let character_scene: Handle<Scene> = ctx
        .asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("characters/base_basic_pbr.glb"));

    info!("Loading character model: characters/base_basic_pbr.glb#Scene0");

    // Spawn the main player entity (parent) with physics components
    // No visible mesh - the GLB model will be the visual representation
    let player_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            Player {
                speed: 3.0,
                velocity: Vec3::ZERO,
                is_grounded: true,
                radius: player_radius,
                half_height: player_half_height,
                target_rotation: 0.0,
                current_rotation: 0.0,
                start_rotation: 0.0,
                rotation_elapsed: 0.0,
                rotation_duration: 0.2, // Fixed 0.2 second duration for all rotations
            },
            CharacterModel::new(character_scene.clone()),
        ))
        .id();

    // Spawn the character model as a child entity
    // Scale down to 0.3 and offset down by 0.3 units to align with collision sphere
    ctx.commands
        .spawn((
            SceneRoot(character_scene),
            Transform::from_translation(Vec3::new(0.0, -0.3, 0.0)).with_scale(Vec3::splat(0.5)),
        ))
        .insert(ChildOf(player_entity));

    // Spawn a spotlight as a child entity to act as a flashlight
    // Points forward in the direction the character is facing
    // Toggle with F key (keyboard) or Y button (gamepad)
    // Position: right at the edge of the collision cylinder (radius = 0.2)
    // Starts hidden (off) by default
    ctx.commands
        .spawn((
            SpotLight {
                color: Color::srgb(1.0, 0.98, 0.9), // Warm white flashlight beam
                intensity: 200000.0,                // Very bright focused beam
                range: 20.0,                        // Good range for exploration
                radius: 0.0,
                shadows_enabled: false, // Disabled for performance
                inner_angle: 0.12,      // ~6.9 degrees - focused center (50% wider)
                outer_angle: 0.375,     // ~21.5 degrees - wider cone (50% wider)
                ..default()
            },
            // Position at chest height (y=0.2), right at collision cylinder edge (z=0.2)
            Transform::from_translation(Vec3::new(0.0, 0.2, 0.2))
                .looking_at(Vec3::new(0.0, 0.1, 10.0), Vec3::Y),
            Visibility::Hidden, // Off by default
            PlayerFlashlight,
        ))
        .insert(ChildOf(player_entity));

    info!(
        "Spawned player with character model and flashlight at position: {:?}",
        position
    );

    // Create collision box as a cylinder (invisible by default, for debugging)
    // The cylinder mesh uses radius and half_height to match the actual collision shape
    let collision_box_mesh = ctx
        .meshes
        .add(Cylinder::new(player_radius, player_half_height * 2.0));
    let collision_box_material = ctx.materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    ctx.commands.spawn((
        Mesh3d(collision_box_mesh),
        MeshMaterial3d(collision_box_material),
        Transform::from_translation(position),
        Visibility::Hidden,
        CollisionBox,
    ));
}

/// Spawn an NPC entity with a 3D character model.
///
/// This function creates:
/// 1. An NPC entity with collision data (no visible mesh)
/// 2. A GLB character model as a child entity for visuals
///
/// NPCs are static (non-moving) and block player movement.
/// Properties can customize the NPC's name, model, and collision radius.
pub fn spawn_npc(
    ctx: &mut EntitySpawnContext,
    position: Vec3,
    properties: &HashMap<String, String>,
) {
    // Parse NPC properties with defaults
    let npc_radius = properties
        .get("radius")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(0.3);

    let npc_name = properties
        .get("name")
        .cloned()
        .unwrap_or_else(|| "NPC".to_string());

    // Load the NPC model (GLB file) - using the same default model as player for now
    // TODO: Support custom models via properties when Bevy supports dynamic asset paths
    let npc_scene: Handle<Scene> = ctx
        .asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("characters/base_basic_pbr.glb"));

    info!("Loading NPC model: characters/base_basic_pbr.glb#Scene0");

    // Spawn the NPC entity (parent) with collision component
    let npc_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            Npc {
                name: npc_name.clone(),
                radius: npc_radius,
            },
        ))
        .id();

    // Spawn the character model as a child entity
    // Scale and offset to align with collision sphere
    ctx.commands
        .spawn((
            SceneRoot(npc_scene),
            Transform::from_translation(Vec3::new(0.0, -0.3, 0.0)).with_scale(Vec3::splat(0.5)),
        ))
        .insert(ChildOf(npc_entity));

    info!(
        "Spawned NPC '{}' at position: {:?} with radius {}",
        npc_name, position, npc_radius
    );
}

/// Spawn a light source entity with a point light.
///
/// Light sources emit light uniformly in all directions (spherical).
/// Properties can customize color, intensity, range, and shadow casting.
pub fn spawn_light_source(
    ctx: &mut EntitySpawnContext,
    position: Vec3,
    properties: &HashMap<String, String>,
) {
    // Parse light properties with defaults
    let intensity = properties
        .get("intensity")
        .and_then(|i| i.parse::<f32>().ok())
        .unwrap_or(10000.0) // 10,000 lumens default - bright enough for interiors
        .clamp(0.0, 1000000.0);

    let range = properties
        .get("range")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(10.0)
        .clamp(0.1, 100.0);

    let shadows_enabled = properties
        .get("shadows")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    let flicker_enabled = parse_flicker_enabled(properties);
    let flicker_amplitude = parse_flicker_amplitude(properties);
    let flicker_speed = parse_flicker_speed(properties);

    // Parse color (format: "r,g,b" with values 0.0-1.0)
    let color = properties
        .get("color")
        .and_then(|c| {
            let parts: Vec<f32> = c.split(',').filter_map(|p| p.trim().parse().ok()).collect();
            if parts.len() == 3 {
                Some(Color::srgb(parts[0], parts[1], parts[2]))
            } else {
                None
            }
        })
        .unwrap_or(Color::WHITE);

    // Spawn light source entity
    let light_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            LightSource {
                color,
                intensity,
                range,
                shadows_enabled,
            },
            PointLight {
                color,
                intensity,
                range,
                radius: 0.0, // Point light (no physical size)
                shadows_enabled,
                ..default()
            },
        ))
        .id();

    if flicker_enabled {
        ctx.commands.entity(light_entity).insert(FlickerLight {
            base_intensity: intensity,
            amplitude: flicker_amplitude,
            speed: flicker_speed,
        });
    }

    info!(
        "Spawned light source at {:?} (intensity: {}, range: {}, shadows: {}, flicker: {})",
        position, intensity, range, shadows_enabled, flicker_enabled
    );
}

/// Parse light intensity from properties with defaults and clamping.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_light_intensity(properties: &HashMap<String, String>) -> f32 {
    properties
        .get("intensity")
        .and_then(|i| i.parse::<f32>().ok())
        .unwrap_or(10000.0)
        .clamp(0.0, 1000000.0)
}

/// Parse light range from properties with defaults and clamping.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_light_range(properties: &HashMap<String, String>) -> f32 {
    properties
        .get("range")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(10.0)
        .clamp(0.1, 100.0)
}

/// Parse shadows enabled from properties with default.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_shadows_enabled(properties: &HashMap<String, String>) -> bool {
    properties
        .get("shadows")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false)
}

/// Parse color from properties in "r,g,b" format.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_color(properties: &HashMap<String, String>) -> Option<Color> {
    properties.get("color").and_then(|c| {
        let parts: Vec<f32> = c.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() == 3 {
            Some(Color::srgb(parts[0], parts[1], parts[2]))
        } else {
            None
        }
    })
}

/// Parse NPC radius from properties with default.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_npc_radius(properties: &HashMap<String, String>) -> f32 {
    properties
        .get("radius")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(0.3)
}

/// Parse NPC name from properties with default.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_npc_name(properties: &HashMap<String, String>) -> String {
    properties
        .get("name")
        .cloned()
        .unwrap_or_else(|| "NPC".to_string())
}

/// Parse flicker enabled from properties with default (false).
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_flicker_enabled(properties: &HashMap<String, String>) -> bool {
    properties
        .get("flicker")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false)
}

/// Parse flicker amplitude from properties with defaults and clamping.
/// Default: 3_000.0, clamped to 0.0..=100_000.0.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_flicker_amplitude(properties: &HashMap<String, String>) -> f32 {
    properties
        .get("flicker_amplitude")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(3000.0)
        .clamp(0.0, 100_000.0)
}

/// Parse flicker speed from properties with defaults and clamping.
/// Default: 4.0, clamped to 0.1..=20.0.
/// Exposed for testing.
#[allow(dead_code)]
pub(crate) fn parse_flicker_speed(properties: &HashMap<String, String>) -> f32 {
    properties
        .get("flicker_speed")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(4.0)
        .clamp(0.1, 20.0)
}

#[cfg(test)]
mod tests;
