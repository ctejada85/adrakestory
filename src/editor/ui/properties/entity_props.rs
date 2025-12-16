//! Entity-specific property editing panels.

use crate::editor::state::EditorState;
use crate::systems::game::map::format::EntityType;
use super::entity_tools::get_entity_icon;
use bevy_egui::egui;

/// Render properties for a single selected entity
pub fn render_single_entity_properties(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    let index = match editor_state.selected_entities.iter().next() {
        Some(&idx) => idx,
        None => return,
    };

    if index >= editor_state.current_map.entities.len() {
        ui.label("Invalid entity");
        return;
    }

    let entity_type = editor_state.current_map.entities[index].entity_type;
    let icon = get_entity_icon(&entity_type);

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(icon);
            ui.strong(format!("{:?}", entity_type));
        });
    });

    ui.add_space(8.0);

    // Position
    ui.group(|ui| {
        ui.label("Position");

        let mut position_changed = false;
        let (mut x, mut y, mut z) = editor_state.current_map.entities[index].position;

        ui.horizontal(|ui| {
            ui.label("X:");
            if ui
                .add(egui::DragValue::new(&mut x).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Y:");
            if ui
                .add(egui::DragValue::new(&mut y).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Z:");
            if ui
                .add(egui::DragValue::new(&mut z).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });

        if position_changed {
            editor_state.current_map.entities[index].position = (x, y, z);
            editor_state.mark_modified();
        }
    });

    // Entity-specific properties
    if entity_type == EntityType::Npc {
        ui.add_space(8.0);
        render_npc_properties(ui, editor_state, index);
    } else if entity_type == EntityType::LightSource {
        ui.add_space(8.0);
        render_light_source_properties(ui, editor_state, index);
    }

    ui.add_space(8.0);

    // Actions
    ui.horizontal(|ui| {
        if ui.button("🗑 Delete").clicked() {
            editor_state.current_map.entities.remove(index);
            editor_state.selected_entities.clear();
            editor_state.mark_modified();
        }
        if ui.button("Clear").clicked() {
            editor_state.selected_entities.clear();
        }
    });
}

/// Render NPC-specific properties
fn render_npc_properties(ui: &mut egui::Ui, editor_state: &mut EditorState, index: usize) {
    ui.group(|ui| {
        ui.label("NPC Properties");

        // Name
        let current_name = editor_state.current_map.entities[index]
            .properties
            .get("name")
            .cloned()
            .unwrap_or_else(|| "NPC".to_string());
        let mut name = current_name;

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut name).changed() {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("name".to_string(), name);
                editor_state.mark_modified();
            }
        });

        // Radius
        let current_radius: f32 = editor_state.current_map.entities[index]
            .properties
            .get("radius")
            .and_then(|r| r.parse().ok())
            .unwrap_or(0.3);
        let mut radius = current_radius;

        ui.horizontal(|ui| {
            ui.label("Radius:");
            if ui
                .add(egui::Slider::new(&mut radius, 0.1..=1.0).step_by(0.05))
                .changed()
            {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("radius".to_string(), format!("{:.2}", radius));
                editor_state.mark_modified();
            }
        });
    });
}

/// Render LightSource-specific properties
fn render_light_source_properties(ui: &mut egui::Ui, editor_state: &mut EditorState, index: usize) {
    ui.group(|ui| {
        ui.label("Light Properties");

        // Intensity
        let current_intensity: f32 = editor_state.current_map.entities[index]
            .properties
            .get("intensity")
            .and_then(|i| i.parse().ok())
            .unwrap_or(1000.0);
        let mut intensity = current_intensity;

        ui.horizontal(|ui| {
            ui.label("Intensity:");
            if ui
                .add(egui::Slider::new(&mut intensity, 0.0..=100000.0).logarithmic(true))
                .changed()
            {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("intensity".to_string(), format!("{:.0}", intensity));
                editor_state.mark_modified();
            }
        });

        // Range
        let current_range: f32 = editor_state.current_map.entities[index]
            .properties
            .get("range")
            .and_then(|r| r.parse().ok())
            .unwrap_or(10.0);
        let mut range = current_range;

        ui.horizontal(|ui| {
            ui.label("Range:");
            if ui
                .add(egui::Slider::new(&mut range, 0.1..=100.0).step_by(0.5))
                .changed()
            {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("range".to_string(), format!("{:.1}", range));
                editor_state.mark_modified();
            }
        });

        // Shadows
        let current_shadows = editor_state.current_map.entities[index]
            .properties
            .get("shadows")
            .map(|s| s == "true")
            .unwrap_or(false);
        let mut shadows = current_shadows;

        if ui.checkbox(&mut shadows, "Cast Shadows").changed() {
            editor_state.current_map.entities[index]
                .properties
                .insert("shadows".to_string(), shadows.to_string());
            editor_state.mark_modified();
        }

        // Color (RGB sliders)
        ui.add_space(4.0);
        ui.label("Color:");

        let color_str = editor_state.current_map.entities[index]
            .properties
            .get("color")
            .cloned()
            .unwrap_or_else(|| "1.0,1.0,1.0".to_string());

        let parts: Vec<f32> = color_str
            .split(',')
            .filter_map(|p| p.trim().parse().ok())
            .collect();

        let (mut r, mut g, mut b) = if parts.len() == 3 {
            (parts[0], parts[1], parts[2])
        } else {
            (1.0, 1.0, 1.0)
        };

        let mut color_changed = false;

        ui.horizontal(|ui| {
            ui.label("R:");
            if ui
                .add(egui::Slider::new(&mut r, 0.0..=1.0).step_by(0.01))
                .changed()
            {
                color_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("G:");
            if ui
                .add(egui::Slider::new(&mut g, 0.0..=1.0).step_by(0.01))
                .changed()
            {
                color_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("B:");
            if ui
                .add(egui::Slider::new(&mut b, 0.0..=1.0).step_by(0.01))
                .changed()
            {
                color_changed = true;
            }
        });

        if color_changed {
            editor_state.current_map.entities[index]
                .properties
                .insert("color".to_string(), format!("{:.2},{:.2},{:.2}", r, g, b));
            editor_state.mark_modified();
        }

        // Color preview
        let preview_color = egui::Color32::from_rgb(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        );
        let (rect, _) = ui.allocate_exact_size(egui::vec2(60.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 2.0, preview_color);
    });
}
