//! Selection highlight rendering.

use super::SelectionHighlight;
use crate::editor::state::EditorState;
use super::UpdateSelectionHighlights;
use bevy::prelude::*;

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
        spawn_selection_highlight(&mut commands, &highlight_mesh, &highlight_material, pos);
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
