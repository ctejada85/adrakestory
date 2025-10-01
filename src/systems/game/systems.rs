use bevy::prelude::*;
use super::components::{Player, Voxel, VoxelType, GameCamera};
use super::resources::VoxelWorld;

pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let voxel_world = VoxelWorld::default();

    // Create voxel mesh (1x1x1 cube)
    let voxel_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    // Render all non-air voxels
    for x in 0..voxel_world.width {
        for y in 0..voxel_world.height {
            for z in 0..voxel_world.depth {
                if let Some(voxel_type) = voxel_world.get_voxel(x, y, z) {
                    if voxel_type != VoxelType::Air {
                        let color = match voxel_type {
                            VoxelType::Grass => Color::srgb(0.2 + (x as f32 * 0.15), 0.5, 0.3),
                            VoxelType::Dirt => Color::srgb(0.6, 0.4, 0.2),
                            VoxelType::Stone => Color::srgb(0.5, 0.5, 0.5),
                            VoxelType::Air => continue,
                        };
                        let voxel_material = materials.add(color);

                        commands.spawn((
                            Mesh3d(voxel_mesh.clone()),
                            MeshMaterial3d(voxel_material),
                            Transform::from_xyz(x as f32, y as f32, z as f32),
                            Voxel { x, y, z, voxel_type },
                        ));
                    }
                }
            }
        }
    }

    commands.insert_resource(voxel_world);

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

            // Clamp player position to room bounds
            // Tiles span from -0.5 to 3.5, with player radius 0.3
            transform.translation.x = transform.translation.x.clamp(-0.2, 3.2);
            transform.translation.z = transform.translation.z.clamp(-0.2, 3.2);
        }
    }
}

pub fn cleanup_game(
    mut commands: Commands,
    voxel_query: Query<Entity, With<Voxel>>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<Entity, With<GameCamera>>,
    light_query: Query<Entity, With<DirectionalLight>>,
) {
    for entity in &voxel_query {
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
    commands.remove_resource::<VoxelWorld>();
}