//! Selection handling for voxels and entities.

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool};
use super::{DragSelectState, UpdateSelectionHighlights, ViewportRaycast};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Handle selection when the tool is active
pub fn handle_selection(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    mut drag_state: ResMut<DragSelectState>,
    viewport: ViewportRaycast,
) {
    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if pointer is over any UI area (panels, buttons, backgrounds, etc.)
    // Also check is_using_pointer() for active interactions like dragging resize handles
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    // Handle mouse release - stop drag selection and handle toggle
    if mouse_button.just_released(MouseButton::Left) {
        if drag_state.is_dragging {
            // If we didn't move during drag and the voxel was already selected, deselect it
            if !drag_state.did_drag_move && drag_state.start_was_selected {
                if let Some(start_pos) = drag_state.start_grid_pos {
                    editor_state.selected_voxels.remove(&start_pos);
                    info!("Deselected voxel at {:?}", start_pos);
                    update_events.send(UpdateSelectionHighlights);
                }
            }

            // Reset drag state
            drag_state.is_dragging = false;
            drag_state.last_grid_pos = None;
            drag_state.start_grid_pos = None;
            drag_state.did_drag_move = false;
            drag_state.start_was_selected = false;
        }
        return;
    }

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get mouse ray for entity selection
    let Ok((camera, camera_transform)) = viewport.camera.get_single() else {
        return;
    };
    let Ok(window) = viewport.window.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // First, try to select an entity using ray-sphere intersection
    let entity_selection_radius = 0.5; // Radius for entity "hitbox"
    let mut closest_entity_index: Option<usize> = None;
    let mut closest_distance = f32::MAX;

    for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
        let (ex, ey, ez) = entity_data.position;
        let entity_pos = Vec3::new(ex, ey, ez);

        // Ray-sphere intersection test
        if let Some(distance) = ray_sphere_intersection(&ray, entity_pos, entity_selection_radius) {
            if distance < closest_distance {
                closest_distance = distance;
                closest_entity_index = Some(index);
            }
        }
    }

    // If we found an entity, select/deselect it (no drag for entities)
    if let Some(entity_idx) = closest_entity_index {
        // Clear voxel selection when selecting entities
        editor_state.selected_voxels.clear();

        if editor_state.selected_entities.contains(&entity_idx) {
            editor_state.selected_entities.remove(&entity_idx);
            info!("Deselected entity at index {}", entity_idx);
        } else {
            editor_state.selected_entities.clear(); // Single selection for now
            editor_state.selected_entities.insert(entity_idx);
            info!("Selected entity at index {}", entity_idx);
        }

        // Trigger highlight update
        update_events.send(UpdateSelectionHighlights);
        return;
    }

    // If no entity was clicked, try voxel selection
    // Get cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Clear entity selection when selecting voxels
    editor_state.selected_entities.clear();

    // Check if this voxel was already selected (for toggle on release)
    let was_selected = editor_state.selected_voxels.contains(&grid_pos);

    // Start drag-select mode
    drag_state.is_dragging = true;
    drag_state.last_grid_pos = Some(grid_pos);
    drag_state.start_grid_pos = Some(grid_pos);
    drag_state.did_drag_move = false;
    drag_state.start_was_selected = was_selected;

    // Add voxel to selection (we'll handle deselect on release if no drag occurred)
    if !was_selected {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?}", grid_pos);
    }

    // Trigger highlight update
    update_events.send(UpdateSelectionHighlights);
}

/// Handle continuous drag selection while mouse is held
pub fn handle_drag_selection(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    mut drag_state: ResMut<DragSelectState>,
) {
    // Only process if we're in drag-select mode
    if !drag_state.is_dragging {
        return;
    }

    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.start_grid_pos = None;
        drag_state.did_drag_move = false;
        drag_state.start_was_selected = false;
        return;
    }

    // Stop drag if mouse is released
    if !mouse_button.pressed(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.start_grid_pos = None;
        drag_state.did_drag_move = false;
        drag_state.start_was_selected = false;
        return;
    }

    // Check if pointer is over any UI area
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Only process if this is a different position than last time
    if drag_state.last_grid_pos == Some(grid_pos) {
        return;
    }

    // Mark that we've moved to a different voxel
    drag_state.did_drag_move = true;

    // Update last position
    drag_state.last_grid_pos = Some(grid_pos);

    // Add voxel to selection if not already selected
    if !editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Drag-selected voxel at {:?}", grid_pos);

        // Trigger highlight update
        update_events.send(UpdateSelectionHighlights);
    }
}

/// Ray-sphere intersection test
/// Returns the distance to the intersection point if there is one
fn ray_sphere_intersection(ray: &Ray3d, center: Vec3, radius: f32) -> Option<f32> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(*ray.direction);
    let b = 2.0 * oc.dot(*ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);

    // Return the closest positive intersection
    if t1 > 0.0 {
        Some(t1)
    } else if t2 > 0.0 {
        Some(t2)
    } else {
        None
    }
}
