//! Map Editor for A Drake's Story
//!
//! A standalone GUI application for creating and editing map files.

use adrakestory::editor::{camera, grid, history, state, tools, ui};
use adrakestory::editor::{EditorHistory, EditorState};
use adrakestory::systems::game::map::format::MapData;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

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
        .add_systems(Startup, setup_editor)
        .add_systems(Update, render_ui)
        .add_systems(Update, camera::handle_camera_input)
        .add_systems(Update, camera::update_editor_camera)
        .add_systems(Update, grid::update_grid_visibility)
        .add_systems(Update, grid::update_cursor_indicator)
        .add_systems(Update, tools::handle_voxel_placement)
        .add_systems(Update, tools::handle_voxel_removal)
        .add_systems(Update, tools::handle_entity_placement)
        .add_systems(Update, tools::handle_selection)
        .run();
}

/// Setup the editor on startup
fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_state: Res<EditorState>,
) {
    info!("Starting Map Editor");

    // Spawn 3D camera for viewport
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::new(2.0, 0.0, 2.0), Vec3::Y),
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

    // Spawn grid
    let world = &editor_state.current_map.world;
    grid::spawn_grid(
        &mut commands,
        &mut meshes,
        &mut materials,
        world.width,
        world.height,
        world.depth,
        editor_state.grid_opacity,
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
) {
    let ctx = contexts.ctx_mut();

    // Render toolbar
    ui::render_toolbar(ctx, &mut editor_state, &mut ui_state, &history);

    // Render properties panel
    ui::render_properties_panel(ctx, &mut editor_state);

    // Render viewport controls
    ui::render_viewport_controls(ctx);

    // Render status bar
    render_status_bar(ctx, &editor_state, &history);

    // Render dialogs
    ui::dialogs::render_dialogs(ctx, &mut editor_state, &mut ui_state);

    // Handle file operations
    ui::dialogs::handle_file_operations(&mut editor_state, &mut ui_state);
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
