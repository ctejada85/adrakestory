use super::components::{CollisionBox, GameCamera, Player, SubVoxel, Voxel, VoxelType};
use super::resources::{GameInitialized, SpatialGrid, VoxelWorld};
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

// === SubVoxel position helpers for ECS-based collision ===

fn calculate_sub_voxel_world_pos(sub_voxel: &SubVoxel) -> Vec3 {
    const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0; // Keep in sync with main const
    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    Vec3::new(
        sub_voxel.parent_x as f32 + offset + (sub_voxel.sub_x as f32 * SUB_VOXEL_SIZE),
        sub_voxel.parent_y as f32 + offset + (sub_voxel.sub_y as f32 * SUB_VOXEL_SIZE),
        sub_voxel.parent_z as f32 + offset + (sub_voxel.sub_z as f32 * SUB_VOXEL_SIZE),
    )
}

fn get_sub_voxel_bounds(sub_voxel: &SubVoxel) -> (Vec3, Vec3) {
    let center = calculate_sub_voxel_world_pos(sub_voxel);
    let half_size = 1.0 / 8.0 / 2.0;
    (
        center - Vec3::splat(half_size),
        center + Vec3::splat(half_size),
    )
}
pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_initialized: Option<Res<GameInitialized>>,
) {
    // If game is already initialized, don't run setup again
    if let Some(initialized) = game_initialized {
        if initialized.0 {
            return;
        }
    }

    // Mark game as initialized
    commands.insert_resource(GameInitialized(true));

    let voxel_world = VoxelWorld::default();
    let mut spatial_grid = SpatialGrid::default();

    // Create sub-voxel mesh (1/8 x 1/8 x 1/8 cube)
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Render all non-air voxels as sub-voxels
    for x in 0..voxel_world.width {
        for y in 0..voxel_world.height {
            for z in 0..voxel_world.depth {
                if let Some(voxel_type) = voxel_world.get_voxel(x, y, z) {
                    if voxel_type != VoxelType::Air {
                        // Spawn parent voxel marker (for reference, no mesh)
                        commands.spawn(Voxel);

                        // Check if this is a corner pillar voxel at y=1
                        let is_corner_pillar =
                            y == 1 && matches!((x, z), (0, 0) | (0, 3) | (3, 0) | (3, 3));

                        // Check if this is a 1-sub-voxel-height platform
                        let is_step_platform = y == 1 && ((x == 1 && z == 1) || (x == 2 && z == 2));

                        // Check if this is a staircase voxel
                        let is_staircase = y == 1 && x == 2 && z == 1;

                        if is_staircase {
                            // For staircase: create 8 steps, each 1 sub-voxel tall
                            // Each step goes from bottom to a certain height
                            for step in 0..SUB_VOXEL_COUNT {
                                let step_height = step + 1; // Height of this step (1 to 8 sub-voxels)

                                for sub_x in step..(step + 1) {
                                    // Each step is 1 sub-voxel wide in X
                                    for sub_y in 0..step_height {
                                        // Height increases with each step
                                        for sub_z in 0..SUB_VOXEL_COUNT {
                                            // Full depth
                                            let color = Color::srgb(
                                                0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                                0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                                0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                            );
                                            let sub_voxel_material = materials.add(color);

                                            let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
                                            let sub_x_pos =
                                                x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
                                            let sub_y_pos =
                                                y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
                                            let sub_z_pos =
                                                z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

                                            let sub_voxel_entity = commands
                                                .spawn((
                                                    Mesh3d(sub_voxel_mesh.clone()),
                                                    MeshMaterial3d(sub_voxel_material),
                                                    Transform::from_xyz(
                                                        sub_x_pos, sub_y_pos, sub_z_pos,
                                                    ),
                                                    SubVoxel {
                                                        parent_x: x,
                                                        parent_y: y,
                                                        parent_z: z,
                                                        sub_x,
                                                        sub_y,
                                                        sub_z,
                                                    },
                                                ))
                                                .id();
                                            let sub_voxel_world_pos =
                                                Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
                                            let grid_coords = SpatialGrid::world_to_grid_coords(
                                                sub_voxel_world_pos,
                                            );
                                            spatial_grid
                                                .cells
                                                .entry(grid_coords)
                                                .or_default()
                                                .push(sub_voxel_entity);
                                        }
                                    }
                                }
                            }
                        } else if is_step_platform {
                            // For step platforms: render only 1 sub-voxel height (8x1x8)
                            for sub_x in 0..SUB_VOXEL_COUNT {
                                for sub_y in 0..1 {
                                    // Only bottom layer
                                    for sub_z in 0..SUB_VOXEL_COUNT {
                                        let color = Color::srgb(
                                            0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
                                            0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
                                            0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
                                        );
                                        let sub_voxel_material = materials.add(color);

                                        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
                                        let sub_x_pos =
                                            x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
                                        let sub_y_pos =
                                            y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
                                        let sub_z_pos =
                                            z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

                                        let sub_voxel_entity = commands
                                            .spawn((
                                                Mesh3d(sub_voxel_mesh.clone()),
                                                MeshMaterial3d(sub_voxel_material),
                                                Transform::from_xyz(
                                                    sub_x_pos, sub_y_pos, sub_z_pos,
                                                ),
                                                SubVoxel {
                                                    parent_x: x,
                                                    parent_y: y,
                                                    parent_z: z,
                                                    sub_x,
                                                    sub_y,
                                                    sub_z,
                                                },
                                            ))
                                            .id();
                                        let sub_voxel_world_pos =
                                            Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
                                        let grid_coords =
                                            SpatialGrid::world_to_grid_coords(sub_voxel_world_pos);
                                        spatial_grid
                                            .cells
                                            .entry(grid_coords)
                                            .or_default()
                                            .push(sub_voxel_entity);
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
                                        let sub_x_pos =
                                            x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
                                        let sub_y_pos =
                                            y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
                                        let sub_z_pos =
                                            z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

                                        let sub_voxel_entity = commands
                                            .spawn((
                                                Mesh3d(sub_voxel_mesh.clone()),
                                                MeshMaterial3d(sub_voxel_material),
                                                Transform::from_xyz(
                                                    sub_x_pos, sub_y_pos, sub_z_pos,
                                                ),
                                                SubVoxel {
                                                    parent_x: x,
                                                    parent_y: y,
                                                    parent_z: z,
                                                    sub_x,
                                                    sub_y,
                                                    sub_z,
                                                },
                                            ))
                                            .id();
                                        let sub_voxel_world_pos =
                                            Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
                                        let grid_coords =
                                            SpatialGrid::world_to_grid_coords(sub_voxel_world_pos);
                                        spatial_grid
                                            .cells
                                            .entry(grid_coords)
                                            .or_default()
                                            .push(sub_voxel_entity);
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
                                        let sub_x_pos =
                                            x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
                                        let sub_y_pos =
                                            y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
                                        let sub_z_pos =
                                            z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

                                        let sub_voxel_entity = commands
                                            .spawn((
                                                Mesh3d(sub_voxel_mesh.clone()),
                                                MeshMaterial3d(sub_voxel_material),
                                                Transform::from_xyz(
                                                    sub_x_pos, sub_y_pos, sub_z_pos,
                                                ),
                                                SubVoxel {
                                                    parent_x: x,
                                                    parent_y: y,
                                                    parent_z: z,
                                                    sub_x,
                                                    sub_y,
                                                    sub_z,
                                                },
                                            ))
                                            .id();
                                        let sub_voxel_world_pos =
                                            Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
                                        let grid_coords =
                                            SpatialGrid::world_to_grid_coords(sub_voxel_world_pos);
                                        spatial_grid
                                            .cells
                                            .entry(grid_coords)
                                            .or_default()
                                            .push(sub_voxel_entity);
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
    commands.insert_resource(spatial_grid);

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

    // Create collision box (invisible by default)
    let collision_box_mesh = meshes.add(Cuboid::new(
        player_radius * 2.0,
        player_radius * 2.0,
        player_radius * 2.0,
    ));
    let collision_box_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(collision_box_mesh),
        MeshMaterial3d(collision_box_material),
        Transform::from_xyz(1.5, 0.5 + player_radius, 1.5),
        Visibility::Hidden,
        CollisionBox,
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
    let mut camera_transform =
        Transform::from_xyz(1.5, 8.0, 5.5).looking_at(Vec3::new(1.5, 0.0, 1.5), Vec3::Y);
    camera_transform.rotate_around(
        Vec3::new(1.5, 0.0, 1.5),
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
    );

    let original_rotation = camera_transform.rotation;

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameCamera {
            original_rotation,
            target_rotation: original_rotation,
            rotation_speed: 5.0,
        },
    ));
}
pub fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(crate::states::GameState::Paused);
    }
}

/*
    Player movement and collision now use SubVoxel component fields for all collision and step-up logic.
    This enables ECS-idiomatic queries and future voxel modification features.
*/
pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues when window regains focus
        let delta = time.delta_secs().min(0.1);

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
            let move_delta = direction * player.speed * delta;
            let max_step_height = SUB_VOXEL_SIZE; // Can step up 1 sub-voxel height

            // Try moving on X axis
            let new_x = current_pos.x + move_delta.x;
            if check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                new_x,
                current_pos.y,
                current_pos.z,
                player.radius,
            ) {
                // Check if we can step up
                let player_collision = PlayerCollision {
                    pos: Vec3::new(new_x, current_pos.y, current_pos.z),
                    radius: player.radius,
                    current_y: current_pos.y,
                };
                if let Some(step_height) = get_step_up_height(
                    &spatial_grid,
                    &sub_voxel_query,
                    &player_collision,
                    max_step_height,
                ) {
                    // Only step up if the height increase is reasonable (within one step)
                    let height_increase = step_height - current_pos.y;
                    if height_increase > 0.001 && height_increase <= max_step_height + 0.001 {
                        transform.translation.x = new_x;
                        transform.translation.y = step_height;
                        player.is_grounded = true;
                        player.velocity.y = 0.0;
                    }
                }
                // If step-up failed, don't move (blocked)
            } else {
                transform.translation.x = new_x;
            }

            // Try moving on Z axis
            let new_z = current_pos.z + move_delta.z;
            if check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                transform.translation.x,
                transform.translation.y,
                new_z,
                player.radius,
            ) {
                // Check if we can step up
                let player_collision = PlayerCollision {
                    pos: Vec3::new(transform.translation.x, transform.translation.y, new_z),
                    radius: player.radius,
                    current_y: transform.translation.y,
                };
                if let Some(step_height) = get_step_up_height(
                    &spatial_grid,
                    &sub_voxel_query,
                    &player_collision,
                    max_step_height,
                ) {
                    // Only step up if the height increase is reasonable (within one step)
                    let height_increase = step_height - transform.translation.y;
                    if height_increase > 0.001 && height_increase <= max_step_height + 0.001 {
                        transform.translation.z = new_z;
                        transform.translation.y = step_height;
                        player.is_grounded = true;
                        player.velocity.y = 0.0;
                    }
                }
                // If step-up failed, don't move (blocked)
            } else {
                transform.translation.z = new_z;
            }
        }
    }
}
/// Struct to group player collision parameters for step-up checks.
struct PlayerCollision {
    pos: Vec3,
    radius: f32,
    current_y: f32,
}

fn get_step_up_height(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    player: &PlayerCollision,
    max_step_height: f32,
) -> Option<f32> {
    let collision_radius = player.radius * 0.8;
    let current_bottom = player.current_y - player.radius;

    // Calculate player's AABB for grid lookup
    let player_min = Vec3::new(
        player.pos.x - collision_radius,
        player.pos.y - player.radius,
        player.pos.z - collision_radius,
    );
    let player_max = Vec3::new(
        player.pos.x + collision_radius,
        player.pos.y + player.radius,
        player.pos.z + collision_radius,
    );

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    let mut all_voxels = Vec::new();

    for entity in relevant_sub_voxel_entities {
        if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Check horizontal overlap with target position
            let closest_x = player.pos.x.clamp(min.x, max.x);
            let closest_z = player.pos.z.clamp(min.z, max.z);
            let dx = player.pos.x - closest_x;
            let dz = player.pos.z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                all_voxels.push(max.y);
            }
        }
    }

    if all_voxels.is_empty() {
        return None;
    }

    // Remove duplicates and sort by height
    all_voxels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    all_voxels.dedup_by(|a, b| (*a - *b).abs() < 0.001);

    // Find voxels above the current bottom (excluding floor)
    let above_floor: Vec<f32> = all_voxels
        .iter()
        .filter(|&&h| h > current_bottom + 0.01)
        .copied()
        .collect();

    // Only step up if there's exactly one level above current position
    if above_floor.len() != 1 {
        return None;
    }

    let step_height = above_floor[0];

    // Check step distance from current bottom
    let step_distance = step_height - current_bottom;

    // Allow step up if within max step height
    if step_distance > 0.005 && step_distance <= max_step_height + 0.005 {
        return Some(step_height + player.radius);
    }

    None
}

fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
) -> bool {
    // Use slightly smaller collision radius for tighter fit
    let collision_radius = radius;

    // Calculate player's AABB for grid lookup
    let player_min = Vec3::new(x - collision_radius, y - radius, z - collision_radius);
    let player_max = Vec3::new(x + collision_radius, y + radius, z + collision_radius);

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    // Check all relevant sub-voxels for collision
    for entity in relevant_sub_voxel_entities {
        if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Only check sub-voxels that overlap with player's height, but not the floor
            // Skip sub-voxels that are below player's center (these are floor/ground)
            if max.y <= y - radius * 0.5 {
                continue;
            }

            // Skip if sub-voxel is too far above
            if min.y > y + radius {
                continue;
            }

            // Quick AABB check for horizontal bounds
            if x + collision_radius < min.x
                || x - collision_radius > max.x
                || z + collision_radius < min.z
                || z - collision_radius > max.z
            {
                continue;
            }

            // Find closest point on sub-voxel AABB to player center (horizontal only)
            let closest_x = x.clamp(min.x, max.x);
            let closest_z = z.clamp(min.z, max.z);

            // Check horizontal distance only
            let dx = x - closest_x;
            let dz = z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                return true; // Collision detected
            }
        }
    }

    false
}

pub fn toggle_collision_box(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut collision_box_query: Query<&mut Visibility, With<CollisionBox>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        for mut visibility in &mut collision_box_query {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

pub fn update_collision_box(
    player_query: Query<&Transform, With<Player>>,
    mut collision_box_query: Query<&mut Transform, (With<CollisionBox>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut box_transform in &mut collision_box_query {
            box_transform.translation = player_transform.translation;
        }
    }
}

pub fn rotate_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform)>,
) {
    if let Ok((mut game_camera, mut transform)) = camera_query.get_single_mut() {
        let center = Vec3::new(1.5, 0.0, 1.5);

        // Check if Delete key is pressed
        if keyboard_input.pressed(KeyCode::Delete) {
            // Rotate 90 degrees to the left around the world Y-axis
            let rotation_offset = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            game_camera.target_rotation = rotation_offset * game_camera.original_rotation;
        } else {
            // Return to original rotation
            game_camera.target_rotation = game_camera.original_rotation;
        }

        // Smoothly interpolate rotation
        let new_rotation = transform.rotation.slerp(
            game_camera.target_rotation,
            game_camera.rotation_speed * time.delta_secs(),
        );

        // Calculate how much we rotated
        let rotation_delta = new_rotation * transform.rotation.inverse();

        // Rotate the camera position around the center point
        let offset = transform.translation - center;
        let rotated_offset = rotation_delta * offset;

        transform.translation = center + rotated_offset;
        transform.rotation = new_rotation;
    }
}

pub fn apply_gravity(time: Res<Time>, mut player_query: Query<&mut Player>) {
    const GRAVITY: f32 = -32.0;

    if let Ok(mut player) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues
        let delta = time.delta_secs().min(0.1);
        player.velocity.y += GRAVITY * delta;
    }
}

pub fn apply_physics(
    time: Res<Time>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues
        let delta = time.delta_secs().min(0.1);

        // Apply velocity
        let new_y = transform.translation.y + player.velocity.y * delta;
        let player_bottom = new_y - player.radius;
        let current_bottom = transform.translation.y - player.radius;

        let mut hit_ground = false;
        let mut highest_collision_y = f32::MIN;

        // Check collision with sub-voxels below
        for sub_voxel in sub_voxel_query.iter() {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Only check sub-voxels that are below the player
            if max.y > new_y {
                continue;
            }

            // Check if player sphere is above this sub-voxel horizontally
            let player_x = transform.translation.x;
            let player_z = transform.translation.z;

            // Check horizontal overlap
            let horizontal_overlap = player_x + player.radius > min.x
                && player_x - player.radius < max.x
                && player_z + player.radius > min.z
                && player_z - player.radius < max.z;

            if horizontal_overlap && player.velocity.y <= 0.0 {
                // Check if player's bottom would go through the top of this sub-voxel
                // Player was above, and would now be at or below the top
                if current_bottom >= max.y && player_bottom <= max.y {
                    highest_collision_y = highest_collision_y.max(max.y);
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
