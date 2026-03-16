# Requirements — Fix Occlusion Systems State Guard

**Source:** Bug report `docs/bugs/2026-03-16-1213-p4-occlusion-systems-run-in-all-game-states.md`  
**Status:** Draft  

---

## 1. Overview

`OcclusionPlugin::build()` registers `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` in the `Update` schedule without a state condition. All three systems run in every `GameState` including `IntroAnimation`, `TitleScreen`, `LoadingMap`, and `Settings`. While each system returns early when required resources are absent, the Bevy scheduler still evaluates the system condition, dispatches the system, and executes the early-exit logic on every frame in non-gameplay states. All other gameplay systems in `main.rs` are correctly gated with `.run_if(in_state(GameState::InGame))`.

The fix adds a `.run_if` condition to the system chain so the three systems are skipped entirely by the scheduler in non-gameplay states.

---

## 2. Functional Requirements

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-1 | `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` must not be dispatched by the scheduler in `GameState::IntroAnimation`, `GameState::TitleScreen`, `GameState::LoadingMap`, or `GameState::Settings`. | 1 |
| FR-2 | All three systems must continue to run in `GameState::InGame`. | 1 |
| FR-3 | All three systems must continue to run in `GameState::Paused`, so that the occlusion visual is preserved correctly while the pause menu is overlaid on the game world. | 1 |
| FR-4 | The chained ordering of the three systems (`detect_interior_system → update_occlusion_uniforms → debug_draw_occlusion_zone`) must be preserved. | 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Phase |
|----|-------------|-------|
| NFR-1 | The change is confined to `OcclusionPlugin::build()` in `src/systems/game/occlusion/mod.rs`. No other files are modified. | 1 |
| NFR-2 | No new imports beyond what is already present in the file. `GameState` is already imported via `crate::states::GameState`. | 1 |

---

## 4. Open Questions

| ID | Question | Status |
|----|----------|--------|
| Q1 | Should the systems also run in `GameState::Settings`? Settings can be entered from the pause menu (game world visible in background) or from the title screen (no game world). | ✅ No — `Settings` entered from the title screen has no game entities; the edge case of settings-over-gameplay is uncommon and the visual glitch (occlusion pauses) is acceptable for a P4 fix |
