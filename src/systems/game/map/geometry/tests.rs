//! Tests for SubVoxelGeometry.

#[cfg(test)]
mod unit_tests {
    use crate::systems::game::map::geometry::{RotationAxis, SubVoxelGeometry};

    #[test]
    fn test_empty_geometry() {
        let geom = SubVoxelGeometry::new();
        assert_eq!(geom.count_occupied(), 0);
        assert!(!geom.is_occupied(0, 0, 0));
    }

    #[test]
    fn test_set_and_check_occupied() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(3, 4, 5);
        assert!(geom.is_occupied(3, 4, 5));
        assert!(!geom.is_occupied(3, 4, 6));
        assert_eq!(geom.count_occupied(), 1);
    }

    #[test]
    fn test_clear() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(2, 3, 4);
        assert!(geom.is_occupied(2, 3, 4));
        geom.clear(2, 3, 4);
        assert!(!geom.is_occupied(2, 3, 4));
    }

    #[test]
    fn test_full_geometry() {
        let geom = SubVoxelGeometry::full();
        assert_eq!(geom.count_occupied(), 512); // 8×8×8
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 7, 7));
    }

    #[test]
    fn test_platform_horizontal() {
        let geom = SubVoxelGeometry::platform_horizontal();
        assert_eq!(geom.count_occupied(), 64); // 8×1×8
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 0, 7));
        assert!(!geom.is_occupied(0, 1, 0));
    }

    #[test]
    fn test_staircase_x() {
        let geom = SubVoxelGeometry::staircase_x();
        // Step 0: 1 high, Step 1: 2 high, ..., Step 7: 8 high
        // Total: (1+2+3+4+5+6+7+8) * 8 = 36 * 8 = 288
        assert_eq!(geom.count_occupied(), 288);
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 7, 7));
        assert!(!geom.is_occupied(0, 1, 0)); // Step 0 is only 1 high
    }

    #[test]
    fn test_pillar() {
        let geom = SubVoxelGeometry::pillar();
        assert_eq!(geom.count_occupied(), 8); // 2×2×2
        assert!(geom.is_occupied(3, 3, 3));
        assert!(geom.is_occupied(4, 4, 4));
        assert!(!geom.is_occupied(2, 3, 3));
    }

    #[test]
    fn test_fence_x() {
        let geom = SubVoxelGeometry::fence_x();
        // Left post: 1×8×1 = 8
        // Right post: 1×8×1 = 8
        // Bottom rail: 6×1×1 = 6
        // Top rail: 6×1×1 = 6
        // Total: 8 + 8 + 6 + 6 = 28
        assert_eq!(geom.count_occupied(), 28);
        // Check left post at z=0 edge
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(0, 7, 0));
        // Check right post at z=0 edge
        assert!(geom.is_occupied(7, 0, 0));
        assert!(geom.is_occupied(7, 7, 0));
        // Check rails at z=0 edge
        assert!(geom.is_occupied(3, 2, 0)); // bottom rail
        assert!(geom.is_occupied(3, 5, 0)); // top rail
                                            // Check gaps
        assert!(!geom.is_occupied(3, 0, 0)); // below bottom rail
        assert!(!geom.is_occupied(3, 4, 0)); // between rails
    }

    #[test]
    fn test_fence_corner() {
        let geom = SubVoxelGeometry::fence_corner();
        // Corner post: 1×8×1 = 8
        // X-axis rails: 7×1×1 × 2 = 14
        // Z-axis rails: 1×1×7 × 2 = 14
        // Total: 8 + 14 + 14 = 36
        assert_eq!(geom.count_occupied(), 36);
        // Check corner post
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(0, 7, 0));
        // Check X-axis rails
        assert!(geom.is_occupied(4, 2, 0)); // bottom rail
        assert!(geom.is_occupied(4, 5, 0)); // top rail
                                            // Check Z-axis rails
        assert!(geom.is_occupied(0, 2, 4)); // bottom rail
        assert!(geom.is_occupied(0, 5, 4)); // top rail
    }

    #[test]
    fn test_fence_post() {
        let geom = SubVoxelGeometry::fence_post();
        // Center post: 2×8×2 = 32
        assert_eq!(geom.count_occupied(), 32);
        // Check center post
        assert!(geom.is_occupied(3, 0, 3));
        assert!(geom.is_occupied(4, 7, 4));
        // Check edges are empty
        assert!(!geom.is_occupied(0, 0, 0));
        assert!(!geom.is_occupied(7, 0, 7));
    }

    #[test]
    fn test_fence_with_connections() {
        // No connections - just post
        let geom = SubVoxelGeometry::fence_with_connections(false, false, false, false);
        assert_eq!(geom.count_occupied(), 32); // Just the center post

        // All connections
        let geom = SubVoxelGeometry::fence_with_connections(true, true, true, true);
        // Center post: 2×8×2 = 32
        // Each rail direction: 3×2×2 × 2 (top+bottom) = 24 per direction
        // But rails overlap with post, so: 32 + 4 directions × (3×2×2) = 32 + 48 = 80
        // Actually: neg_x rails (3×2×2)=12, pos_x rails (3×2×2)=12, neg_z (3×2×2)=12, pos_z (3×2×2)=12
        // Two rail heights each, so 12×2 = 24 per direction, but we need to count actual sub-voxels
        // Let's just verify connectivity
        assert!(geom.is_occupied(0, 2, 3)); // neg_x rail
        assert!(geom.is_occupied(7, 2, 3)); // pos_x rail
        assert!(geom.is_occupied(3, 2, 0)); // neg_z rail
        assert!(geom.is_occupied(3, 2, 7)); // pos_z rail

        // Single connection (pos_x only)
        let geom = SubVoxelGeometry::fence_with_connections(false, true, false, false);
        assert!(geom.is_occupied(3, 0, 3)); // center post
        assert!(geom.is_occupied(7, 2, 3)); // pos_x rail
        assert!(!geom.is_occupied(0, 2, 3)); // no neg_x rail
    }

    #[test]
    fn test_rotation_preserves_count() {
        let geom = SubVoxelGeometry::platform_horizontal();
        let count = geom.count_occupied();

        let rotated_x = geom.rotate(RotationAxis::X, 1);
        assert_eq!(rotated_x.count_occupied(), count);

        let rotated_y = geom.rotate(RotationAxis::Y, 1);
        assert_eq!(rotated_y.count_occupied(), count);

        let rotated_z = geom.rotate(RotationAxis::Z, 1);
        assert_eq!(rotated_z.count_occupied(), count);
    }

    #[test]
    fn test_rotation_360_returns_original() {
        let geom = SubVoxelGeometry::staircase_x();

        let rotated = geom
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1);

        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_rotation_180_twice_returns_original() {
        let geom = SubVoxelGeometry::platform_horizontal();

        let rotated = geom.rotate(RotationAxis::X, 2).rotate(RotationAxis::X, 2);

        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_no_rotation() {
        let geom = SubVoxelGeometry::full();
        let rotated = geom.rotate(RotationAxis::Y, 0);
        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_platform_rotation_x_90() {
        let platform = SubVoxelGeometry::platform_horizontal();
        let rotated = platform.rotate(RotationAxis::X, 1);

        // After 90° rotation around X, horizontal platform (8×1×8) becomes vertical (8×8×1)
        // All sub-voxels should be at z=0
        for (_, _, z) in rotated.occupied_positions() {
            assert_eq!(z, 0, "All sub-voxels should be at z=0 after X rotation");
        }
        assert_eq!(rotated.count_occupied(), 64);
    }

    #[test]
    fn test_platform_rotation_z_90() {
        let platform = SubVoxelGeometry::platform_horizontal();
        let rotated = platform.rotate(RotationAxis::Z, 1);

        // After 90° rotation around Z, horizontal platform (8×1×8) becomes vertical (1×8×8)
        // All sub-voxels should be at x=7 (rotated to the far side)
        for (x, _, _) in rotated.occupied_positions() {
            assert_eq!(x, 7, "All sub-voxels should be at x=7 after Z rotation");
        }
        assert_eq!(rotated.count_occupied(), 64);
    }

    #[test]
    fn test_occupied_positions_iterator() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(1, 2, 3);
        geom.set_occupied(4, 5, 6);

        let positions: Vec<_> = geom.occupied_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(1, 2, 3)));
        assert!(positions.contains(&(4, 5, 6)));
    }

    #[test]
    fn test_bounds_checking() {
        let mut geom = SubVoxelGeometry::new();

        // Out of bounds should not panic
        geom.set_occupied(-1, 0, 0);
        geom.set_occupied(8, 0, 0);
        geom.set_occupied(0, -1, 0);
        geom.set_occupied(0, 8, 0);

        assert_eq!(geom.count_occupied(), 0);
        assert!(!geom.is_occupied(-1, 0, 0));
        assert!(!geom.is_occupied(8, 0, 0));
    }
}
