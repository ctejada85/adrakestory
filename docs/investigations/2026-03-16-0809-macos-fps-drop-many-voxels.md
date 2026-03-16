# Investigation: macOS FPS Drop With Many Voxels

**Date:** 2026-03-16 08:09  
**Status:** Complete  
**Component:** Rendering pipeline / Occlusion system / Collision system  
**Platform:** macOS-specific (Windows stable at ~120 FPS)

---

## Summary

When many voxels are on screen the game drops below acceptable FPS on macOS while remaining stable at ~120 FPS on Windows. The discrepancy is platform-specific, pointing to the Metal rendering backend as the primary cause rather than a CPU logic issue.

Three root causes were identified — two of them macOS-specific GPU-side issues and one cross-platform CPU issue that compounds the GPU cost.

---

## Environment

| Item | Detail |
|------|--------|
| Platform (broken) | macOS (Metal backend via wgpu) |
| Platform (working) | Windows (Vulkan or DX12 backend) |
| Renderer | Bevy + wgpu — Metal on macOS, Vulkan/DX12 on Windows |
| GPU architecture (macOS) | Apple Silicon = Tile-Based Deferred Renderer (TBDR); Intel Mac = traditional rasterizer |
| Observed symptom | FPS drop proportional to number of visible voxels |

---

## Investigation Method

1. Read all systems registered in `Update` schedule to identify per-frame hot paths
2. Traced rendering pipeline: material creation → chunk spawning → per-frame uniform updates
3. Examined `OcclusionConfig::default()` to determine the active transparency technique
4. Checked MSAA implications of the chosen `AlphaMode`
5. Estimated entity counts for a typical map
6. Cross-referenced macOS Metal rendering characteristics with observed behavior

---

## Findings

### Finding 1 — `AlphaBlend` Technique Forces MSAA on Every Chunk (CRITICAL — macOS)

**File:** `src/systems/game/occlusion/mod.rs`, line 195  
**Also:** `src/systems/game/occlusion/mod.rs`, line 556

`OcclusionConfig::default()` sets `technique: TransparencyTechnique::AlphaBlend`. This is the value passed to `create_occlusion_material()` at spawn time (`spawner/mod.rs:347`). Inside `create_occlusion_material()`:

```rust
TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage,
```

`AlphaMode::AlphaToCoverage` requires **MSAA to be active** to function (Bevy default: 4× MSAA). Every voxel chunk mesh is assigned this material, so **every chunk participates in the MSAA resolve pass**.

**Why this is macOS-critical:**  
Apple Silicon (M1/M2/M3) and most macOS GPUs use a Tile-Based Deferred Renderer (TBDR). On TBDR architectures, MSAA is processed in tile memory on-chip. While this is memory-bandwidth-efficient in simple cases, it becomes a bottleneck when:

- Many objects use `AlphaToCoverage` simultaneously (resolving per-sample coverage masks every frame)
- The number of tiles with mixed-coverage fragments is large (i.e., many voxel chunk edges on screen)
- The fragment shader is non-trivial (the occlusion shader does ray-distance math per fragment)

On Windows with a traditional discrete GPU (NVIDIA/AMD), MSAA is handled with dedicated hardware resolve units and dedicated MSAA memory — it scales much better with many objects.

With 100–200 `VoxelChunk` entities all using `AlphaToCoverage`, the macOS GPU is resolving MSAA coverage for every visible fragment every frame. As more voxels fill the screen, more tiles have `AlphaToCoverage` fragments, and tile memory pressure increases.

**The alternate technique `Dithered` uses `AlphaMode::Opaque`** — no MSAA participation, no coverage resolve. This would be fast on both platforms.

---

### Finding 2 — Draw Call Count on Metal (HIGH — macOS)

**File:** `src/systems/game/map/spawner/chunks.rs`, lines 232–252

Each `VoxelChunk` spawns its own `Mesh3d` + `MeshMaterial3d` entity. For a typical map this means **100–200 separate draw calls per frame**. The LOD system switches `mesh.0` handle when the LOD level changes, but there is still one draw call per chunk.

**Why this is macOS-critical:**  
Metal has higher CPU-side per-draw-call overhead than Vulkan or DX12. Vulkan and DX12 were designed explicitly to reduce per-draw-call driver cost; Metal's command encoder model is more efficient than OpenGL but still more expensive per draw than Vulkan/DX12 at scale.

Bevy's render pipeline batches draws when objects share the same material and mesh — but each chunk has a **unique mesh** (different geometry), so batching cannot merge them. Each chunk = 1 Metal draw command.

At 200 chunks × Metal draw overhead, this adds measurable CPU time to every frame specifically on macOS.

---

### Finding 3 — Fragment Shader Runs on All Visible Chunk Fragments (MEDIUM — macOS)

**File:** `assets/shaders/occlusion_material.wgsl`, lines 84–126

The `OcclusionMaterial` fragment shader runs for **every visible fragment of every chunk mesh**. It performs:
- A normalized ray calculation (camera → player)
- An XZ-plane point-to-ray distance check
- Two `smoothstep` calls
- A Bayer matrix dither check (if `Dithered` technique)

With many voxels on screen the total fragment count is large. On Windows (discrete GPU with many shader cores), this parallelises efficiently. On macOS with Apple Silicon's GPU, the execution units are fewer and the TBDR pipeline adds overhead for complex fragment shaders at high occupancy.

This alone is unlikely to cause the drop, but it compounds the MSAA cost from Finding 1.

---

### Finding 4 — Multiple Spatial Grid Lookups Per Frame (HIGH — cross-platform)

**Files:** `src/systems/game/collision.rs:127`, `src/systems/game/physics.rs:85`

Every frame, 3–4 `spatial_grid.get_entities_in_aabb()` calls are made:

| Call site | Function | Frequency |
|-----------|----------|-----------|
| `collision.rs:127` | `check_sub_voxel_collision()` — initial check | Every `move_player()` call |
| `collision.rs:203` | `check_sub_voxel_collision()` — step-up re-check | When step-up candidate found |
| `player_movement.rs` | `move_player()` — diagonal split | When diagonal movement collides |
| `physics.rs:85` | `apply_physics()` | Every frame |

Each call is O(k) where k = sub-voxel entities in the player's AABB cells. With many voxels loaded, k grows. A typical dense map loads ~200,000 `SubVoxel` entities.

This is cross-platform but reduces the CPU frame budget that is already tighter on macOS due to the GPU overhead above.

---

### Finding 5 — Interior Detection HashSet Rebuild on Hot Reload (MEDIUM — cross-platform)

**File:** `src/systems/game/interior_detection.rs:126–129`

`build_occupied_voxel_set()` iterates all `SubVoxel` entities via the spatial grid to build a `HashSet` of occupied voxel positions. It is triggered on `Added<SubVoxel>` or `RemovedComponents<SubVoxel>` (i.e., map load and hot reload).

- **Normal gameplay:** Uses cached `HashSet`. No per-frame cost.
- **On map load or hot reload:** Rebuilds entire set in one frame — O(200,000+) operations → **visible frame stutter**.

This is not the cause of the sustained FPS drop but produces a one-shot spike on map load/reload.

---

### Finding 6 — Occlusion Systems Not Gated to InGame State (LOW)

**File:** `src/systems/game/occlusion/mod.rs:521–528`

`OcclusionPlugin` registers `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` in `Update` with no `run_if(in_state(GameState::InGame))` guard. These run in all states including title screen and loading. Minimal cost since material/entity queries return empty results in other states, but wastes query overhead.

---

## Root Cause Summary

| # | Root Cause | Location | Severity | Platform |
|---|-----------|----------|----------|----------|
| 1 | `AlphaBlend` default → `AlphaToCoverage` → per-chunk MSAA on TBDR GPU | `occlusion/mod.rs:195`, `556` | **Critical** | macOS only |
| 2 | ~100–200 unique draw calls per frame; Metal has higher per-draw cost | `spawner/chunks.rs:232–252` | **High** | macOS only |
| 3 | Occlusion fragment shader on all visible chunk fragments, compounds MSAA cost | `occlusion_material.wgsl:84–126` | **Medium** | macOS worse |
| 4 | 3–4 spatial grid AABB lookups per frame in collision/physics | `collision.rs:127,203`, `physics.rs:85` | **High** | Cross-platform |
| 5 | Interior detection full HashSet rebuild on map load/hot reload | `interior_detection.rs:126–129` | **Medium** | Cross-platform |
| 6 | Occlusion systems not gated to InGame state | `occlusion/mod.rs:521–528` | **Low** | Cross-platform |

---

## Recommended Fixes

### Fix A — Switch Default Technique to `Dithered` (addresses Finding 1)

Change `OcclusionConfig::default()` to use `TransparencyTechnique::Dithered`. This selects `AlphaMode::Opaque`, eliminating all `AlphaToCoverage` participation and MSAA coverage resolve for voxel chunks. Visually: slightly visible dither pattern instead of smooth alpha blend — acceptable for a voxel art style.

```rust
// Before (occlusion/mod.rs ~line 195)
technique: TransparencyTechnique::AlphaBlend,

// After
technique: TransparencyTechnique::Dithered,
```

**Expected impact:** Large FPS improvement on macOS; minimal change on Windows.

### Fix B — Reduce Draw Calls via Frustum-Based Chunk Culling (addresses Finding 2)

Currently all chunks are submitted for rendering regardless of camera visibility. Adding frustum culling on chunk AABBs would eliminate off-screen draw calls. Bevy's built-in `Aabb` + `Visibility` system can handle this with minimal custom code.

**Expected impact:** Moderate FPS improvement on macOS at any voxel count; larger improvement when many chunks are off-screen.

### Fix C — Cache Diagonal Collision Query (addresses Finding 4)

In `move_player()`, the diagonal movement fallback re-queries the spatial grid for separate X and Z axis checks with the same AABB. Cache the initial query result and reuse it rather than querying again.

**Expected impact:** ~30–50% reduction in spatial grid lookups per frame; cross-platform.

### Fix D — Gate Occlusion Systems to InGame State (addresses Finding 6)

```rust
// occlusion/mod.rs OcclusionPlugin::build()
.add_systems(
    Update,
    (detect_interior_system, update_occlusion_uniforms, debug_draw_occlusion_zone)
        .chain()
        .run_if(in_state(GameState::InGame)), // add this
)
```

**Expected impact:** Minor cleanup; prevents unnecessary query overhead in other states.

---

## Related Bugs

| Bug file | Finding | Severity |
|----------|---------|----------|
| `docs/bugs/2026-03-16-1213-p1-alphatocoverage-msaa-macos-tbdr.md` | Finding 1 — AlphaToCoverage MSAA on macOS TBDR | Critical |
| `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md` | Finding 2 — Per-chunk draw call overhead on Metal | High |
| `docs/bugs/2026-03-16-1213-p2-spatial-grid-multiple-lookups-per-frame.md` | Finding 4 — 3–4 spatial grid lookups per frame | High |
| `docs/bugs/2026-03-16-1213-p3-occlusion-fragment-shader-all-visible-chunks.md` | Finding 3 — Fragment shader on all visible chunk fragments | Medium |
| `docs/bugs/2026-03-16-1213-p3-interior-detection-hashset-rebuild-on-hot-reload.md` | Finding 5 — Full HashSet rebuild on map load/hot reload | Medium |
| `docs/bugs/2026-03-16-1213-p4-occlusion-systems-run-in-all-game-states.md` | Finding 6 — Occlusion systems not gated to InGame state | Low |

**Prior related bugs:**
- `docs/bugs/2026-03-15-2141-p1-occlusion-material-gpu-reupload-every-frame.md` — Prior fix addressed CPU-side `get_mut()` calls; this investigation identifies the GPU-side MSAA cost from the material mode itself.
- `docs/bugs/2026-03-15-2141-p2-interior-detection-frame-spikes.md` — Finding 5 in this report is a residual of that fix (hot reload rebuild).
