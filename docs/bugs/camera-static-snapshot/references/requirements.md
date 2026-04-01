# Requirements — Camera Behaviour Parameters Not Configurable Per-Map

**Source:** Map Format Analysis Investigation — 2026-03-22
**Bug:** `docs/bugs/camera-static-snapshot/ticket.md`
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

`CameraData` (`src/systems/game/map/format/camera.rs:7–13`) stores only three
fields: `position`, `look_at`, and `rotation_offset`. These define the camera's
starting pose, but two critical runtime parameters are hardcoded in
`spawn_camera()` (`src/systems/game/map/spawner/mod.rs:607–609`):

```rust
rotation_speed: 5.0,
follow_speed: 15.0,
```

A third behavioural parameter — field of view — uses the Bevy `Camera3d`
default (~60°) and cannot be influenced from the map file at all.

Two observable defects follow:

- **Hardcoded feel parameters** — level designers cannot tune camera
  responsiveness per-map without modifying engine code. A map needing a slower,
  more cinematic follow (e.g. a cutscene area) or a tighter, more responsive
  follow (e.g. a fast-action corridor) must use the same feel as every other map.
- **No FOV control** — some map layouts benefit from a wider or narrower field of
  view. There is no format-level mechanism to set FOV.

A secondary issue is that `rotation_offset` stores a raw radian value in the RON
file (`-1.5707964` in `assets/maps/default.ron:3814`) with no explanation in the
field doc comment or in the map-format spec. This is confusing for map authors.

The fix is purely additive: three optional fields are added to `CameraData` with
`#[serde(default)]`. When absent (all existing map files), each falls back to the
current default. No existing map file breaks. No migration pass is needed.

---

## 2. Functional Requirements

### 2.1 New Optional Fields on CameraData

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | Add `follow_speed: Option<f32>` to `CameraData`. When `Some(v)`, the spawned `GameCamera.follow_speed` is `v`. When `None`, the fallback is `15.0` (current hardcoded value). | Phase 1 |
| FR-2.1.2 | Add `rotation_speed: Option<f32>` to `CameraData`. When `Some(v)`, the spawned `GameCamera.rotation_speed` is `v`. When `None`, the fallback is `5.0` (current hardcoded value). | Phase 1 |
| FR-2.1.3 | Add `fov_degrees: Option<f32>` to `CameraData`. When `Some(deg)`, the camera is spawned with `Projection::Perspective(PerspectiveProjection { fov: deg.to_radians(), ..default() })`. When `None`, the default `Camera3d` projection is used. | Phase 1 |
| FR-2.1.4 | All three new fields must be decorated with `#[serde(default)]` so that map files that omit them parse without error. | Phase 1 |

### 2.2 Spawner Reads New Fields

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | `spawn_camera()` in `src/systems/game/map/spawner/mod.rs` must read `follow_speed` and `rotation_speed` from `CameraData` using `.unwrap_or(default)` and pass the resulting values to `GameCamera`. | Phase 1 |
| FR-2.2.2 | `spawn_camera()` must conditionally attach `Projection::Perspective` when `fov_degrees` is `Some`. When `None`, no explicit `Projection` component is added and the Bevy default applies. | Phase 1 |

### 2.3 Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | All existing map files (including `assets/maps/default.ron`) must load without error and produce identical runtime behaviour after the fix. | Phase 1 |
| FR-2.3.2 | A map file that does not include any of the new fields must behave exactly as today. | Phase 1 |

### 2.4 Documentation and Comments

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | The `rotation_offset` field doc comment must be updated to state that the value is in radians and that `-π/2` (≈ `-1.5707964`) rotates the camera 90° left around the world Y axis. | Phase 1 |
| FR-2.4.2 | `docs/api/map-format-spec.md` must be updated to document all five `CameraData` fields, including the three new optional ones and the clarified `rotation_offset` semantics. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The `#[serde(default)]` pattern must match the convention already used in the format layer (e.g. `VoxelData.pattern`, `VoxelData.rotation`). No custom deserialiser is needed. | Phase 1 |
| NFR-3.2 | `CameraData` must retain its existing derives: `Serialize, Deserialize, Clone, Debug`. | Phase 1 |
| NFR-3.3 | The change is purely additive. No existing field is removed, renamed, or given a different type. | Phase 1 |
| NFR-3.4 | `GameCamera` field types remain `f32` (not `Option<f32>`). The `Option` is resolved in `spawn_camera()`, not propagated into the component. | Phase 1 |
| NFR-3.5 | Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings. | Phase 1 |
| NFR-3.6 | Unit tests must cover default field absence (RON round-trip), and the `follow_speed` and `fov_degrees` fields serialise and deserialise correctly when present. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (this fix)

- `follow_speed`, `rotation_speed`, `fov_degrees` optional fields added to
  `CameraData` with `#[serde(default)]`.
- `spawn_camera()` reads all three fields.
- `rotation_offset` doc comment updated.
- `docs/api/map-format-spec.md` updated.
- Unit tests added.
- Both binaries compile cleanly.

### Phase 2 — Future (out of scope)

- Smooth camera intro animation on map load (spawn-time interpolation from above
  the look_at point to the configured position) — this is a runtime system concern,
  not a format concern.
- Camera mode selection (orbit, fixed, follow, cinematic rail) expressed as a
  typed enum in the format.
- Per-map camera constraints (min/max zoom distance, pitch limits).

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The `#[serde(default)]` attribute on `Option<f32>` fields defaults to `None` via `Option`'s standard `Default` impl. No helper function is required (unlike `VoxelData` which uses helpers for non-`None` defaults). |
| 2 | Bevy's `Camera3d` default FOV is approximately 60° in the vertical direction. Map authors setting `fov_degrees` should be aware this is the vertical FOV convention in Bevy's `PerspectiveProjection`. |
| 3 | The `map_editor` binary uses `Camera2d` for its own editor viewport, not `Camera3d`. The editor spawns the game camera only when previewing the map (hot-reload path). The fix applies equally to both binary entry points because both invoke `spawn_camera()` from the shared library. |
| 4 | The snap-on-load behaviour (camera jumps instantly to `position` on map spawn rather than animating in) is a separate runtime concern and is explicitly out of scope for this ticket. |
| 5 | Values below 0 or above 180 for `fov_degrees` are nonsensical but no clamp is required in Phase 1; the spec should document recommended range (5°–150°). |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should the optional fields use `Option<f32>` or a newtype with custom `Default`? | `Option<f32>` with `#[serde(default)]`. This is the simplest approach and consistent with how the codebase handles purely optional fields. |
| 2 | Should `GameCamera` store the optional values or resolved `f32`? | Resolved `f32` in `GameCamera`. The `Option` is resolved at spawn time in `spawn_camera()`. Propagating `Option` into the component would require every camera system to unwrap values each frame. |
| 3 | Is FOV stored in degrees or radians in the format? | Degrees in the format (human-readable), converted to radians in `spawn_camera()` via `.to_radians()`. Consistent with `rotation_offset` being the one exception (it uses radians, which is a legacy issue documented by FR-2.4.1). |
| 4 | Is the snap-on-load behaviour in scope? | No. Smooth intro animation is a runtime system change, not a format change. Adding a format field for it (e.g. `intro_duration_secs`) would be premature without a runtime implementation. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Findings 1–7 fixes (format and geometry layer stability) | **Done** | Team |

No blockers. This is an independent, additive format change.

---

## 8. Reference: CameraData Flow

```
CameraData (RON)
    │
    ├─ position, look_at, rotation_offset
    │       → spawn_camera() → camera Transform
    │
    ├─ follow_speed (new, Option<f32>)
    │       → spawn_camera() → GameCamera.follow_speed
    │
    ├─ rotation_speed (new, Option<f32>)
    │       → spawn_camera() → GameCamera.rotation_speed
    │
    └─ fov_degrees (new, Option<f32>)
            → spawn_camera() → Projection::Perspective(fov)
```

**Runtime consumers of `GameCamera` fields:**

| Field | Consumer system | File |
|-------|----------------|------|
| `follow_speed` | `follow_player_camera` | `src/systems/game/camera.rs:47` |
| `rotation_speed` | `rotate_camera` | `src/systems/game/camera.rs:86` |

---

## 9. Reference: Files Affected

| File | Change |
|------|--------|
| `src/systems/game/map/format/camera.rs` | **Modified** — add three optional fields; improve `rotation_offset` doc comment; add unit tests |
| `src/systems/game/map/spawner/mod.rs` | **Modified** — read new fields in `spawn_camera()`; conditionally set `Projection` |
| `docs/api/map-format-spec.md` | **Modified** — document all five `CameraData` fields |

---

*Created: 2026-03-31*
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 8*
