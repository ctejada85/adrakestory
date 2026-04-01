# Fix: Camera Behaviour Parameters Are Not Configurable Per-Map

**Date:** 2026-03-31
**Severity:** Low (p3)
**Component:** Map format — `src/systems/game/map/format/camera.rs`, camera spawner — `src/systems/game/map/spawner/mod.rs`

---

## Story

As a level designer, I want to be able to set the camera's follow speed, rotation
speed, and field of view in the map file so that I can tune the camera feel per-map
without touching engine code.

---

## Description

`CameraData` (`src/systems/game/map/format/camera.rs:7–13`) exposes `position`,
`look_at`, and `rotation_offset`. These three fields control where the camera
starts, but not how it behaves. Two critical runtime parameters —
`follow_speed` (15.0) and `rotation_speed` (5.0) — are hardcoded in
`spawn_camera()` (`src/systems/game/map/spawner/mod.rs:607–609`). A third
parameter, field of view, uses the Bevy `Camera3d` default (~60°) and cannot be
overridden from the map file at all.

Two concrete problems result:

1. **Hardcoded feel parameters.** A map that needs a slower, more cinematic camera
   or a tighter, more responsive follow cannot express that preference. The
   designer must change engine source code or accept the defaults.
2. **No FOV control.** Some maps benefit from a wider or narrower field of view
   (e.g. a claustrophobic interior vs. a wide outdoor vista). There is no map-level
   mechanism to set FOV.

The fix adds three optional fields to `CameraData` with `#[serde(default)]`:
`follow_speed`, `rotation_speed`, and `fov_degrees`. When absent (all existing
map files), each falls back to the current hardcoded default. When present, the
value is forwarded to `GameCamera` and `Projection` at spawn. No existing map
file breaks.

A secondary issue is that `rotation_offset` stores a raw radian value
(`-1.5707964` in `default.ron`) with no documentation in the format spec or field
comment. This is addressed by improving the doc comment and the spec entry.

---

## Acceptance Criteria

1. `CameraData` has three new optional fields: `follow_speed: Option<f32>`,
   `rotation_speed: Option<f32>`, `fov_degrees: Option<f32>`.
2. Each field is decorated with `#[serde(default)]`; the corresponding `Default`
   implementations return `None`.
3. `spawn_camera()` reads `follow_speed` and `rotation_speed` from `CameraData`
   and passes them to `GameCamera`, falling back to 15.0 and 5.0 respectively when
   `None`.
4. `spawn_camera()` reads `fov_degrees` from `CameraData`; when `Some(deg)`, it
   spawns the camera with a `Projection::Perspective(PerspectiveProjection { fov:
   deg.to_radians(), ..default() })`. When `None`, the default `Camera3d`
   projection (~60°) is used.
5. All existing map files (including `assets/maps/default.ron`) load without
   error and produce identical runtime behaviour to before.
6. A map file that sets `follow_speed: Some(5.0)` produces a camera that follows
   the player at speed 5.0 rather than 15.0.
7. A map file that sets `fov_degrees: Some(90.0)` produces a 90° horizontal FOV.
8. The `rotation_offset` field doc comment is updated to state that the value is
   in radians and that `−π/2` (≈ `−1.5707964`) means "rotated 90° left around Y".
9. `docs/api/map-format-spec.md` is updated to document all five `CameraData`
   fields including the three new optional ones and the clarified `rotation_offset`
   semantics.
10. `cargo build` succeeds for both `adrakestory` and `map_editor` with zero new
    errors or warnings.
11. `cargo test` passes with no new failures.
12. `cargo clippy` reports zero new errors.

---

## Non-Functional Requirements

- The `#[serde(default)]` pattern must be used consistently with how it is
  applied elsewhere in the format layer (e.g. `VoxelData.pattern`,
  `VoxelData.rotation`). Helper functions returning the default value are
  acceptable if the `Default` impl returns `None`.
- `CameraData` must retain its existing derives: `Serialize, Deserialize, Clone,
  Debug`.
- The change must be purely additive at the format layer — no existing field is
  removed, renamed, or given a different type.
- Both binaries must compile without new warnings after the change.
- No runtime system other than `spawn_camera()` needs modification; `GameCamera`
  field types are unchanged (fields remain `f32`, not `Option<f32>`).

---

## Tasks

1. Add `follow_speed: Option<f32>`, `rotation_speed: Option<f32>`, and
   `fov_degrees: Option<f32>` to `CameraData` in
   `src/systems/game/map/format/camera.rs`, each with `#[serde(default)]` and
   a `Default` impl returning `None`.
2. Update `spawn_camera()` in `src/systems/game/map/spawner/mod.rs` to read
   `follow_speed` and `rotation_speed` from `CameraData` (with `.unwrap_or`
   fallback), and to conditionally set `Projection::Perspective` when
   `fov_degrees` is `Some`.
3. Improve the `rotation_offset` doc comment in `camera.rs` to document units
   (radians) and the meaning of the default value (`-π/2`).
4. Update `docs/api/map-format-spec.md` to document all five `CameraData` fields
   and the semantics of `rotation_offset`.
5. Add unit tests in `src/systems/game/map/format/camera.rs` (inline `#[cfg(test)]`):
   - `camera_data_defaults_produce_none_optionals` — asserts all three optional
     fields are `None` when not present in RON.
   - `camera_data_follow_speed_round_trips` — serialise/deserialise a
     `CameraData` with `follow_speed: Some(8.0)` and assert the value survives.
   - `camera_data_fov_degrees_round_trips` — same for `fov_degrees`.
6. Run `cargo build`, `cargo test`, `cargo clippy`; fix any failures or new
   warnings.
