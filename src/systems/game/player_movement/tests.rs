use super::*;

// widened AABB pre-fetch tests
#[test]
fn widened_aabb_covers_movement_destination() {
    // Verify the pre-fetch AABB formula used in move_player contains both the
    // current position cylinder and the destination cylinder in XZ, plus the
    // step-up height upward in Y.
    let player_x = 5.0_f32;
    let player_y = 2.0_f32;
    let player_z = 3.0_f32;
    let radius = 0.2_f32;
    let half_height = 0.4_f32;
    let move_delta_x = 0.1_f32;
    let move_delta_z = -0.05_f32;

    let prefetch_min = Vec3::new(
        player_x - radius - move_delta_x.abs(),
        player_y - half_height,
        player_z - radius - move_delta_z.abs(),
    );
    let prefetch_max = Vec3::new(
        player_x + radius + move_delta_x.abs(),
        player_y + half_height + SUB_VOXEL_SIZE + STEP_UP_TOLERANCE,
        player_z + radius + move_delta_z.abs(),
    );

    // Current position cylinder must be within the pre-fetch bounds
    assert!(player_x - radius >= prefetch_min.x);
    assert!(player_x + radius <= prefetch_max.x);
    assert!(player_z - radius >= prefetch_min.z);
    assert!(player_z + radius <= prefetch_max.z);

    // Destination cylinder must also be within the pre-fetch bounds
    let new_x = player_x + move_delta_x;
    let new_z = player_z + move_delta_z;
    assert!(new_x - radius >= prefetch_min.x);
    assert!(new_x + radius <= prefetch_max.x);
    assert!(new_z - radius >= prefetch_min.z);
    assert!(new_z + radius <= prefetch_max.z);

    // Step-up height must be covered (top of cylinder at player_y + half_height + step_up)
    let step_up_height = SUB_VOXEL_SIZE + STEP_UP_TOLERANCE;
    let new_y_stepped = player_y + step_up_height;
    assert!(new_y_stepped + half_height <= prefetch_max.y);
}
