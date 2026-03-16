# Requirements — Depth Prepass for OcclusionMaterial Fragment Overdraw

**Source:** Performance investigation — benchmarks `profile_1773690244.csv` / `profile_1773690261.csv`
**Status:** Draft

---

## 1. Overview

When many voxels are visible, the game's GPU fragment throughput becomes the primary bottleneck.
The `OcclusionMaterial` fragment shader uses `discard` to implement dithered transparency. On modern
GPUs, any shader containing `discard` disables the hardware's **early-Z** optimization: the driver
cannot determine whether a fragment will survive the shader before executing it. As a result, the
full PBR + occlusion WGSL shader runs for every overlapping fragment — not just the nearest visible
one. In a dense voxel scene with 3–5× overdraw, this multiplies expensive fragment shader invocations
by the same factor.

Adding Bevy's `DepthPrepass` component to the 3D camera resolves this. Bevy runs a lightweight
depth-only pass before the main forward pass, pre-populating the depth buffer. In the main pass,
fragments that fail the depth test are rejected **before** the fragment shader fires. The
`OcclusionMaterial` WGSL already contains `#ifdef PREPASS_PIPELINE` conditional logic, so
depth-prepass-aware rendering is structurally supported without a shader rewrite.

---

## 3. Functional Requirements

### 3.1 Depth Prepass Activation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | The game's 3D camera entity shall have `bevy::core_pipeline::prepass::DepthPrepass` inserted when spawned. | Phase 1 |
| FR-3.1.2 | `DepthPrepass` shall be inserted only on the game camera (`GameCamera` component), not on any UI or editor camera. | Phase 1 |
| FR-3.1.3 | The prepass shall remain active when the player is paused (`GameState::Paused`). | Phase 1 |

### 3.2 Shader Prepass Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | `OcclusionExtension` shall override `prepass_fragment_shader()` to return the same WGSL file (`shaders/occlusion_material.wgsl`). | Phase 1 |
| FR-3.2.2 | The dithered `discard` and region-based `discard` shall run identically in the prepass and main pass. The existing WGSL already handles this — all discards precede the `#ifdef PREPASS_PIPELINE` output branch. | Phase 1 |
| FR-3.2.3 | The depth buffer produced by the prepass shall accurately represent which fragments survive the dither and region checks, so the main pass can reject occluded fragments before executing the expensive PBR shader. | Phase 1 |

### 3.3 Visual Correctness

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | Visual output (dithering pattern, occlusion fade, region hide) shall be identical with and without `DepthPrepass`. | Phase 1 |
| FR-3.3.2 | No z-fighting artefacts shall appear on chunk surfaces after enabling the prepass. | Phase 1 |
| FR-3.3.3 | Shadow rendering (when `ShadowQuality != None`) shall be unaffected by the depth prepass change. | Phase 1 |

### 3.4 Hot-Reload Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | When the map is hot-reloaded, the new 3D camera entity shall receive `DepthPrepass` automatically because it is part of the spawn bundle. | Phase 1 |

### 3.5 Performance Measurement

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | Before and after benchmarks shall be captured using the existing `FrameProfiler` to confirm a measurable improvement in `frame_interval_us` p95 when overdraw is present. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | The depth prepass shall not degrade average FPS in scenes with low overdraw (e.g., open sky view). | Phase 1 |
| NFR-4.2 | Implementation shall compile without warnings on macOS with Apple Silicon (TBDR GPU). | Phase 1 |
| NFR-4.3 | No new Bevy plugins or crates are required; `DepthPrepass` is part of `bevy::core_pipeline`. | Phase 1 |
| NFR-4.4 | The change shall not alter the editor binary's camera behaviour. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- Add `DepthPrepass` to the game camera spawn bundle
- Override `prepass_fragment_shader()` in `OcclusionExtension` to use the same WGSL — ensures dither and region discards run in the prepass, giving accurate depth
- Confirm visual output is unchanged
- Run benchmarks to measure improvement

### Phase 2 — Enhanced

- Evaluate switching `TransparencyTechnique::Dithered` to `AlphaMode::Mask` (alpha cutout) which
  participates in a depth prepass with correct discard semantics natively and requires no
  `prepass_fragment_shader()` override

### Future Phases

- Evaluate switching `TransparencyTechnique::Dithered` to `AlphaMode::Mask` (alpha cutout) which
  participates in a depth prepass with correct discard semantics natively

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `ExtendedMaterial<StandardMaterial, OcclusionExtension>` automatically participates in Bevy's depth prepass pipeline because `StandardMaterial` registers a prepass shader. Verify at runtime. |
| 2 | The game runs in **forward rendering** mode (not deferred). `DepthPrepass` is a forward-pass optimization. |
| 3 | The editor camera is a separate entity with `EditorCamera` marker component — it must not receive `DepthPrepass`. |
| 4 | Apple TBDR GPUs (M-series) handle depth prepass differently than desktop GPUs (tile-based deferred rasterization); the prepass may have negligible overhead on macOS, making the net win larger. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | Does `ExtendedMaterial<StandardMaterial, OcclusionExtension>` register a prepass variant automatically, or does `OcclusionExtension` need to implement `prepass_fragment_shader()`? | Developer |
| 2 | Does the dithered discard run during the prepass? If so, the prepass depth may have "holes" matching the dither pattern — acceptable or needs fixing? | Developer |
| 3 | Is there z-fighting risk given that depth prepass uses `AlphaMode::Opaque` for dithered surfaces while main pass also uses `AlphaMode::Opaque`? | Developer |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `FrameProfiler` (debug benchmarks) must be present to measure improvement | Done | — |
| 2 | `ShadowQuality` settings must be stable before benchmarking prepass effect | Done | — |

---

*Created: 2026-03-16*
*Source: Performance investigation — benchmark profile CSVs, WGSL shader analysis*
