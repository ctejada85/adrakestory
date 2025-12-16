//! Hotbar system for quick item selection.
//!
//! Provides a 9-slot hotbar similar to Minecraft for quick access to
//! voxel types, patterns, entities, and tools.

use crate::editor::state::EditorTool;
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};

/// What can be stored in a hotbar slot.
#[derive(Clone, Debug, PartialEq)]
pub enum HotbarItem {
    /// Empty slot
    Empty,
    /// Voxel with specific type and pattern
    Voxel {
        voxel_type: VoxelType,
        pattern: SubVoxelPattern,
    },
    /// Entity placement
    Entity { entity_type: EntityType },
    /// Editor tool
    Tool(EditorTool),
}

impl Default for HotbarItem {
    fn default() -> Self {
        Self::Empty
    }
}

impl HotbarItem {
    /// Get a display name for this item.
    pub fn name(&self) -> String {
        match self {
            Self::Empty => "Empty".to_string(),
            Self::Voxel {
                voxel_type,
                pattern,
            } => {
                let type_name = match voxel_type {
                    VoxelType::Air => "Air",
                    VoxelType::Grass => "Grass",
                    VoxelType::Dirt => "Dirt",
                    VoxelType::Stone => "Stone",
                };
                let pattern_name = match pattern {
                    SubVoxelPattern::Full => "",
                    SubVoxelPattern::PlatformXZ => " Platform",
                    SubVoxelPattern::PlatformXY => " Wall-Z",
                    SubVoxelPattern::PlatformYZ => " Wall-X",
                    SubVoxelPattern::StaircaseX => " Stairs+X",
                    SubVoxelPattern::StaircaseNegX => " Stairs-X",
                    SubVoxelPattern::StaircaseZ => " Stairs+Z",
                    SubVoxelPattern::StaircaseNegZ => " Stairs-Z",
                    SubVoxelPattern::Pillar => " Pillar",
                    SubVoxelPattern::Fence => " Fence",
                };
                format!("{}{}", type_name, pattern_name)
            }
            Self::Entity { entity_type } => match entity_type {
                EntityType::PlayerSpawn => "Player Spawn".to_string(),
                EntityType::Npc => "NPC".to_string(),
                EntityType::Enemy => "Enemy".to_string(),
                EntityType::Item => "Item".to_string(),
                EntityType::Trigger => "Trigger".to_string(),
                EntityType::LightSource => "Light".to_string(),
            },
            Self::Tool(tool) => tool.name().to_string(),
        }
    }

    /// Get an icon/emoji for this item.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Empty => "  ",
            Self::Voxel { voxel_type, .. } => match voxel_type {
                VoxelType::Air => "  ",
                VoxelType::Grass => "🟩",
                VoxelType::Dirt => "🟫",
                VoxelType::Stone => "⬜",
            },
            Self::Entity { entity_type } => match entity_type {
                EntityType::PlayerSpawn => "🟢",
                EntityType::Npc => "🔵",
                EntityType::Enemy => "🔴",
                EntityType::Item => "🟡",
                EntityType::Trigger => "🟣",
                EntityType::LightSource => "💡",
            },
            Self::Tool(tool) => match tool {
                EditorTool::VoxelPlace { .. } => "✏️",
                EditorTool::VoxelRemove => "🗑️",
                EditorTool::EntityPlace { .. } => "📍",
                EditorTool::Select => "🔲",
                EditorTool::Camera => "📷",
            },
        }
    }

    /// Check if this item can place voxels.
    pub fn is_voxel(&self) -> bool {
        matches!(self, Self::Voxel { .. })
    }

    /// Check if this item can place entities.
    pub fn is_entity(&self) -> bool {
        matches!(self, Self::Entity { .. })
    }

    /// Check if this is a tool.
    pub fn is_tool(&self) -> bool {
        matches!(self, Self::Tool(_))
    }

    /// Check if this slot is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get voxel data if this is a voxel item.
    pub fn as_voxel(&self) -> Option<(VoxelType, SubVoxelPattern)> {
        match self {
            Self::Voxel {
                voxel_type,
                pattern,
            } => Some((*voxel_type, *pattern)),
            _ => None,
        }
    }

    /// Get entity type if this is an entity item.
    pub fn as_entity(&self) -> Option<EntityType> {
        match self {
            Self::Entity { entity_type } => Some(*entity_type),
            _ => None,
        }
    }
}

/// Categories for the item palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaletteCategory {
    #[default]
    Voxels,
    Patterns,
    Entities,
    Tools,
}

impl PaletteCategory {
    /// Get all categories in order.
    pub fn all() -> &'static [PaletteCategory] {
        &[
            PaletteCategory::Voxels,
            PaletteCategory::Patterns,
            PaletteCategory::Entities,
            PaletteCategory::Tools,
        ]
    }

    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Voxels => "Voxels",
            Self::Patterns => "Patterns",
            Self::Entities => "Entities",
            Self::Tools => "Tools",
        }
    }

    /// Get the next category (wrapping).
    pub fn next(&self) -> Self {
        match self {
            Self::Voxels => Self::Patterns,
            Self::Patterns => Self::Entities,
            Self::Entities => Self::Tools,
            Self::Tools => Self::Voxels,
        }
    }

    /// Get the previous category (wrapping).
    pub fn prev(&self) -> Self {
        match self {
            Self::Voxels => Self::Tools,
            Self::Patterns => Self::Voxels,
            Self::Entities => Self::Patterns,
            Self::Tools => Self::Entities,
        }
    }

    /// Get items in this category.
    pub fn items(&self) -> Vec<HotbarItem> {
        match self {
            Self::Voxels => vec![
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Grass,
                    pattern: SubVoxelPattern::Full,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Dirt,
                    pattern: SubVoxelPattern::Full,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::Full,
                },
            ],
            Self::Patterns => vec![
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::Full,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::PlatformXZ,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::PlatformXY,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::PlatformYZ,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::StaircaseX,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::StaircaseNegX,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::StaircaseZ,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::StaircaseNegZ,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::Pillar,
                },
                HotbarItem::Voxel {
                    voxel_type: VoxelType::Stone,
                    pattern: SubVoxelPattern::Fence,
                },
            ],
            Self::Entities => vec![
                HotbarItem::Entity {
                    entity_type: EntityType::PlayerSpawn,
                },
                HotbarItem::Entity {
                    entity_type: EntityType::Npc,
                },
                HotbarItem::Entity {
                    entity_type: EntityType::Enemy,
                },
                HotbarItem::Entity {
                    entity_type: EntityType::Item,
                },
                HotbarItem::Entity {
                    entity_type: EntityType::Trigger,
                },
                HotbarItem::Entity {
                    entity_type: EntityType::LightSource,
                },
            ],
            Self::Tools => vec![
                HotbarItem::Tool(EditorTool::Select),
                HotbarItem::Tool(EditorTool::VoxelRemove),
                HotbarItem::Tool(EditorTool::Camera),
            ],
        }
    }
}

/// The default hotbar configuration.
pub fn default_hotbar() -> [HotbarItem; 9] {
    [
        HotbarItem::Voxel {
            voxel_type: VoxelType::Grass,
            pattern: SubVoxelPattern::Full,
        },
        HotbarItem::Voxel {
            voxel_type: VoxelType::Dirt,
            pattern: SubVoxelPattern::Full,
        },
        HotbarItem::Voxel {
            voxel_type: VoxelType::Stone,
            pattern: SubVoxelPattern::Full,
        },
        HotbarItem::Voxel {
            voxel_type: VoxelType::Grass,
            pattern: SubVoxelPattern::StaircaseX,
        },
        HotbarItem::Voxel {
            voxel_type: VoxelType::Stone,
            pattern: SubVoxelPattern::PlatformXZ,
        },
        HotbarItem::Entity {
            entity_type: EntityType::PlayerSpawn,
        },
        HotbarItem::Entity {
            entity_type: EntityType::LightSource,
        },
        HotbarItem::Tool(EditorTool::Select),
        HotbarItem::Empty,
    ]
}

#[cfg(test)]
mod tests {
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
            pattern: SubVoxelPattern::StaircaseX,
        };
        assert_eq!(stairs.name(), "Stone Stairs+X");
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
}
