# Requirements — Upgrade Bevy 0.15 → 0.18

**Source:** Bevy migration guides + codebase analysis — 2026-03-21
**Participants:** Developer (Engineer)
**Status:** Draft — pending team review

---

## 1. Overview

Migrate the game and map editor from Bevy 0.15.3 to Bevy 0.18.1. The upgrade spans three intermediate versions (0.16, 0.17, 0.18) each with breaking changes. The migration must preserve all existing functionality with no user-visible regressions: gameplay, rendering, occlusion, physics, input, map loading, and the map editor.

The upgrade also resolves an open bug: the occlusion shadow-casting fix (`docs/bugs/fix-occlusion-shadow-casting/`) depends on shader behavior that changed in 0.18. The migration is the right time to implement the correct fix (Option B — view projection check) rather than patching 0.15.

---

## 2. Data Domains

### 2.1 Migration Domains

| Domain | Description | Typical Migration Signal |
|--------|-------------|--------------------------|
| **Dependencies** | `Cargo.toml` version pins for `bevy` and `bevy_egui` | `cargo build` fails with version incompatibility errors |
| **Rust Code** | All `.rs` source files using Bevy APIs (camera, materials, lights, mesh, handles) | Compiler errors on renamed types, moved import paths, or removed fields |
| **WGSL Shaders** | Custom shader files (`occlusion_material.wgsl`, `occlusion_material_prepass.wgsl`) | Shader compilation errors logged at runtime; incorrect rendering output |
| **Map Editor (bevy_egui)** | Editor binary and all `bevy_egui` UI code across 25+ files | Editor fails to start or UI panels stop rendering after dependency update |
| **Game Binary** | Game entry point, all gameplay systems, state transitions, and debug overlays | Game fails to start, visual regressions, or broken gameplay after migration |

**Requirement:** Each domain must be migrated and verified as a discrete compile/run milestone before proceeding to the next domain.

---

## 3. Functional Requirements

### 3.1 Dependency Updates

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | `bevy` in `Cargo.toml` must be updated to `0.18` | Phase 1 |
| FR-3.1.2 | `bevy_egui` must be updated to `0.39` (the version compatible with Bevy 0.18) | Phase 1 |
| FR-3.1.3 | After dependency update, `cargo build` must compile without errors or warnings about deprecated API usage | Phase 1 |

---

### 3.2 Rust Code Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | All camera spawning code must remain compatible. `Camera3d::default()` and `Camera2d` are already in use and are forward-compatible with 0.18 (verified: no `Camera { target: ... }` pattern exists in the codebase). Verify no new required camera components are introduced by 0.18. | Phase 1 |
| FR-3.2.2 | All `StandardMaterial` usages must compile and render correctly. Field names (`base_color`, `metallic`, `roughness`, `alpha_mode`, etc.) must be verified against 0.18. `AlphaMode::Mask(0.001)` must still function as expected for the occlusion material. `AlphaMode::AlphaToCoverage` must still be supported. | Phase 1 |
| FR-3.2.3 | The `ExtendedMaterial` / `MaterialExtension` API must be verified and updated. `OcclusionMaterial = ExtendedMaterial<StandardMaterial, OcclusionExtension>` must remain valid. `impl MaterialExtension for OcclusionExtension` must implement all required methods for 0.18. Shader bind group indices (currently group 2, binding 100) must be verified. | Phase 1 |
| FR-3.2.4 | `MaterialPlugin::<OcclusionMaterial>` registration must use 0.18 API. `enable_prepass` and `enable_shadows` are now `Material`/`MaterialExtension` methods, not `MaterialPlugin` fields (0.17→0.18 change). If `MaterialPlugin` fields were in use, convert to trait methods. | Phase 1 |
| FR-3.2.5 | All light types must render correctly. `DirectionalLight`, `AmbientLight` field names must be verified (`illuminance`, `brightness`, `shadows_enabled`, etc.). `CascadeShadowConfigBuilder` API must be verified (currently used in 4 locations). `NotShadowCaster` component must still be available (types moved to `bevy_light` in 0.16→0.17; import via `bevy::pbr` or `bevy::light`). | Phase 1 |
| FR-3.2.6 | All import paths must be updated for the `bevy_render` reorganization (0.16→0.17). Camera types moved to `bevy_camera` (accessible via `bevy::camera`). Light types moved to `bevy_light` (accessible via `bevy::light` or `bevy::pbr`). `NotShadowCaster`, `NotShadowReceiver` moved to `bevy_light`. Mesh types moved to `bevy_mesh` (accessible via `bevy::mesh`). All affected imports must be updated to compile. | Phase 1 |
| FR-3.2.7 | `Mesh::new(topology, usage)` API must be verified; update if signature changed. | Phase 1 |
| FR-3.2.8 | `Handle::weak_from_u128()` deprecation: verified not present in this codebase — no migration action required. If any new code is added during migration, use the `weak_handle!` macro instead. | Phase 1 |

---

### 3.3 WGSL Shader Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | `occlusion_material.wgsl` must compile without errors in Bevy 0.18. All `#import bevy_pbr::...` paths must exist in 0.18 (`pbr_fragment`, `pbr_functions`, `prepass_io`, `forward_io`, `pbr_deferred_functions`). `VertexOutput` and `FragmentOutput` struct layouts must be verified. Update any renamed or moved shader imports. | Phase 1 |
| FR-3.3.2 | `occlusion_material_prepass.wgsl` must compile and implement the shadow fix. `#import bevy_pbr::prepass_io::VertexOutput` must still be valid. `#import bevy_render::view::View` must be added. The shadow-pass detection guard (view projection matrix check) from `docs/bugs/fix-occlusion-shadow-casting/references/architecture.md` Option B must be implemented. Note: `DEPTH_CLAMP_ORTHO` is **not present** in the current shader (verified) — no removal needed. | Phase 1 |
| FR-3.3.3 | Shader behavior must be functionally identical to the current 0.15 behavior for all non-shadow cases. | Phase 1 |

---

### 3.4 Map Editor Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | The map editor binary (`cargo run --bin map_editor`) must start and render without errors. | Phase 1 |
| FR-3.4.2 | All `bevy_egui` usages must be updated for bevy_egui 0.39: `EguiPlugin` registration pattern, `EguiContexts` system parameter, and any UI context API calls used across the 25+ editor UI files. | Phase 1 |
| FR-3.4.3 | All editor tools (paint, erase, select, fill, etc.) must function correctly after migration. | Phase 1 |
| FR-3.4.4 | Map saving and loading from the editor must work correctly. | Phase 1 |
| FR-3.4.5 | Hot-reload (F5 / Ctrl+R) from the editor must work correctly. | Phase 1 |

---

### 3.5 Game Binary Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | The game binary must start, display the intro animation, title screen, and load a map. | Phase 1 |
| FR-3.5.2 | Voxel rendering must be visually correct (greedy meshing, LOD switching, chunk culling). | Phase 1 |
| FR-3.5.3 | Occlusion transparency must work correctly: ceiling voxels invisible, floor voxels visible. | Phase 1 |
| FR-3.5.4 | Shadows from the directional light (sun) must appear correctly inside occluded rooms. | Phase 1 |
| FR-3.5.5 | Physics, collision, and player movement must function without regression. | Phase 1 |
| FR-3.5.6 | Input systems (keyboard, gamepad) must function correctly. | Phase 1 |
| FR-3.5.7 | All `GameState` transitions (Intro → Title → Loading → InGame → Paused) must work. | Phase 1 |
| FR-3.5.8 | FPS counter and debug overlays must render without errors. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | **No performance regression.** Frame time at the standard map benchmark scene must not increase by more than 5% compared to the 0.15 baseline. | Phase 1 |
| NFR-4.2 | **Incremental migration.** The migration must be verifiable as a compile/run step at each intermediate milestone (§3.1, §3.2, §3.3, §3.4, §3.5) before proceeding. | Phase 1 |
| NFR-4.3 | **No new deprecation warnings.** The final build must produce zero `deprecated` warnings from the `bevy` or `bevy_egui` crates. | Phase 1 |
| NFR-4.4 | **Preserved test suite.** All existing `cargo test` tests must pass after migration. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP
- Dependencies updated and `cargo build` compiles clean
- All Rust code updated for 0.18 API changes (camera, materials, lights, mesh, imports)
- WGSL shaders compile and render correctly; shadow fix (Option B) implemented
- Game binary starts, all state transitions work, voxel rendering and occlusion correct
- Map editor binary starts and egui UI renders

### Phase 2 — Enhanced
- All editor tools (paint, erase, select, fill) verified functional
- Map save/load and hot-reload verified in editor
- Full `cargo test` suite passing with zero regressions
- Performance benchmark verified ≤5% regression

### Future Phases
- Adopt new Bevy 0.16/0.17/0.18 features beyond migration requirements
- Migrate to `bevy_light` module paths as first-class imports (currently using `bevy::pbr` re-exports)
- Point and spot light shadow casting from occluded voxels (tracked separately)
- Hybrid mode 3 shader-based occlusion re-enablement (separate ticket)

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | **Bevy 0.18.1** is the target version (latest stable as of 2026-03-21). |
| 2 | **bevy_egui 0.39.x** is the compatible version for Bevy 0.18. |
| 3 | All three intermediate migration guides must be consulted: 0.15→0.16, 0.16→0.17, 0.17→0.18 (see §8). |
| 4 | The shadow casting fix (Option B) is included in this migration as part of FR-3.3.2. |
| 5 | **Out of scope:** Adopting any new Bevy 0.16/0.17/0.18 features beyond what is required for migration. |
| 6 | **Out of scope:** Migrating to the new `bevy_light` module paths as first-class imports (using `bevy::pbr` re-exports is acceptable if they still exist). |
| 7 | **Out of scope:** Point and spot light shadow casting from occluded voxels (tracked separately in the shadow casting bug; currently `shadows_enabled: false`). |
| 8 | **Out of scope:** Hybrid mode 3 shader-based occlusion re-enablement (separate ticket). |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | Are `bevy::pbr` re-export paths for light types (`NotShadowCaster`, `DirectionalLight`, etc.) still available in 0.18, or must imports switch to `bevy::light`? | Verify during FR-3.2.6 |
| 2 | Does `AlphaMode::AlphaToCoverage` still exist in 0.18, or was it renamed? | Verify during FR-3.2.2 |
| 3 | Does `bevy_pbr::pbr_fragment::pbr_input_from_standard_material` still exist in 0.18 shaders? | Verify during FR-3.3.1 |
| 4 | Does bevy_egui 0.39 require any changes to `EguiPlugin` registration or `EguiContexts` usage? | Verify during FR-3.4.2 |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Bevy migration guides: [0.15→0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/), [0.16→0.17](https://bevyengine.org/learn/migration-guides/0-16-to-0-17/), [0.17→0.18](https://bevyengine.org/learn/migration-guides/0-17-to-0-18/) | Not started | Developer |
| 2 | bevy_egui 0.39 changelog / migration notes | Not started | Developer |
| 3 | Shadow casting fix architecture doc (`docs/bugs/fix-occlusion-shadow-casting/references/architecture.md`) — Option B implementation required for FR-3.3.2 | Pending | Developer |

---

## 9. Key Contacts

| Person | Role | Reach Out For |
|--------|------|---------------|
| Developer | Engineer | All topics |

---

## 10. Reference: Sample Scenarios

| Type | Example Verification Scenario | Complexity |
|------|-------------------------------|------------|
| Dependency | Run `cargo build --release` after updating `Cargo.toml`; confirm zero errors and zero deprecation warnings | Low |
| Rust Code — Camera | Spawn game with `Camera3d` + `Camera2d`; verify no panics about missing required components (code already uses forward-compatible patterns) | Medium |
| Rust Code — Materials | Load a map with occlusion voxels; verify `AlphaMode::Mask` and `AlphaToCoverage` render correctly | Medium |
| Rust Code — Imports | Confirm `NotShadowCaster`, `DirectionalLight`, `Mesh` compile via their 0.18 import paths | Low |
| WGSL Shader | Launch game with occlusion material; verify no shader compilation errors in logs and ceiling voxels are transparent | High |
| WGSL Shadow Fix | Enter an occluded room; verify sun shadows appear on the floor and walls (shadow fix Option B active) | High |
| Map Editor | Run `cargo run --bin map_editor`; open a map, paint a voxel, save, and verify round-trip | Medium |
| Game State | Start game; verify Intro → Title → LoadingMap → InGame → Paused transitions all complete without panic | Medium |
| Performance | Run benchmark map; verify frame time within 5% of 0.15 baseline | High |
| Test Suite | Run `cargo test`; verify all tests pass with zero failures | Low |

---

*Created: 2026-03-21*
*Source: Bevy migration guides 0.15→0.16, 0.16→0.17, 0.17→0.18 | Codebase analysis (adrakestory)*
