use super::*;

#[test]
fn test_hotbar_item_default() {
    let item = HotbarItem::default();
    assert!(item.is_empty());
}

#[test]
fn test_hotbar_item_voxel() {
    let item = HotbarItem::Voxel {
        voxel_type: VoxelType::Grass,
        pattern: SubVoxelPattern::Full,
    };
    assert!(item.is_voxel());
    assert!(!item.is_entity());
    assert!(!item.is_tool());
    assert!(!item.is_empty());

    let (vtype, pattern) = item.as_voxel().unwrap();
    assert_eq!(vtype, VoxelType::Grass);
    assert_eq!(pattern, SubVoxelPattern::Full);
}

#[test]
fn test_hotbar_item_entity() {
    let item = HotbarItem::Entity {
        entity_type: EntityType::PlayerSpawn,
    };
    assert!(item.is_entity());
    assert!(!item.is_voxel());

    let entity = item.as_entity().unwrap();
    assert_eq!(entity, EntityType::PlayerSpawn);
}

#[test]
fn test_hotbar_item_name() {
    let grass = HotbarItem::Voxel {
        voxel_type: VoxelType::Grass,
        pattern: SubVoxelPattern::Full,
    };
    assert_eq!(grass.name(), "Grass");

    let stairs = HotbarItem::Voxel {
        voxel_type: VoxelType::Stone,
        pattern: SubVoxelPattern::Staircase,
    };
    assert_eq!(stairs.name(), "Stone Stairs");
}

#[test]
fn test_palette_category_cycle() {
    let cat = PaletteCategory::Voxels;
    assert_eq!(cat.next(), PaletteCategory::Patterns);
    assert_eq!(cat.prev(), PaletteCategory::Tools);

    // Full cycle
    let mut c = PaletteCategory::Voxels;
    for _ in 0..4 {
        c = c.next();
    }
    assert_eq!(c, PaletteCategory::Voxels);
}

#[test]
fn test_default_hotbar() {
    let hotbar = default_hotbar();
    assert_eq!(hotbar.len(), 9);
    assert!(hotbar[0].is_voxel());
    assert!(hotbar[5].is_entity());
    assert!(hotbar[7].is_tool());
    assert!(hotbar[8].is_empty());
}
