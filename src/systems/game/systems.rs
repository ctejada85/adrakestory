use bevy::prelude::*;
use super::components::{Player, Voxel, SubVoxel, VoxelType, GameCamera};
use super::resources::VoxelWorld;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let voxel_world = VoxelWorld::default();

    // Create sub-voxel mesh (1/8 x 1/8 x 1/8 cube)
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Render all non-air voxels as 8x8x8 sub-voxels
    for x in 0..voxel_world.width {
        for y in 0..voxel_world.height {
            for z in 0..voxel_world.depth {
                if let Some(voxel_type) = voxel_world.get_voxel(x, y, z) {
                    if voxel_type != VoxelType::Air {
                        // Spawn parent voxel marker (for reference, no mesh)
                        let parent_entity = commands.spawn(Voxel { x, y, z, voxel_type }).id();

                        // Spawn 8x8x8 sub-voxels
                        for sub_x in 0..SUB_VOXEL_COUNT {
                            for sub_y in 0..SUB_VOXEL_COUNT {
                                for sub_z in 0..SUB_VOXEL_COUNT {
                                    // Generate unique color based on position
                                    let color = Color::srgb(
                                        0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                        0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                        0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                    );
                                    let sub_voxel_material = materials.add(color);

                                    // Calculate world position for this sub-voxel
                                    // Center of voxel is at (x, y, z)
                                    // Sub-voxels span from -0.5 to 0.5 relative to center
                                    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5; // Start offset
                                    let sub_x_pos = x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
                                    let sub_y_pos = y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
                                    let sub_z_pos = z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

                                    commands.spawn((
                                        Mesh3d(sub_voxel_mesh.clone()),
                                        MeshMaterial3d(sub_voxel_material),
                                        Transform::from_xyz(sub_x_pos, sub_y_pos, sub_z_pos),
                                        SubVoxel {
                                            parent_x: x,
                                            parent_y: y,
                                            parent_z: z,
                                            sub_x,
                                            sub_y,
                                            sub_z,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    commands.insert_resource(voxel_world);

    // Create player (sphere) - positioned on top of voxel floor
    let player_radius = 0.3;
    let player_mesh = meshes.add(Sphere::new(player_radius));
    let player_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(1.5, 0.5 + player_radius, 1.5),
        Player {
            speed: 3.0,
            velocity: Vec3::ZERO,
            is_grounded: true,
            radius: player_radius,
        },
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
    voxel_world: Res<VoxelWorld>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
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

        // Jump
        if keyboard_input.just_pressed(KeyCode::Space) && player.is_grounded {
            player.velocity.y = 8.0;
            player.is_grounded = false;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();

            let current_pos = transform.translation;
            let move_delta = direction * player.speed * time.delta_secs();

            // Try moving on X axis
            let new_x = current_pos.x + move_delta.x;
            if !check_voxel_collision(&voxel_world, new_x, current_pos.y, current_pos.z, player.radius) {
                transform.translation.x = new_x;
            }

            // Try moving on Z axis
            let new_z = current_pos.z + move_delta.z;
            if !check_voxel_collision(&voxel_world, transform.translation.x, current_pos.y, new_z, player.radius) {
                transform.translation.z = new_z;
            }
        }
    }
}

fn check_voxel_collision(voxel_world: &VoxelWorld, x: f32, y: f32, z: f32, radius: f32) -> bool {
    // Check voxels around the player's horizontal position
    // Only check at the player's current level, not floor level (to allow walking over ground)
    let player_y = y.floor() as i32;

    // Get all voxels that might intersect with the player sphere in horizontal plane
    let min_x = (x - radius).floor() as i32;
    let max_x = (x + radius).ceil() as i32;
    let min_z = (z - radius).floor() as i32;
    let max_z = (z + radius).ceil() as i32;

    for voxel_x in min_x..=max_x {
        for voxel_z in min_z..=max_z {
            // Check voxel at player's height (not the floor they're standing on)
            if let Some(voxel_type) = voxel_world.get_voxel(voxel_x, player_y, voxel_z) {
                if voxel_type != VoxelType::Air {
                    // Voxel AABB bounds
                    let voxel_min_x = voxel_x as f32 - 0.5;
                    let voxel_max_x = voxel_x as f32 + 0.5;
                    let voxel_min_z = voxel_z as f32 - 0.5;
                    let voxel_max_z = voxel_z as f32 + 0.5;

                    // Find closest point on voxel's horizontal face to player center
                    let closest_x = x.clamp(voxel_min_x, voxel_max_x);
                    let closest_z = z.clamp(voxel_min_z, voxel_max_z);

                    // Check horizontal distance only
                    let dx = x - closest_x;
                    let dz = z - closest_z;
                    let distance_squared = dx * dx + dz * dz;

                    if distance_squared < radius * radius {
                        return true; // Collision detected
                    }
                }
            }
        }
    }

    false
}

pub fn apply_gravity(
    time: Res<Time>,
    mut player_query: Query<&mut Player>,
) {
    const GRAVITY: f32 = -32.0;

    if let Ok(mut player) = player_query.get_single_mut() {
        player.velocity.y += GRAVITY * time.delta_secs();
    }
}

pub fn apply_physics(
    time: Res<Time>,
    voxel_world: Res<VoxelWorld>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Apply velocity
        let new_y = transform.translation.y + player.velocity.y * time.delta_secs();
        let player_bottom = new_y - player.radius;

        // Check collision with voxels below - no clamping, allow falling off edges
        let player_x = transform.translation.x.round() as i32;
        let player_z = transform.translation.z.round() as i32;
        let check_y = player_bottom.floor() as i32;

        let mut hit_ground = false;

        // Check if there's a solid voxel at the position where player's bottom would be
        // Only check if player is within world bounds
        if player_x >= 0 && player_x < voxel_world.width && player_z >= 0 && player_z < voxel_world.depth {
            if let Some(voxel_type) = voxel_world.get_voxel(player_x, check_y, player_z) {
                if voxel_type != VoxelType::Air {
                    // Voxel occupies space from check_y to check_y+1
                    let voxel_top = (check_y + 1) as f32;

                    // If player is falling and would go below voxel top, snap to top
                    if player_bottom <= voxel_top && player.velocity.y <= 0.0 {
                        transform.translation.y = voxel_top + player.radius;
                        player.velocity.y = 0.0;
                        player.is_grounded = true;
                        hit_ground = true;
                    }
                }
            }
        }

        if !hit_ground {
            transform.translation.y = new_y;
            player.is_grounded = false;
        }
    }
}

pub fn cleanup_game(
    mut commands: Commands,
    voxel_query: Query<Entity, With<Voxel>>,
    sub_voxel_query: Query<Entity, With<SubVoxel>>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<Entity, With<GameCamera>>,
    light_query: Query<Entity, With<DirectionalLight>>,
) {
    for entity in &voxel_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &sub_voxel_query {
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