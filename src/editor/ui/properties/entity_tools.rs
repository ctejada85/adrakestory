//! Entity tool property panels for entity placement.

use crate::systems::game::map::format::EntityType;
use bevy_egui::egui;

/// Entity Place tool content
pub fn render_entity_place_content(ui: &mut egui::Ui, entity_type: &mut EntityType) {
    ui.group(|ui| {
        ui.label("Entity Type");
        ui.horizontal(|ui| {
            let icon = get_entity_icon(entity_type);
            ui.label(icon);

            egui::ComboBox::from_id_salt("entity_type_prop")
                .selected_text(format!("{:?}", entity_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(entity_type, EntityType::PlayerSpawn, "🟢 Player Spawn");
                    ui.selectable_value(entity_type, EntityType::Npc, "🔵 NPC");
                    ui.selectable_value(entity_type, EntityType::Enemy, "🔴 Enemy");
                    ui.selectable_value(entity_type, EntityType::Item, "🟡 Item");
                    ui.selectable_value(entity_type, EntityType::Trigger, "🟣 Trigger");
                    ui.selectable_value(entity_type, EntityType::LightSource, "💡 Light Source");
                });
        });
    });

    ui.add_space(8.0);

    // Entity type description
    ui.group(|ui| {
        let description = match entity_type {
            EntityType::PlayerSpawn => "Starting position for the player character.",
            EntityType::Npc => "Non-player character that can have dialog and interactions.",
            EntityType::Enemy => "Hostile entity that the player can fight.",
            EntityType::Item => "Collectible item or interactive object.",
            EntityType::Trigger => "Invisible trigger zone for events.",
            EntityType::LightSource => "Point light that illuminates in all directions.",
        };
        ui.small(description);
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Click to place entity");
        ui.small("• Edit properties in Outliner");
    });
}

/// Get icon for an entity type
pub fn get_entity_icon(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢",
        EntityType::Npc => "🔵",
        EntityType::Enemy => "🔴",
        EntityType::Item => "🟡",
        EntityType::Trigger => "🟣",
        EntityType::LightSource => "💡",
    }
}
