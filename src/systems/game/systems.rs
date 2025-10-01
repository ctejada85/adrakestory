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

    // Render all non-air voxels as sub-voxels
    for x in 0..voxel_world.width {
        for y in 0..voxel_world.height {
            for z in 0..voxel_world.depth {
                if let Some(voxel_type) = voxel_world.get_voxel(x, y, z) {
                    if voxel_type != VoxelType::Air {
                        // Spawn parent voxel marker (for reference, no mesh)
                        let parent_entity = commands.spawn(Voxel { x, y, z, voxel_type }).id();

                        // Check if this is a corner pillar voxel at y=1
                        let is_corner_pillar = y == 1 &&
                            ((x == 0 && z == 0) || (x == 0 && z == 3) || (x == 3 && z == 0) || (x == 3 && z == 3));

                        // Check if this is a 1-sub-voxel-height platform
                        let is_step_platform = y == 1 &&
                            ((x == 1 && z == 1) || (x == 2 && z == 2));

                        if is_step_platform {
                            // For step platforms: render only 1 sub-voxel height (8x1x8)
                            for sub_x in 0..SUB_VOXEL_COUNT {
                                for sub_y in 0..1 {  // Only bottom layer
                                    for sub_z in 0..SUB_VOXEL_COUNT {
                                        let color = Color::srgb(
                                            0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                            0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                            0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                        );
                                        let sub_voxel_material = materials.add(color);

                                        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
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
                        } else if is_corner_pillar {
                            // For corner pillars: render only 2x2x2 sub-voxels (using standard 1/8 size)
                            // This creates a small pillar in the center, 2/8 = 1/4 the width of a full voxel
                            let pillar_count = 2;
                            let pillar_start = 3; // Start at the 3rd sub-voxel position (centered)

                            for sub_x in pillar_start..(pillar_start + pillar_count) {
                                for sub_y in pillar_start..(pillar_start + pillar_count) {
                                    for sub_z in pillar_start..(pillar_start + pillar_count) {
                                        let color = Color::srgb(
                                            0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                            0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                            0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                        );
                                        let sub_voxel_material = materials.add(color);

                                        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
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
                        } else {
                            // For normal voxels: render full 8x8x8 sub-voxels
                            for sub_x in 0..SUB_VOXEL_COUNT {
                                for sub_y in 0..SUB_VOXEL_COUNT {
                                    for sub_z in 0..SUB_VOXEL_COUNT {
                                        let color = Color::srgb(
                                            0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                            0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                            0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                        );
                                        let sub_voxel_material = materials.add(color);

                                        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
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
    sub_voxel_query: Query<(&SubVoxel, &Transform), Without<Player>>,
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
            let max_step_height = SUB_VOXEL_SIZE; // Can step up 1 sub-voxel height

            // Try moving on X axis
            let new_x = current_pos.x + move_delta.x;
            if check_sub_voxel_collision(&sub_voxel_query, new_x, current_pos.y, current_pos.z, player.radius) {
                // Check if we can step up
                if let Some(step_height) = get_step_up_height(&sub_voxel_query, new_x, current_pos.y, current_pos.z, player.radius, max_step_height) {
                    transform.translation.x = new_x;
                    transform.translation.y = step_height;
                }
            } else {
                transform.translation.x = new_x;
            }

            // Try moving on Z axis
            let new_z = current_pos.z + move_delta.z;
            if check_sub_voxel_collision(&sub_voxel_query, transform.translation.x, transform.translation.y, new_z, player.radius) {
                // Check if we can step up
                if let Some(step_height) = get_step_up_height(&sub_voxel_query, transform.translation.x, transform.translation.y, new_z, player.radius, max_step_height) {
                    transform.translation.z = new_z;
                    transform.translation.y = step_height;
                }
            } else {
                transform.translation.z = new_z;
            }
        }
    }
}

fn get_step_up_height(
    sub_voxel_query: &Query<(&SubVoxel, &Transform), Without<Player>>,
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    max_step_height: f32,
) -> Option<f32> {
    let collision_radius = radius * 0.85;
    let player_bottom = y - radius;

    // Find all sub-voxels that are blocking the player horizontally
    let mut highest_blocking_top = None;

    for (sub_voxel, sub_transform) in sub_voxel_query.iter() {
        let sub_pos = sub_transform.translation;
        let half_size = SUB_VOXEL_SIZE / 2.0;

        let min_x = sub_pos.x - half_size;
        let max_x = sub_pos.x + half_size;
        let min_y = sub_pos.y - half_size;
        let max_y = sub_pos.y + half_size;
        let min_z = sub_pos.z - half_size;
        let max_z = sub_pos.z + half_size;

        // Check horizontal overlap
        if x + collision_radius < min_x || x - collision_radius > max_x || z + collision_radius < min_z || z - collision_radius > max_z {
            continue;
        }

        // Check if this sub-voxel is at a steppable height
        // It should be above the ground but within max_step_height
        let height_difference = max_y - player_bottom;
        if height_difference > 0.0 && height_difference <= max_step_height {
            // This is a valid step - check if we can stand on top of it
            let closest_x = x.clamp(min_x, max_x);
            let closest_z = z.clamp(min_z, max_z);
            let dx = x - closest_x;
            let dz = z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                // This sub-voxel is blocking us, record its top
                highest_blocking_top = Some(highest_blocking_top.unwrap_or(max_y).max(max_y));
            }
        }
    }

    // If we found a step, return the new height (top of sub-voxel + player radius)
    highest_blocking_top.map(|top| top + radius)
}

fn check_sub_voxel_collision(
    sub_voxel_query: &Query<(&SubVoxel, &Transform), Without<Player>>,
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
) -> bool {
    // Use slightly smaller collision radius for tighter fit
    let collision_radius = radius * 0.85;

    // Check all sub-voxels for collision
    for (sub_voxel, sub_transform) in sub_voxel_query.iter() {
        let sub_pos = sub_transform.translation;
        let half_size = SUB_VOXEL_SIZE / 2.0;

        // Sub-voxel AABB bounds
        let min_x = sub_pos.x - half_size;
        let max_x = sub_pos.x + half_size;
        let min_y = sub_pos.y - half_size;
        let max_y = sub_pos.y + half_size;
        let min_z = sub_pos.z - half_size;
        let max_z = sub_pos.z + half_size;

        // Only check sub-voxels that overlap with player's height, but not the floor
        // Skip sub-voxels that are below player's center (these are floor/ground)
        if max_y <= y - radius * 0.5 {
            continue;
        }

        // Skip if sub-voxel is too far above
        if min_y > y + radius {
            continue;
        }

        // Quick AABB check for horizontal bounds
        if x + collision_radius < min_x || x - collision_radius > max_x || z + collision_radius < min_z || z - collision_radius > max_z {
            continue;
        }

        // Find closest point on sub-voxel AABB to player center (horizontal only)
        let closest_x = x.clamp(min_x, max_x);
        let closest_z = z.clamp(min_z, max_z);

        // Check horizontal distance only
        let dx = x - closest_x;
        let dz = z - closest_z;
        let distance_squared = dx * dx + dz * dz;

        if distance_squared < collision_radius * collision_radius {
            return true; // Collision detected
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
    sub_voxel_query: Query<(&SubVoxel, &Transform), Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Apply velocity
        let new_y = transform.translation.y + player.velocity.y * time.delta_secs();
        let player_bottom = new_y - player.radius;
        let current_bottom = transform.translation.y - player.radius;

        let mut hit_ground = false;
        let mut highest_collision_y = f32::MIN;

        // Check collision with sub-voxels below
        for (sub_voxel, sub_transform) in sub_voxel_query.iter() {
            let sub_pos = sub_transform.translation;
            let half_size = SUB_VOXEL_SIZE / 2.0;

            // Sub-voxel AABB bounds
            let sub_min_x = sub_pos.x - half_size;
            let sub_max_x = sub_pos.x + half_size;
            let sub_min_z = sub_pos.z - half_size;
            let sub_max_z = sub_pos.z + half_size;
            let sub_min_y = sub_pos.y - half_size;
            let sub_max_y = sub_pos.y + half_size;

            // Only check sub-voxels that are below the player
            if sub_max_y > new_y {
                continue;
            }

            // Check if player sphere is above this sub-voxel horizontally
            let player_x = transform.translation.x;
            let player_z = transform.translation.z;

            // Check horizontal overlap
            let horizontal_overlap =
                player_x + player.radius > sub_min_x &&
                player_x - player.radius < sub_max_x &&
                player_z + player.radius > sub_min_z &&
                player_z - player.radius < sub_max_z;

            if horizontal_overlap && player.velocity.y <= 0.0 {
                // Check if player's bottom would go through the top of this sub-voxel
                // Player was above, and would now be at or below the top
                if current_bottom >= sub_max_y && player_bottom <= sub_max_y {
                    highest_collision_y = highest_collision_y.max(sub_max_y);
                    hit_ground = true;
                }
            }
        }

        if hit_ground {
            transform.translation.y = highest_collision_y + player.radius;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        } else {
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