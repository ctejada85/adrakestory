# Requirements — Camera Lerp Frame-Rate Independence Fix

**Source:** Bug report `2026-03-15-2141-p3-camera-lerp-visual-lag.md` — 2026-03-15  
**Status:** Draft

---

## 1. Overview

The `follow_player_camera` and `rotate_camera` systems interpolate the camera using `lerp(target, speed * delta_secs)` and `slerp(target, speed * delta)`. This is a common approximation that is **not frame-rate-independent**: the convergence time varies with frame rate, and at `follow_speed = 5.0` the camera takes ~400–500 ms to reach 99% of its target position, producing a "floaty" lag that is perceptible to players.

The correct formulation is the exponential decay `alpha = 1 - e^(-speed * delta)`, which guarantees identical convergence time regardless of whether the game runs at 30, 60, or 120 fps. Additionally, `follow_speed = 5.0` at the spawn site is too conservative; a value of 15.0 yields a responsive third-person camera without overshooting.

This fix is purely arithmetic — two formula replacements in `camera.rs` and one constant change in `spawner/mod.rs`. No new types, systems, or components are introduced.

---

## 2. Functional Requirements

### 2.1 Frame-Rate-Independent Follow

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `follow_player_camera` must compute the lerp factor as `1.0 - (-follow_speed * delta).exp()` instead of `follow_speed * delta`. | Phase 1 |
| FR-2.1.2 | With the fixed formula and `follow_speed = 15.0`, the camera must reach 99% of its target position within the same wall-clock duration regardless of whether the game runs at 30, 60, or 120 fps. | Phase 1 |

### 2.2 Frame-Rate-Independent Rotation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | `rotate_camera` must compute the slerp factor as `1.0 - (-rotation_speed * delta).exp()` instead of `rotation_speed * delta`. | Phase 1 |
| FR-2.2.2 | The rotation behaviour on the Delete key (90° left, smooth return) must be functionally unchanged after the formula fix. | Phase 1 |

### 2.3 Follow Speed Default

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | The `follow_speed` value in `GameCamera` at spawn time (in `spawner/mod.rs`) must be changed from `5.0` to `15.0`. | Phase 1 |
| FR-2.3.2 | `rotation_speed` at spawn time remains `5.0`; only the lerp formula is corrected. | Phase 1 |

### 2.4 Existing Behaviour Preservation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `GameCamera` component fields (`original_rotation`, `target_rotation`, `rotation_speed`, `follow_offset`, `follow_speed`, `target_position`) must remain unchanged. | Phase 1 |
| FR-2.4.2 | The `follow_player_camera` system must continue to update `game_camera.target_position` before computing the lerp. | Phase 1 |
| FR-2.4.3 | The `rotate_camera` system's keyboard guard (`InputSource::KeyboardMouse`) and offset-pivot logic must remain unchanged. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The fix introduces no new heap allocations — `f32::exp()` is a single CPU instruction. | Phase 1 |
| NFR-3.2 | The `GameCamera` component struct must not gain new fields. | Phase 1 |
| NFR-3.3 | No new systems or plugins are introduced; all changes are internal to existing functions. | Phase 1 |
| NFR-3.4 | All existing unit tests must pass without modification. | Phase 1 |
| NFR-3.5 | The fix is scoped to the game binary only; the map editor camera is unaffected. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 (all delivered together)

- Fix lerp formula in `follow_player_camera` (exponential decay)
- Fix slerp formula in `rotate_camera` (exponential decay)
- Raise `follow_speed` spawn default from `5.0` → `15.0`

### Future Phases

- Make `follow_speed` and `rotation_speed` hot-reloadable for easier tuning
- Add configurable camera lag as an accessibility option for players who prefer a softer follow

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `f32::exp()` is available in `std` (Rust `no_std` is not a target). |
| 2 | `follow_speed = 15.0` with exponential decay yields a feel comparable to a tight third-person camera; the exact value is tunable post-ship. |
| 3 | `rotation_speed = 5.0` with exponential decay produces acceptable rotation smoothness; no value change is required. |
| 4 | The delta time clamping in physics systems (see `coding-guardrails.md` §3) is not required here because the camera lerp diverging after focus loss is benign — the camera simply snaps to the correct position. |

---

## 6. Open Questions

No open questions.

---

## 7. Dependencies & Blockers

No blockers. This fix is independent of the P1 and P2 fixes.

---

*Created: 2026-03-15*  
*Source: Bug report `docs/bugs/2026-03-15-2141-p3-camera-lerp-visual-lag.md`*  
*Companion document: [Architecture](./architecture.md)*
