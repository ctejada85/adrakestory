# Architecture Overview

Complete guide to the system architecture and design of A Drake's Story.

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [ECS Architecture](#ecs-architecture)
- [Game States](#game-states)
- [System Organization](#system-organization)
- [Core Components](#core-components)
- [Resources](#resources)
- [Module Structure](#module-structure)
- [Data Flow](#data-flow)
- [Design Patterns](#design-patterns)

## Overview

A Drake's Story is built using the **Entity Component System (ECS)** architecture pattern via the Bevy game engine. This architecture provides:

- **Modularity**: Systems are independent and composable
- **Performance**: Data-oriented design for cache efficiency
- **Flexibility**: Easy to add/remove features
- **Maintainability**: Clear separation of concerns

## Technology Stack

### Core Technologies

- **Language**: Rust 2021 Edition
- **Game Engine**: Bevy 0.18
- **Build System**: Cargo
- **Serialization**: Serde + RON

### Key Dependencies

```toml
[dependencies]
bevy = { version = "0.18", features = ["bevy_gltf"] }  # Game engine with GLTF support
serde = { version = "1.0", features = ["derive"] }     # Serialization
ron = "0.8"                                            # Map format
thiserror = "1.0"                                      # Error handling
```

### Development Tools

- **Debugger**: CodeLLDB (VSCode)
- **Linter**: Clippy
- **Formatter**: rustfmt
- **Documentation**: rustdoc

## ECS Architecture

### Entity Component System Basics

**Entities**: Unique identifiers for game objects
```rust
// Examples: player, voxel, camera, UI element
let entity = commands.spawn(/* components */);
```

**Components**: Data attached to entities
```rust
#[derive(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,
}
```

**Systems**: Logic that operates on components
```rust
fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    // Update player positions
}
```

### Bevy's ECS Features

- **Queries**: Efficient component access
- **Resources**: Global state
- **Events**: Communication between systems
- **States**: Game state management
- **Schedules**: System execution order

## Game States

The game uses a finite state machine for flow control:

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    IntroAnimation,  // Opening splash screen
    TitleScreen,     // Main menu
    LoadingMap,      // Map loading with progress
    InGame,          // Active gameplay
    Paused,          // Pause menu
    Settings,        // Settings menu (planned)
}
```

### State Transitions

```
IntroAnimation → TitleScreen → LoadingMap → InGame ⇄ Paused
                                                ↓
                                          TitleScreen
```

### State-Specific Systems

Systems can be configured to run only in specific states:

```rust
app.add_systems(OnEnter(GameState::InGame), setup_game)
   .add_systems(Update, player_movement.run_if(in_state(GameState::InGame)))
   .add_systems(OnExit(GameState::InGame), cleanup_game);
```

### Camera Management Across States

The game uses different cameras for different states to prevent rendering conflicts:

- **2D Camera**: Used for UI-only states (IntroAnimation, TitleScreen, LoadingMap, Paused)
  - Spawned at startup in [`setup()`](../../src/main.rs:121)
  - Automatically despawned when entering InGame state via [`cleanup_2d_camera()`](../../src/main.rs:127)

- **3D Camera**: Used for gameplay (InGame state)
  - Spawned when entering InGame state in [`spawn_camera()`](../../src/systems/game/map/spawner/)
  - Includes [`GameCamera`](../../src/systems/game/components.rs:38) component for rotation control

This separation prevents camera order ambiguity warnings and ensures only one camera is active per render target at any time.

## System Organization

### Module Hierarchy

```
src/
├── main.rs                 # Application entry point
├── states.rs               # Game state definitions
├── systems/
│   ├── mod.rs              # Systems module root
│   ├── game/               # Core gameplay systems
│   │   ├── mod.rs
│   │   ├── components.rs   # Game components
│   │   ├── resources.rs    # Game resources
│   │   ├── systems.rs      # System re-exports
│   │   ├── camera.rs       # Camera control
│   │   ├── character/      # Character model system
│   │   │   └── mod.rs      # CharacterModel component
│   │   ├── collision.rs    # Collision detection
│   │   ├── occlusion/      # Occlusion material system
│   │   │   ├── mod.rs      # OcclusionPlugin, OcclusionUniforms, update system
│   │   │   └── tests.rs    # Unit tests for uniform caching logic
│   │   ├── hot_reload/     # Hot reload system
│   │   │   ├── mod.rs
│   │   │   ├── state.rs
│   │   │   ├── watcher.rs
│   │   │   ├── reload_handler.rs
│   │   │   ├── systems.rs
│   │   │   └── notifications.rs
│   │   ├── input.rs        # Input handling
│   │   ├── physics.rs      # Physics simulation
│   │   ├── player_movement.rs  # Player controls
│   │   └── map/            # Map loading system
│   │       ├── mod.rs
│   │       ├── format/     # Map data structures
│   │       │   ├── mod.rs
│   │       │   ├── camera.rs
│   │       │   ├── defaults.rs
│   │       │   ├── entities.rs
│   │       │   ├── lighting.rs
│   │       │   ├── metadata.rs
│   │       │   ├── patterns.rs
│   │       │   ├── rotation.rs
│   │       │   └── world.rs
│   │       ├── geometry/   # Sub-voxel geometry
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── patterns.rs
│   │       │   ├── rotation.rs
│   │       │   └── utils.rs
│   │       ├── loader.rs   # Map file loading
│   │       ├── spawner/    # World instantiation
│   │       │   ├── mod.rs
│   │       │   ├── meshing/ # Mesh generation
│   │       │   │   ├── mod.rs
│   │       │   │   ├── occupancy.rs
│   │       │   │   ├── greedy_mesher.rs
│   │       │   │   ├── mesh_builder.rs
│   │       │   │   └── palette.rs
│   │       │   ├── entities.rs
│   │       │   ├── chunks.rs
│   │       │   └── systems.rs
│   │       ├── validation.rs  # Map validation
│   │       └── error.rs    # Error types
│   ├── intro_animation/    # Intro screen
│   ├── title_screen/       # Title screen
│   ├── loading_screen/     # Loading UI
│   └── pause_menu/         # Pause menu
├── editor/                 # Map editor
│   ├── mod.rs
│   ├── cursor/             # Cursor state management
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── raycasting.rs
│   ├── tools/              # Editor tools
│   │   ├── mod.rs
│   │   ├── input/          # Input handling
│   │   │   ├── mod.rs
│   │   │   ├── events.rs
│   │   │   ├── keyboard.rs
│   │   │   └── operations.rs
│   │   ├── selection_tool/ # Selection tool
│   │   │   ├── mod.rs
│   │   │   ├── preview.rs  # Unified transform/rotation preview (single system)
│   │   │   ├── selection.rs
│   │   │   ├── movement.rs
│   │   │   └── rotation.rs
│   │   └── voxel_tool/     # Voxel placement/removal
│   │       ├── mod.rs
│   │       ├── drag_state.rs
│   │       ├── placement.rs
│   │       └── removal.rs
│   └── ui/                 # Editor UI
│       ├── mod.rs
│       ├── toolbar/        # Toolbar panel
│       │   ├── mod.rs
│       │   ├── file_menu.rs
│       │   ├── edit_menu.rs
│       │   ├── view_menu.rs
│       │   └── tool_buttons.rs
│       ├── dialogs/        # Dialog windows
│       │   ├── mod.rs
│       │   ├── events.rs
│       │   ├── rendering.rs
│       │   ├── file_operations.rs
│       │   └── window_handling.rs
│       └── properties/     # Properties panel
│           ├── mod.rs
│           ├── voxel.rs
│           ├── entity.rs
│           ├── lighting.rs
│           └── map_info.rs
└── components/             # Shared components
```

### Editor Tools: TransformPreview Lifecycle

`render_transform_preview` (`src/editor/tools/selection_tool/preview.rs`) is the **sole system** that owns the full `TransformPreview` entity lifecycle. It handles all three transform modes in a single `match`:

| Mode | Behavior |
|------|----------|
| `TransformMode::None` | Despawns all `TransformPreview` entities and returns. |
| `TransformMode::Move` | Despawns existing previews, then spawns coarse 1-voxel cube previews at the offset destination. |
| `TransformMode::Rotate` | Despawns existing previews, then spawns sub-voxel geometry previews at the rotated destination. |

The single cleanup loop runs **before** any spawning on every changed frame. This eliminates the double-despawn bug that existed when a separate `render_rotation_preview` system also iterated `With<TransformPreview>` in the same frame.

**Rule**: Never add a second system that queries `With<TransformPreview>` and calls `despawn()`. All `TransformPreview` entity creation and destruction must stay inside `render_transform_preview`.



**1. Game Systems** (`systems/game/`)
- Core gameplay logic
- Character model management
- Physics and collision
- Player movement
- Camera control
- Map loading
- Occlusion material uniform updates

**2. UI Systems** (`systems/*/`)
- Menu interfaces
- Loading screens
- HUD (planned)

**3. Rendering Systems** (Bevy built-in)
- 3D rendering
- Lighting
- Camera projection

## Core Components

### Player Component

```rust
#[derive(Component)]
pub struct Player {
    pub speed: f32,          // Movement speed
    pub velocity: Vec3,      // Current velocity
    pub is_grounded: bool,   // On ground?
    pub radius: f32,         // Horizontal collision radius (0.2)
    pub half_height: f32,    // Vertical half-height (0.4)
}
```

**Purpose**: Represents the player character with physics state. Uses a **cylinder collider** with horizontal `radius` and vertical `half_height` for collision detection. The visual representation is handled separately by the [`CharacterModel`](#charactermodel-component) component.

### CharacterModel Component

```rust
#[derive(Component)]
pub struct CharacterModel {
    pub scene_handle: Handle<Scene>,  // GLB/GLTF scene handle
    pub scale: f32,                   // Model scale factor
    pub offset: Vec3,                 // Position offset
}
```

**Purpose**: Manages the player's 3D visual model (GLB/GLTF format). The model is spawned as a child entity of the player, separating visuals from physics. See [Character System](systems/character-system.md) for details.

### Voxel Component

```rust
#[derive(Component)]
pub struct Voxel;
```

**Purpose**: Marker component for voxel entities.

### SubVoxel Component

```rust
#[derive(Component)]
pub struct SubVoxel {
    pub bounds: (Vec3, Vec3),  // (min, max) AABB bounds
}
```

**Purpose**: Represents individual sub-voxels within a voxel (8×8×8 grid). The bounds are pre-calculated and cached at spawn time for efficient collision detection, eliminating the need to compute them during physics updates. See [`physics-analysis.md`](systems/physics-analysis.md) for optimization details.

### GameCamera Component

```rust
#[derive(Component)]
pub struct GameCamera {
    pub original_rotation: Quat,   // Original rotation quaternion
    pub target_rotation: Quat,     // Target rotation for smooth transitions
    pub rotation_speed: f32,       // Speed of rotation interpolation (default: 5.0)
    pub follow_offset: Vec3,       // Offset from player in local camera space
    pub follow_speed: f32,         // Speed of position follow (default: 15.0)
    pub target_position: Vec3,     // Current follow target (player position)
}
```

**Purpose**: Camera control and rotation state with smooth interpolation.

**Lerp formula**: Both `follow_player_camera` and `rotate_camera` use the frame-rate-independent exponential decay formula `alpha = 1 - exp(-speed * delta)` instead of the approximation `speed * delta`. This guarantees identical convergence time regardless of frame rate (30/60/120 fps).

**Map-configurable feel**: `follow_speed` and `rotation_speed` are sourced from `CameraData` at spawn time via `spawn_camera()`. Map files may set them as `Option<f32>` fields; when absent, the engine defaults (15.0 and 5.0) are used. `fov_degrees` (also optional) sets the vertical field of view via `Projection::Perspective` when present.

### CollisionBox Component

```rust
#[derive(Component)]
pub struct CollisionBox;
```

**Purpose**: Debug visualization for collision boundaries. Displays a cylinder mesh matching the player's collision shape when toggled with the 'C' key.

## Resources

### LoadedMapData

```rust
#[derive(Resource)]
pub struct LoadedMapData {
    pub map: MapData,
}
```

**Purpose**: Stores the currently loaded map data.

### MapLoadProgress

```rust
#[derive(Resource, Default)]
pub struct MapLoadProgress {
    pub current: Option<LoadProgress>,
    pub events: Vec<LoadProgress>,
}
```

**Purpose**: Tracks map loading progress for UI display.

### SpatialGrid

```rust
#[derive(Resource)]
pub struct SpatialGrid {
    cells: HashMap<IVec3, Vec<Entity>>,
    cell_size: f32,
}
```

**Purpose**: Spatial partitioning for efficient collision detection.

### PreFetchedCollisionEntities

```rust
#[derive(Resource, Default)]
pub struct PreFetchedCollisionEntities {
    pub entities: Vec<Entity>,
    pub bounds: Option<(Vec3, Vec3)>,
}
```

**Purpose**: Frame-level cache that shares a single `SpatialGrid` AABB lookup across `move_player` and `apply_physics`. `move_player` (Movement set) writes a widened AABB result; `apply_physics` (Physics set) reads it when the player's physics AABB is fully contained within the cached bounds, falling back to its own query otherwise.

### GameInitialized

```rust
#[derive(Resource, Default)]
pub struct GameInitialized(pub bool);
```

**Purpose**: Prevents duplicate world setup. Tuple struct for simple boolean flag.

## Module Structure

### Game Module (`systems/game/`)

**Responsibilities:**
- Core gameplay logic
- Character model management
- Physics simulation
- Collision detection
- Player movement
- Camera control
- Map loading

**Key Files:**
- `components.rs`: Game-specific components
- `resources.rs`: Game-specific resources
- `systems.rs`: System function re-exports
- `camera.rs`: Camera control system
- `character/mod.rs`: Character model component
- `collision.rs`: Collision detection
- `input.rs`: Input handling
- `physics.rs`: Physics simulation
- `player_movement.rs`: Player controls

### Map Module (`systems/game/map/`)

**Responsibilities:**
- Map file loading
- Map validation
- World spawning
- Progress tracking

**Key Files:**
- `format/`: Map data structures (split into modules)
  - `camera.rs`, `defaults.rs`, `entities.rs`, `lighting.rs`, `metadata.rs`, `patterns.rs`, `rotation.rs`, `voxel_type.rs`, `world.rs`
  - `voxel_type.rs` defines `VoxelType` — re-exported via `format/mod.rs` and `components.rs`
- `loader.rs`: File I/O and parsing
- `spawner/`: Entity instantiation (split into modules)
  - `mod.rs`: Constants, types, main system
  - `meshing/`: Mesh generation (`occupancy.rs`, `greedy_mesher.rs`, `mesh_builder.rs`, `palette.rs`)
  - `entities.rs`, `chunks.rs`, `systems.rs`
- `geometry/`: Sub-voxel geometry calculations
  - `types.rs`, `patterns.rs`, `rotation.rs`, `utils.rs`
- `validation.rs`: Map validation
- `error.rs`: Error types

### UI Modules

Each UI module follows a consistent pattern:

```
module/
├── mod.rs          # Module exports
├── components.rs   # UI components
├── resources.rs    # UI resources (optional)
└── systems.rs      # UI systems
```

## Data Flow

### Startup Flow

```
1. main() → App initialization
2. Setup systems run
3. IntroAnimation state entered
4. Intro systems run
5. Transition to TitleScreen
```

### Game Start Flow

```
1. User clicks "New Game"
2. Transition to LoadingMap state
3. Map loader starts
4. Progress updates emitted
5. Map validated
6. World spawned
7. Transition to InGame state
```

### Game Loop Flow

```
1. Input systems read controls
2. Physics systems update velocities
3. Collision systems check boundaries
4. Movement systems update positions
5. Camera systems follow player
6. Rendering systems draw frame
```

### Pause Flow

```
1. User presses ESC
2. Transition to Paused state
3. Game systems stop
4. Pause menu systems run
5. User selects option
6. Transition back or to TitleScreen
```

## Design Patterns

### State Pattern

Game states control system execution:

```rust
app.add_systems(
    Update,
    player_movement.run_if(in_state(GameState::InGame))
);
```

### Message Pattern (Bevy 0.18+)

The polling-based event system uses `Message`/`MessageReader`/`MessageWriter` (renamed from `Event`/`EventReader`/`EventWriter` in Bevy 0.18):

```rust
fn emit_message(mut messages: MessageWriter<CustomMessage>) {
    messages.write(CustomMessage { /* ... */ });
}

fn handle_message(mut messages: MessageReader<CustomMessage>) {
    for msg in messages.read() {
        // Handle message
    }
}
```

Note: `#[derive(Message)]` and `app.add_message::<T>()` replace the old `#[derive(Event)]` / `app.add_event::<T>()`. The `Event` trait is now reserved for the observer/trigger pattern only.

### Single-Entity Query Pattern (Bevy 0.15+)

When a system is guaranteed to have exactly one entity matching a query (e.g. the player, the primary camera, the primary window), use the `Single<>` system parameter instead of `Query::single()` / `single_mut()`. This is the idiomatic Bevy 0.15+ approach used throughout this codebase.

```rust
// Old — panics at runtime if count != 1
fn move_player(mut query: Query<(&mut Player, &mut Transform)>) {
    if let Ok((mut player, mut transform)) = query.single_mut() {
        // ...
    }
}

// New — enforced at schedule-build time; cleaner signature
fn move_player(mut player: Single<(&mut Player, &mut Transform)>) {
    let (ref mut p, ref mut t) = *player;
    // ...
}
```

Use `Option<Single<>>` when the entity may legitimately be absent (e.g. during hot-reload gaps, or for optional gameplay entities like the flashlight):

```rust
fn update_flashlight(flashlight: Option<Single<&mut SpotLight, With<Flashlight>>>) {
    let Some(mut light) = flashlight else { return };
    // ...
}
```

**Rule**: Never use `Query::single()` or `Query::single_mut()` for player, camera, or window queries. Use `Single<>` or `Option<Single<>>` instead. Existing `Query<>` params are appropriate only when the count is unknown or can be zero/many.

### Component Pattern

Data-oriented design with components:

```rust
#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Damage(f32);

fn damage_system(
    mut query: Query<&mut Health>,
    damage_query: Query<&Damage>,
) {
    // Apply damage to health
}
```

### Resource Pattern

Global state management:

```rust
#[derive(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: Difficulty,
}
```

### Builder Pattern

Map creation:

```rust
let map = MapData {
    metadata: MapMetadata { /* ... */ },
    world: WorldData { /* ... */ },
    // ...
};
```

## Performance Considerations

### Spatial Partitioning

The `SpatialGrid` resource divides the world into cells for efficient collision queries:

```rust
// O(n²) → O(n) for collision checks
let nearby = spatial_grid.query_cell(position);
```

### Pre-fetched Collision Cache

`move_player` issues a single widened AABB lookup at the start of each movement frame and stores the result in `PreFetchedCollisionEntities`. All axis checks within `move_player` reuse this slice, and `apply_physics` uses the same cache when the player's physics AABB is within the cached bounds — eliminating the 3–4 redundant `SpatialGrid` queries that previously occurred per frame.

- `move_player` runs in `GameSystemSet::Movement` and writes the resource.
- `apply_physics` runs in `GameSystemSet::Physics` (after Movement) and reads the resource.
- The widened AABB expands horizontally by `|move_delta|` and vertically by `SUB_VOXEL_SIZE + STEP_UP_TOLERANCE` to cover step-up geometry.

### Conditional GPU Uniform Updates

The occlusion system (`systems/game/occlusion/`) uses a **three-level cache** to prevent unconditional GPU re-uploads:

**Level 1 — Position quantization:** before computing any uniforms, player and camera world positions are snapped to a configurable grid via `quantize_position()`. The step size is `OcclusionConfig.uniform_quantization_step` (default `0.25` units). Sub-step movement produces identical quantized values, so the downstream cache comparison always passes — eliminating ~85–95% of `get_mut()` calls during normal movement.

**Level 2 — Sub-struct equality cache:** `OcclusionUniforms` is split into two private `Copy + PartialEq` sub-structs:
- `StaticOcclusionUniforms` — config-driven fields; cached in `Local<Option<StaticOcclusionUniforms>>`
- `DynamicOcclusionUniforms` — positional fields (quantized player/camera position, interior region); cached in `Local<Option<DynamicOcclusionUniforms>>`

`get_mut()` is only called when at least one sub-struct cache differs from the newly computed value.

**Level 3 — Read-before-write guard:** a read-only `materials.get()` comparison runs before any `get_mut()` call as a second layer of protection against Bevy change detection stamping assets dirty unnecessarily.

Bevy `Ref<Transform>` queries and `is_changed()` gates skip sub-struct recomputation independently: a camera move does not recompute static config fields and vice versa.

**Why `get_mut()` is expensive here:** ALL voxel chunks share a single `OcclusionMaterialHandle`. Any `get_mut()` call stamps the shared asset `Changed`, causing Bevy's render world to re-extract and re-prepare GPU bind groups for all 100–200 chunk entities simultaneously — producing 13–18 ms frame spikes on movement-heavy frames (see `docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`).

**Transparency technique and `AlphaMode`:**

`OcclusionConfig.technique` controls both shader behavior and `AlphaMode`:

| Technique | `AlphaMode` | MSAA | Notes |
|-----------|-------------|------|-------|
| `Dithered` (default) | `Opaque` | None | Uses `discard` in shader; no MSAA cost |
| `AlphaBlend` | `AlphaToCoverage` | 4× per chunk | Smooth transparency; high cost on macOS TBDR |

The default is `Dithered` — `AlphaBlend` is available via the in-game settings menu. This prevents forced MSAA coverage-resolve on all 100–200 chunk meshes every frame on Apple Silicon and Intel macOS GPUs. See `docs/bugs/fix-alphatocoverage-msaa-macos/` for full context.

**Rule**: Any system that writes to a Bevy `Assets<T>` via `get_mut()` must guard the call behind an actual value change. See `coding-guardrails.md` §1.

### Camera Lerp Frame-Rate Independence

The camera follow and rotation systems use exponential decay to ensure smooth, frame-rate-independent interpolation:

```rust
// alpha = 1 - exp(-speed * delta)  — correct at any frame rate
let alpha = 1.0 - (-speed * delta).exp();
transform.translation = transform.translation.lerp(target, alpha);
```

The naive approximation `t = speed * delta` is only valid when `t ≪ 1`. At 30 fps with `speed = 5`, `t ≈ 0.17` — large enough to cause over-interpolation. The exponential formula is exact at all delta sizes.

`follow_speed` defaults to `15.0` (responsive third-person). `rotation_speed` defaults to `5.0`. Both values are resolved from `CameraData` in `spawn_camera()` — map files can override them via the optional `follow_speed` and `rotation_speed` fields on `CameraData`. A third optional field, `fov_degrees`, overrides the default vertical FOV (~60°) via `Projection::Perspective`.

**Rule**: All camera lerp/slerp calls must use `lerp_alpha(speed, delta)` from `camera.rs`, never `speed * delta` directly.

### Interior Detection Cache Invalidation

The interior detection system (`systems/game/interior_detection.rs`) maintains a `HashSet<IVec3>` occupancy cache of all voxel positions for its BFS flood-fill. Rebuilding this cache is expensive (iterates all `SubVoxel` entities), so it is deferred until the spawn wave settles:

- Change detection uses Bevy's `Added<SubVoxel>` query filter and `RemovedComponents<SubVoxel>` system parameter.
- When either is non-empty, `InteriorState.rebuild_pending` is set to `true` and the system returns early — **no rebuild during the spawn frame**.
- On the first frame where both are empty and `rebuild_pending` is `true`, the flag is cleared, the cache is reset, and exactly one full rebuild runs. Detection resumes on the same frame.
- On cold start (cache `None`, no spawn in progress, `rebuild_pending` false), the rebuild runs inline as a one-time initialisation.
- During steady-state gameplay (no map changes), the cache is reused across all detection cycles.
- The default `OcclusionMode` is `Hybrid`. `ShaderBased` mode skips the BFS path entirely.
- The BFS throttle interval defaults to 60 frames (~1×/sec at 60 fps).

**Rule**: Never use entity-count comparison as a cache-invalidation key. Use `Added<C>` / `RemovedComponents<C>` instead — they are O(1) and event-driven.

**Rule**: Never rebuild the occupancy cache during a spawn frame. Set `rebuild_pending = true` and wait for the settle frame to avoid O(200k) work while the player is live in-game.

### LOD Update Throttling

`update_chunk_lods` (`systems/game/map/spawner/mod.rs`) runs every frame but skips the O(N) chunk iteration unless the camera has moved or new chunks have spawned:

- **Distance guard**: `camera_pos.distance(last_camera_pos) >= lod_config.movement_threshold` — stored in `Local<Vec3>` per-system state. When the camera is stationary the system returns after one `distance()` call — O(1) instead of O(N).
- **New-chunk bypass**: `Added<VoxelChunk>` query detects chunks spawned this frame (e.g. on map load or hot-reload) and forces a full pass so newly spawned chunks get their correct LOD immediately.
- **`LodConfig` resource**: `movement_threshold: f32` (default `0.5` from `LOD_MOVEMENT_THRESHOLD`) allows runtime tuning of the dead zone without recompilation. Registered via `app.init_resource::<LodConfig>()`.

**Rule**: Never inline `LOD_MOVEMENT_THRESHOLD` in the guard — always read from `lod_config.movement_threshold` so the value is tunable at runtime.

### Depth Prepass

The game camera has `bevy::core_pipeline::prepass::DepthPrepass` inserted at spawn time in `spawn_camera()`. This activates a depth-only GPU pass before the main forward pass.

**Why it exists:** `OcclusionMaterial` uses `discard` in its WGSL fragment shader for dithered transparency. `discard` prevents GPU hardware early-Z optimisation in the main forward pass. With typical 3–5× overdraw in dense voxel areas, the expensive PBR + occlusion shader would run for all overlapping fragments. The depth prepass pre-populates the depth buffer so occluded fragments fail the hardware depth test before the fragment shader fires.

**Prepass shader:** `OcclusionExtension::prepass_fragment_shader()` returns the same `occlusion_material.wgsl` used by the main pass. The shader's `#ifdef PREPASS_PIPELINE` branch gates the output (depth-only vs full PBR), while the dither and region-based `discard` calls run before the branch in both passes. This ensures the prepass depth buffer accurately excludes fragments that would be discarded in the main pass.

**Rule:** Only the game camera (`GameCamera` component) has `DepthPrepass`. The editor camera must not receive it.

### Shadow Quality

Shadow rendering quality is controlled by `OcclusionConfig.shadow_quality: ShadowQuality`, persisted in `settings.ron`.

**`ShadowQuality` variants:**

| Variant | `shadows_enabled` | Cascades | Max distance | `NotShadowCaster` on chunks |
|---|---|---|---|---|
| `None` | `false` | — | — | No |
| `CharactersOnly` | `true` | 2 | 20 units | **Yes** |
| `Low` (default) | `true` | 2 | 20 units | No |
| `High` | `true` | 4 | 100 units | No (original behaviour) |

**Application paths:**
1. **Spawn time** — `spawn_lighting()` calls `shadow_params_for_quality(config.shadow_quality)` to set `DirectionalLight.shadows_enabled` and `CascadeShadowConfig` when the map loads. `spawn_voxels_chunked()` inserts `NotShadowCaster` on each `VoxelChunk` if quality is `CharactersOnly`.
2. **Runtime** — `apply_shadow_quality_system` (`GameSystemSet::Visual`, `InGame | Paused`) watches `OcclusionConfig::is_changed()` and `Added<VoxelChunk>`. On change it updates `DirectionalLight`, `CascadeShadowConfig`, and adds/removes `NotShadowCaster` on all chunks. On hot-reload it applies only to newly added chunks.

**Performance note (profiled):** The default `Low` reduces p95 frame spikes from ~38ms (`High`) to under 15ms by cutting shadow cascade volume from 4×(100u)³ to 2×(20u)³.

**Rule**: All shadow quality changes must go through `OcclusionConfig.shadow_quality`. Never hard-code `shadows_enabled: true` or a specific `CascadeShadowConfigBuilder` — always call `shadow_params_for_quality()`.

### VSync and Frame Pacing

Display settings are managed by `VsyncConfig` (`src/systems/settings/vsync.rs`), a separate resource from `OcclusionConfig`. Both are serialized to the same `settings.ron` via `AppSettings` (a combined serde struct using `#[serde(flatten)]`).

**Key types:**

| Type | Kind | Purpose |
|------|------|---------|
| `VsyncConfig` | `Resource` + Serialize | `vsync_enabled: bool`, `vsync_multiplier: f32`, `dirty: bool` (skip) |
| `MonitorInfo` | `Resource` (runtime only) | Cached `refresh_hz: f32`; populated by `detect_monitor_refresh_system` |
| `FrameLimiterState` | `Local<T>` in `apply_vsync_system` | Tracks `next_frame_deadline` + `target_frame_time` for self-correcting sleep-based pacing |

**Systems:**

- `detect_monitor_refresh_system` — runs each `Update` frame until the `Monitor` entity's `refresh_rate_millihertz` is read; sets `VsyncConfig.dirty = true` to trigger reapplication with the correct Hz.
- `apply_vsync_system` — runs in the `First` schedule; applies sleep-based frame pacing at the start of each frame, then (when `dirty`) mutates `Window.present_mode` and configures `FrameLimiterState.target_frame_time`.

**Frame cap logic:**

- When `vsync_enabled = true` and `vsync_multiplier <= 1.0`: `present_mode = Fifo`; target frame time is `1.0 / (refresh_hz × multiplier)`. Hardware VSync caps at the native refresh rate.
- When `vsync_enabled = true` and `vsync_multiplier > 1.0`: `present_mode = AutoNoVsync`; software cap drives fps above native Hz (e.g. 2× = 120 fps on 60 Hz monitor). `Fifo` cannot be used here as it hard-caps at the monitor refresh rate.
- When `vsync_enabled = false`: `present_mode = AutoNoVsync`; no software cap is applied.

**Self-correcting deadline algorithm:** `FrameLimiterState` advances `next_frame_deadline` by exactly `target_frame_time` each frame instead of recording `last_frame_end` after the sleep. Sleep overshoots in frame N shorten the sleep in frame N+1, preventing drift accumulation. `precise_sleep()` uses an OS sleep for the bulk of the wait and a spin-wait for the final 2 ms to overcome macOS/Windows scheduler wakeup latency. `next_frame_deadline` is reset on settings changes to prevent burst catch-up after a stall.

**Multiplier steps (UI):** `0.25×`, `0.5×`, `1.0×`, `2×`–`16×` (integer steps). Stored as `f32`, clamped to `[0.25, 16.0]`.

**Dirty-flag rule:** `VsyncConfig.dirty` must be set to `true` whenever `vsync_enabled` or `vsync_multiplier` changes. `apply_vsync_system` clears it after applying. This prevents per-frame `Window` mutation (guardrail §1 equivalent for display settings).

### Settings File (`settings.ron`)

All persistent settings are written to and read from `settings.ron` in the working directory. The file is deserialized into the `AppSettings` struct which uses `#[serde(flatten)]` to combine two config resources into one file:

```rust
#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(flatten)]
    pub occlusion: OcclusionConfig,
    #[serde(flatten)]
    pub vsync: VsyncConfig,
}
```

**Fields and defaults:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vsync_enabled` | `bool` | `false` | Enable hardware VSync + software frame cap |
| `vsync_multiplier` | `f32` | `1.0` | Frame rate target as a multiple of the monitor refresh rate; clamped to `[0.25, 16.0]` |
| `occlusion_mode` | `OcclusionMode` | `Hybrid` | Interior occlusion algorithm (`None`, `ShaderBased`, `RegionBased`, `Hybrid`) |
| `transparency_technique` | `TransparencyTechnique` | `Dithered` | Voxel transparency method (`Dithered`, `AlphaBlend`) |
| `shadow_quality` | `ShadowQuality` | `Low` | Directional light shadow cascade quality (`Off`, `Low`, `High`) |
| `uniform_quantization_step` | `f32` | `0.25` | Position grid step for occlusion uniform quantization (smaller = more updates, larger = fewer) |

**Rules:**
- Missing fields fall back to `Default` values — do not remove fields from `AppSettings` without a serde default.
- `dirty: bool` on `VsyncConfig` is tagged `#[serde(skip)]` and must never be written to disk.
- Both configs share the same flat namespace — field names must be unique across `OcclusionConfig` and `VsyncConfig`.

### Sub-Voxel Rendering

### Build Profiles

Optimized profiles balance compilation speed and runtime performance:

```toml
[profile.dev]
opt-level = 1              # Slight optimization

[profile.dev.package."*"]
opt-level = 3              # Full optimization for deps
```

## Extension Points

### Adding New Systems

1. Create system function
2. Add to appropriate module
3. Register in `main.rs`
4. Configure state conditions

### Adding New Components

1. Define component struct
2. Derive `Component` trait
3. Add to `components.rs`
4. Use in systems

### Adding New States

1. Add variant to `GameState` enum
2. Create state-specific systems
3. Define transitions
4. Add UI if needed

### Adding New Entity Types

1. Define in `map/format/entities.rs`
2. Add spawning logic in `map/spawner/entities.rs`
3. Create component if needed
4. Add systems for behavior

## Best Practices

### System Design

1. **Single Responsibility**: Each system does one thing
2. **Query Efficiency**: Use specific queries
3. **State Conditions**: Run systems only when needed
4. **Error Handling**: Use `Result` types

### Component Design

1. **Data Only**: Components are pure data
2. **Small**: Keep components focused
3. **Composable**: Combine for complex behavior
4. **Derive Traits**: Use `#[derive]` when possible

### Resource Design

1. **Global State**: Only for truly global data
2. **Initialization**: Use `Default` or `FromWorld`
3. **Access**: Minimize mutable access
4. **Lifetime**: Consider cleanup

## Testing Strategy

### Unit Tests

Test individual functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        // Test collision logic
    }
}
```

### Integration Tests

Test system interactions:

```rust
#[test]
fn test_player_movement() {
    let mut app = App::new();
    // Setup and test
}
```

### Manual Testing

- Test in debug mode
- Use collision visualization
- Test all game states
- Verify map loading

## Documentation

### Code Documentation

```rust
/// Handles player movement based on input.
///
/// # Arguments
/// * `time` - Game time resource
/// * `input` - Keyboard input resource
/// * `query` - Query for player entities
pub fn player_movement_system(/* ... */) {
    // Implementation
}
```

### Module Documentation

```rust
//! # Player Movement Module
//!
//! Handles all player movement logic including:
//! - Keyboard input processing
//! - Velocity calculation
//! - Ground detection
```

## Related Documentation

- **[Debugging Guide](debugging.md)** - Debug setup and tips
- **[Contributing Guide](contributing.md)** - Contribution workflow
- **[Character System](systems/character-system.md)** - Character model management
- **[Map Loader System](systems/map-loader.md)** - Map system details
- **[Map Editor Documentation](systems/map-editor/README.md)** - Complete map editor guide

---

**Architecture Version:** 2.5.0
**Last Updated:** 2026-03-31