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
