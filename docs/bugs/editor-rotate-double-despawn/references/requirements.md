# Requirements: Fix Editor Rotate Double-Despawn

## Problem

When rotating voxels in the map editor, Bevy emits "entity already despawned" warnings
every frame the rotation preview updates. The warnings are a symptom of two independent
systems competing to clean up the same `TransformPreview` entities.

## Functional Requirements

### FR-1: Single system owns `TransformPreview` lifecycle

The `TransformPreview` component lifetime (spawn + despawn) must be managed by exactly
one system. No two systems may both query `With<TransformPreview>` and issue `despawn()`
on the results in the same frame.

### FR-2: Preview content is mode-appropriate

- `TransformMode::None` — all `TransformPreview` entities are removed; nothing is spawned.
- `TransformMode::Move` — coarse 0.95-unit cube previews (green valid, red collision).
- `TransformMode::Rotate` — sub-voxel geometry previews (blue valid, red collision).
- Switching modes clears the previous mode's previews before spawning the new ones.

### FR-3: No "entity already despawned" warnings during rotation

Running the editor, selecting voxels, pressing `R`, and rotating must produce zero
`WARN bevy_ecs::error::handler: Entity despawned` messages.

### FR-4: Backward-compatible observable behaviour

The visual output for move previews and rotate previews must be identical to the
pre-fix implementation. Existing functionality must not regress.

## Non-Functional Requirements

### NFR-1: Single system registration entry

The merged system is registered once in `src/bin/map_editor/main.rs`, replacing the two
existing lines for `render_transform_preview` and `render_rotation_preview`.

### NFR-2: No new Bevy `App` test infrastructure

Unit tests must target pure helper functions only (e.g. `rotate_position`). No full
`bevy::App` test setup is required for this fix.

## Out of Scope

- Changes to `render_selection_highlights`
- Changes to `handle_transformation_operations` or confirm/cancel logic
- Changes to `TransformPreview` component fields
