//! Editor setup and initialization.

use adrakestory::editor::ui::dialogs::MapDataChangedEvent;
use adrakestory::editor::{camera, grid, EditorState};
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use grid::InfiniteGridConfig;

/// Setup the editor on startup
pub fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_config: Res<InfiniteGridConfig>,
    editor_state: Res<EditorState>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    info!("Starting Map Editor");

    // Spawn 3D camera for viewport
    let camera_pos = Vec3::new(10.0, 10.0, 10.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(Vec3::new(2.0, 0.0, 2.0), Vec3::Y),
        camera::EditorCamera::new(),
    ));

    // Get lighting configuration from the current map
    let lighting = &editor_state.current_map.lighting;

    // Spawn directional light using map configuration with high-quality shadows
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
            "Spawned directional light with high-quality shadows: illuminance={}, color={:?}, direction={:?}",
            dir_light.illuminance, dir_light.color, dir_light.direction
        );
    }

    // Spawn ambient light using map configuration
    // Convert 0.0-1.0 intensity to brightness (scale by 1000 for Bevy's lighting system)
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: ambient_brightness,
    });

    info!(
        "Spawned ambient light with brightness: {}",
        ambient_brightness
    );

    // Spawn infinite grid (frustum not available at startup, will be updated on first frame)
    grid::spawn_infinite_grid(
        &mut commands,
        &mut meshes,
        &mut materials,
        &grid_config,
        camera_pos,
        None, // Frustum will be used on subsequent updates
    );

    // Spawn cursor indicator
    grid::spawn_cursor_indicator(&mut commands, &mut meshes, &mut materials);

    // Send event to trigger initial lighting setup
    map_changed_events.send(MapDataChangedEvent);

    info!("Map editor setup complete");
}
