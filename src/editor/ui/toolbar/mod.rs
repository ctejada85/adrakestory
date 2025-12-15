//! Toolbar UI with menu bar and quick actions.
//!
//! This module provides the top toolbar including:
//! - Menu bar (File, Edit, View, Run, Tools, Help)
//! - Tool selection buttons
//! - Context-sensitive tool options
//! - Play/test controls
//! - View toggles

mod controls;
mod menus;
mod tool_buttons;
mod tool_options;

pub use controls::{render_play_controls, render_view_toggles};
pub use menus::{
    render_edit_menu, render_file_menu, render_help_menu, render_run_menu, render_tools_menu,
    render_view_menu,
};
pub use tool_buttons::render_tool_buttons;
pub use tool_options::{entity_type_display, pattern_short_name, render_tool_options};

use crate::editor::file_io::{SaveMapAsEvent, SaveMapEvent};
use crate::editor::history::EditorHistory;
use crate::editor::play::{PlayMapEvent, PlayTestState, StopGameEvent};
use crate::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use crate::editor::shortcuts::{RedoEvent, UndoEvent};
use crate::editor::state::{EditorState, EditorUIState, ToolMemory};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render the top toolbar with menu bar and quick actions
#[allow(clippy::too_many_arguments)]
pub fn render_toolbar(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    tool_memory: &mut ToolMemory,
    history: &EditorHistory,
    recent_files: &mut RecentFiles,
    play_state: &mut PlayTestState,
    save_events: &mut EventWriter<SaveMapEvent>,
    save_as_events: &mut EventWriter<SaveMapAsEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
    play_events: &mut EventWriter<PlayMapEvent>,
    stop_events: &mut EventWriter<StopGameEvent>,
    undo_events: &mut EventWriter<UndoEvent>,
    redo_events: &mut EventWriter<RedoEvent>,
) {
    // Menu bar panel
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            render_file_menu(
                ui,
                editor_state,
                ui_state,
                recent_files,
                save_events,
                save_as_events,
                open_recent_events,
            );
            render_edit_menu(ui, history, undo_events, redo_events);
            render_view_menu(ui, editor_state);
            render_run_menu(ui, play_state, play_events, stop_events);
            render_tools_menu(ui, editor_state, tool_memory);
            render_help_menu(ui, ui_state);

            // Spacer to push map name to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let map_name = &editor_state.current_map.metadata.name;
                let modified = if editor_state.is_modified { " *" } else { "" };
                ui.label(format!("{}{}", map_name, modified));
            });
        });
    });

    // Horizontal toolbar panel (below menu bar)
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // === Tool Buttons ===
            render_tool_buttons(ui, editor_state, tool_memory);

            ui.separator();

            // === Context-Sensitive Options ===
            render_tool_options(ui, editor_state, tool_memory);

            ui.separator();

            // === Play/Test Controls ===
            render_play_controls(ui, play_state, play_events, stop_events);

            ui.separator();

            // === View Toggles ===
            render_view_toggles(ui, editor_state);
        });
    });
}
