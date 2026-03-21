//! Window close and app exit handling.

use crate::editor::play::PlayTestState;
use crate::editor::state::{EditorState, EditorUIState, PendingAction};
use bevy::prelude::*;

use super::events::AppExitEvent;

/// System to intercept window close requests and prompt for unsaved changes
pub fn handle_window_close_request(
    mut window_close_events: MessageReader<bevy::window::WindowCloseRequested>,
    editor_state: Res<EditorState>,
    mut ui_state: ResMut<EditorUIState>,
    mut exit_events: MessageWriter<AppExitEvent>,
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
            exit_events.write(AppExitEvent);
        }
    }
}

/// System to handle the actual app exit
pub fn handle_app_exit(
    mut exit_events: MessageReader<AppExitEvent>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
    mut play_state: ResMut<PlayTestState>,
) {
    for _ in exit_events.read() {
        // Stop any running game before exiting
        if play_state.is_running {
            info!("Stopping running game before exit");
            play_state.stop_game();
        }

        info!("Application exit requested");
        app_exit.write(bevy::app::AppExit::Success);
    }
}
