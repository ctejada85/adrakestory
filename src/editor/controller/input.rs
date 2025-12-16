//! Controller input handling for editing operations.
//!
//! Handles trigger presses for placement/removal, hotbar cycling,
//! and palette navigation.

use crate::editor::controller::camera::ControllerCameraMode;
use crate::editor::controller::cursor::ControllerCursor;
use crate::editor::controller::hotbar::{default_hotbar, HotbarItem, PaletteCategory};
use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::state::EditorState;
use crate::editor::MapRenderState;
use crate::systems::game::map::format::{EntityData, SubVoxelPattern, VoxelData};
use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource managing the controller editing mode state.
#[derive(Resource)]
pub struct ControllerEditMode {
    /// Whether controller edit mode is enabled
    pub enabled: bool,
    /// Current hotbar slot (0-8)
    pub hotbar_slot: usize,
    /// Hotbar contents
    pub hotbar: [HotbarItem; 9],
    /// Whether the palette is open
    pub palette_open: bool,
    /// Current palette category
    pub palette_category: PaletteCategory,
    /// Selected item in palette (index within category)
    pub palette_selection: usize,
    /// Cooldown timer for trigger actions (prevents rapid-fire)
    pub action_cooldown: f32,
    /// Cooldown timer for hotbar cycling
    pub hotbar_cooldown: f32,
}

impl Default for ControllerEditMode {
    fn default() -> Self {
        Self {
            enabled: true,
            hotbar_slot: 0,
            hotbar: default_hotbar(),
            palette_open: false,
            palette_category: PaletteCategory::default(),
            palette_selection: 0,
            action_cooldown: 0.0,
            hotbar_cooldown: 0.0,
        }
    }
}

impl ControllerEditMode {
    /// Get the current hotbar item.
    pub fn current_item(&self) -> &HotbarItem {
        &self.hotbar[self.hotbar_slot]
    }

    /// Set the current hotbar slot's item.
    pub fn set_current_item(&mut self, item: HotbarItem) {
        self.hotbar[self.hotbar_slot] = item;
    }

    /// Cycle to the next hotbar slot.
    pub fn next_slot(&mut self) {
        self.hotbar_slot = (self.hotbar_slot + 1) % 9;
    }

    /// Cycle to the previous hotbar slot.
    pub fn prev_slot(&mut self) {
        self.hotbar_slot = if self.hotbar_slot == 0 {
            8
        } else {
            self.hotbar_slot - 1
        };
    }

    /// Go to a specific slot.
    pub fn goto_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.hotbar_slot = slot;
        }
    }

    /// Get items in the current palette category.
    pub fn palette_items(&self) -> Vec<HotbarItem> {
        self.palette_category.items()
    }

    /// Get the currently selected palette item.
    pub fn selected_palette_item(&self) -> Option<HotbarItem> {
        let items = self.palette_items();
        items.get(self.palette_selection).cloned()
    }

    /// Move palette selection.
    pub fn move_palette_selection(&mut self, delta: i32) {
        let items = self.palette_items();
        if items.is_empty() {
            return;
        }

        let new_selection = self.palette_selection as i32 + delta;
        if new_selection < 0 {
            self.palette_selection = items.len() - 1;
        } else if new_selection >= items.len() as i32 {
            self.palette_selection = 0;
        } else {
            self.palette_selection = new_selection as usize;
        }
    }

    /// Switch palette category and reset selection.
    pub fn switch_category(&mut self, next: bool) {
        self.palette_category = if next {
            self.palette_category.next()
        } else {
            self.palette_category.prev()
        };
        self.palette_selection = 0;
    }

    /// Confirm palette selection (put item in hotbar).
    pub fn confirm_palette_selection(&mut self) {
        if let Some(item) = self.selected_palette_item() {
            self.hotbar[self.hotbar_slot] = item;
            self.palette_open = false;
        }
    }
}

/// System to handle controller hotbar input.
pub fn handle_controller_hotbar(
    mode: Res<ControllerCameraMode>,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut edit_mode: ResMut<ControllerEditMode>,
) {
    // Only process in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // Don't process hotbar while palette is open
    if edit_mode.palette_open {
        return;
    }

    // Update cooldowns
    edit_mode.hotbar_cooldown = (edit_mode.hotbar_cooldown - time.delta_secs()).max(0.0);

    if edit_mode.hotbar_cooldown > 0.0 {
        return;
    }

    let mut cycled = false;

    // Gamepad: LB/RB for hotbar cycling
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::LeftTrigger) {
            edit_mode.prev_slot();
            cycled = true;
        }
        if gamepad.just_pressed(GamepadButton::RightTrigger) {
            edit_mode.next_slot();
            cycled = true;
        }

        // D-pad left/right for hotbar cycling
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
            edit_mode.prev_slot();
            cycled = true;
        }
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            edit_mode.next_slot();
            cycled = true;
        }
    }

    // Keyboard: Number keys 1-9 for direct slot selection
    let number_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    for (i, key) in number_keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            edit_mode.goto_slot(i);
            cycled = true;
        }
    }

    // Keyboard: scroll wheel equivalent with [ and ]
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        edit_mode.prev_slot();
        cycled = true;
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        edit_mode.next_slot();
        cycled = true;
    }

    if cycled {
        edit_mode.hotbar_cooldown = 0.1;
        info!("Hotbar slot: {} - {}", edit_mode.hotbar_slot + 1, edit_mode.current_item().name());
    }
}

/// System to handle controller palette input.
pub fn handle_controller_palette(
    mode: Res<ControllerCameraMode>,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut edit_mode: ResMut<ControllerEditMode>,
) {
    // Only process in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // Y button or E key to toggle palette
    let mut toggle_palette = keyboard.just_pressed(KeyCode::KeyE);
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::North) {
            toggle_palette = true;
        }
    }

    if toggle_palette {
        edit_mode.palette_open = !edit_mode.palette_open;
        if edit_mode.palette_open {
            info!("Palette opened");
        } else {
            info!("Palette closed");
        }
        return;
    }

    // Only process palette navigation if palette is open
    if !edit_mode.palette_open {
        return;
    }

    // Palette navigation
    for gamepad in gamepads.iter() {
        // D-pad up/down to navigate items
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            edit_mode.move_palette_selection(-1);
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            edit_mode.move_palette_selection(1);
        }

        // LB/RB to switch categories
        if gamepad.just_pressed(GamepadButton::LeftTrigger) {
            edit_mode.switch_category(false);
        }
        if gamepad.just_pressed(GamepadButton::RightTrigger) {
            edit_mode.switch_category(true);
        }

        // A button to confirm selection
        if gamepad.just_pressed(GamepadButton::South) {
            edit_mode.confirm_palette_selection();
        }

        // B button to close palette
        if gamepad.just_pressed(GamepadButton::East) {
            edit_mode.palette_open = false;
        }

        // X button to clear current slot
        if gamepad.just_pressed(GamepadButton::West) {
            edit_mode.set_current_item(HotbarItem::Empty);
        }

        // Left stick for navigation
        let left_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        if left_y > 0.5 {
            edit_mode.move_palette_selection(-1);
        } else if left_y < -0.5 {
            edit_mode.move_palette_selection(1);
        }
    }

    // Keyboard navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        edit_mode.move_palette_selection(-1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        edit_mode.move_palette_selection(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        edit_mode.switch_category(false);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        edit_mode.switch_category(true);
    }
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        edit_mode.confirm_palette_selection();
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        edit_mode.palette_open = false;
    }
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        edit_mode.set_current_item(HotbarItem::Empty);
    }
}

/// System to handle controller editing actions (place/remove).
pub fn handle_controller_editing(
    mode: Res<ControllerCameraMode>,
    gamepads: Query<&Gamepad>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    cursor: Res<ControllerCursor>,
    mut edit_mode: ResMut<ControllerEditMode>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mut render_state: ResMut<MapRenderState>,
) {
    // Only process in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // Don't process while palette is open
    if edit_mode.palette_open {
        return;
    }

    // Update cooldown
    edit_mode.action_cooldown = (edit_mode.action_cooldown - time.delta_secs()).max(0.0);

    if edit_mode.action_cooldown > 0.0 {
        return;
    }

    // Check if cursor is in reach
    if !cursor.in_reach {
        return;
    }

    let mut place_action = false;
    let mut remove_action = false;

    // Gamepad: RT to place, LT to remove
    for gamepad in gamepads.iter() {
        let rt = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
        let lt = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0);

        if rt > 0.5 {
            place_action = true;
        }
        if lt > 0.5 {
            remove_action = true;
        }
    }

    // Mouse fallback: Left click to place, right click to remove
    if mouse_button.pressed(MouseButton::Left) {
        place_action = true;
    }
    if mouse_button.pressed(MouseButton::Right) {
        remove_action = true;
    }

    // Handle placement
    if place_action {
        if let Some(placement_pos) = cursor.placement_position {
            let item = edit_mode.current_item().clone();
            match &item {
                HotbarItem::Voxel {
                    voxel_type,
                    pattern,
                } => {
                    let pos = (placement_pos.x, placement_pos.y, placement_pos.z);

                    // Check if voxel already exists at this position
                    let exists = editor_state
                        .current_map
                        .world
                        .voxels
                        .iter()
                        .any(|v| v.pos == pos);

                    if !exists {
                        let voxel_data = VoxelData {
                            pos,
                            voxel_type: *voxel_type,
                            pattern: Some(*pattern),
                            rotation_state: None,
                        };

                        editor_state.current_map.world.voxels.push(voxel_data.clone());
                        editor_state.mark_modified();
                        render_state.needs_render = true;

                        history.push(EditorAction::PlaceVoxel {
                            pos,
                            data: voxel_data,
                        });
                        edit_mode.action_cooldown = 0.15;

                        info!("Placed {} at {:?}", item.name(), pos);
                    }
                }
                HotbarItem::Entity { entity_type } => {
                    let entity_data = EntityData {
                        entity_type: *entity_type,
                        position: (
                            placement_pos.x as f32 + 0.5,
                            placement_pos.y as f32,
                            placement_pos.z as f32 + 0.5,
                        ),
                        properties: HashMap::new(),
                    };

                    let index = editor_state.current_map.entities.len();
                    editor_state.current_map.entities.push(entity_data.clone());
                    editor_state.mark_modified();
                    render_state.needs_render = true;

                    history.push(EditorAction::PlaceEntity {
                        index,
                        data: entity_data,
                    });
                    edit_mode.action_cooldown = 0.25;

                    info!("Placed {} at {:?}", item.name(), placement_pos);
                }
                HotbarItem::Tool(_) | HotbarItem::Empty => {
                    // Tools don't place anything
                }
            }
        }
    }

    // Handle removal
    if remove_action {
        if let Some(target_pos) = cursor.target_voxel {
            let pos = (target_pos.x, target_pos.y, target_pos.z);

            // Find and remove the voxel
            if let Some(idx) = editor_state
                .current_map
                .world
                .voxels
                .iter()
                .position(|v| v.pos == pos)
            {
                let removed = editor_state.current_map.world.voxels.remove(idx);
                editor_state.mark_modified();
                render_state.needs_render = true;

                history.push(EditorAction::RemoveVoxel {
                    pos,
                    data: removed,
                });
                edit_mode.action_cooldown = 0.15;

                info!("Removed voxel at {:?}", pos);
            }
        }
    }
}

/// System to handle pick block (X button / middle mouse).
pub fn handle_controller_pick_block(
    mode: Res<ControllerCameraMode>,
    gamepads: Query<&Gamepad>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor: Res<ControllerCursor>,
    editor_state: Res<EditorState>,
    mut edit_mode: ResMut<ControllerEditMode>,
) {
    // Only process in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // Don't process while palette is open
    if edit_mode.palette_open {
        return;
    }

    let mut pick_action = false;

    // Gamepad: X button
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::West) {
            pick_action = true;
        }
    }

    // Mouse: Middle click
    if mouse_button.just_pressed(MouseButton::Middle) {
        pick_action = true;
    }

    if !pick_action {
        return;
    }

    // Pick the voxel under cursor
    if let Some(target_pos) = cursor.target_voxel {
        let pos = (target_pos.x, target_pos.y, target_pos.z);

        if let Some(voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .find(|v| v.pos == pos)
        {
            let item = HotbarItem::Voxel {
                voxel_type: voxel.voxel_type,
                pattern: voxel.pattern.unwrap_or(SubVoxelPattern::Full),
            };
            edit_mode.set_current_item(item.clone());
            info!("Picked block: {}", item.name());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_edit_mode_default() {
        let mode = ControllerEditMode::default();
        assert!(mode.enabled);
        assert_eq!(mode.hotbar_slot, 0);
        assert!(!mode.palette_open);
    }

    #[test]
    fn test_hotbar_cycling() {
        let mut mode = ControllerEditMode::default();

        mode.next_slot();
        assert_eq!(mode.hotbar_slot, 1);

        mode.prev_slot();
        assert_eq!(mode.hotbar_slot, 0);

        mode.prev_slot();
        assert_eq!(mode.hotbar_slot, 8); // Wraps around
    }

    #[test]
    fn test_goto_slot() {
        let mut mode = ControllerEditMode::default();

        mode.goto_slot(5);
        assert_eq!(mode.hotbar_slot, 5);

        mode.goto_slot(100); // Invalid, should not change
        assert_eq!(mode.hotbar_slot, 5);
    }

    #[test]
    fn test_palette_navigation() {
        let mut mode = ControllerEditMode::default();
        mode.palette_open = true;

        let initial_items = mode.palette_items().len();
        assert!(initial_items > 0);

        mode.move_palette_selection(1);
        assert_eq!(mode.palette_selection, 1);

        mode.switch_category(true);
        assert_eq!(mode.palette_category, PaletteCategory::Patterns);
        assert_eq!(mode.palette_selection, 0); // Reset on category change
    }

    #[test]
    fn test_confirm_palette_selection() {
        let mut mode = ControllerEditMode::default();
        mode.palette_open = true;
        mode.palette_category = PaletteCategory::Entities;
        mode.palette_selection = 0;

        mode.confirm_palette_selection();

        assert!(!mode.palette_open);
        assert!(mode.current_item().is_entity());
    }
}
