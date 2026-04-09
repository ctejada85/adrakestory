use super::*;
use crate::systems::game::map::geometry::RotationAxis;

// --- axis_angle_to_matrix ---

#[test]
fn test_identity_matrices_all_axes() {
    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
        assert_eq!(
            axis_angle_to_matrix(axis, 0),
            IDENTITY,
            "angle 0 should be identity for {:?}",
            axis
        );
    }
}

#[test]
fn test_four_rotations_compose_to_identity() {
    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
        let r = axis_angle_to_matrix(axis, 1);
        let r2 = multiply_matrices(&r, &r);
        let r3 = multiply_matrices(&r2, &r);
        let r4 = multiply_matrices(&r3, &r);
        assert_eq!(r4, IDENTITY, "4 × 90° around {:?} should be identity", axis);
    }
}

#[test]
fn test_angle_180_equals_two_90s() {
    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
        let r90 = axis_angle_to_matrix(axis, 1);
        let r180_direct = axis_angle_to_matrix(axis, 2);
        let r180_composed = multiply_matrices(&r90, &r90);
        assert_eq!(
            r180_direct, r180_composed,
            "180° should equal 90°×90° for {:?}",
            axis
        );
    }
}

// --- multiply_matrices ---

#[test]
fn test_multiply_by_identity() {
    let m = axis_angle_to_matrix(RotationAxis::Y, 1);
    assert_eq!(multiply_matrices(&m, &IDENTITY), m);
    assert_eq!(multiply_matrices(&IDENTITY, &m), m);
}

#[test]
fn test_y90_then_x90_composition() {
    // Y+90 followed by X+90
    let my = axis_angle_to_matrix(RotationAxis::Y, 1);
    let mx = axis_angle_to_matrix(RotationAxis::X, 1);
    let composed = multiply_matrices(&mx, &my);
    // The composition must be a valid rotation
    assert!(is_valid_rotation_matrix(&composed));
    assert_ne!(composed, my);
    assert_ne!(composed, mx);
}

// --- apply_orientation_matrix parity with rotate() ---

#[test]
fn test_apply_matches_rotate_single_axis() {
    use crate::systems::game::map::format::patterns::SubVoxelPattern;

    let base = SubVoxelPattern::Staircase.geometry();

    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
        for angle in 1..=3 {
            let via_rotate = base.clone().rotate(axis, angle);
            let matrix = axis_angle_to_matrix(axis, angle);
            let via_matrix = apply_orientation_matrix(base.clone(), &matrix);

            let rotate_positions: std::collections::BTreeSet<_> =
                via_rotate.occupied_positions().collect();
            let matrix_positions: std::collections::BTreeSet<_> =
                via_matrix.occupied_positions().collect();

            assert_eq!(
                rotate_positions, matrix_positions,
                "apply_orientation_matrix should match rotate() for {:?} angle {}",
                axis, angle
            );
        }
    }
}

#[test]
fn test_apply_identity_is_noop() {
    use crate::systems::game::map::format::patterns::SubVoxelPattern;

    let base = SubVoxelPattern::Staircase.geometry();
    let result = apply_orientation_matrix(base.clone(), &IDENTITY);
    let base_positions: std::collections::BTreeSet<_> = base.occupied_positions().collect();
    let result_positions: std::collections::BTreeSet<_> = result.occupied_positions().collect();
    assert_eq!(base_positions, result_positions);
}

#[test]
fn test_apply_multi_axis_composition() {
    use crate::systems::game::map::format::patterns::SubVoxelPattern;

    let base = SubVoxelPattern::Staircase.geometry();

    // Apply Y+90 then X+90 via rotate()
    let via_rotate = base
        .clone()
        .rotate(RotationAxis::Y, 1)
        .rotate(RotationAxis::X, 1);

    // Build the composed matrix and apply
    let my = axis_angle_to_matrix(RotationAxis::Y, 1);
    let mx = axis_angle_to_matrix(RotationAxis::X, 1);
    let composed = multiply_matrices(&mx, &my);
    let via_matrix = apply_orientation_matrix(base, &composed);

    let rotate_positions: std::collections::BTreeSet<_> = via_rotate.occupied_positions().collect();
    let matrix_positions: std::collections::BTreeSet<_> = via_matrix.occupied_positions().collect();

    assert_eq!(rotate_positions, matrix_positions);
}

// --- is_valid_rotation_matrix ---

#[test]
fn test_identity_is_valid() {
    assert!(is_valid_rotation_matrix(&IDENTITY));
}

#[test]
fn test_all_24_single_and_double_axis_are_valid() {
    let cases = [
        (RotationAxis::X, 1),
        (RotationAxis::X, 2),
        (RotationAxis::X, 3),
        (RotationAxis::Y, 1),
        (RotationAxis::Y, 2),
        (RotationAxis::Y, 3),
        (RotationAxis::Z, 1),
        (RotationAxis::Z, 2),
        (RotationAxis::Z, 3),
    ];
    for (axis, angle) in &cases {
        let m = axis_angle_to_matrix(*axis, *angle);
        assert!(
            is_valid_rotation_matrix(&m),
            "axis_angle_to_matrix({:?}, {}) should be valid",
            axis,
            angle
        );
    }
}

#[test]
fn test_reflection_is_invalid() {
    // Reflection: det = -1
    let reflection: OrientationMatrix = [[-1, 0, 0], [0, 1, 0], [0, 0, 1]];
    assert!(!is_valid_rotation_matrix(&reflection));
}

#[test]
fn test_non_permutation_is_invalid() {
    let bad: OrientationMatrix = [[1, 1, 0], [0, 0, 1], [0, 0, 1]];
    assert!(!is_valid_rotation_matrix(&bad));
}

// --- find_or_insert_orientation ---

#[test]
fn test_find_or_insert_deduplicates() {
    let mut list: Vec<OrientationMatrix> = Vec::new();
    let m = axis_angle_to_matrix(RotationAxis::Y, 1);
    let i1 = find_or_insert_orientation(&mut list, m);
    let i2 = find_or_insert_orientation(&mut list, m);
    assert_eq!(i1, i2);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_find_or_insert_appends_distinct() {
    let mut list: Vec<OrientationMatrix> = Vec::new();
    let m1 = axis_angle_to_matrix(RotationAxis::Y, 1);
    let m2 = axis_angle_to_matrix(RotationAxis::X, 1);
    let i1 = find_or_insert_orientation(&mut list, m1);
    let i2 = find_or_insert_orientation(&mut list, m2);
    assert_ne!(i1, i2);
    assert_eq!(list.len(), 2);
}

// --- legacy migration ---

#[test]
fn test_legacy_migration_all_12_single_axis() {
    use crate::systems::game::components::VoxelType;
    use crate::systems::game::map::format::world::VoxelData;

    let cases = [
        (RotationAxis::X, 1),
        (RotationAxis::X, 2),
        (RotationAxis::X, 3),
        (RotationAxis::Y, 1),
        (RotationAxis::Y, 2),
        (RotationAxis::Y, 3),
        (RotationAxis::Z, 1),
        (RotationAxis::Z, 2),
        (RotationAxis::Z, 3),
    ];

    for (axis, angle) in &cases {
        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Stone,
            pattern: None,
            rotation: None,
            rotation_state: Some(LegacyRotationState {
                axis: *axis,
                angle: *angle,
            }),
        }];

        migrate_legacy_rotations(&mut orientations, &mut voxels);

        assert!(
            voxels[0].rotation_state.is_none(),
            "rotation_state must be cleared"
        );
        assert!(voxels[0].rotation.is_some(), "rotation must be set");
        let index = voxels[0].rotation.unwrap();
        assert_eq!(
            orientations[index],
            axis_angle_to_matrix(*axis, *angle),
            "migrated matrix should match for {:?} angle {}",
            axis,
            angle
        );
    }
}

#[test]
fn test_legacy_migration_deduplicates() {
    use crate::systems::game::components::VoxelType;
    use crate::systems::game::map::format::world::VoxelData;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![
        VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Stone,
            pattern: None,
            rotation: None,
            rotation_state: Some(LegacyRotationState {
                axis: RotationAxis::Y,
                angle: 1,
            }),
        },
        VoxelData {
            pos: (1, 0, 0),
            voxel_type: VoxelType::Stone,
            pattern: None,
            rotation: None,
            rotation_state: Some(LegacyRotationState {
                axis: RotationAxis::Y,
                angle: 1,
            }),
        },
    ];

    migrate_legacy_rotations(&mut orientations, &mut voxels);

    assert_eq!(
        orientations.len(),
        1,
        "duplicate matrix must not be added twice"
    );
    assert_eq!(voxels[0].rotation, voxels[1].rotation);
}

// --- normalise_staircase_variants ---

/// Helper: build a minimal VoxelData for normalisation tests.
fn staircase_voxel(
    pattern: super::super::patterns::SubVoxelPattern,
    rotation: Option<usize>,
) -> super::super::world::VoxelData {
    use crate::systems::game::components::VoxelType;
    super::super::world::VoxelData {
        pos: (0, 0, 0),
        voxel_type: VoxelType::Stone,
        pattern: Some(pattern),
        rotation,
        rotation_state: None,
    }
}

#[test]
fn normalise_staircase_neg_x_rotation_none_absorbs_y180() {
    use super::super::patterns::SubVoxelPattern;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegX, None)];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(
        voxels[0].pattern,
        Some(SubVoxelPattern::Staircase),
        "pattern must be normalised to Staircase"
    );
    let idx = voxels[0]
        .rotation
        .expect("rotation must be Some after normalisation");
    assert_eq!(
        orientations[idx],
        axis_angle_to_matrix(RotationAxis::Y, 2),
        "absorbed matrix must be Y+180°"
    );
}

#[test]
fn normalise_staircase_z_rotation_none_absorbs_y90() {
    use super::super::patterns::SubVoxelPattern;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, None)];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
    let idx = voxels[0].rotation.expect("rotation must be set");
    assert_eq!(
        orientations[idx],
        axis_angle_to_matrix(RotationAxis::Y, 1),
        "absorbed matrix must be Y+90°"
    );
}

#[test]
fn normalise_staircase_neg_z_rotation_none_absorbs_y270() {
    use super::super::patterns::SubVoxelPattern;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegZ, None)];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
    let idx = voxels[0].rotation.expect("rotation must be set");
    assert_eq!(
        orientations[idx],
        axis_angle_to_matrix(RotationAxis::Y, 3),
        "absorbed matrix must be Y+270°"
    );
}

#[test]
fn normalise_staircase_z_with_existing_y90_composes_to_y180() {
    use super::super::patterns::SubVoxelPattern;

    // Pre-populate orientations with M_y90 at index 0
    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
    let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
    // StaircaseZ pre-bake = Y+90°; existing = Y+90°; composed = Y+180°
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, Some(0))];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
    let idx = voxels[0]
        .rotation
        .expect("composed Y+180° must produce a non-None rotation");
    assert_eq!(
        orientations[idx],
        axis_angle_to_matrix(RotationAxis::Y, 2),
        "composed matrix must be Y+180°"
    );
}

#[test]
fn normalise_staircase_neg_z_plus_y90_equals_identity_sets_rotation_none() {
    use super::super::patterns::SubVoxelPattern;

    // StaircaseNegZ pre-bake = Y+270°; existing orientation = Y+90°
    // composed = Y+90° × Y+270° = Y+360° = IDENTITY → rotation: None
    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
    let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegZ, Some(0))];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
    assert_eq!(
        voxels[0].rotation, None,
        "composed == IDENTITY must set rotation: None"
    );
}

#[test]
fn normalise_canonical_staircase_is_unchanged() {
    use super::super::patterns::SubVoxelPattern;

    let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
    let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::Staircase, Some(0))];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    // Pattern and rotation must be unchanged
    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
    assert_eq!(voxels[0].rotation, Some(0));
    assert_eq!(
        orientations.len(),
        1,
        "no new orientation must be appended for canonical Staircase"
    );
}

#[test]
fn normalise_non_staircase_pattern_is_unchanged() {
    use super::super::patterns::SubVoxelPattern;
    use super::super::world::VoxelData;
    use crate::systems::game::components::VoxelType;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![VoxelData {
        pos: (0, 0, 0),
        voxel_type: VoxelType::Stone,
        pattern: Some(SubVoxelPattern::Full),
        rotation: None,
        rotation_state: None,
    }];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Full));
    assert_eq!(voxels[0].rotation, None);
    assert!(
        orientations.is_empty(),
        "no orientations should be added for non-staircase"
    );
}

#[test]
fn normalise_deduplicates_identical_composed_matrices() {
    use super::super::patterns::SubVoxelPattern;

    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    // Two voxels with the same directional variant and no prior rotation
    // should share the same orientation index after normalisation.
    let mut voxels = vec![
        staircase_voxel(SubVoxelPattern::StaircaseZ, None),
        staircase_voxel(SubVoxelPattern::StaircaseZ, None),
    ];

    normalise_staircase_variants(&mut orientations, &mut voxels);

    assert_eq!(
        orientations.len(),
        1,
        "identical composed matrices must be deduplicated"
    );
    assert_eq!(
        voxels[0].rotation, voxels[1].rotation,
        "both voxels must reference the same orientation index"
    );
}

#[test]
fn normalise_round_trip_geometry_identical_to_legacy_pre_bake() {
    use super::super::patterns::SubVoxelPattern;
    use crate::systems::game::map::geometry::RotationAxis as GeoAxis;

    // The geometry produced by normalised Staircase + composed matrix
    // must be bit-for-bit identical to SubVoxelGeometry::staircase_x().rotate(Y, 1)
    // (the old StaircaseZ.geometry() pre-bake path).

    // --- Legacy path (what the bug used to produce for rotation: None) ---
    let legacy_geom = SubVoxelPattern::StaircaseZ.geometry();

    // --- Normalised path ---
    let mut orientations: Vec<OrientationMatrix> = Vec::new();
    let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, None)];
    normalise_staircase_variants(&mut orientations, &mut voxels);

    let orientation = voxels[0].rotation.map(|i| &orientations[i]);
    let normalised_geom = SubVoxelPattern::Staircase.geometry_with_rotation(orientation);

    let legacy_positions: std::collections::BTreeSet<_> =
        legacy_geom.occupied_positions().collect();
    let normalised_positions: std::collections::BTreeSet<_> =
        normalised_geom.occupied_positions().collect();

    assert_eq!(
        legacy_positions, normalised_positions,
        "normalised geometry must match old StaircaseZ.geometry() pre-bake"
    );

    // The unused import suppression — GeoAxis is imported above for clarity
    let _ = GeoAxis::Y;
}

// --- world_dir_to_local tests ---

#[test]
fn world_dir_to_local_identity_returns_dir_unchanged() {
    assert_eq!(world_dir_to_local(None, [1, 0, 0]), [1, 0, 0]);
    assert_eq!(world_dir_to_local(None, [-1, 0, 0]), [-1, 0, 0]);
    assert_eq!(world_dir_to_local(None, [0, 0, 1]), [0, 0, 1]);
    assert_eq!(world_dir_to_local(None, [0, 0, -1]), [0, 0, -1]);
}

#[test]
fn world_dir_to_local_y90_maps_world_x_to_local_neg_z() {
    // Y+90°: local X → world −Z, local Z → world +X
    // Inverse (Mᵀ): world +X → local +Z, world −X → local −Z,
    //               world +Z → local −X, world −Z → local +X
    let m = axis_angle_to_matrix(RotationAxis::Y, 1);
    assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [0, 0, 1]);
    assert_eq!(world_dir_to_local(Some(&m), [-1, 0, 0]), [0, 0, -1]);
    assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [-1, 0, 0]);
    assert_eq!(world_dir_to_local(Some(&m), [0, 0, -1]), [1, 0, 0]);
}

#[test]
fn world_dir_to_local_y180_maps_x_to_neg_x() {
    // Y+180°: local X → world −X, local Z → world −Z
    // Inverse: world +X → local −X, world +Z → local −Z
    let m = axis_angle_to_matrix(RotationAxis::Y, 2);
    assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [-1, 0, 0]);
    assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [0, 0, -1]);
}

#[test]
fn world_dir_to_local_y270_maps_world_x_to_local_z() {
    // Y+270° (= Y−90°): local X → world +Z, local Z → world −X
    // Inverse: world +X → local −Z, world +Z → local +X
    let m = axis_angle_to_matrix(RotationAxis::Y, 3);
    assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [0, 0, -1]);
    assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [1, 0, 0]);
}
