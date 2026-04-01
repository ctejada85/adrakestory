# Runtime Light Mutation

**Date:** 2026-04-01
**Component:** Lighting / Game Systems

---

## Story

As a game developer, I want to mutate `LightSource` component fields at runtime so that lights can flicker, change colour, or be toggled by gameplay events without requiring direct access to Bevy's `PointLight`.

---

## Description

`LightSource` (`src/systems/game/components.rs`) was designed as the gameplay API for light entities: other systems should write to `LightSource` and a sync system propagates the values to Bevy's `PointLight` for rendering. The `sync_light_sources` system (`src/systems/game/input.rs`) has now been implemented and registered, completing the core pipeline. This ticket covers the usage layer: adding example/gameplay systems that mutate `LightSource` (e.g., a flicker effect) and confirming the full round-trip works end-to-end. Out of scope: area lights, emissive mesh lighting, baked lightmaps.

---

## Acceptance Criteria

1. A system that mutates `LightSource::intensity` on a spawned light entity causes the rendered `PointLight` intensity to change within the same frame (via `sync_light_sources`).
2. `sync_light_sources` only runs for entities whose `LightSource` has actually changed (`Changed<LightSource>` filter), producing no overhead when lights are static.
3. All four fields (`color`, `intensity`, `range`, `shadows_enabled`) are correctly synchronised to the corresponding `PointLight` fields.
4. A unit test spawns a `LightSource` + `PointLight` entity, mutates `LightSource`, runs `sync_light_sources`, and asserts the `PointLight` values match.
5. At least one gameplay demonstration system (e.g., a simple intensity-flicker system gated behind a feature flag or a test map entity) exists in the codebase or in a dedicated example.

---

## Non-Functional Requirements

- `sync_light_sources` must run in `GameSystemSet::Visual` after physics and movement, before camera.
- The system must use `Changed<LightSource>` and must not touch `PointLight` entities that lack a `LightSource` component.
- Must not add a dependency on any new crate.

---

## Tasks

1. Write a unit test for `sync_light_sources` using Bevy's `World` directly (assert all four fields sync correctly).
2. Add a `flicker_lights` example system (or a simple demo in a test map) that oscillates `LightSource::intensity` over time.
3. Verify `sync_light_sources` is in `GameSystemSet::Visual` and gated to `InGame`/`Paused` states.
4. Manually verify in the running game that mutating `LightSource` fields causes visible light changes with no one-frame lag.
