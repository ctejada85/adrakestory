# Requirements — Movement Frame Drop Spikes Fix

**Source:** Performance investigation — 2026-03-21
**Investigation:** `docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`
**Status:** Draft — pending team review

---

## 1. Overview

During gameplay on Windows, sustained player/camera movement causes frame-time spikes of 13–18 ms per frame across clusters of 10–20 consecutive frames (measured as `frame_cpu_us` in the frame profiler). Normal idle frames measure ~2 ms CPU. The spikes reduce effective FPS from ~140 to ~60 during movement and are perceptible as stutter.

Root cause analysis traced 99% of spike CPU time to Bevy's built-in PostUpdate / render-extraction pipeline — not to any instrumented game system. The dominant trigger is the `update_occlusion_uniforms` system calling `Assets::get_mut()` on a **single shared material** used by all 100–200 voxel chunks. Every such call marks the material asset as changed, forcing Bevy to re-extract the material for every chunk entity. With pipelined rendering, this creates a feedback loop where render-thread overhead stalls the next frame's main-thread sync point.

Secondary contributors include LOD mesh-handle swaps triggering per-entity change detection, missing `PresentMode` configuration amplifying vsync-related stutter on Windows DWM, and a one-time O(N) interior-detection cache rebuild after map load.

---

## 2. Data Domains

### 2.1 Frame Spike Categories

| Domain | Description | Frequency |
|--------|-------------|-----------|
| **Movement spikes** | 13–18 ms CPU per frame during sustained player/camera movement. Caused by shared-material mutation triggering mass render re-extraction. | Every movement frame |
| **LOD transition spikes** | Additive 2–5 ms when camera crosses LOD distance thresholds (50, 100, 200 units). Multiple chunks swap `Mesh3d` handles simultaneously. | On LOD boundary crossing |
| **Post-spawn cache rebuild** | Single 14.5 ms spike when `detect_interior_system` rebuilds its `HashSet<IVec3>` cache after map load/hot-reload. | Once per map load |
| **Frame pacing stutter** | Vsync deadline misses amplified by Windows DWM compositing. Not a CPU spike per se, but makes 13 ms spikes visually worse. | Continuous (Windows) |

**Requirement:** Each spike category must be addressed independently. Movement spikes (highest frequency, highest user impact) are Phase 1 priority.

---

## 3. Functional Requirements

### 3.1 Material Mutation Reduction

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | The `update_occlusion_uniforms` system must not call `Assets::get_mut()` when the computed uniform values are identical to the previously written values, thereby avoiding Bevy change-detection firing. | Phase 1 |
| FR-3.1.2 | Player and camera positions used for occlusion uniforms must be quantized to a configurable grid (default 0.25 world units) so that small sub-grid movements do not trigger material mutations. | Phase 1 |
| FR-3.1.3 | The quantization grid size must be configurable via `OcclusionConfig` so it can be tuned without code changes. | Phase 1 |
| FR-3.1.4 | When quantized positions match the previous frame's quantized positions, the system must skip all material access (both read and write) for zero overhead. | Phase 1 |

### 3.2 Render Extraction Overhead

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | When the shared `OcclusionMaterial` asset has not been mutated in a frame, Bevy's render extraction must not re-process any chunk entities for that material. (Achieved implicitly by FR-3.1.1 — no `get_mut()` = no change detection = no extraction.) | Phase 1 |
| FR-3.2.2 | The number of LOD mesh-handle swaps per frame must be capped to a configurable maximum (default: 8 chunks/frame) to bound the render extraction cost from `Mesh3d` change detection. | Phase 2 |
| FR-3.2.3 | Remaining LOD changes beyond the per-frame cap must be queued and applied across subsequent frames in priority order (nearest-to-camera first). | Phase 2 |

### 3.3 Frame Pacing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | The game window must configure an explicit `PresentMode` to control frame presentation timing on Windows. | Phase 1 |
| FR-3.3.2 | The chosen `PresentMode` must avoid blocking the main thread on vsync when the frame finishes early, to prevent coupling between render-thread latency and main-thread stalls. | Phase 1 |
| FR-3.3.3 | `PresentMode` must not introduce visible screen tearing in borderless fullscreen on Windows (DWM already composites, so `Mailbox`/`AutoNoVsync` should not tear). | Phase 1 |

### 3.4 Interior Detection Cache Rebuild

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | The `build_occupied_voxel_set` function must not process all spatial-grid cells in a single frame. | Phase 2 |
| FR-3.4.2 | Cache rebuild must be spread across multiple frames, processing a configurable number of cells per frame (default: 2000). | Phase 2 |
| FR-3.4.3 | During incremental rebuild, the system must use the stale cache (if available) rather than blocking on completion. | Phase 2 |
| FR-3.4.4 | The system must signal cache-rebuild completion so downstream consumers (flood-fill) operate on fresh data. | Phase 2 |

### 3.5 Profiling Instrumentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | A `profile_scope!` instrumentation point must be added to a PostUpdate system to measure time spent in Bevy's built-in PostUpdate phase (transform propagation, visibility, render extraction). | Phase 1 |
| FR-3.5.2 | The profiler CSV output must include the new PostUpdate label so future investigations can distinguish game-system time from framework time. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | Movement frame spikes must be reduced to ≤ 4 ms CPU per frame (from current 13–18 ms) on a Windows debug build. | Phase 1 |
| NFR-4.2 | Idle-frame CPU time must not regress beyond 10% of current baseline (~2 ms). | Phase 1 |
| NFR-4.3 | Visual correctness of the occlusion system must be preserved — quantized positions must not produce visible popping or discontinuities in the occlusion fade region. | Phase 1 |
| NFR-4.4 | All existing tests must continue to pass (`cargo test`). | Phase 1 |
| NFR-4.5 | Changes must be contained to the occlusion system, window configuration, interior detection, and diagnostics modules. No changes to the spawner chunk-mesh pipeline. | Phase 1 |
| NFR-4.6 | The LOD rate-limiting system (Phase 2) must not produce visible LOD popping when the camera moves at normal gameplay speeds. | Phase 2 |
| NFR-4.7 | Frame profiler overhead must remain negligible (< 50 µs per frame) after adding PostUpdate instrumentation. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- Quantize occlusion uniform positions to reduce `get_mut()` frequency (FR-3.1.1–FR-3.1.4)
- Set explicit `PresentMode::AutoNoVsync` on the game window (FR-3.3.1–FR-3.3.3)
- Add PostUpdate profiling instrumentation (FR-3.5.1–FR-3.5.2)
- Validate movement spikes reduced to ≤ 4 ms (NFR-4.1)

### Phase 2 — Enhanced

- Cap LOD mesh swaps per frame with priority queue (FR-3.2.2–FR-3.2.3)
- Incremental interior-detection cache rebuild (FR-3.4.1–FR-3.4.4)

### Future Phases

- Per-chunk material instances to eliminate shared-material extraction entirely
- GPU-driven occlusion uniforms via push constants (bypasses `Assets` change detection)
- Async render extraction (Bevy roadmap — depends on upstream changes)

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `Assets::get_mut()` always fires Bevy change detection regardless of whether the written value differs from the existing value. This is a Bevy invariant, not a bug. |
| 2 | Pipelined rendering is enabled via `DefaultPlugins` in Bevy 0.18. The render extraction sync point is the primary amplifier of per-frame render overhead. |
| 3 | All voxel chunks share a single `OcclusionMaterialHandle` (confirmed in `spawner/mod.rs:356` and `chunks.rs:238`). Changing this to per-chunk materials is out of scope for Phase 1. |
| 4 | Windows DWM composites borderless-fullscreen frames, so `PresentMode::AutoNoVsync` / `Mailbox` will not cause tearing. |
| 5 | The profiler only runs in debug builds (`#[cfg(debug_assertions)]`). Spike magnitude in release builds is expected to be lower but the proportional pattern is the same. |
| 6 | Quantizing positions to 0.25 units is below the visual perception threshold for the dithered occlusion fade effect (occlusion_radius is typically 3–8 units). |
| 7 | LOD distance thresholds are `[50.0, 100.0, 200.0, f32::MAX]`. Chunk count at each boundary varies by map but is typically 10–30 chunks per threshold. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | What quantization grid size provides the best trade-off between update frequency and visual smoothness? 0.25 units is the proposal — should this be validated with visual testing at different values? | Developer |
| 2 | Should `PresentMode` be a user-facing setting in the settings menu, or hardcoded? | Developer |
| 3 | Is the 8-chunks-per-frame LOD cap (Phase 2) sufficient for maps with 200+ chunks, or does it need to scale with chunk count? | Developer |
| 4 | Should the PostUpdate profiling instrumentation be permanent or removed after this investigation? | Developer |
| 5 | Are release-build frame spikes also user-visible, or is this primarily a debug-build issue amplified by `opt-level = 1`? A release-build profile should be captured to confirm. | Developer |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Investigation report (`docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`) | Done | Developer |
| 2 | Architecture document (`references/architecture.md`) | Done | Developer |
| 3 | `OcclusionUniforms` must implement `PartialEq` for read-only comparison (FR-3.1.1) | Already done (`#[derive(PartialEq)]` at `occlusion/mod.rs:130`) | — |
| 4 | `DynamicOcclusionUniforms` must implement `PartialEq` | Already done (`#[derive(PartialEq)]` at `occlusion/mod.rs:277`) | — |

---

## 9. Key Contacts

| Person | Role | Reach Out For |
|--------|------|---------------|
| Developer | Engineer | All implementation decisions |

---

## 10. Reference: Sample Scenarios

| Type | Example Verification Scenario | Complexity |
|------|-------------------------------|------------|
| Movement spike | Walk player continuously for 5 seconds — verify no frame exceeds 4 ms CPU | Medium |
| Idle baseline | Stand still for 5 seconds — verify frames stay at ~2 ms CPU | Low |
| LOD transition | Walk towards distant chunks crossing 200→100→50 unit thresholds — verify smooth LOD changes | Medium |
| Visual correctness | Walk in/out of buildings — verify occlusion fade is smooth with quantized positions | Medium |
| Post-spawn spike | Load a large map — verify the cache-rebuild spike is ≤ 5 ms (Phase 2) or documented (Phase 1) | Medium |
| Windows frame pacing | Run in borderless fullscreen on 60 Hz monitor — verify no visible stutter during movement | High |
| Profiler output | Run debug build — verify CSV contains `post_update` label with timing data | Low |

---

*Created: 2026-03-21*
*Source: Performance investigation — `docs/investigations/2026-03-21-1048-windows-frame-drop-spikes.md`*
