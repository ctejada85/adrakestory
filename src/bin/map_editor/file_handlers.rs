//! File handling systems for the map editor.

use adrakestory::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use adrakestory::editor::state;
use adrakestory::editor::ui::dialogs::MapDataChangedEvent;
use adrakestory::editor::EditorState;
use bevy::prelude::*;

/// System to handle opening a recent file
pub fn handle_open_recent_file(
    mut events: EventReader<OpenRecentFileEvent>,
    mut editor_state: ResMut<EditorState>,
    mut ui_state: ResMut<state::EditorUIState>,
    mut recent_files: ResMut<RecentFiles>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    for event in events.read() {
        info!("Opening recent file: {:?}", event.path);

        // Try to load the map
        match std::fs::read_to_string(&event.path) {
            Ok(contents) => {
                match ron::from_str::<adrakestory::systems::game::map::format::MapData>(&contents) {
                    Ok(map_data) => {
                        info!("Successfully loaded map from: {:?}", event.path);
                        editor_state.current_map = map_data;
                        editor_state.file_path = Some(event.path.clone());
                        editor_state.clear_modified();
                        editor_state.clear_selections();

                        // Update recent files (moves to front)
                        recent_files.add(event.path.clone());

                        // Send event to trigger lighting update
                        map_changed_events.send(MapDataChangedEvent);
                    }
                    Err(e) => {
                        error!("Failed to parse map file: {}", e);
                        ui_state.error_message = format!("Failed to parse map file:\n{}", e);
                        ui_state.error_dialog_open = true;
                    }
                }
            }
            Err(e) => {
                error!("Failed to read file: {}", e);
                ui_state.error_message = format!("Failed to read file:\n{}", e);
                ui_state.error_dialog_open = true;
            }
        }
    }
}
