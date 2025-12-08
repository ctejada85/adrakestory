//! Map Editor for A Drake's Story
//!
//! A standalone GUI application for creating and editing map files.

use adrakestory::editor::tools::ActiveTransform;
use adrakestory::editor::ui::dialogs::MapDataChangedEvent;
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
    outliner_state: ResMut<'w, ui::OutlinerState>,
    dialog_receiver: ResMut<'w, ui::dialogs::FileDialogReceiver>,
}

/// Bundle of read-only editor state resources
#[derive(bevy::ecs::system::SystemParam)]
struct EditorReadResources<'w> {
    cursor_state: Res<'w, CursorState>,
    history: Res<'w, EditorHistory>,
    active_transform: Res<'w, ActiveTransform>,
    keyboard_mode: Res<'w, KeyboardEditMode>,
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
        .init_resource::<ui::OutlinerState>()
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
        .add_systems(Update, renderer::render_entities_system)
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
    read_resources: EditorReadResources,
    mut transform_events: TransformEvents,
    mut save_events: SaveEvents,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
    mut selection_events: EventWriter<tools::UpdateSelectionHighlights>,
    mut render_events: EventWriter<RenderMapEvent>,
) {
    let ctx = contexts.ctx_mut();

    // Render toolbar
    ui::render_toolbar(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.ui_state,
        &read_resources.history,
        &mut save_events.save,
        &mut save_events.save_as,
    );

    // Render outliner panel (left side)
    ui::render_outliner_panel(
        ctx,
        &mut ui_resources.editor_state,
        &mut ui_resources.outliner_state,
        &mut selection_events,
        &mut render_events,
    );

    // Render properties panel (right side)
    ui::render_properties_panel(
        ctx,
        &mut ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.active_transform,
        &mut transform_events,
    );

    // Render viewport overlays (keyboard mode indicator, selection tooltip, etc.)
    ui::render_viewport_overlays(
        ctx,
        &ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.keyboard_mode,
        &read_resources.active_transform,
    );

    // Render status bar
    render_status_bar(
        ctx,
        &ui_resources.editor_state,
        &read_resources.cursor_state,
        &read_resources.history,
        &read_resources.keyboard_mode,
        &read_resources.active_transform,
    );

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
    cursor_state: &CursorState,
    history: &EditorHistory,
    keyboard_mode: &KeyboardEditMode,
    active_transform: &ActiveTransform,
) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // === Section 1: Current Tool with Icon ===
            let (tool_icon, tool_name) = get_tool_display(&editor_state.active_tool);
            ui.label(format!("{} {}", tool_icon, tool_name));

            ui.separator();

            // === Section 2: Operation Status (if transform active) ===
            if active_transform.mode != tools::TransformMode::None {
                let mode_text = match active_transform.mode {
                    tools::TransformMode::Move => {
                        let offset = active_transform.current_offset;
                        format!(
                            "🔄 MOVING {} voxel{} │ Offset: ({}, {}, {})",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            },
                            offset.x,
                            offset.y,
                            offset.z
                        )
                    }
                    tools::TransformMode::Rotate => {
                        format!(
                            "↻ ROTATING {} voxel{} │ Axis: {:?} │ Angle: {}°",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            },
                            active_transform.rotation_axis,
                            active_transform.rotation_angle * 90
                        )
                    }
                    tools::TransformMode::None => String::new(),
                };
                ui.colored_label(egui::Color32::YELLOW, mode_text);
                ui.label("│ ENTER: confirm, ESC: cancel");
                ui.separator();
            }

            // === Section 3: Cursor Position ===
            if let Some(grid_pos) = cursor_state.grid_pos {
                ui.label(format!(
                    "Cursor: ({}, {}, {})",
                    grid_pos.0, grid_pos.1, grid_pos.2
                ));
            } else {
                ui.label("Cursor: --");
            }

            ui.separator();

            // === Section 4: Map Statistics ===
            ui.label(format!(
                "Voxels: {}",
                editor_state.current_map.world.voxels.len()
            ));
            ui.label(format!(
                "Entities: {}",
                editor_state.current_map.entities.len()
            ));

            ui.separator();

            // === Section 5: Selection Info ===
            let voxel_sel = editor_state.selected_voxels.len();
            let entity_sel = editor_state.selected_entities.len();
            if voxel_sel > 0 || entity_sel > 0 {
                ui.label(format!("Sel: {}v {}e", voxel_sel, entity_sel));
                ui.separator();
            }

            // === Section 6: Keyboard Mode Indicator ===
            if keyboard_mode.enabled {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "⌨ KEYBOARD");
                ui.separator();
            }

            // === Section 7: Modified Indicator ===
            if editor_state.is_modified {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "● Modified");
            } else {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✓ Saved");
            }

            // === Right-aligned: History Stats ===
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Undo: {} │ Redo: {}",
                    history.undo_count(),
                    history.redo_count()
                ));
            });
        });
    });
}

/// Get tool icon and display name
fn get_tool_display(tool: &state::EditorTool) -> (&'static str, &'static str) {
    match tool {
        state::EditorTool::Select => ("🔲", "Select"),
        state::EditorTool::VoxelPlace { .. } => ("✏️", "Voxel Place"),
        state::EditorTool::VoxelRemove => ("🗑️", "Voxel Remove"),
        state::EditorTool::EntityPlace { .. } => ("📍", "Entity Place"),
        state::EditorTool::Camera => ("📷", "Camera"),
    }
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
