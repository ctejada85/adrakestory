# Requirements — Fix: Occlusion Material GPU Re-Upload Every Frame

**Source:** Bug report [2026-03-15-2141-p1-occlusion-material-gpu-reupload-every-frame.md](../2026-03-15-2141-p1-occlusion-material-gpu-reupload-every-frame.md) — 2026-03-15  
**Status:** Draft — pending team review

---

## 1. Overview

The `update_occlusion_uniforms` system in `src/systems/game/occlusion.rs` calls `Assets::get_mut()` unconditionally on every frame, even when the underlying uniform values (player position, camera position, occlusion config) have not changed. In Bevy, `get_mut()` always marks the asset as modified regardless of whether any data was actually written. This causes the render world to re-extract and re-prepare `OcclusionMaterial` bind groups on every tick, resulting in unnecessary GPU re-uploads and degraded frame rate in the game binary.

The fix introduces change detection at the system level: compute the new `OcclusionUniforms` value first, compare it against a cached copy from the previous frame, and only call `get_mut()` — and therefore only trigger a GPU re-upload — when the values differ. This aligns GPU work with actual state changes rather than performing it unconditionally every frame.

Additionally, `OcclusionUniforms` is split into two private Rust-side helper structs — `StaticOcclusionUniforms` (config-driven fields that change only when `OcclusionConfig` is mutated) and `DynamicOcclusionUniforms` (positional fields that change when the player or camera moves or interior state changes). Each sub-struct has its own `Local` cache, enabling finer-grained dirty tracking: a camera movement does not require comparing config fields, and a config change does not require comparing transform fields. The GPU-facing `OcclusionUniforms` struct and its binding are unchanged.

---

## 2. Data Domains

Not applicable — this fix operates on a single uniform struct (`OcclusionUniforms`) within one system.

---

## 3. Functional Requirements

### 3.1 Change Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | The system must compute `StaticOcclusionUniforms` and `DynamicOcclusionUniforms` separately before accessing the material asset. | Phase 1 |
| FR-3.1.2 | The system must cache last-written values in `Local<Option<StaticOcclusionUniforms>>` and `Local<Option<DynamicOcclusionUniforms>>` separately. | Phase 1 |
| FR-3.1.3 | The system must skip calling `materials.get_mut()` when both sub-struct caches match the newly computed values. | Phase 1 |
| FR-3.1.4 | The system must call `materials.get_mut()` and write the full assembled `OcclusionUniforms` only when at least one sub-struct cache differs. | Phase 1 |
| FR-3.1.5 | The system must update the relevant `Local` cache(s) whenever new uniforms are written. | Phase 1 |

### 3.2 Equality Check

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | `StaticOcclusionUniforms` and `DynamicOcclusionUniforms` must each derive `PartialEq`. | Phase 1 |
| FR-3.2.2 | `OcclusionUniforms` (the GPU-facing struct) must also derive `PartialEq` for consistency, even though comparison is now performed on the sub-structs. | Phase 1 |

### 3.3 Bevy Resource Change Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | The system must gate static field computation on `config.is_changed()`, skipping `StaticOcclusionUniforms` recomputation when config has not changed. | Phase 1 |
| FR-3.3.2 | The system must gate dynamic field computation on `Ref<Transform>` change detection (camera and player) and `interior_state.is_changed()`, skipping `DynamicOcclusionUniforms` recomputation when none of these have changed. | Phase 1 |
| FR-3.3.3 | The `Local` sub-struct cache comparisons (FR-3.1.3) must be retained alongside the Bevy change-detection gates as a safety fallback, since Bevy change detection can produce false positives on startup and after state transitions. | Phase 1 |

### 3.4 Uniform Split

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | `StaticOcclusionUniforms` must contain all fields sourced from `OcclusionConfig`: `min_alpha`, `occlusion_radius`, `height_threshold`, `falloff_softness`, `technique`, `mode`. | Phase 1 |
| FR-3.4.2 | `DynamicOcclusionUniforms` must contain all per-frame positional fields: `player_position`, `camera_position`, `region_min`, `region_max`. | Phase 1 |
| FR-3.4.3 | Both sub-structs are private Rust-only types. The GPU-facing `OcclusionUniforms` struct, its `#[uniform(100)]` binding, and the WGSL shader are unchanged. | Phase 1 |
| FR-3.4.4 | The system must assemble the full `OcclusionUniforms` from the current sub-struct values when a write is required. | Phase 1 |

### 3.5 Existing Behavior Preservation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | All existing early-return conditions (occlusion disabled, mode is None, material handle not yet available) must be preserved unchanged. | Phase 1 |
| FR-3.5.2 | The debug logging that fires every 120 frames must be preserved. | Phase 1 |
| FR-3.5.3 | The frame counter used for logging must continue to increment regardless of whether uniforms were updated. | Phase 1 |
| FR-3.5.4 | On the first frame where the material handle becomes available, uniforms must be written unconditionally (both caches start as `None`). | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | The fix must not introduce any per-frame heap allocation. The `Local` cache is stack-resident; `OcclusionUniforms` is `Copy`-sized and must not box or clone into a heap structure. | Phase 1 |
| NFR-4.2 | The system's time complexity remains O(1) — no iteration over entities or assets is added. | Phase 1 |
| NFR-4.3 | The `OcclusionUniforms` struct must remain `#[derive(ShaderType)]`-compatible after adding `PartialEq`. | Phase 1 |
| NFR-4.4 | GPU bind group preparation for `OcclusionMaterial` must not occur on frames where player position, camera position, and `OcclusionConfig` are all unchanged. | Phase 1 |
| NFR-4.5 | The fix must not change public API surface — `OcclusionUniforms`, `OcclusionExtension`, `OcclusionMaterialHandle`, and `update_occlusion_uniforms` signatures remain unchanged (adding `PartialEq` to `OcclusionUniforms` is additive). | Phase 1 |
| NFR-4.6 | Frame rate of the game binary under a static scene must be comparable to the map editor after the fix is applied. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP (all changes delivered together)

- `OcclusionUniforms` derives `PartialEq` (not `Eq` — floats disallow it)
- `OcclusionUniforms` split into `StaticOcclusionUniforms` (config-driven) and `DynamicOcclusionUniforms` (positional) for finer-grained dirty tracking
- Each sub-struct derives `PartialEq` and has its own `Local` cache
- `get_mut()` is only called when at least one sub-struct cache differs from the newly computed value
- Camera and player queries use `Ref<Transform>` for change detection
- Bevy `is_changed()` / `Ref<Transform>` early-out gates each sub-struct computation independently
- `Local` sub-struct cache comparisons retained as safety fallback alongside Bevy change detection
- GPU-facing `OcclusionUniforms` struct, `#[uniform(100)]` binding, and WGSL shader are unchanged
- All existing early-return paths and debug logging are preserved
- First-frame write is unconditional (both caches initialized to `None`)

### Future Phases

- None identified.

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `OcclusionUniforms` fields are all `Copy` types; adding `PartialEq` via `#[derive]` is safe and correct. |
| 2 | Bevy's render world reads the asset's change-detection tick to decide whether to re-prepare bind groups. Avoiding `get_mut()` is sufficient to prevent re-uploads. |
| 3 | The fix targets the game binary only; the map editor does not use `OcclusionMaterial` and is unaffected. |
| 4 | Padding fields (`_padding1`–`_padding4`) are always zero and are covered by `PartialEq` without special handling. |
| 5 | A single shared `OcclusionMaterialHandle` exists; there is no per-chunk material instance that would require iterating multiple handles. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| ~~1~~ | ~~Should `OcclusionUniforms` also derive `Eq` (not just `PartialEq`) given that it contains `f32` fields?~~ | **Resolved:** Derive only `PartialEq`. Rust disallows `#[derive(Eq)]` on types containing `f32`/`Vec3`/`Vec4`. `PartialEq` is sufficient for the cache comparison; if a NaN ever appears in a position value, `PartialEq` returns `false`, causing a re-upload — safe, conservative behavior. |
| ~~2~~ | ~~Are there any other systems that call `materials.get_mut()` on `OcclusionMaterialHandle` that would re-introduce the unconditional dirty marking?~~ | **Resolved:** A full codebase search confirms `occlusion.rs:259` is the only `Assets<T>::get_mut()` call on `OcclusionMaterial`. No other system can re-introduce dirty marking. |
| ~~3~~ | ~~Should the Phase 2 Bevy change-detection gates be implemented in the same PR or tracked as a follow-up?~~ | **Resolved:** All changes (sub-struct split + Bevy change-detection gates) are implemented together in the same set of changes. |
| ~~4~~ | ~~Are there other materials in the codebase that exhibit the same unconditional `get_mut()` pattern?~~ | **Resolved:** Only two `Assets<T>::get_mut()` call sites exist in the codebase. `occlusion.rs:259` is this bug. `editor/grid/systems.rs:70` is editor-only and correctly gated behind a camera-movement threshold — not the same pattern. |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Bug confirmed as reproducible in the game binary | Done | — |
| 2 | `OcclusionUniforms` derives `PartialEq` (required for FR-3.2.1) | Not started | Implementer |

---

## 9. Key Contacts

| Person | Role | Reach Out For |
|--------|------|---------------|
| — | Rendering / GPU | Bevy material change-detection behavior, bind group lifecycle |
| — | Gameplay | Acceptable frame-rate target for static scenes |

---

## 10. Reference: Relevant Code Locations

| Location | Description |
|----------|-------------|
| `src/systems/game/occlusion.rs:209–300` | `update_occlusion_uniforms` — the system being fixed |
| `src/systems/game/occlusion.rs:99–147` | `OcclusionUniforms` struct definition — needs `PartialEq` |
| `src/systems/game/occlusion.rs:259` | `materials.get_mut()` call that triggers GPU re-upload every frame |
| `src/systems/game/occlusion.rs:272–287` | Unconditional uniform assignment |

---

*Created: 2026-03-15*  
*Source: Bug report 2026-03-15-2141-p1-occlusion-material-gpu-reupload-every-frame.md*
