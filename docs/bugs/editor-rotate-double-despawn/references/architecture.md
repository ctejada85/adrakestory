# Architecture: Fix Editor Rotate Double-Despawn

## Current Architecture

### Affected files

| File | Role |
|------|------|
| `src/editor/tools/selection_tool/preview.rs` | Contains `render_transform_preview` and `render_rotation_preview` |
| `src/editor/tools/selection_tool/mod.rs` | Exports both functions |
| `src/editor/tools/mod.rs` | Re-exports both functions |
| `src/bin/map_editor/main.rs:170–171` | Registers both systems with no ordering constraint |

### Current system topology

```
Update schedule (unordered):
  render_transform_preview  ──┐  both query With<TransformPreview>
  render_rotation_preview   ──┘  both call despawn() on all results
```

During `TransformMode::Rotate`, both systems pass their guards and both enqueue
`despawn()` for the same entity IDs before any command flush. The second despawn targets
an entity whose generation has already incremented → warning.

### Data flow (current)

```
ActiveTransform.is_changed()
        │
        ├── render_transform_preview
        │     cleanup loop (lines 32–34)  ← despawn #1
        │     spawn coarse cube previews
        │
        └── render_rotation_preview
              cleanup loop (lines 107–110) ← despawn #2 (same entity IDs!)
              spawn sub-voxel previews
```

---

## Target Architecture

### Merged system

Replace both functions with a single `render_transform_preview` that owns the full
`TransformPreview` lifecycle and switches on `active_transform.mode`:

```
Update schedule:
  render_transform_preview (merged)
      │
      ├── mode == None     → despawn all, return
      ├── mode == Move     → despawn all, spawn coarse cubes
      └── mode == Rotate   → despawn all, spawn sub-voxel previews
```

One cleanup loop. One spawn path. No shared-query race possible.

### Files changed

| File | Change |
|------|--------|
| `src/editor/tools/selection_tool/preview.rs` | Delete `render_rotation_preview`; expand `render_transform_preview` with a `Rotate` branch |
| `src/editor/tools/selection_tool/mod.rs` | Remove `render_rotation_preview` from `pub use` |
| `src/editor/tools/mod.rs` | Remove `render_rotation_preview` from `pub use` |
| `src/bin/map_editor/main.rs` | Remove line 171 (`render_rotation_preview` registration) |

### Merged function signature

The merged function has the union of both current parameter sets — identical since both
functions already share all parameters:

```rust
pub fn render_transform_preview(
    mut commands: Commands,
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_previews: Query<Entity, With<TransformPreview>>,
)
```

### Pseudocode

```rust
pub fn render_transform_preview(...) {
    // mode == None: clean up and exit
    if active_transform.mode == TransformMode::None {
        for entity in existing_previews.iter() { commands.entity(entity).despawn(); }
        return;
    }

    // Only update when state changed
    if !active_transform.is_changed() { return; }

    // Single cleanup — runs exactly once per frame
    for entity in existing_previews.iter() { commands.entity(entity).despawn(); }

    match active_transform.mode {
        TransformMode::Move   => { /* spawn coarse cube previews — existing logic */ }
        TransformMode::Rotate => { /* spawn sub-voxel previews  — existing logic */ }
        TransformMode::None   => unreachable!(),
    }
}
```

---

## Appendix D — Code Template

### preview.rs replacement

```rust
//! Transform and rotation preview rendering.

use super::{ActiveTransform, TransformMode, TransformPreview};
use crate::editor::state::EditorState;
use crate::systems::game::map::format::{apply_orientation_matrix, axis_angle_to_matrix, SubVoxelPattern};
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

/// Render transform previews for both move and rotate modes.
///
/// Owns the full lifecycle of `TransformPreview` entities.
/// A single cleanup loop runs before any spawn so no entity is despawned twice.
pub fn render_transform_preview(
    mut commands: Commands,
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_previews: Query<Entity, With<TransformPreview>>,
) {
    // mode == None: clean up and return
    if active_transform.mode == TransformMode::None {
        for entity in existing_previews.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    if !active_transform.is_changed() {
        return;
    }

    // Single despawn loop — runs exactly once per frame regardless of mode
    for entity in existing_previews.iter() {
        commands.entity(entity).despawn();
    }

    match active_transform.mode {
        TransformMode::None => {}   // handled above
        TransformMode::Move => {
            // ... spawn_move_previews logic (existing render_transform_preview lines 36–85)
        }
        TransformMode::Rotate => {
            // ... spawn_rotation_previews logic (existing render_rotation_preview lines 112–185)
        }
    }
}

// rotate_position() stays unchanged
```

### main.rs change

Remove line:
```rust
.add_systems(Update, tools::render_rotation_preview)
```

No other changes to `main.rs`.
