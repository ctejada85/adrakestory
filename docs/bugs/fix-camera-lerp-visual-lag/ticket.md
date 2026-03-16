# User Story — Camera Lerp Frame-Rate Independence Fix

**Ticket ID:** fix-camera-lerp-visual-lag  
**Type:** Bug Fix  
**Priority:** P3  
**Status:** Ready for implementation

---

## Story

As a player, I want the camera to follow my character smoothly and responsively at any frame rate, so that the game feels tight and polished regardless of whether I run at 30, 60, or 120 fps.

---

## Description

The `follow_player_camera` and `rotate_camera` systems use `lerp(target, speed * delta)` and `slerp(target, speed * delta)` — a common but frame-rate-dependent approximation. The correct formula is the exponential decay `alpha = 1 - exp(-speed * delta)`, which gives identical convergence time at any frame rate.

Additionally, the spawn-time `follow_speed = 5.0` default produces a sluggish ~400–800 ms camera lag. Raising it to `15.0` (with the fixed formula) gives a responsive third-person feel.

**Root cause files:**
- `src/systems/game/camera.rs` — lines ~43, ~83
- `src/systems/game/map/spawner/mod.rs` — line ~484

**Reference:** `docs/bugs/2026-03-15-2141-p3-camera-lerp-visual-lag.md`

---

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-1 | `follow_player_camera` computes lerp factor as `1.0 - (-follow_speed * delta).exp()` |
| AC-2 | `rotate_camera` computes slerp factor as `1.0 - (-rotation_speed * delta).exp()` |
| AC-3 | `follow_speed` at spawn time is `15.0` (was `5.0`) |
| AC-4 | `rotation_speed` at spawn time remains `5.0` |
| AC-5 | `GameCamera` component struct fields are unchanged (no additions, no removals) |
| AC-6 | The Delete-key rotation logic is functionally unchanged |
| AC-7 | Unit tests confirm the exponential formula is frame-rate-independent |
| AC-8 | All existing tests pass |
| AC-9 | `cargo clippy` produces no new warnings |
| AC-10 | Release build succeeds |

---

## Non-Functional Requirements

| # | Requirement |
|---|-------------|
| NFR-1 | No new heap allocations — `f32::exp()` is a single scalar operation |
| NFR-2 | No new systems, plugins, or component fields introduced |
| NFR-3 | Changes are limited to `camera.rs` and `spawner/mod.rs` (plus tests) |
| NFR-4 | Fix is scoped to the game binary; the map editor is unaffected |

---

## Tasks

1. **Fix lerp formula in `follow_player_camera`** — Replace `follow_speed * delta` with `1.0 - (-follow_speed * delta).exp()` in `camera.rs` line ~43
2. **Fix slerp formula in `rotate_camera`** — Replace `rotation_speed * delta` with `1.0 - (-rotation_speed * delta).exp()` in `camera.rs` line ~83
3. **Update `follow_speed` spawn default** — Change `5.0` → `15.0` in `spawner/mod.rs` line ~484
4. **Write unit tests** — Add tests in `src/systems/game/tests/camera_tests.rs`:
   - `exponential_alpha_is_frame_rate_independent`
   - `exponential_alpha_approaches_1_at_large_delta`
   - `exponential_alpha_approaches_0_at_tiny_delta`
   - `linear_approximation_differs_at_low_fps`
5. **Validate** — `cargo test --lib`, `cargo clippy --lib`, `cargo build --release`
6. **Update `docs/developer-guide/architecture.md`** — Add camera lerp fix to camera system notes
7. **Commit** — All changes in a single `fix(camera)` commit

---

## Dependencies / Blockers

None. This is independent of P1 (occlusion) and P2 (interior detection) fixes.

---

## Pre-existing Failures to Document (not fix)

`editor::state::tests::test_get_display_name_with_path` — fails before this change; unrelated.

---

*Created: 2026-03-15*  
*Documents: [Requirements](./requirements.md) · [Architecture](./architecture.md)*
