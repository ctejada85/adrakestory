use bevy::prelude::*;
use super::components::{Player, FloorTile, GameCamera};
use super::resources::RoomSize;

pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let room_size = RoomSize::default();
    commands.insert_resource(room_size);

    // Create floor tiles (4x4 room) with different colors
    let tile_mesh = meshes.add(Cuboid::new(1.0, 0.1, 1.0));

    for x in 0..4 {
        for z in 0..4 {
            // Generate a different color for each tile based on position
            let color = Color::srgb(
                0.2 + (x as f32 * 0.15),
                0.3 + (z as f32 * 0.15),
                0.4 + ((x + z) as f32 * 0.1),
            );
            let tile_material = materials.add(color);

            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(tile_material),
                Transform::from_xyz(x as f32, 0.0, z as f32),
                FloorTile,
            ));
        }
    }

    // Create player (sphere)
    let player_mesh = meshes.add(Sphere::new(0.3));
    let player_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(1.5, 0.5, 1.5),
        Player { speed: 3.0 },
    ));

    // Add light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Add camera (isometric-style view, tilted 30 degrees and rotated)
    let mut camera_transform = Transform::from_xyz(1.5, 8.0, 5.5)
        .looking_at(Vec3::new(1.5, 0.0, 1.5), Vec3::Y);
    camera_transform.rotate_around(
        Vec3::new(1.5, 0.0, 1.5),
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
    );

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameCamera,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform)>,
) {
    if let Ok((player, mut transform)) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        // Adjusted for camera rotation: up moves in +X, right moves in -Z
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.z += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * player.speed * time.delta_secs();

            // Clamp player position to room bounds (0 to 3 in both x and z)
            transform.translation.x = transform.translation.x.clamp(0.3, 3.7);
            transform.translation.z = transform.translation.z.clamp(0.3, 3.7);
        }
    }
}

pub fn cleanup_game(
    mut commands: Commands,
    tile_query: Query<Entity, With<FloorTile>>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<Entity, With<GameCamera>>,
    light_query: Query<Entity, With<DirectionalLight>>,
) {
    for entity in &tile_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &player_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &light_query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<RoomSize>();
}