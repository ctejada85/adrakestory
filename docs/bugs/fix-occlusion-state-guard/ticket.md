# Fix Occlusion Systems State Guard

**Date:** 2026-03-16  
**Severity:** Low (P4)  
**Component:** Occlusion system / System registration  

---

## Story

As a developer, I want occlusion systems to be skipped by the scheduler in non-gameplay states so that there is no unnecessary CPU overhead during the title screen, loading screen, and intro animation.

---

## Description

`OcclusionPlugin::build()` registers `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` without a `.run_if` state condition, so they are dispatched every frame in all `GameState` variants. Every other gameplay system in `main.rs` is gated with `.run_if(in_state(GameState::InGame))`. The fix adds `.run_if(in_state(GameState::InGame).or(in_state(GameState::Paused)))` to the system chain in `OcclusionPlugin::build()`. The change is one line in one file.

---

## Acceptance Criteria

1. `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` are not dispatched in `GameState::TitleScreen`, `GameState::IntroAnimation`, `GameState::LoadingMap`, or `GameState::Settings`.
2. All three systems continue to run in `GameState::InGame` with unchanged behaviour.
3. All three systems continue to run in `GameState::Paused` (occlusion visual preserved behind pause menu).
4. The chained ordering of the three systems is preserved.
5. `cargo test --lib`, `cargo clippy --lib`, and `cargo build --release` all pass.

---

## Non-Functional Requirements

- The change is confined to `OcclusionPlugin::build()` in `src/systems/game/occlusion/mod.rs`; no other files are modified.

---

## Tasks

1. In `OcclusionPlugin::build()`, append `.run_if(in_state(GameState::InGame).or(in_state(GameState::Paused)))` after `.chain()` on the system tuple (see `architecture.md` §2.1 for the exact code).
2. Run `cargo test --lib`, `cargo clippy --lib`, and `cargo build --release`; fix any failures.
3. Verify via `RUST_LOG=debug cargo run --release 2>&1 | grep -i occlusion` that no occlusion log lines appear during the title screen or loading screen states.
