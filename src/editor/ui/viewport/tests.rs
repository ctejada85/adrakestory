use super::*;
use crate::systems::game::map::format::EntityData;
use std::collections::HashMap;

// --- should_show_label tests ---

#[test]
fn named_npc_is_shown() {
    assert!(should_show_label("Alice", "NPC"));
}

#[test]
fn default_placeholder_is_suppressed() {
    assert!(!should_show_label("NPC", "NPC"));
}

#[test]
fn empty_name_is_suppressed() {
    assert!(!should_show_label("", "NPC"));
}

#[test]
fn lowercase_npc_is_shown() {
    // Filter is case-sensitive: "npc" != "NPC"
    assert!(should_show_label("npc", "NPC"));
}

#[test]
fn npc_with_trailing_space_is_shown() {
    // Only the exact string "NPC" is suppressed
    assert!(should_show_label("NPC ", "NPC"));
}

#[test]
fn enemy_default_is_suppressed() {
    assert!(!should_show_label("Enemy", "Enemy"));
}

#[test]
fn named_enemy_is_shown() {
    assert!(should_show_label("Goblin", "Enemy"));
}

#[test]
fn item_default_is_suppressed() {
    assert!(!should_show_label("Item", "Item"));
}

#[test]
fn named_item_is_shown() {
    assert!(should_show_label("Health Potion", "Item"));
}

#[test]
fn trigger_default_is_suppressed() {
    assert!(!should_show_label("Trigger", "Trigger"));
}

#[test]
fn named_trigger_is_shown() {
    assert!(should_show_label("Door Trigger", "Trigger"));
}

#[test]
fn light_source_default_is_suppressed() {
    assert!(!should_show_label("LightSource", "LightSource"));
}

#[test]
fn named_light_source_is_shown() {
    assert!(should_show_label("Torch", "LightSource"));
}

// --- entity_tooltip_lines tests ---

fn make_entity(entity_type: EntityType, props: &[(&str, &str)]) -> EntityData {
    EntityData {
        entity_type,
        position: (1.0, 2.0, 3.0),
        properties: props
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>(),
    }
}

#[test]
fn tooltip_common_lines_always_present() {
    let entity = make_entity(EntityType::Enemy, &[("name", "Goblin")]);
    let lines = entity_tooltip_lines(&entity, 5);
    assert_eq!(lines[0], "Name: Goblin");
    assert_eq!(lines[1], "Type: Enemy");
    assert_eq!(lines[2], "Position: (1.00, 2.00, 3.00)");
    assert_eq!(lines[3], "Index: 5");
}

#[test]
fn tooltip_npc_with_radius_appends_radius_line() {
    let entity = make_entity(EntityType::Npc, &[("name", "Alice"), ("radius", "2.5")]);
    let lines = entity_tooltip_lines(&entity, 0);
    assert!(lines.iter().any(|l| l == "Radius: 2.5"));
}

#[test]
fn tooltip_npc_without_radius_has_no_radius_line() {
    let entity = make_entity(EntityType::Npc, &[("name", "Alice")]);
    let lines = entity_tooltip_lines(&entity, 0);
    assert!(!lines.iter().any(|l| l.starts_with("Radius:")));
}

#[test]
fn tooltip_light_source_with_props_appends_them() {
    let entity = make_entity(
        EntityType::LightSource,
        &[
            ("name", "Torch"),
            ("intensity", "800"),
            ("range", "5.0"),
            ("color", "#ffaa00"),
            ("shadows", "true"),
        ],
    );
    let lines = entity_tooltip_lines(&entity, 1);
    assert!(lines.iter().any(|l| l == "intensity: 800"));
    assert!(lines.iter().any(|l| l == "range: 5.0"));
    assert!(lines.iter().any(|l| l == "color: #ffaa00"));
    assert!(lines.iter().any(|l| l == "shadows: true"));
}

#[test]
fn tooltip_light_source_without_props_has_only_common_lines() {
    let entity = make_entity(EntityType::LightSource, &[("name", "Torch")]);
    let lines = entity_tooltip_lines(&entity, 2);
    assert_eq!(lines.len(), 4);
}

#[test]
fn tooltip_missing_name_shows_empty_name() {
    let entity = make_entity(EntityType::Enemy, &[]);
    let lines = entity_tooltip_lines(&entity, 3);
    assert_eq!(lines[0], "Name: ");
}
