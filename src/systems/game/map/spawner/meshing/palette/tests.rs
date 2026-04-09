use super::*;

#[test]
fn test_palette_size_is_64() {
    assert_eq!(VoxelMaterialPalette::PALETTE_SIZE, 64);
}

#[test]
fn test_material_index_in_range() {
    // Test various positions to ensure index is always in valid range
    let test_positions = [
        (0, 0, 0, 0, 0, 0),
        (1, 2, 3, 4, 5, 6),
        (-1, -2, -3, 0, 0, 0),
        (100, 200, 300, 7, 7, 7),
        (i32::MAX, i32::MAX, i32::MAX, 7, 7, 7),
        (i32::MIN, i32::MIN, i32::MIN, 0, 0, 0),
    ];

    for (x, y, z, sx, sy, sz) in test_positions {
        let index = VoxelMaterialPalette::get_material_index(x, y, z, sx, sy, sz);
        assert!(
            index < VoxelMaterialPalette::PALETTE_SIZE,
            "Index {} out of range for position ({}, {}, {}, {}, {}, {})",
            index,
            x,
            y,
            z,
            sx,
            sy,
            sz
        );
    }
}

#[test]
fn test_same_position_same_index() {
    let index1 = VoxelMaterialPalette::get_material_index(5, 10, 15, 3, 4, 5);
    let index2 = VoxelMaterialPalette::get_material_index(5, 10, 15, 3, 4, 5);
    assert_eq!(index1, index2);
}

#[test]
fn test_different_positions_may_differ() {
    let index1 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 0);
    let index2 = VoxelMaterialPalette::get_material_index(1, 0, 0, 0, 0, 0);
    let index3 = VoxelMaterialPalette::get_material_index(0, 1, 0, 0, 0, 0);
    let index4 = VoxelMaterialPalette::get_material_index(0, 0, 1, 0, 0, 0);

    // While not guaranteed to be different, they should not all be the same
    // due to the prime-based hash
    let all_same = index1 == index2 && index2 == index3 && index3 == index4;
    assert!(!all_same, "Hash collision for adjacent positions");
}

#[test]
fn test_hash_distribution() {
    // Test that the hash function produces reasonable distribution
    // by checking that we get multiple different indices over a range
    let mut indices = std::collections::HashSet::new();

    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                let index = VoxelMaterialPalette::get_material_index(x, y, z, 0, 0, 0);
                indices.insert(index);
            }
        }
    }

    // We should get a reasonable spread of indices (at least 10 unique values)
    assert!(
        indices.len() >= 10,
        "Poor hash distribution: only {} unique indices",
        indices.len()
    );
}

#[test]
fn test_sub_voxel_coords_affect_index() {
    let index1 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 0);
    let index2 = VoxelMaterialPalette::get_material_index(0, 0, 0, 1, 0, 0);
    let index3 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 1, 0);
    let index4 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 1);

    // Sub-voxel coordinates should affect the hash
    let all_same = index1 == index2 && index2 == index3 && index3 == index4;
    assert!(!all_same, "Sub-voxel coordinates don't affect hash");
}
