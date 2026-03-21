# Epic: Fix Movement Frame Drop Spikes

**Date:** 2026-03-21
**Type:** Epic
**Priority:** p2
**Component:** Occlusion system / frame pacing / diagnostics
**Investigation:** `docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`

---

## Overview

Player movement causes 13–18 ms frame spikes on Windows, resulting in visible stuttering during otherwise smooth gameplay. Profiling reveals that all voxel chunks share a single `OcclusionMaterial` handle, and every frame the player moves, `Assets::get_mut()` fires Bevy change detection. This forces render extraction to re-process 100–200 chunk entities, stalling the main thread at the pipelined rendering sync point. This epic eliminates the per-frame material mutation overhead through position quantization, improves frame pacing with an explicit `PresentMode`, and adds instrumentation to detect future regressions.

---

## Epic Story

As a player, I want smooth frame pacing during movement so that walking and running never produces visible stuttering or hitching.

---

## Child Stories

| # | Title | Phase | Status |
|---|-------|-------|--------|
| 1 | Quantize occlusion uniform positions | Phase 1 (MVP) | Backlog |
| 2 | Set explicit PresentMode | Phase 1 (MVP) | Backlog |
| 3 | Add PostUpdate profiling scope | Phase 1 (MVP) | Backlog |
| 4 | Rate-limit LOD mesh swaps | Phase 2 | Backlog |
| 5 | Incremental interior cache rebuild | Phase 2 | Backlog |

---

## Story 1 — Quantize Occlusion Uniform Positions

As a player, I want the occlusion system to avoid redundant GPU material updates during movement so that walking does not cause frame spikes.

### Description

Currently, `update_occlusion_uniforms` calls `materials.get_mut()` every frame the player or camera moves, because the raw floating-point positions always differ slightly. This fires Bevy's change detection for the shared `OcclusionMaterial` and forces render extraction of all 100–200 chunk entities. The fix quantizes player and camera positions to a configurable grid (default 0.25 units) so that positions only change meaningfully every ~8–17 frames at typical movement speeds. A read-only `materials.get()` comparison is added before `get_mut()` as a second layer of protection.

### Acceptance Criteria

1. `update_occlusion_uniforms` calls `materials.get_mut()` **only** when quantized uniform values differ from the currently stored GPU values.
2. When the player moves at walking speed (~2 units/s), `get_mut()` is called ≤ 10 times/second (not every frame).
3. When the player is stationary, `get_mut()` is **never** called.
4. Occlusion fade-in/fade-out remains visually indistinguishable from the current behavior at the default quantization step of 0.25 units.
5. The quantization step is configurable via `OcclusionConfig::uniform_quantization_step`.
6. All existing unit tests pass (`cargo test`).
7. A new unit test verifies `quantize_position()` correctness for positive, negative, zero, and sub-step inputs.

### Non-Functional Requirements

- `quantize_position()` must be `#[inline]` and have zero heap allocation.
- The combined overhead of quantize + cache comparison must be < 5 µs per frame in release builds.
- No changes to the GPU shader (uniform layout unchanged).

### Tasks

1. Add `uniform_quantization_step: f32` field to `OcclusionConfig` with default `0.25` in `src/systems/game/occlusion/mod.rs`.
2. Implement `quantize_position(pos: Vec3, step: f32) -> Vec3` helper function.
3. Modify `update_occlusion_uniforms` to quantize player and camera positions before computing `DynamicOcclusionUniforms`.
4. Add read-only `materials.get()` comparison before `materials.get_mut()` — only call `get_mut()` when the final `OcclusionUniforms` struct differs (`PartialEq` already derived).
5. Update local caches (`dynamic_cache`, `static_cache`) unconditionally so next-frame dirty checks remain accurate.
6. Write unit tests for `quantize_position()`.
7. Manual verification: run `cargo run --release`, walk around map, confirm no visible occlusion stepping or popping at 0.25 step.

---

## Story 2 — Set Explicit PresentMode

As a player, I want the game window to use a non-blocking present mode so that frame pacing is not degraded by vsync-related stalls on Windows.

### Description

The window configuration in `main.rs` does not set `present_mode`, so Bevy defaults to `PresentMode::AutoVsync` (maps to `Fifo` on most drivers). On Windows, `Fifo` can introduce extra latency at the compositor level, compounding the render extraction stall into a visible hitch. Switching to `AutoNoVsync` (prefers `Mailbox` when available, falls back to `Immediate`) eliminates the compositor-side blocking while still allowing the display to pick the best non-blocking mode.

### Acceptance Criteria

1. The `Window` configuration in `main.rs` explicitly sets `present_mode: PresentMode::AutoNoVsync`.
2. The game renders without screen tearing artifacts on the primary development target (Windows with DWM compositor).
3. No import or compilation errors from the added `PresentMode` usage.
4. `cargo clippy` reports no new warnings.

### Non-Functional Requirements

- Single-line change — no new structs or systems.
- Applies to both debug and release builds.

### Tasks

1. Add `PresentMode` to the `bevy::window` import in `src/main.rs`.
2. Add `present_mode: PresentMode::AutoNoVsync` to the `Window` struct.
3. Build and run on Windows to verify no visible tearing (DWM handles compositing).
4. Run `cargo clippy` to confirm no warnings.

---

## Story 3 — Add PostUpdate Profiling Scope

As a developer, I want to measure time spent in Bevy's PostUpdate schedule so that I can attribute frame spikes to render extraction and sync stalls.

### Description

The current `FrameProfiler` records `frame_cpu_us` (wall-clock from `First` to `Last`) and per-system scopes for game systems, but 99% of spike-frame CPU time is unaccounted because PostUpdate (transform propagation, visibility culling, render extraction, pipelined sync) is not instrumented. Adding a profiling scope in PostUpdate closes this observability gap.

### Acceptance Criteria

1. A new `post_update` column appears in the profiler CSV output when running a debug build.
2. The `post_update` value captures wall-clock time for the PostUpdate schedule (± scheduling jitter).
3. The profiling scope is `#[cfg(debug_assertions)]`-gated (no release build overhead).
4. Existing profiler output remains backward-compatible (new column is additive).
5. `cargo test` passes.

### Non-Functional Requirements

- Must use the existing `profile_scope!` macro pattern — no new profiling infrastructure.
- Must not alter system ordering or introduce any game-logic side effects.
- Debug-build only (`#[cfg(debug_assertions)]`).

### Tasks

1. Add a `profile_post_update` system to `src/diagnostics/mod.rs` that uses `profile_scope!`.
2. Register the system in `PostUpdate` schedule inside `FrameProfilerPlugin::build()`.
3. Run a debug build and capture CSV output; verify the `post_update` column appears with non-zero values.
4. Run `cargo test` to confirm no regressions.

---

## Story 4 — Rate-Limit LOD Mesh Swaps

As a player, I want LOD transitions to be spread across frames so that crossing an LOD boundary does not cause a one-frame spike from bulk mesh swaps.

### Description

When the player crosses an LOD distance threshold, `update_chunk_lods` swaps the `Mesh3d` handle for all affected chunks in a single frame. Each swap triggers render extraction (mesh rebind), and a threshold crossing can affect 20–50 chunks simultaneously. Rate-limiting to N swaps per frame (e.g., 8) bounds the per-frame cost and spreads the transition over 2–6 frames. Chunks are prioritized by distance to camera (closest first) for visual quality.

### Acceptance Criteria

1. A new `LodUpdateQueue` resource queues pending LOD transitions.
2. `update_chunk_lods` processes at most `max_lod_swaps_per_frame` mesh swaps per frame (configurable, default 8).
3. Remaining swaps are deferred to subsequent frames in distance-sorted order (closest first).
4. LOD transitions complete within 1 second under normal gameplay conditions.
5. No visual artifacts from partial LOD states (chunks at mixed LODs are acceptable during transition).
6. Existing `update_chunk_lods` tests pass.

### Non-Functional Requirements

- Queue processing cost < 10 µs per frame (sorting + dequeue) in release builds.
- Queue must drain completely — no stuck entries.
- `max_lod_swaps_per_frame` configurable via `LodConfig`.

### Tasks

1. Create `LodUpdateQueue` resource (priority queue sorted by camera distance).
2. Modify `update_chunk_lods` in `src/systems/game/map/spawner/mod.rs` to enqueue all needed transitions.
3. Add per-frame dequeue loop with configurable cap (`LodConfig::max_lod_swaps_per_frame`, default 8).
4. Add `max_lod_swaps_per_frame` field to `LodConfig`.
5. Write unit tests for queue ordering and per-frame cap behavior.
6. Manual verification: walk across LOD boundaries in a large map; confirm no single-frame spike > 5 ms from LOD.

---

## Story 5 — Incremental Interior Cache Rebuild

As a player, I want interior detection cache rebuilds to be spread across frames so that entering a new region does not cause a 14 ms stall.

### Description

When `detect_interior_system` determines a cache rebuild is needed (player moved > 0.3 units since last build), it calls `build_occupied_voxel_set()` which iterates all sub-voxels in the loaded map to build a `HashSet<IVec3>`. For large maps this takes 14+ ms in a single frame. Converting this to an incremental state machine that processes N voxels per frame (e.g., 5000) spreads the cost across multiple frames.

### Acceptance Criteria

1. `build_occupied_voxel_set` is replaced with an `IncrementalCacheBuilder` that processes a bounded number of voxels per frame.
2. The per-frame cost of incremental building is < 2 ms in debug builds.
3. While the cache is rebuilding, the previous cache remains active (no gap in interior detection).
4. The completed cache produces identical results to the current single-frame build.
5. `detect_interior_system` unit tests pass.
6. A new unit test verifies incremental builder produces the same set as the one-shot builder.

### Non-Functional Requirements

- The previous cache must remain active during multi-frame rebuild (no detection gap).
- Incremental chunk size is configurable (default 5000 voxels/frame).
- Memory: only one complete cache + one partial builder in memory at any time.

### Tasks

1. Create `IncrementalCacheBuilder` resource in `src/systems/game/interior_detection.rs`.
2. Implement state machine: `Idle` → `Building { iterator, partial_set }` → `Complete`.
3. Modify `detect_interior_system` to start a builder instead of calling `build_occupied_voxel_set()`.
4. Add per-frame progress function that processes N sub-voxels and checks for completion.
5. On completion, swap new cache into `InteriorState::occupied_voxels_cache` atomically.
6. Write unit test comparing incremental result against one-shot result for a known map.
7. Manual verification: load a large map, walk to trigger rebuild, confirm no single-frame spike > 3 ms from interior detection.

---

## Epic Acceptance Criteria

1. All five child stories are complete and individually verified.
2. Movement at walking speed (~2 units/s) produces no frame spikes > 5 ms on Windows in a release build, measured over 60 seconds of continuous movement.
3. Movement at running speed (~5 units/s) produces no frame spikes > 8 ms on Windows in a release build.
4. Stationary gameplay produces zero `get_mut()` calls on the occlusion material per frame.
5. `cargo test` passes with no regressions.
6. `cargo clippy` reports no new warnings.
7. Profiler CSV includes `post_update` column for future observability.

---

## Epic Non-Functional Requirements

- All changes are backward-compatible — no map format changes, no save-breaking changes, no shader layout changes.
- Phase 1 (Stories 1–3) is deployable independently of Phase 2 (Stories 4–5).
- No new crate dependencies introduced.
- Performance improvements must be measurable via the existing profiling framework (`FrameProfiler`).

---

## Dependencies & Risks

| # | Item | Type | Status | Notes |
|---|------|------|--------|-------|
| 1 | `OcclusionUniforms` has `#[derive(PartialEq)]` | Dependency | ✅ Done | Verified at `occlusion/mod.rs:130` |
| 2 | `DynamicOcclusionUniforms` has `#[derive(PartialEq)]` | Dependency | ✅ Done | Verified at `occlusion/mod.rs:277` |
| 3 | Bevy 0.18 `Assets::get()` does not fire change detection | Dependency | ✅ Done | Confirmed — only `get_mut()` fires change detection |
| 4 | Quantization step 0.25 may produce visible occlusion stepping | Risk | Open | Mitigation: manual visual test at 0.125, 0.25, 0.5; make configurable |
| 5 | `AutoNoVsync` may cause tearing on non-DWM compositors | Risk | Open | Mitigation: DWM prevents tearing on Windows 10+; Linux/macOS testing deferred |
| 6 | Release-build spike magnitude unknown | Risk | Open | Mitigation: capture release profile before declaring fix complete |
| 7 | LOD queue could starve if player moves rapidly through many thresholds | Risk | Open | Mitigation: cap queue length; force-flush if queue exceeds 100 entries |

---

## Related

- **Investigation:** `docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`
- **Requirements:** `docs/bugs/fix-movement-frame-drops/references/requirements.md`
- **Architecture:** `docs/bugs/fix-movement-frame-drops/references/architecture.md`
