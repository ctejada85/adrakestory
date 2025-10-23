# Map Editor Architecture

## System Architecture Diagram

```mermaid
graph TB
    subgraph "Map Editor Application"
        Entry[map_editor.rs<br/>Entry Point]
        
        subgraph "Core Systems"
            State[EditorState<br/>Current Map Data]
            History[EditorHistory<br/>Undo/Redo Stack]
            Tools[EditorTools<br/>Active Tool]
        end
        
        subgraph "UI Layer - bevy_egui"
            Toolbar[Toolbar Panel<br/>File, Edit, Tools]
            Props[Properties Panel<br/>Voxel/Entity Config]
            Viewport[3D Viewport<br/>Interactive Preview]
            Status[Status Bar<br/>Info Display]
        end
        
        subgraph "Rendering"
            Camera[Editor Camera<br/>Orbit Controls]
            Grid[Infinite Grid Renderer<br/>Dynamic Visual Guide]
            VoxelRender[Voxel Renderer<br/>Reuse Game Code]
        end
        
        subgraph "File I/O"
            Loader[Map Loader<br/>Read .ron Files]
            Saver[Map Saver<br/>Write .ron Files]
            Validator[Map Validator<br/>Check Integrity]
        end
    end
    
    Entry --> State
    Entry --> History
    Entry --> Tools
    
    State --> Toolbar
    State --> Props
    State --> Viewport
    State --> Status
    
    Tools --> Viewport
    History --> State
    
    Viewport --> Camera
    Viewport --> Grid
    Viewport --> VoxelRender
    
    Toolbar --> Loader
    Toolbar --> Saver
    Saver --> Validator
    Loader --> Validator
    
    Validator --> State
```

## Data Flow Diagram

```mermaid
flowchart LR
    subgraph "User Input"
        Mouse[Mouse Events]
        Keyboard[Keyboard Events]
        UI[UI Interactions]
    end
    
    subgraph "Unified Input System"
        KeyboardHandler[Keyboard Input Handler<br/>Single Entry Point]
        MouseHandlers[Mouse Input Handlers<br/>Tool-Specific]
        UIHandler[UI Event Handler]
    end
    
    subgraph "Event System"
        InputEvents[EditorInputEvent<br/>Semantic Events]
        UIEvents[UI Button Events]
    end
    
    subgraph "Execution Layer"
        TransformOps[Transformation Operations<br/>Event-Driven Execution]
        ToolOps[Tool Operations<br/>Direct Execution]
    end
    
    subgraph "State Management"
        EditorState[Editor State]
        History[History System]
    end
    
    subgraph "Rendering"
        Viewport[3D Viewport]
        UIRender[UI Panels]
    end
    
    Keyboard --> KeyboardHandler
    Mouse --> MouseHandlers
    UI --> UIHandler
    
    KeyboardHandler --> InputEvents
    UIHandler --> UIEvents
    
    InputEvents --> TransformOps
    UIEvents --> TransformOps
    MouseHandlers --> ToolOps
    
    TransformOps --> EditorState
    ToolOps --> EditorState
    
    EditorState --> History
    EditorState --> Viewport
    EditorState --> UIRender
    
    History -.Undo/Redo.-> EditorState
```

## Component Interaction Flow

```mermaid
sequenceDiagram
    participant User
    participant UI as UI Panel
    participant Tool as Active Tool
    participant State as Editor State
    participant History as History System
    participant Render as Renderer
    
    User->>UI: Select Voxel Tool
    UI->>State: Update Active Tool
    State->>Render: Update UI Display
    
    User->>Tool: Click in Viewport
    Tool->>Tool: Calculate Grid Position
    Tool->>State: Place Voxel at Position
    State->>History: Record Action
    State->>Render: Update 3D View
    
    User->>UI: Press Undo (Ctrl+Z)
    UI->>History: Request Undo
    History->>State: Restore Previous State
    State->>Render: Update 3D View
```

## File Operation Workflow

```mermaid
flowchart TD
    Start([User Action])
    
    Start --> New{New Map?}
    Start --> Open{Open Map?}
    Start --> Save{Save Map?}
    
    New -->|Yes| CreateDefault[Create Default Map]
    CreateDefault --> UpdateState[Update Editor State]
    
    Open -->|Yes| ShowDialog[Show File Dialog<br/>Non-blocking Thread]
    ShowDialog --> LoadFile[Load .ron File]
    LoadFile --> Parse[Parse RON Data]
    Parse --> Validate[Validate Map]
    Validate -->|Valid| UpdateState
    Validate -->|Invalid| ShowError[Show Error Dialog]
    
    Save -->|Yes| HasPath{Has File Path?}
    HasPath -->|No| ShowSaveDialog[Show Save Dialog<br/>Non-blocking Thread]
    HasPath -->|Yes| AutoExpand[Auto-Expand Dimensions<br/>Fit All Voxels]
    ShowSaveDialog --> AutoExpand
    AutoExpand --> ValidateMap[Validate Current Map]
    ValidateMap -->|Valid| SerializeMap[Serialize to RON]
    ValidateMap -->|Invalid| ShowError
    SerializeMap --> WriteFile[Write to File]
    WriteFile --> ClearDirty[Clear Modified Flag<br/>Update Window Title]
    
    UpdateState --> Render[Render Editor]
    ShowError --> Render
    ClearDirty --> Render
    Render --> End([Done])
```

## Tool System Architecture

```mermaid
classDiagram
    class EditorTool {
        <<enumeration>>
        VoxelPlace
        VoxelRemove
        EntityPlace
        Select
        Camera
    }
    
    class VoxelPlaceTool {
        +voxel_type: VoxelType
        +pattern: SubVoxelPattern
        +handle_click(pos)
        +preview_placement()
    }
    
    class VoxelRemoveTool {
        +handle_click(pos)
        +preview_removal()
    }
    
    class EntityPlaceTool {
        +entity_type: EntityType
        +handle_click(pos)
        +preview_placement()
    }
    
    class SelectTool {
        +selected_items: HashSet
        +handle_click(pos)
        +handle_drag(start, end)
    }
    
    class CameraTool {
        +orbit_speed: f32
        +pan_speed: f32
        +handle_drag(delta)
        +handle_zoom(delta)
    }
    
    EditorTool --> VoxelPlaceTool
    EditorTool --> VoxelRemoveTool
    EditorTool --> EntityPlaceTool
    EditorTool --> SelectTool
    EditorTool --> CameraTool
```

## State Management Structure

```mermaid
classDiagram
    class EditorState {
        +current_map: MapData
        +file_path: Option~PathBuf~
        +is_modified: bool
        +active_tool: EditorTool
        +selected_voxels: HashSet
        +selected_entities: HashSet
        +show_grid: bool
        +snap_to_grid: bool
    }
    
    class MapData {
        +metadata: MapMetadata
        +world: WorldData
        +entities: Vec~EntityData~
        +lighting: LightingData
        +camera: CameraData
        +custom_properties: HashMap
    }
    
    class EditorHistory {
        +undo_stack: Vec~EditorAction~
        +redo_stack: Vec~EditorAction~
        +max_history: usize
        +push(action)
        +undo()
        +redo()
        +clear()
    }
    
    class EditorAction {
        <<enumeration>>
        PlaceVoxel
        RemoveVoxel
        PlaceEntity
        RemoveEntity
        ModifyMetadata
    }
    
    EditorState --> MapData
    EditorState --> EditorHistory
    EditorHistory --> EditorAction
```

## UI Panel Layout

```mermaid
graph TB
    subgraph "Main Window"
        subgraph "Top Bar"
            Menu[Menu Bar<br/>File Edit View Tools Help]
            Toolbar[Quick Actions<br/>New Open Save Undo Redo]
        end
        
        subgraph "Content Area"
            subgraph "Left - Viewport 70%"
                Viewport[3D Viewport<br/>Interactive Map View]
                ViewControls[View Controls<br/>Grid Snap Camera]
            end
            
            subgraph "Right - Properties 30%"
                ToolPanel[Tool Settings<br/>Current Tool Options]
                VoxelPanel[Voxel Properties<br/>Type Pattern Position]
                EntityPanel[Entity Properties<br/>Type Position Custom]
                MapPanel[Map Info<br/>Metadata Dimensions]
            end
        end
        
        subgraph "Bottom Bar"
            Status[Status Bar<br/>Info Messages Validation]
        end
    end
```

## Rendering Pipeline

```mermaid
flowchart LR
    subgraph "Input"
        MapData[Map Data]
        EditorState[Editor State]
    end
    
    subgraph "Processing"
        VoxelSpawner[Voxel Spawner<br/>Reuse Game Code]
        GridGenerator[Grid Generator]
        CursorRenderer[Cursor Renderer]
    end
    
    subgraph "Bevy Rendering"
        Meshes[Mesh Components]
        Materials[Material Components]
        Transforms[Transform Components]
    end
    
    subgraph "Output"
        Screen[3D Viewport Display]
    end
    
    MapData --> VoxelSpawner
    EditorState --> GridGenerator
    EditorState --> CursorRenderer
    
    VoxelSpawner --> Meshes
    GridGenerator --> Meshes
    CursorRenderer --> Meshes
    
    Meshes --> Materials
    Materials --> Transforms
    Transforms --> Screen
```

## Key Design Patterns

### 1. Command Pattern (Undo/Redo)
Every editing action is encapsulated as a command that can be executed, undone, and redone.

### 2. State Pattern (Tools)
Different tools implement the same interface but behave differently based on the active tool state.

### 3. Observer Pattern (UI Updates)
UI panels observe the editor state and update automatically when state changes.

### 4. Strategy Pattern (Validation)
Different validation strategies can be applied based on map requirements.

### 5. Factory Pattern (Tool Creation)
Tools are created through a factory based on the selected tool type.

## Performance Considerations

### Optimization Strategies

1. **Lazy Rendering**
   - Only re-render when state changes
   - Use change detection for UI updates

2. **Infinite Grid System**
   - Camera-relative grid generation
   - Only renders visible grid sections (±50 units from camera)
   - Regenerates only when camera moves > 2 units
   - Aligned with voxel centers for accurate placement

3. **Spatial Partitioning**
   - Use spatial hash for voxel lookups
   - Frustum culling for large maps

4. **Batch Operations**
   - Group multiple edits into single history entry
   - Batch mesh updates for better performance

5. **Memory Management**
   - Limit history stack size
   - Use sparse data structures for voxels
   - Unload unused resources

## Error Handling Strategy

```mermaid
flowchart TD
    Operation[User Operation]
    
    Operation --> Validate{Validate Input}
    Validate -->|Invalid| ShowError[Show Error Message]
    Validate -->|Valid| Execute[Execute Operation]
    
    Execute --> Success{Success?}
    Success -->|No| LogError[Log Error]
    Success -->|Yes| UpdateState[Update State]
    
    LogError --> ShowError
    ShowError --> Recover[Attempt Recovery]
    Recover --> Continue[Continue Editing]
    
    UpdateState --> Continue
```

## Testing Strategy

### Unit Tests
- Tool behavior
- History system
- Validation logic
- File I/O operations

### Integration Tests
- UI interactions
- State management
- Rendering pipeline
- File operations

### End-to-End Tests
- Complete editing workflows
- Save/load cycles
- Undo/redo chains
- Error recovery

## Grid System Details

### Infinite Grid Architecture

The editor uses an infinite grid system that dynamically generates grid lines based on camera position:

**Key Features:**
- **Infinite Spanning**: Grid extends infinitely in all directions
- **Camera-Relative**: Only renders visible portion (configurable render distance)
- **Voxel Alignment**: Grid lines at integer coordinates (0, 1, 2, ...) align with voxel centers
- **Dynamic Regeneration**: Updates when camera moves beyond threshold
- **Major Grid Lines**: Every Nth line rendered with different color/opacity

**Configuration:**
```rust
InfiniteGridConfig {
    spacing: 1.0,              // Aligns with voxel positions
    render_distance: 50.0,     // Units from camera
    major_line_interval: 10,   // Every 10th line is major
    opacity: 0.3,              // Grid transparency
    regeneration_threshold: 2.0 // Camera movement threshold
}
```

**Performance:**
- Regenerates only when camera moves > 2 units
- Limits grid to visible area (typically 100x100 lines)
- Uses efficient LineList topology
- Minimal CPU overhead during static camera

## Input System Architecture (Updated October 2025)

The map editor uses a **unified, event-driven input architecture**:

### System Count Reduction

| Component | Before | After | Change |
|-----------|--------|-------|--------|
| Input Handler Systems | 7 | 1 | **-86%** |
| Transformation Systems | 8 | 1 | **-88%** |
| Rendering Systems | 3 | 3 | 0% |
| **Total Systems** | **18** | **5** | **-72%** |

### Key Components

1. **Unified Keyboard Handler** ([`handle_keyboard_input()`](../../../../src/editor/tools/input.rs:105))
   - Single entry point for all keyboard input
   - Context-aware key mapping based on mode
   - One UI focus check instead of 7+

2. **Transformation Operations** ([`handle_transformation_operations()`](../../../../src/editor/tools/input.rs:234))
   - Event-driven execution
   - Handles both keyboard and UI button events
   - Separated from input reading

3. **Event System** ([`EditorInputEvent`](../../../../src/editor/tools/input.rs:15))
   - Semantic events (StartMove, RotateDelta, etc.)
   - Decouples input reading from execution
   - Enables better testing and maintainability

### Benefits

- **Single Responsibility**: Input reading separated from execution
- **DRY Principle**: One UI focus check instead of 7+
- **Maintainability**: All shortcuts in one place
- **Testability**: Can test input mapping separately
- **Performance**: Fewer systems to run each frame

See [Input Refactoring Summary](archive/input-refactoring-summary.md) for complete details.

---

**Document Version**: 2.1.0
**Last Updated**: 2025-10-23
**Status**: Updated for save functionality with auto-expand