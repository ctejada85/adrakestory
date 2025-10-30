//! Map Editor for A Drake's Story
//!
//! A standalone GUI application for creating and editing map files.

use adrakestory::editor::tools::ActiveTransform;
use adrakestory::editor::ui::properties::TransformEvents;
use adrakestory::editor::{camera, cursor, file_io, grid, renderer, state, tools, ui};
use adrakestory::editor::{
    handle_keyboard_cursor_movement, handle_keyboard_selection, handle_tool_switching,
    toggle_keyboard_edit_mode,
};
use adrakestory::editor::{
    CursorState, EditorHistory, EditorState, KeyboardEditMode, MapRenderState, RenderMapEvent,
};
use adrakestory::editor::{FileSavedEvent, SaveFileDialogReceiver, SaveMapAsEvent, SaveMapEvent};
use adrakestory::editor::ui::dialogs::MapDataChangedEvent;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use grid::InfiniteGridConfig;

/// Bundle of event writers for save operations
#[derive(bevy::ecs::system::SystemParam)]
struct SaveEvents<'w> {
    save: EventWriter<'w, SaveMapEvent>,
    save_as: EventWriter<'w, SaveMapAsEvent>,
}

/// Bundle of UI-related resources
#[derive(bevy::ecs::system::SystemParam)]
struct UIResources<'w> {
    editor_state: ResMut<'w, EditorState>,
    ui_state: ResMut<'w, state::EditorUIState>,
    dialog_receiver: ResMut<'w, ui::dialogs::FileDialogReceiver>,
}

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
        .init_resource::<CursorState>()
        .init_resource::<EditorHistory>()
        .init_resource::<state::EditorUIState>()
        .init_resource::<camera::CameraInputState>()
        .init_resource::<ui::dialogs::FileDialogReceiver>()
        .init_resource::<SaveFileDialogReceiver>()
        .init_resource::<MapRenderState>()
        .init_resource::<InfiniteGridConfig>()
        .init_resource::<ActiveTransform>()
        .init_resource::<KeyboardEditMode>()
        .add_event::<ui::dialogs::FileSelectedEvent>()
        .add_event::<SaveMapEvent>()
        .add_event::<SaveMapAsEvent>()
        .add_event::<FileSavedEvent>()
        .add_event::<RenderMapEvent>()
        .add_event::<MapDataChangedEvent>()
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
        .add_systems(Update, update_lighting_on_map_change)
        .add_systems(Update, render_ui)
        .add_systems(Update, ui::dialogs::check_file_dialog_result)
        .add_systems(Update, ui::dialogs::handle_file_selected)
        .add_systems(Update, file_io::handle_save_map)
        .add_systems(Update, file_io::handle_save_map_as)
        .add_systems(Update, file_io::check_save_dialog_result)
        .add_systems(Update, file_io::handle_file_saved)
        .add_systems(
            Update,
            (
                toggle_keyboard_edit_mode,
                handle_tool_switching,
                cursor::update_cursor_position,
                handle_keyboard_cursor_movement.after(cursor::update_cursor_position),
                handle_keyboard_selection,
            ),
        )
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
    editor_state: Res<EditorState>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
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

    // Get lighting configuration from the current map
    let lighting = &editor_state.current_map.lighting;

    // Spawn directional light using map configuration with high-quality shadows
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let cascade_shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            minimum_distance: 0.1,
            overlap_proportion: 0.3,
        }
        .build();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));

        info!("Spawned directional light with high-quality shadows: illuminance={}, color={:?}, direction={:?}",
              dir_light.illuminance, dir_light.color, dir_light.direction);
    }

    // Spawn ambient light using map configuration
    // Convert 0.0-1.0 intensity to brightness (scale by 1000 for Bevy's lighting system)
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: ambient_brightness,
    });

    info!(
        "Spawned ambient light with brightness: {}",
        ambient_brightness
    );

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

    // Send event to trigger initial lighting setup
    map_changed_events.send(MapDataChangedEvent);

    info!("Map editor setup complete");
}

/// Render the UI
fn render_ui(
    mut contexts: EguiContexts,
    mut ui_resources: UIResources,
    cursor_state: Res<CursorState>,
    history: Res<EditorHistory>,
    active_transform: Res<ActiveTransform>,
    keyboard_mode: Res<KeyboardEditMode>,
    mut transform_events: TransformEvents,
    mut save_events: SaveEvents,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    let ctx = contexts.ctx_mut();

    // Render toolbar
    ui::render_toolbar(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.ui_state,
        &history,
        &mut save_events.save,
        &mut save_events.save_as,
    );

    // Render properties panel
    ui::render_properties_panel(
        ctx,
        &mut ui_resources.editor_state,
        &cursor_state,
        &active_transform,
        &mut transform_events,
    );

    // Render viewport controls
    ui::render_viewport_controls(ctx);

    // Render status bar
    render_status_bar(ctx, &ui_resources.editor_state, &history, &keyboard_mode);

    // Render dialogs
    ui::dialogs::render_dialogs(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.ui_state,
        &mut save_events.save,
        &mut map_changed_events,
    );

    // Handle file operations
    ui::dialogs::handle_file_operations(&mut ui_resources.ui_state, ui_resources.dialog_receiver);
}

/// Render the status bar at the bottom
fn render_status_bar(
    ctx: &egui::Context,
    editor_state: &EditorState,
    history: &EditorHistory,
    keyboard_mode: &KeyboardEditMode,
) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Keyboard edit mode indicator
            if keyboard_mode.enabled {
                ui.colored_label(egui::Color32::GREEN, "⌨ KEYBOARD MODE");
                ui.separator();
            }

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

/// System to update lighting when the map changes (e.g., after loading a new map)
fn update_lighting_on_map_change(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut ambient_light: ResMut<AmbientLight>,
    directional_lights: Query<Entity, With<DirectionalLight>>,
    mut map_changed_events: EventReader<MapDataChangedEvent>,
) {
    // Only update if we received a map data changed event
    if map_changed_events.read().next().is_none() {
        return;
    }

    let lighting = &editor_state.current_map.lighting;

    // Update ambient light
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    ambient_light.brightness = ambient_brightness;

    info!(
        "Updated ambient light brightness to: {}",
        ambient_brightness
    );

    // Remove existing directional lights
    for entity in directional_lights.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new directional light with map configuration and high-quality shadows
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let cascade_shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            minimum_distance: 0.1,
            overlap_proportion: 0.3,
        }
        .build();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));

        info!("Updated directional light with high-quality shadows: illuminance={}, color={:?}, direction={:?}",
              dir_light.illuminance, dir_light.color, dir_light.direction);
    }
}
