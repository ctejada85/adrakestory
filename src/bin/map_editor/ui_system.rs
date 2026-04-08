//! UI rendering system.

use super::status_bar::render_status_bar;
use adrakestory::editor::play::{PlayMapEvent, PlayTestState, StopGameEvent};
use adrakestory::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use adrakestory::editor::tools::ActiveTransform;
use adrakestory::editor::ui::dialogs::{AppExitEvent, MapDataChangedEvent};
use adrakestory::editor::ui::properties::TransformEvents;
use adrakestory::editor::{state, tools, ui};
use adrakestory::editor::{
    CursorState, EditorHistory, EditorState, KeyboardEditMode, RedoEvent, RenderMapEvent, UndoEvent,
};
use adrakestory::editor::{SaveMapAsEvent, SaveMapEvent};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Bundle of event writers for save operations
#[derive(bevy::ecs::system::SystemParam)]
pub struct SaveEvents<'w> {
    pub save: MessageWriter<'w, SaveMapEvent>,
    pub save_as: MessageWriter<'w, SaveMapAsEvent>,
}

/// Bundle of event writers for UI operations
#[derive(bevy::ecs::system::SystemParam)]
pub struct UIEventWriters<'w> {
    pub map_changed: MessageWriter<'w, MapDataChangedEvent>,
    pub selection: MessageWriter<'w, tools::UpdateSelectionHighlights>,
    pub render: MessageWriter<'w, RenderMapEvent>,
    pub exit: MessageWriter<'w, AppExitEvent>,
    pub open_recent: MessageWriter<'w, OpenRecentFileEvent>,
    pub play: MessageWriter<'w, PlayMapEvent>,
    pub stop: MessageWriter<'w, StopGameEvent>,
    pub undo: MessageWriter<'w, UndoEvent>,
    pub redo: MessageWriter<'w, RedoEvent>,
}

/// Bundle of UI-related resources
#[derive(bevy::ecs::system::SystemParam)]
pub struct UIResources<'w> {
    pub editor_state: ResMut<'w, EditorState>,
    pub ui_state: ResMut<'w, state::EditorUIState>,
    pub tool_memory: ResMut<'w, state::ToolMemory>,
    pub outliner_state: ResMut<'w, ui::OutlinerState>,
    pub recent_files: ResMut<'w, RecentFiles>,
    pub dialog_receiver: ResMut<'w, ui::dialogs::FileDialogReceiver>,
    pub play_state: ResMut<'w, PlayTestState>,
}

/// Bundle of read-only editor state resources
#[derive(bevy::ecs::system::SystemParam)]
pub struct EditorReadResources<'w> {
    pub cursor_state: Res<'w, CursorState>,
    pub history: ResMut<'w, EditorHistory>,
    pub active_transform: Res<'w, ActiveTransform>,
    pub keyboard_mode: Res<'w, KeyboardEditMode>,
}

/// Render the UI
pub fn render_ui(
    mut contexts: EguiContexts,
    mut ui_resources: UIResources,
    mut read_resources: EditorReadResources,
    mut transform_events: TransformEvents,
    mut save_events: SaveEvents,
    mut ui_events: UIEventWriters,
) {
    let ctx = contexts.ctx_mut().expect("egui context");

    // Render toolbar
    ui::render_toolbar(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.ui_state,
        &mut ui_resources.tool_memory,
        &read_resources.history,
        &mut ui_resources.recent_files,
        &mut ui_resources.play_state,
        &mut save_events.save,
        &mut save_events.save_as,
        &mut ui_events.open_recent,
        &mut ui_events.play,
        &mut ui_events.stop,
        &mut ui_events.undo,
        &mut ui_events.redo,
    );

    // Render status bar (before side panels and overlays so its height is known)
    render_status_bar(
        ctx,
        &ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.history,
        &read_resources.keyboard_mode,
        &read_resources.active_transform,
    );

    // Render outliner panel (left side)
    ui::render_outliner_panel(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.outliner_state,
        &mut read_resources.history,
        &mut ui_events.selection,
        &mut ui_events.render,
    );

    // Render properties panel (right side)
    ui::render_properties_panel(
        ctx,
        &mut ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.active_transform,
        &mut transform_events,
        &mut read_resources.history,
    );

    // Render viewport overlays (keyboard mode indicator, selection tooltip, etc.)
    ui::render_viewport_overlays(
        ctx,
        &ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.keyboard_mode,
        &read_resources.active_transform,
    );

    // Render dialogs
    ui::dialogs::render_dialogs(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.ui_state,
        &mut save_events.save,
        &mut ui_events.map_changed,
        &mut ui_events.exit,
        &mut ui_events.open_recent,
    );

    // Handle file operations
    ui::dialogs::handle_file_operations(&mut ui_resources.ui_state, ui_resources.dialog_receiver);
}
