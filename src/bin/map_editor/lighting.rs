//! Lighting system for map editor.

use adrakestory::editor::ui::dialogs::MapDataChangedEvent;
use adrakestory::editor::EditorState;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

/// System to update lighting when the map changes (e.g., after loading a new map)
pub fn update_lighting_on_map_change(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut ambient_light: ResMut<AmbientLight>,
    directional_lights: Query<Entity, With<DirectionalLight>>,
    mut map_changed_events: EventReader<MapDataChangedEvent>,
) {
    // Only update if we received a map data changed event
    if map_changed_events.read().next().is_none() {
        return;
    }

    let lighting = &editor_state.current_map.lighting;

    // Update ambient light
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    ambient_light.brightness = ambient_brightness;

    info!(
        "Updated ambient light brightness to: {}",
        ambient_brightness
    );

    // Remove existing directional lights
    for entity in directional_lights.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new directional light with map configuration and high-quality shadows
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let cascade_shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            minimum_distance: 0.1,
            overlap_proportion: 0.3,
        }
        .build();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));

        info!(
            "Updated directional light with high-quality shadows: illuminance={}, color={:?}, direction={:?}",
            dir_light.illuminance, dir_light.color, dir_light.direction
        );
    }
}
