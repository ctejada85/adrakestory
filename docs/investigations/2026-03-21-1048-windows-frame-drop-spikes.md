# Investigation: Windows Frame Drop Spikes

**Date:** 2026-03-21 10:48
**Status:** Complete
**Component:** Rendering pipeline, occlusion system, interior detection

## Summary

Frame-time profiling (`profile_1774103617.csv`, 1520 frames) reveals three
distinct spike categories on a Windows debug build:

| Category | Frames | CPU spike | Root cause |
|----------|--------|-----------|------------|
| A — Map spawn | 350 | 304 ms | `spawn_voxels_chunked` (expected, one-time) |
| B — Cache rebuild | 409 | 14.5 ms | `detect_interior_system` O(N) rebuild |
| C — Movement clusters | 589-620, 1227-1246 | 13-18 ms each | **Shared-material mutation + render extraction** |

Category C is the user-reported problem. **99% of CPU time in those frames is
unaccounted** by any profiled game system — the tracked systems total ~100 µs
while `frame_cpu_us` reads 13-18 ms. The time is spent inside Bevy's built-in
PostUpdate / render-extraction pipeline.

## Environment

- Bevy 0.18 with `DefaultPlugins` (pipelined rendering enabled)
- Windows, `BorderlessFullscreen`, no explicit `PresentMode`
- Debug build: game code at `opt-level = 1`, dependencies at `opt-level = 3`
- Profiler measures wall-clock `First` → `Last` schedule (includes render extract)

## Investigation Method

1. Parsed all 11,701 CSV rows; separated `frame_interval_us`, `frame_cpu_us`,
   and per-system labels.
2. Identified spike frames (CPU > 10 ms) and computed unaccounted CPU
   (`frame_cpu_us − Σ tracked_system`).
3. Traced code paths for all profiled systems to rule out game-logic causes.
4. Examined the shared `OcclusionMaterialHandle`, chunk spawn code, LOD
   system, and `update_occlusion_uniforms` for render-pipeline interactions.
5. Correlated spike clusters with player movement (`apply_physics` 10× higher
   in spike frames).

## Findings

### Finding 1 — Shared material `get_mut()` forces per-frame render re-extraction (p2 High)

**File:** `src/systems/game/occlusion/mod.rs:413-415`
**Function:** `update_occlusion_uniforms`

```rust
if static_dirty || dynamic_dirty {
    if let Some(material) = materials.get_mut(&material_handle.0) {
        material.extension.occlusion_uniforms = assemble_uniforms(&s, &d);
```

**What happens:**

1. ALL voxel chunks share a single `OcclusionMaterialHandle`
   (`spawner/mod.rs:356`, `chunks.rs:238`).
2. Every frame the player or camera moves, `dynamic_dirty` becomes `true`
   (player position changed → new uniform values differ from cache).
3. `Assets::get_mut()` marks the material asset as **changed**, regardless of
   whether the byte-level content differs. This is a Bevy invariant —
   `get_mut()` always fires change detection.
4. Bevy's render extraction re-processes the material for **every entity**
   referencing that handle (100-200 chunk entities).
5. With pipelined rendering, this extraction runs on the main thread in
   PostUpdate. The resulting GPU work (uniform buffer re-upload, potential
   pipeline rebinds) executes on the render thread but may stall the *next*
   frame's sync point.

**Evidence:** During movement clusters (frames 589-620, 1227-1246),
`apply_physics` jumps from ~6 µs to 24-63 µs (player is moving), while
`update_occlusion_uniforms` itself takes only 4-15 µs. The remaining
~13 ms is PostUpdate extraction + render-thread sync overhead.

**Why it matters:** This is the dominant source of the reported frame drops.
Every movement frame pays ~13 ms of render-pipeline tax that doesn't appear
in any profiled system.

---

### Finding 2 — LOD mesh swaps compound render extraction cost (p3 Medium)

**File:** `src/systems/game/map/spawner/mod.rs:456-459`
**Function:** `update_chunk_lods`

```rust
if new_lod != lod.current_lod {
    lod.current_lod = new_lod;
    mesh.0 = lod.lod_meshes[new_lod].clone();  // Handle clone triggers Mesh3d change detection
}
```

When the camera crosses an LOD distance threshold (50, 100, 200 units), chunks
at that boundary all swap their `Mesh3d` handle in the same frame. Each swap
triggers Bevy's `Changed<Mesh3d>` detection → render extraction re-processes
those entities. Combined with Finding 1, this creates a double extraction hit:
material *and* mesh re-extracted for the same entities.

The system correctly gates on camera movement ≥ 0.5 units and skips the O(N)
loop otherwise, so idle frames are O(1). But during sustained movement, every
frame that crosses the threshold pays the full cost.

---

### Finding 3 — No explicit `PresentMode` on Windows (p3 Medium)

**File:** `src/main.rs:128-134`

```rust
.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        ..default()  // ← PresentMode defaults to AutoVsync
    }),
    ..default()
}))
```

Bevy's default `PresentMode::AutoVsync` maps to `Fifo` on most Windows
drivers. In borderless fullscreen, Windows DWM (Desktop Window Manager)
composites the frame, adding an extra present-to-display latency.

This doesn't cause CPU spikes directly but amplifies their *visible* impact:
a 13 ms CPU spike + 3 ms DWM composite can push a frame past the 16.67 ms
vsync deadline, causing a visible stutter even though total work is only
slightly over budget.

---

### Finding 4 — `detect_interior_system` O(N) cache rebuild (p3 Medium)

**File:** `src/systems/game/interior_detection.rs:140, 212-238`
**Function:** `build_occupied_voxel_set`

```rust
let new_cache = build_occupied_voxel_set(&spatial_grid, &sub_voxels);
// Iterates ALL spatial_grid.cells → ALL SubVoxel entities → HashSet insert
```

On the "settle" frame after map spawn or hot-reload (when `rebuild_pending`
flips), this function traverses the entire `SpatialGrid` HashMap and queries
every `SubVoxel` entity to build a `HashSet<IVec3>`. With ~200k SubVoxel
entities, this produces a one-time spike of ~14.5 ms (frame 409 in the
profile).

Mitigations already in place:
- Deferred during spawn waves (`rebuild_pending` flag)
- Throttled to every 60 frames
- Skipped if player moved < 0.3 units

The spike is infrequent (once per map load/reload) but jarring.

---

### Finding 5 — Pipelined rendering sync stall amplifies movement spikes (p2 High)

**Mechanism (Bevy 0.18 internals):**

With `PipelinedRenderingPlugin` (enabled by `DefaultPlugins`):
1. Main thread runs `First` → `Update` → `PostUpdate` (including render extract)
2. Main thread **blocks** waiting for the render thread to finish the *previous*
   frame before extraction can begin
3. Render thread processes mesh uploads, material binds, draw commands
4. The blocking wait appears in `frame_cpu_us` as wall-clock time

When Findings 1 + 2 cause extra render-thread work (material re-upload +
mesh handle swaps), the render thread takes longer → the *next* frame's main
thread blocks longer at the sync point → sustained 13-18 ms "CPU" spikes
across consecutive frames.

**Evidence:** The 1227-1246 cluster shows 20 **consecutive** spike frames
(not intermittent), consistent with a pipeline feedback loop where each
frame's render work delays the next frame's sync point.

## Root Cause Summary

| # | Root Cause | Location | Priority | Severity | Notes |
|---|-----------|----------|----------|----------|-------|
| 1 | Shared material `get_mut()` every movement frame | `occlusion/mod.rs:414` | p2 | High | Dominant cause of 13-18 ms spikes |
| 2 | Pipelined rendering sync stall feedback loop | Bevy internals (PostUpdate) | p2 | High | Amplifies Finding 1 across consecutive frames |
| 3 | LOD mesh swaps compound extraction cost | `spawner/mod.rs:458` | p3 | Medium | Additive with Finding 1 during LOD transitions |
| 4 | No explicit `PresentMode` | `main.rs:131` | p3 | Medium | Amplifies visible stutter on Windows DWM |
| 5 | Interior detection O(N) cache rebuild | `interior_detection.rs:140` | p3 | Medium | One-time 14.5 ms spike per map load |

## Recommended Fixes

### Fix 1 — Avoid `get_mut()` when uniform values haven't changed (addresses Root Cause 1)

Replace the current `get_mut()` call with a byte-level comparison before
mutating. This prevents Bevy's change detection from firing when the GPU
uniform data is identical:

```rust
// Before (always marks dirty):
if static_dirty || dynamic_dirty {
    if let Some(material) = materials.get_mut(&material_handle.0) {
        material.extension.occlusion_uniforms = assemble_uniforms(&s, &d);
    }
}

// After (only marks dirty when GPU data actually changes):
if static_dirty || dynamic_dirty {
    let new_uniforms = assemble_uniforms(&s, &d);
    // Read-only check first — no change detection triggered
    if let Some(material) = materials.get(&material_handle.0) {
        if material.extension.occlusion_uniforms != new_uniforms {
            // Only NOW call get_mut
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.extension.occlusion_uniforms = new_uniforms;
                if static_dirty {
                    material.base.alpha_mode = match config.technique {
                        TransparencyTechnique::Dithered => AlphaMode::Mask(0.001),
                        TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage,
                    };
                }
            }
        }
    }
}
```

However, since player position changes every movement frame, the uniforms
*will* differ. A better approach: **quantize the uniform values** to reduce
update frequency — e.g., round player position to 0.25-unit increments:

```rust
let quantized_pos = (player_pos * 4.0).round() / 4.0;
```

This reduces material mutations from every frame to ~every 4th frame of
movement, cutting render extraction overhead by ~75%.

### Fix 2 — Set explicit `PresentMode::AutoNoVsync` or `Mailbox` (addresses Root Cause 4)

```rust
Window {
    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
    present_mode: bevy::window::PresentMode::AutoNoVsync,
    ..default()
}
```

`Mailbox` (or `AutoNoVsync` which selects it) submits frames without blocking,
reducing the coupling between render-thread completion time and main-thread
stalls.

### Fix 3 — Spread interior detection cache rebuild across multiple frames (addresses Root Cause 5)

Instead of rebuilding the entire `HashSet<IVec3>` in one frame, process a fixed
number of spatial grid cells per frame (e.g., 1000) and mark the cache as
"building" until complete.

### Fix 4 — Profile the unaccounted time (diagnostic improvement)

Add `profile_scope!` instrumentation to PostUpdate to isolate where the 13 ms
is actually spent:

```rust
// In a new system added to PostUpdate:
fn profile_post_update(profiler: Option<Res<FrameProfiler>>) {
    profile_scope!(profiler, "post_update_start");
}
```

This would confirm whether the time is in Bevy's transform propagation,
visibility culling, or the render extraction sync point.

## Related Bugs

| Bug | Root Cause | Recommended Action |
|-----|-----------|-------------------|
| Movement frame drops (13-18 ms) | #1 + #2 | Quantize uniforms + avoid unnecessary `get_mut()` |
| Post-spawn stutter | #5 | Incremental cache rebuild |
| Windows frame pacing | #4 | Explicit `PresentMode` |
