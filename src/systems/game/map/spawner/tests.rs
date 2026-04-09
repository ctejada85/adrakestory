use super::*;

#[test]
fn test_sub_voxel_count_is_8() {
    assert_eq!(SUB_VOXEL_COUNT, 8);
}

#[test]
fn test_sub_voxel_size_is_eighth() {
    assert!((SUB_VOXEL_SIZE - 0.125).abs() < 0.001);
}

#[test]
fn test_chunk_size_is_16() {
    assert_eq!(CHUNK_SIZE, 16);
}

#[test]
fn test_lod_levels_is_4() {
    assert_eq!(LOD_LEVELS, 4);
}

#[test]
fn test_lod_distances_increasing() {
    for i in 0..LOD_DISTANCES.len() - 1 {
        assert!(
            LOD_DISTANCES[i] < LOD_DISTANCES[i + 1],
            "LOD distances should be increasing"
        );
    }
}

#[test]
fn test_face_normal_pos_x() {
    let normal = Face::PosX.normal();
    assert_eq!(normal, [1.0, 0.0, 0.0]);
}

#[test]
fn test_face_normal_neg_x() {
    let normal = Face::NegX.normal();
    assert_eq!(normal, [-1.0, 0.0, 0.0]);
}

#[test]
fn test_face_normal_pos_y() {
    let normal = Face::PosY.normal();
    assert_eq!(normal, [0.0, 1.0, 0.0]);
}

#[test]
fn test_face_normal_neg_y() {
    let normal = Face::NegY.normal();
    assert_eq!(normal, [0.0, -1.0, 0.0]);
}

#[test]
fn test_face_normal_pos_z() {
    let normal = Face::PosZ.normal();
    assert_eq!(normal, [0.0, 0.0, 1.0]);
}

#[test]
fn test_face_normal_neg_z() {
    let normal = Face::NegZ.normal();
    assert_eq!(normal, [0.0, 0.0, -1.0]);
}

#[test]
fn test_face_offset_pos_x() {
    assert_eq!(Face::PosX.offset(), (1, 0, 0));
}

#[test]
fn test_face_offset_neg_x() {
    assert_eq!(Face::NegX.offset(), (-1, 0, 0));
}

#[test]
fn test_face_offset_pos_y() {
    assert_eq!(Face::PosY.offset(), (0, 1, 0));
}

#[test]
fn test_face_offset_neg_y() {
    assert_eq!(Face::NegY.offset(), (0, -1, 0));
}

#[test]
fn test_face_offset_pos_z() {
    assert_eq!(Face::PosZ.offset(), (0, 0, 1));
}

#[test]
fn test_face_offset_neg_z() {
    assert_eq!(Face::NegZ.offset(), (0, 0, -1));
}

#[test]
fn test_opposite_faces_have_opposite_offsets() {
    let pairs = [
        (Face::PosX, Face::NegX),
        (Face::PosY, Face::NegY),
        (Face::PosZ, Face::NegZ),
    ];

    for (pos, neg) in pairs {
        let (px, py, pz) = pos.offset();
        let (nx, ny, nz) = neg.offset();
        assert_eq!(px + nx, 0);
        assert_eq!(py + ny, 0);
        assert_eq!(pz + nz, 0);
    }
}

#[test]
fn test_opposite_faces_have_opposite_normals() {
    let pairs = [
        (Face::PosX, Face::NegX),
        (Face::PosY, Face::NegY),
        (Face::PosZ, Face::NegZ),
    ];

    for (pos, neg) in pairs {
        let pn = pos.normal();
        let nn = neg.normal();
        assert!((pn[0] + nn[0]).abs() < 0.001);
        assert!((pn[1] + nn[1]).abs() < 0.001);
        assert!((pn[2] + nn[2]).abs() < 0.001);
    }
}

// --- LOD movement threshold tests ---

#[test]
fn lod_threshold_constant_is_well_below_lod_distances() {
    // Threshold must be much smaller than the smallest LOD transition distance
    // so it never causes a false skip near a LOD boundary.
    assert!(LOD_MOVEMENT_THRESHOLD < LOD_DISTANCES[0] / 10.0);
}

#[test]
fn lod_threshold_guard_skips_when_camera_stationary() {
    let camera_pos = Vec3::new(10.0, 5.0, 20.0);
    let last_pos = camera_pos; // no movement
    assert!(camera_pos.distance(last_pos) < LOD_MOVEMENT_THRESHOLD);
}

#[test]
fn lod_threshold_guard_runs_when_camera_moves_beyond_threshold() {
    let last_pos = Vec3::new(10.0, 5.0, 20.0);
    let camera_pos = last_pos + Vec3::new(1.0, 0.0, 0.0); // 1.0 unit — beyond 0.5
    assert!(camera_pos.distance(last_pos) >= LOD_MOVEMENT_THRESHOLD);
}

#[test]
fn lod_threshold_exact_boundary_skips() {
    // At exactly LOD_MOVEMENT_THRESHOLD distance the strict < guard should skip.
    let last_pos = Vec3::ZERO;
    let camera_pos = Vec3::new(LOD_MOVEMENT_THRESHOLD, 0.0, 0.0);
    assert!(!(camera_pos.distance(last_pos) < LOD_MOVEMENT_THRESHOLD));
}

#[test]
fn lod_threshold_cold_start_passes_at_non_origin_position() {
    // On first frame last_camera_pos = Vec3::ZERO; a typical camera spawn position
    // is far from the origin, so the guard must pass and run the full pass.
    let last_pos = Vec3::ZERO;
    let camera_pos = Vec3::new(50.0, 10.0, 30.0);
    assert!(camera_pos.distance(last_pos) >= LOD_MOVEMENT_THRESHOLD);
}

// --- LodConfig resource tests ---

#[test]
fn lod_config_default_matches_constant() {
    let config = LodConfig::default();
    assert_eq!(config.movement_threshold, LOD_MOVEMENT_THRESHOLD);
}
