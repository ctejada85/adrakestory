# Map Editor NPC Management Implementation Plan

## Overview
Add NPC management support to the map editor, allowing users to place, select, edit, and delete NPCs visually in the editor viewport.

## Current State
- The editor already has an `EntityPlace` tool that supports entity types
- `EntityType` enum includes `Npc` (just added)
- Entity placement records position and properties
- The editor renders voxels but does NOT currently render entities visually

## Requirements
1. Add `Npc` to the entity type dropdown in the properties panel
2. Render NPC entities visually in the editor viewport (placeholder markers)
3. Allow editing NPC properties (name, radius) in the properties panel
4. Support selecting and deleting NPC entities
5. Show NPC indicators with labels in the 3D view

## Implementation Steps

### Step 1: Add Npc to Entity Type Dropdown

**File:** `src/editor/ui/properties.rs`

Update the entity type ComboBox to include `Npc`:

```rust
EditorTool::EntityPlace { entity_type } => {
    ui.label("Entity Place Tool");

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("entity_type")
            .selected_text(format!("{:?}", entity_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(entity_type, EntityType::PlayerSpawn, "Player Spawn");
                ui.selectable_value(entity_type, EntityType::Npc, "NPC");
                ui.selectable_value(entity_type, EntityType::Enemy, "Enemy");
                ui.selectable_value(entity_type, EntityType::Item, "Item");
                ui.selectable_value(entity_type, EntityType::Trigger, "Trigger");
            });
    });

    // Show NPC-specific properties when Npc is selected
    if *entity_type == EntityType::Npc {
        ui.separator();
        ui.label("NPC Properties:");
        // Name input (stored in properties HashMap)
        // Radius slider
    }
}
```

### Step 2: Create Entity Renderer System

**File:** `src/editor/renderer.rs` (add to existing file)

Add marker component and rendering for entities:

```rust
/// Marker component for entity indicators spawned by the editor
#[derive(Component)]
pub struct EditorEntityMarker {
    pub entity_index: usize,
}

/// System to render entity markers in the viewport
pub fn render_entities_system(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<Entity, With<EditorEntityMarker>>,
) {
    // Despawn existing markers
    for entity in existing_markers.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Spawn markers for each entity
    for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
        let (x, y, z) = entity_data.position;
        let position = Vec3::new(x, y, z);
        
        let (color, size) = match entity_data.entity_type {
            EntityType::PlayerSpawn => (Color::srgb(0.0, 1.0, 0.0), 0.5),
            EntityType::Npc => (Color::srgb(0.0, 0.5, 1.0), 0.4),
            EntityType::Enemy => (Color::srgb(1.0, 0.0, 0.0), 0.4),
            EntityType::Item => (Color::srgb(1.0, 1.0, 0.0), 0.3),
            EntityType::Trigger => (Color::srgba(1.0, 0.0, 1.0, 0.5), 0.6),
        };

        // Spawn sphere marker
        let mesh = meshes.add(Sphere::new(size));
        let material = materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            EditorEntityMarker { entity_index: index },
        ));
    }
}
```

### Step 3: Add Entity Selection Support

**File:** `src/editor/tools/entity_tool.rs`

Add entity selection via click:

```rust
/// Handle entity selection when clicking near an entity
pub fn handle_entity_selection(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
) {
    // Only handle in Select tool mode
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
    if ui_wants_input {
        return;
    }

    let Some(cursor_pos) = cursor_state.position else {
        return;
    };

    // Find closest entity within selection radius
    let selection_radius = 0.5;
    let mut closest_index: Option<usize> = None;
    let mut closest_distance = f32::MAX;

    for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
        let (ex, ey, ez) = entity_data.position;
        let entity_pos = Vec3::new(ex, ey, ez);
        let distance = cursor_pos.distance(entity_pos);

        if distance < selection_radius && distance < closest_distance {
            closest_distance = distance;
            closest_index = Some(index);
        }
    }

    // Update selection
    editor_state.selected_entities.clear();
    if let Some(index) = closest_index {
        editor_state.selected_entities.insert(index);
    }
}
```

### Step 4: Add NPC Properties Editor in UI

**File:** `src/editor/ui/properties.rs`

Add a section to edit selected entity properties:

```rust
/// Render properties for selected entities
fn render_selected_entity_properties(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    if editor_state.selected_entities.is_empty() {
        ui.label("No entity selected");
        return;
    }

    // For simplicity, edit first selected entity
    let index = *editor_state.selected_entities.iter().next().unwrap();
    
    if let Some(entity) = editor_state.current_map.entities.get_mut(index) {
        ui.label(format!("Entity: {:?}", entity.entity_type));
        
        // Position editing
        ui.horizontal(|ui| {
            ui.label("Position:");
            let (mut x, mut y, mut z) = entity.position;
            ui.add(egui::DragValue::new(&mut x).speed(0.1).prefix("X: "));
            ui.add(egui::DragValue::new(&mut y).speed(0.1).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut z).speed(0.1).prefix("Z: "));
            entity.position = (x, y, z);
        });

        // NPC-specific properties
        if entity.entity_type == EntityType::Npc {
            ui.separator();
            ui.label("NPC Properties:");

            // Name
            let mut name = entity.properties
                .get("name")
                .cloned()
                .unwrap_or_else(|| "NPC".to_string());
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut name).changed() {
                    entity.properties.insert("name".to_string(), name);
                    editor_state.mark_modified();
                }
            });

            // Radius
            let mut radius: f32 = entity.properties
                .get("radius")
                .and_then(|r| r.parse().ok())
                .unwrap_or(0.3);
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::Slider::new(&mut radius, 0.1..=1.0)).changed() {
                    entity.properties.insert("radius".to_string(), radius.to_string());
                    editor_state.mark_modified();
                }
            });
        }

        // Delete button
        if ui.button("Delete Entity").clicked() {
            // Will need to emit delete event
        }
    }
}
```

### Step 5: Add Entity Deletion Support

**File:** `src/editor/tools/entity_tool.rs`

Add entity deletion with undo support:

```rust
/// Event to delete selected entities
#[derive(Event)]
pub struct DeleteSelectedEntities;

/// Handle entity deletion
pub fn handle_entity_deletion(
    mut events: EventReader<DeleteSelectedEntities>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
) {
    for _ in events.read() {
        // Collect entities to delete (in reverse order to maintain indices)
        let mut indices: Vec<usize> = editor_state.selected_entities.iter().copied().collect();
        indices.sort_by(|a, b| b.cmp(a)); // Reverse sort

        for index in indices {
            if index < editor_state.current_map.entities.len() {
                let removed = editor_state.current_map.entities.remove(index);
                history.push(EditorAction::RemoveEntity {
                    index,
                    data: removed,
                });
            }
        }

        editor_state.selected_entities.clear();
        editor_state.mark_modified();
    }
}
```

### Step 6: Update History System for Entity Actions

**File:** `src/editor/history.rs`

Ensure entity actions are properly recorded:

```rust
pub enum EditorAction {
    // ... existing variants ...
    PlaceEntity { index: usize, data: EntityData },
    RemoveEntity { index: usize, data: EntityData },
    ModifyEntity { index: usize, old_data: EntityData, new_data: EntityData },
}
```

### Step 7: Register New Systems in Map Editor

**File:** `src/bin/map_editor.rs`

Add the new systems:

```rust
.add_systems(
    Update,
    (
        render_entities_system.run_if(resource_changed::<EditorState>),
        handle_entity_selection,
        handle_entity_deletion,
    )
)
```

### Step 8: Highlight Selected Entities

**File:** `src/editor/renderer.rs`

Add visual feedback for selected entities:

```rust
// In render_entities_system, check if entity is selected
let is_selected = editor_state.selected_entities.contains(&index);
let color = if is_selected {
    // Brighter color for selection
    color.lighter(0.3)
} else {
    color
};

// Add outline or glow effect for selected entities
```

## File Changes Summary

| File | Change |
|------|--------|
| `src/editor/ui/properties.rs` | Add Npc to entity type dropdown, add NPC properties editor |
| `src/editor/renderer.rs` | Add entity marker rendering system |
| `src/editor/tools/entity_tool.rs` | Add entity selection and deletion |
| `src/editor/history.rs` | Add entity modification history actions |
| `src/bin/map_editor.rs` | Register new systems |

## Visual Indicators

| Entity Type | Color | Shape | Size |
|-------------|-------|-------|------|
| PlayerSpawn | Green | Sphere | 0.5 |
| Npc | Blue | Sphere | 0.4 |
| Enemy | Red | Sphere | 0.4 |
| Item | Yellow | Sphere | 0.3 |
| Trigger | Magenta (transparent) | Sphere | 0.6 |

## Testing Checklist

- [ ] Npc appears in entity type dropdown
- [ ] Clicking in viewport places NPC at cursor position
- [ ] NPC entities show as blue spheres in viewport
- [ ] Selecting an NPC highlights it
- [ ] NPC properties (name, radius) can be edited
- [ ] Changes are saved to map file
- [ ] Undo/Redo works for NPC operations
- [ ] Delete key removes selected NPC
- [ ] Multiple NPCs can be placed
- [ ] Loading a map with NPCs shows them in editor

## Future Enhancements (Out of Scope)

- NPC model preview in editor (load actual GLB)
- Drag to move selected entities
- Copy/paste entities
- Multi-select entities
- Entity layer visibility toggle
- Entity snapping to grid or voxel surfaces

## Estimated Effort

~3-4 hours for full implementation
