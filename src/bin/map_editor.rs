//! Map Editor for A Drake's Story
//!
//! A standalone GUI application for creating and editing map files.

use adrakestory::editor::{camera, cursor, file_io, grid, renderer, state, tools, ui};
use adrakestory::editor::{EditorHistory, EditorState, MapRenderState, RenderMapEvent};
use adrakestory::editor::{FileSavedEvent, SaveFileDialogReceiver, SaveMapAsEvent, SaveMapEvent};
use adrakestory::editor::tools::ActiveTransform;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use grid::InfiniteGridConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Map Editor - A Drake's Story".to_string(),
                resolution: (1600.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<EditorState>()
        .init_resource::<EditorHistory>()
        .init_resource::<state::EditorUIState>()
        .init_resource::<camera::CameraInputState>()
        .init_resource::<ui::dialogs::FileDialogReceiver>()
        .init_resource::<SaveFileDialogReceiver>()
        .init_resource::<MapRenderState>()
        .init_resource::<InfiniteGridConfig>()
        .init_resource::<ActiveTransform>()
        .add_event::<ui::dialogs::FileSelectedEvent>()
        .add_event::<SaveMapEvent>()
        .add_event::<SaveMapAsEvent>()
        .add_event::<FileSavedEvent>()
        .add_event::<RenderMapEvent>()
        .add_event::<tools::UpdateSelectionHighlights>()
        // New unified input event
        .add_event::<tools::EditorInputEvent>()
        // Keep these events for UI button compatibility
        .add_event::<tools::DeleteSelectedVoxels>()
        .add_event::<tools::StartMoveOperation>()
        .add_event::<tools::StartRotateOperation>()
        .add_event::<tools::ConfirmTransform>()
        .add_event::<tools::CancelTransform>()
        .add_event::<tools::UpdateTransformPreview>()
        .add_event::<tools::UpdateRotation>()
        .add_event::<tools::SetRotationAxis>()
        .add_systems(Startup, setup_editor)
        .add_systems(Update, render_ui)
        .add_systems(Update, ui::dialogs::check_file_dialog_result)
        .add_systems(Update, ui::dialogs::handle_file_selected)
        .add_systems(Update, file_io::handle_save_map)
        .add_systems(Update, file_io::handle_save_map_as)
        .add_systems(Update, file_io::check_save_dialog_result)
        .add_systems(Update, file_io::handle_file_saved)
        .add_systems(Update, cursor::update_cursor_position)
        .add_systems(Update, renderer::detect_map_changes)
        .add_systems(Update, renderer::render_map_system)
        .add_systems(Update, camera::handle_camera_input)
        .add_systems(Update, camera::update_editor_camera)
        .add_systems(Update, grid::update_infinite_grid)
        .add_systems(Update, grid::update_grid_visibility)
        .add_systems(Update, grid::update_cursor_indicator)
        .add_systems(Update, tools::handle_voxel_placement)
        .add_systems(Update, tools::handle_voxel_removal)
        .add_systems(Update, tools::handle_entity_placement)
        .add_systems(Update, tools::handle_selection)
        // NEW: Unified input handling systems (replaces 15 old systems)
        .add_systems(Update, tools::handle_keyboard_input)
        .add_systems(Update, tools::handle_transformation_operations)
        // Keep rendering systems
        .add_systems(Update, tools::render_selection_highlights)
        .add_systems(Update, tools::render_transform_preview)
        .add_systems(Update, tools::render_rotation_preview)
        .run();
}

/// Setup the editor on startup
fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_config: Res<InfiniteGridConfig>,
) {
    info!("Starting Map Editor");

    // Spawn 3D camera for viewport
    let camera_pos = Vec3::new(10.0, 10.0, 10.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(Vec3::new(2.0, 0.0, 2.0), Vec3::Y),
        camera::EditorCamera::new(),
    ));

    // Spawn directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));

    // Spawn ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Spawn infinite grid
    grid::spawn_infinite_grid(
        &mut commands,
        &mut meshes,
        &mut materials,
        &grid_config,
        camera_pos,
    );

    // Spawn cursor indicator
    grid::spawn_cursor_indicator(&mut commands, &mut meshes, &mut materials);

    info!("Map editor setup complete");
}

/// Render the UI
fn render_ui(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut ui_state: ResMut<state::EditorUIState>,
    history: Res<EditorHistory>,
    active_transform: Res<ActiveTransform>,
    dialog_receiver: ResMut<ui::dialogs::FileDialogReceiver>,
    mut delete_events: EventWriter<tools::DeleteSelectedVoxels>,
    mut move_events: EventWriter<tools::StartMoveOperation>,
    mut rotate_events: EventWriter<tools::StartRotateOperation>,
    mut confirm_events: EventWriter<tools::ConfirmTransform>,
    mut cancel_events: EventWriter<tools::CancelTransform>,
    mut save_events: EventWriter<SaveMapEvent>,
    mut save_as_events: EventWriter<SaveMapAsEvent>,
) {
    let ctx = contexts.ctx_mut();

    // Render toolbar
    ui::render_toolbar(
        ctx,
        &mut editor_state,
        &mut ui_state,
        &history,
        &mut save_events,
        &mut save_as_events,
    );

    // Render properties panel
    ui::render_properties_panel(
        ctx,
        &mut editor_state,
        &active_transform,
        &mut delete_events,
        &mut move_events,
        &mut rotate_events,
        &mut confirm_events,
        &mut cancel_events,
    );

    // Render viewport controls
    ui::render_viewport_controls(ctx);

    // Render status bar
    render_status_bar(ctx, &editor_state, &history);

    // Render dialogs
    ui::dialogs::render_dialogs(ctx, &mut editor_state, &mut ui_state, &mut save_events);

    // Handle file operations
    ui::dialogs::handle_file_operations(&mut ui_state, dialog_receiver);
}

/// Render the status bar at the bottom
fn render_status_bar(ctx: &egui::Context, editor_state: &EditorState, history: &EditorHistory) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Current tool
            ui.label(format!("Tool: {}", editor_state.active_tool.name()));

            ui.separator();

            // Map stats
            ui.label(format!(
                "Voxels: {}",
                editor_state.current_map.world.voxels.len()
            ));
            ui.label(format!(
                "Entities: {}",
                editor_state.current_map.entities.len()
            ));

            ui.separator();

            // History stats
            ui.label(format!("Undo: {}", history.undo_count()));
            ui.label(format!("Redo: {}", history.redo_count()));

            ui.separator();

            // Modified indicator
            if editor_state.is_modified {
                ui.label("● Modified");
            } else {
                ui.label("Saved");
            }

            // Push remaining content to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Map: {}", editor_state.get_display_name()));
            });
        });
    });
}
