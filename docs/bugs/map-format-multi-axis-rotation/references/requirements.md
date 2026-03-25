# Requirements — Multi-Axis Voxel Rotation

**Source:** Map Format Analysis Investigation — 2026-03-22  
**Status:** Final — all questions resolved 2026-03-25

---

## 1. Overview

The map format stores each voxel's orientation as a `RotationState` — a single axis + angle pair (`rotation.rs:8–14`). When the map editor composes two rotations around different axes (e.g., rotate around Y, then around X), `RotationState::compose()` silently discards the first rotation and stores only the new one (`rotation.rs:32–36`). The resulting map file encodes only the last axis used, meaning any multi-axis orientation is permanently lost on save.

The fix introduces an `orientations` table at the top level of the map file. Each entry is a 3×3 integer rotation matrix representing one distinct orientation used by the map. Voxels reference an entry by integer index into this table. This decouples orientation storage from the per-voxel data, keeps the matrix representation explicit and human-readable in the file, and ensures any composition of 90° rotations is faithfully round-tripped without floating-point types.

---

## 2. Functional Requirements

### 2.1 Orientations Table (Map-Level)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `MapData` must include a top-level `orientations` field containing a list of 3×3 integer rotation matrices (`Vec<[[i32; 3]; 3]>`). | Phase 1 |
| FR-2.1.2 | The orientations list is sparse — only matrices actually referenced by voxels in the map need to be present. An empty list is valid for maps with no rotated voxels. | Phase 1 |
| FR-2.1.3 | Each matrix in the list must represent a valid 90°-grid rotation (determinant = 1, all entries ∈ {−1, 0, 1}, exactly one non-zero entry per row and per column). Invalid matrices must be rejected at validation time. | Phase 1 |
| FR-2.1.4 | The identity orientation (no rotation) does not need to be present in the list — a voxel with `rotation: None` is treated as identity. | Phase 1 |

### 2.2 Voxel Orientation Reference

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | Each `VoxelData` entry must have an optional `rotation` field containing a zero-based index into the map's `orientations` list. | Phase 1 |
| FR-2.2.2 | `rotation: None` (or the field being absent via `#[serde(default)]`) must mean identity — no transformation applied to the pattern geometry. | Phase 1 |
| FR-2.2.3 | A `rotation` index that is out of bounds for the `orientations` list must be rejected at validation time with a clear error. | Phase 1 |

### 2.3 Map File Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | Existing map files that use the old `rotation_state: Some((axis: Y, angle: 1))` syntax must continue to load correctly. The loader must convert the axis/angle pair into the equivalent 3×3 matrix, append it to the orientations list if not already present, and set the voxel's `rotation` index accordingly. | Phase 1 |
| FR-2.3.2 | The new format must be valid RON parseable by the `ron` crate via `#[derive(Serialize, Deserialize)]`. | Phase 1 |
| FR-2.3.3 | The `pattern: None` default (identity orientation) must remain valid and unchanged. | Phase 1 |

### 2.4 Geometry Application

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | At spawn time, the loader must look up the voxel's orientation index in the map's `orientations` list, retrieve the 3×3 matrix, and apply it to the base `SubVoxelGeometry` using `SubVoxelGeometry::rotate()` decomposed from the matrix. | Phase 1 |
| FR-2.4.2 | The geometry produced for any single-axis orientation (e.g., Y+90°) must be identical before and after this change. | Phase 1 |
| FR-2.4.3 | Any orientation achievable by composing 90° rotations around any combination of X, Y, Z axes must be expressible and must produce the same geometry as applying those rotations directly via `SubVoxelGeometry::rotate()`. | Phase 1 |

### 2.5 Map Editor Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | When the editor rotates a voxel, it must compute the resulting 3×3 matrix by composing the current matrix with the new single-axis rotation, append the matrix to the map's orientations list if not already present (equality check by matrix value), and store the index on the voxel. | Phase 1 |
| FR-2.5.2 | When saving a map, the editor must serialise the orientations list as a top-level field in the RON file. | Phase 1 |
| FR-2.5.3 | The editor must display the current voxel orientation in a human-readable form in the properties panel. | Phase 2 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | All matrix arithmetic must use integer types only — no `f32` or floating-point anywhere in orientation composition or application. | Phase 1 |
| NFR-3.2 | The orientations list must not duplicate equivalent matrices — the editor must deduplicate by value before appending. | Phase 1 |
| NFR-3.3 | Map load time must not increase measurably; orientation lookup is O(1) per voxel (index into a `Vec`). | Phase 1 |
| NFR-3.4 | The fix must not change the public API of `SubVoxelGeometry::rotate()` or `rotate_point()`. | Phase 1 |
| NFR-3.5 | Both binaries (`adrakestory` and `map_editor`) are affected; changes must compile for both. | Phase 1 |
| NFR-3.6 | Existing unit tests in `rotation.rs` and `geometry/tests.rs` must continue to pass. New tests must cover matrix application, deduplication, out-of-bounds index rejection, and legacy axis/angle conversion. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- `MapData` has a top-level `orientations: Vec<[[i32; 3]; 3]>` field.
- `VoxelData` has `rotation: Option<usize>` replacing `rotation_state`.
- Validation rejects out-of-bounds rotation indices and invalid matrices.
- Legacy `rotation_state: Some((axis, angle))` syntax loads via a compatibility shim.
- `geometry_with_rotation()` applies the referenced matrix correctly.
- All existing tests pass; new tests cover matrix lookup, deduplication, and legacy conversion.

### Phase 2 — Enhanced

- Map editor properties panel shows current orientation in human-readable form (e.g., axis labels derived from the matrix).
- Editor undo/redo correctly handles orientation changes.
- Editor garbage-collects unused orientation entries from the list on save.

### Future Phases

- Orientation presets in the editor (e.g., "upside-down", "wall-mounted").

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Only 90°-increment rotations are needed; arbitrary-angle rotations are out of scope. |
| 2 | `SubVoxelGeometry::rotate()` and `rotate_point()` in `geometry/rotation.rs` are correct and are not changed by this fix. |
| 3 | The RON file format and `serde` derive pipeline are the serialisation mechanism; no binary format is introduced. |
| 4 | The `Fence` pattern's bypass of `geometry_with_rotation` is a separate issue and is not addressed here. |
| 5 | Staircase directional variant pre-baking (`StaircaseZ` etc.) is a separate issue and is not addressed here. |
| 6 | The maximum number of distinct orientations a map can reference is 24 (the rotation group of the cube); in practice most maps will reference fewer than 10. |

---

## 6. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | ~~Should the new format use an orientation index (0–23) or a matrix/sequence?~~ **Resolved:** use a top-level `orientations` list of 3×3 integer matrices; voxels reference by index. | Team |
| 2 | ~~Does the editor's `RotationState::compose()` call site in `src/editor/tools/input/helpers.rs` need updating, or is `compose()` only called from format code?~~ **Resolved:** Yes — `helpers.rs` calls `compose()` and must be updated to use `multiply_matrices()` + deduplication into `map_data.orientations`. This is covered by Task 8 in the ticket. | Implementer |
| 3 | ~~Should old single-axis files be auto-migrated on save, or only on explicit "re-save"?~~ **Resolved:** Auto-migrate on next save. Loader converts in memory; file is written in new format on next user save. | Team |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | ~~Decision on orientation representation format~~ **Resolved:** orientations table + index. | Done | Team |

---

## 8. Reference: Example RON Snippets

| Type | Example | Notes |
|------|---------|-------|
| Map with two orientations | `orientations: [[[1,0,0],[0,0,-1],[0,1,0]], [[-1,0,0],[0,1,0],[0,0,-1]]]` | First = X+90°, second = Y+180° + X+90° |
| Voxel using first orientation | `(pos:(1,0,2), voxel_type:Stone, pattern:Some(StaircaseX), rotation:Some(0))` | Index 0 in orientations list |
| Voxel with no rotation | `(pos:(0,0,0), voxel_type:Grass)` | rotation absent → identity |
| Legacy (backward compat) | `(pos:(3,0,1), voxel_type:Stone, rotation_state:Some((axis:Y,angle:1)))` | Shim converts to matrix on load |

---

*Created: 2026-03-22*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 1*

