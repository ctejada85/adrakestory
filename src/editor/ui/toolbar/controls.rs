//! Helper UI components for the toolbar.

use crate::editor::play::{PlayMapEvent, PlayTestState, StopGameEvent};
use crate::editor::state::EditorState;
use bevy::prelude::*;
use bevy_egui::egui;

/// Render view toggle buttons
pub fn render_view_toggles(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Grid toggle
    let grid_icon = if editor_state.show_grid { "▦" } else { "▢" };
    let grid_text = format!("{} Grid", grid_icon);
    if ui
        .selectable_label(editor_state.show_grid, grid_text)
        .on_hover_text("Toggle grid (G)")
        .clicked()
    {
        editor_state.show_grid = !editor_state.show_grid;
        info!("Grid toggled: {}", editor_state.show_grid);
    }

    // Snap toggle
    let snap_icon = if editor_state.snap_to_grid {
        "⊞"
    } else {
        "⊟"
    };
    let snap_text = format!("{} Snap", snap_icon);
    if ui
        .selectable_label(editor_state.snap_to_grid, snap_text)
        .on_hover_text("Toggle snap to grid (Shift+G)")
        .clicked()
    {
        editor_state.snap_to_grid = !editor_state.snap_to_grid;
        info!("Snap toggled: {}", editor_state.snap_to_grid);
    }
}

/// Render play/stop buttons for map testing
pub fn render_play_controls(
    ui: &mut egui::Ui,
    play_state: &mut PlayTestState,
    play_events: &mut EventWriter<PlayMapEvent>,
    stop_events: &mut EventWriter<StopGameEvent>,
) {
    if play_state.is_running {
        // Show Stop button when game is running
        let stop_button = egui::Button::new("⏹ Stop")
            .fill(egui::Color32::from_rgb(180, 60, 60))
            .min_size(egui::vec2(65.0, 24.0));

        if ui
            .add(stop_button)
            .on_hover_text("Stop the running game (Shift+F5)")
            .clicked()
        {
            stop_events.send(StopGameEvent);
        }

        // Running indicator
        ui.label(
            egui::RichText::new("● Running")
                .color(egui::Color32::from_rgb(100, 200, 100))
                .small(),
        );

        // Hot reload indicator
        ui.separator();
        ui.label(
            egui::RichText::new("🔄 Hot Reload Active")
                .color(egui::Color32::from_rgb(100, 180, 100))
                .small(),
        )
        .on_hover_text("Map changes will automatically reload in the running game");
    } else {
        // Show Play button when game is not running
        let play_button = egui::Button::new("▶ Play")
            .fill(egui::Color32::from_rgb(60, 140, 60))
            .min_size(egui::vec2(65.0, 24.0));

        if ui
            .add(play_button)
            .on_hover_text("Test map in game (F5)")
            .clicked()
        {
            play_events.send(PlayMapEvent);
        }
    }
}
