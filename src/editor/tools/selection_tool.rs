//! Selection tool for selecting and manipulating objects.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::{EditorState, EditorTool};
use bevy::prelude::*;

/// Marker component for selection highlight visuals
#[derive(Component)]
pub struct SelectionHighlight {
    pub voxel_pos: (i32, i32, i32),
}

/// Event to trigger selection highlight update
#[derive(Event)]
pub struct UpdateSelectionHighlights;

/// Event to trigger deletion of selected voxels
#[derive(Event)]
pub struct DeleteSelectedVoxels;

/// Handle selection when the tool is active
pub fn handle_selection(
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
) {
    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = editor_state.cursor_grid_pos else {
        return;
    };

    // Toggle selection of voxel at this position
    if editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.remove(&grid_pos);
        info!("Deselected voxel at {:?}", grid_pos);
    } else {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?}", grid_pos);
    }

    // Trigger highlight update
    update_events.send(UpdateSelectionHighlights);
}

/// Render selection highlights for selected voxels
pub fn render_selection_highlights(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_highlights: Query<Entity, With<SelectionHighlight>>,
    mut update_events: EventReader<UpdateSelectionHighlights>,
) {
    // Only update if event received
    if update_events.read().count() == 0 {
        return;
    }

    // Despawn existing highlights
    for entity in existing_highlights.iter() {
        commands.entity(entity).despawn();
    }

    // Don't render if no selection
    if editor_state.selected_voxels.is_empty() {
        return;
    }

    // Create highlight material (bright yellow with emission)
    let highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.0, 0.6),
        emissive: LinearRgba::new(1.0, 1.0, 0.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Create wireframe cube mesh (slightly larger than voxel)
    let highlight_mesh = meshes.add(Cuboid::new(1.05, 1.05, 1.05));

    // Spawn highlight for each selected voxel
    for &pos in &editor_state.selected_voxels {
        spawn_selection_highlight(
            &mut commands,
            &highlight_mesh,
            &highlight_material,
            pos,
        );
    }

    info!(
        "Rendered {} selection highlights",
        editor_state.selected_voxels.len()
    );
}

/// Spawn a single selection highlight at the given position
fn spawn_selection_highlight(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    pos: (i32, i32, i32),
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(pos.0 as f32, pos.1 as f32, pos.2 as f32),
        SelectionHighlight { voxel_pos: pos },
    ));
}

/// Handle deletion of selected voxels via Delete/Backspace keys
pub fn handle_delete_selected(
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    mut delete_events: EventReader<DeleteSelectedVoxels>,
) {
    // Only active when Select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check for Delete or Backspace key, or delete event from UI
    let should_delete = keyboard.just_pressed(KeyCode::Delete)
        || keyboard.just_pressed(KeyCode::Backspace)
        || delete_events.read().count() > 0;

    if !should_delete {
        return;
    }

    // Nothing to delete
    if editor_state.selected_voxels.is_empty() {
        info!("No voxels selected to delete");
        return;
    }

    // Create batch action for undo/redo
    let mut actions = Vec::new();
    let selected_count = editor_state.selected_voxels.len();

    // Collect selected positions to avoid borrow checker issues
    let selected_positions: Vec<(i32, i32, i32)> =
        editor_state.selected_voxels.iter().copied().collect();

    // Find and remove each selected voxel
    for pos in selected_positions {
        if let Some(index) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .position(|v| v.pos == pos)
        {
            let voxel_data = editor_state.current_map.world.voxels[index].clone();
            actions.push(EditorAction::RemoveVoxel {
                pos,
                data: voxel_data,
            });
            editor_state.current_map.world.voxels.remove(index);
        }
    }

    // Push batch action to history
    if !actions.is_empty() {
        history.push(EditorAction::Batch {
            description: format!("Delete {} voxel{}", actions.len(), if actions.len() == 1 { "" } else { "s" }),
            actions,
        });

        editor_state.mark_modified();
        info!("Deleted {} selected voxels", selected_count);
    }

    // Clear selection
    editor_state.selected_voxels.clear();

    // Trigger re-render
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}
