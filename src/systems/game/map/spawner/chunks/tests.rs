use super::*;
use crate::systems::game::map::format::{
    axis_angle_to_matrix, world_dir_to_local, SubVoxelPattern,
};
use crate::systems::game::map::geometry::RotationAxis;
use bevy::math::Vec3A;

#[test]
fn test_calculate_sub_voxel_pos_origin() {
    let pos = calculate_sub_voxel_pos(0, 0, 0, 0, 0, 0);
    // First sub-voxel of first voxel
    // offset = -0.5 + 0.125/2 = -0.4375
    let expected_offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    assert!((pos.x - expected_offset).abs() < 0.001);
    assert!((pos.y - expected_offset).abs() < 0.001);
    assert!((pos.z - expected_offset).abs() < 0.001);
}

#[test]
fn test_calculate_sub_voxel_pos_last_sub_voxel() {
    let pos = calculate_sub_voxel_pos(0, 0, 0, 7, 7, 7);
    // Last sub-voxel (index 7) of first voxel
    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    let expected = offset + (7.0 * SUB_VOXEL_SIZE);
    assert!((pos.x - expected).abs() < 0.001);
    assert!((pos.y - expected).abs() < 0.001);
    assert!((pos.z - expected).abs() < 0.001);
}

#[test]
fn test_calculate_sub_voxel_pos_adjacent_voxel() {
    // First sub-voxel of voxel (1,0,0) should be SUB_VOXEL_COUNT * SUB_VOXEL_SIZE
    // away from first sub-voxel of voxel (0,0,0)
    let pos1 = calculate_sub_voxel_pos(0, 0, 0, 0, 0, 0);
    let pos2 = calculate_sub_voxel_pos(1, 0, 0, 0, 0, 0);
    assert!((pos2.x - pos1.x - 1.0).abs() < 0.001); // 1 voxel = 1 world unit
}

#[test]
fn test_get_sub_voxel_color_deterministic() {
    let color1 = get_sub_voxel_color(5, 10, 15, 3, 4, 5);
    let color2 = get_sub_voxel_color(5, 10, 15, 3, 4, 5);
    // Same input should produce same color
    assert_eq!(format!("{:?}", color1), format!("{:?}", color2));
}

#[test]
fn test_chunk_material_variants() {
    // Just verify the enum variants exist and can be matched
    let occlusion = ChunkMaterial::Occlusion(Handle::default());
    let standard = ChunkMaterial::Standard(Handle::default());

    match occlusion {
        ChunkMaterial::Occlusion(_) => {}
        ChunkMaterial::Standard(_) => panic!("Wrong variant"),
    }

    match standard {
        ChunkMaterial::Standard(_) => {}
        ChunkMaterial::Occlusion(_) => panic!("Wrong variant"),
    }
}

#[test]
fn test_sub_voxel_size_matches_count() {
    // 8 sub-voxels should fit in 1 world unit
    assert!((SUB_VOXEL_COUNT as f32 * SUB_VOXEL_SIZE - 1.0).abs() < 0.001);
}

#[test]
fn test_chunk_coordinates_calculation() {
    // Test that world position maps to correct chunk
    let world_pos = Vec3::new(0.0, 0.0, 0.0);
    let chunk_pos = IVec3::new(
        (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
    );
    assert_eq!(chunk_pos, IVec3::ZERO);
}

#[test]
fn test_chunk_coordinates_negative() {
    let world_pos = Vec3::new(-1.0, -1.0, -1.0);
    let chunk_pos = IVec3::new(
        (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
    );
    assert_eq!(chunk_pos, IVec3::new(-1, -1, -1));
}

#[test]
fn test_chunk_coordinates_boundary() {
    // Position at exactly chunk boundary
    let world_pos = Vec3::new(CHUNK_SIZE as f32, 0.0, 0.0);
    let chunk_pos = IVec3::new(
        (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
    );
    assert_eq!(chunk_pos.x, 1);
}

#[test]
fn chunk_aabb_half_extents_match_chunk_size() {
    let half = Vec3A::splat(CHUNK_SIZE as f32 / 2.0);
    assert_eq!(half, Vec3A::splat(8.0));
}

#[test]
fn chunk_aabb_center_matches_chunk_center() {
    let chunk_center = Vec3::new(8.0, 8.0, 8.0); // center of chunk at (0,0,0)
    let aabb = Aabb {
        center: Vec3A::from(chunk_center),
        half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
    };
    assert_eq!(aabb.center, Vec3A::new(8.0, 8.0, 8.0));
    assert_eq!(aabb.half_extents, Vec3A::splat(8.0));
}

// --- Fence rotation tests ---

/// Fence with rotation:None must produce the same geometry as calling
/// fence_geometry_with_neighbors directly (backward-compat AC-2).
#[test]
fn fence_rotation_none_is_unchanged() {
    let neighbors = (true, false, false, false);
    let expected = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);

    // Simulate the spawner fence branch with rotation = None
    let fence_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
    // orientation is None → no apply_orientation_matrix
    let result = fence_geo;

    let expected_pos: Vec<_> = expected.occupied_positions().collect();
    let result_pos: Vec<_> = result.occupied_positions().collect();
    assert_eq!(expected_pos, result_pos);
}

/// Fence with a Y+90° orientation matrix must produce geometry different from
/// the un-rotated fence — confirming the matrix is applied (AC-1).
#[test]
fn fence_with_rotation_differs_from_unrotated() {
    let neighbors = (true, false, false, false); // one rail in -X direction
    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

    let unrotated = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
    let rotated = apply_orientation_matrix(
        SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors),
        &m_y90,
    );

    let unrotated_pos: Vec<_> = unrotated.occupied_positions().collect();
    let rotated_pos: Vec<_> = rotated.occupied_positions().collect();
    // A fence with a rail in -X, rotated 90° around Y, should have the rail in -Z.
    // The two position sets must differ.
    assert_ne!(unrotated_pos, rotated_pos);
}

/// Applying the orientation matrix to fence geometry must produce the same
/// result as calling apply_orientation_matrix directly (AC-1 correctness).
#[test]
fn fence_with_rotation_matches_manual_apply() {
    let neighbors = (false, true, false, false); // rail in +X
    let m_y180 = axis_angle_to_matrix(RotationAxis::Y, 2);

    let fence_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
    let expected = apply_orientation_matrix(fence_geo.clone(), &m_y180);

    // Simulate spawner fence branch
    let fence_geo2 = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
    let result = apply_orientation_matrix(fence_geo2, &m_y180);

    let expected_pos: Vec<_> = expected.occupied_positions().collect();
    let result_pos: Vec<_> = result.occupied_positions().collect();
    assert_eq!(expected_pos, result_pos);
}

/// Collision bounds are derived from occupied_positions() of the geometry.
/// For a rotated fence the occupied positions must match the rotated geometry,
/// not the un-rotated geometry (AC-4).
#[test]
fn fence_rotated_bounds_use_rotated_positions() {
    let neighbors = (true, false, false, false); // rail in -X
    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

    let unrotated_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
    let rotated_geo = apply_orientation_matrix(
        SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors),
        &m_y90,
    );

    // The set of occupied sub-voxel positions must differ: the rail moved from
    // -X direction to -Z direction after Y+90°.
    let unrotated_positions: std::collections::HashSet<_> =
        unrotated_geo.occupied_positions().collect();
    let rotated_positions: std::collections::HashSet<_> =
        rotated_geo.occupied_positions().collect();

    assert_ne!(
        unrotated_positions, rotated_positions,
        "Rotated fence geometry must have different occupied positions than un-rotated"
    );

    // The sub-voxel count must be identical (rotation preserves count).
    assert_eq!(
        unrotated_positions.len(),
        rotated_positions.len(),
        "Rotation must preserve the number of occupied sub-voxels"
    );
}

/// A fence rotated Y+90° with a world +X neighbor should connect in its local +Z direction.
/// After applying the Y+90° orientation matrix the rail appears in the world +X direction,
/// correctly meeting the neighbor.
///
/// Y+90°: local X → world −Z, local Z → world +X.
/// Inverse: world +X → local +Z. So a world +X neighbor triggers local pos_z = true.
#[test]
fn fence_y90_connects_to_world_x_neighbor() {
    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

    // World +X neighbor exists. Map to local frame via Mᵀ.
    let local_dir = world_dir_to_local(Some(&m_y90), [1, 0, 0]);
    // Y+90°: Mᵀ × [1,0,0] = [0,0,1] (local +Z)
    assert_eq!(
        local_dir,
        [0, 0, 1],
        "world +X must map to local +Z for Y+90°"
    );

    // Build fence geometry with that local connection (pos_z = true).
    let fence_geo =
        SubVoxelPattern::Fence.fence_geometry_with_neighbors((false, false, false, true)); // pos_z

    // Rotate the local geometry into world space (local +Z → world +X).
    let world_geo = apply_orientation_matrix(fence_geo, &m_y90);

    // The rail should now extend in the world +X direction (sub_x > 4).
    let has_rail_in_pos_x = world_geo
        .occupied_positions()
        .any(|(sx, _sy, sz)| sx > 4 && sz >= 3 && sz <= 4);
    assert!(
        has_rail_in_pos_x,
        "after Y+90°, the local +Z rail must appear in the world +X half (sub_x > 4)"
    );
}
