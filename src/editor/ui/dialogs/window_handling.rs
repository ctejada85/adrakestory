//! Window close and app exit handling.

use crate::editor::play::PlayTestState;
use crate::editor::state::{EditorState, EditorUIState, PendingAction};
use bevy::prelude::*;

use super::events::AppExitEvent;

/// System to intercept window close requests and prompt for unsaved changes
pub fn handle_window_close_request(
    mut window_close_events: EventReader<bevy::window::WindowCloseRequested>,
    editor_state: Res<EditorState>,
    mut ui_state: ResMut<EditorUIState>,
    mut exit_events: EventWriter<AppExitEvent>,
) {
    for _event in window_close_events.read() {
        // Check if there are unsaved changes
        if editor_state.is_modified {
            info!("Window close requested with unsaved changes");

            // Show unsaved changes dialog
            ui_state.unsaved_changes_dialog_open = true;
            ui_state.pending_action = Some(PendingAction::Quit);
        } else {
            // No unsaved changes, just quit
            exit_events.send(AppExitEvent);
        }
    }
}

/// System to handle the actual app exit
pub fn handle_app_exit(
    mut exit_events: EventReader<AppExitEvent>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut play_state: ResMut<PlayTestState>,
) {
    for _ in exit_events.read() {
        // Stop any running game before exiting
        if play_state.is_running {
            info!("Stopping running game before exit");
            play_state.stop_game();
        }

        info!("Application exit requested");
        app_exit.send(bevy::app::AppExit::Success);
    }
}
