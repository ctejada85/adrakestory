# Fix: Entity Properties Silent Parse Failures

**Date:** 2026-03-31  
**Severity:** Medium (p2)  
**Component:** Map validation — `src/systems/game/map/validation.rs`, entity spawner — `src/systems/game/map/spawner/entities.rs`

---

## Story

As a level designer, I want the map loader to reject a map file that contains unparseable entity property values so that misconfigured entities are caught at load time instead of silently spawning with hardcoded defaults that do not match the author's intent.

---

## Description

All entity configuration is stored as `HashMap<String, String>` in `EntityData.properties` (`src/systems/game/map/format/entities.rs:14–15`). At spawn time, the spawner (`src/systems/game/map/spawner/entities.rs:184–242`) parses these strings into typed values. Any value that fails to parse — a non-numeric intensity, an out-of-range range, a malformed color — falls through to a hardcoded default with no log entry and no error. The author receives no feedback and the entity silently spawns with wrong configuration.

The fix adds a `validate_entity_properties()` helper called from `validate_entities()` in `validation.rs`. For each `LightSource` entity it validates that `intensity` (if present) parses as a non-negative `f32`, `range` (if present) parses as a positive `f32`, `shadows` (if present) is `"true"`, `"false"`, `"1"`, or `"0"`, and `color` (if present) is three comma-separated `f32` values in `[0.0, 1.0]`. For `Npc` entities, it validates that `radius` (if present) parses as a positive `f32`. Unknown keys and unknown entity types are accepted without error (forward-compatibility).

Out of scope: replacing `HashMap<String, String>` with typed serde structs — that is a larger format change tracked separately.

---

## Acceptance Criteria

1. Loading a map where a `LightSource` entity has `intensity: "not_a_number"` returns `Err(MapLoadError::ValidationError(...))` before any spawning occurs.
2. Loading a map where a `LightSource` entity has `range: "-5.0"` (non-positive) returns `Err(MapLoadError::ValidationError(...))` before any spawning occurs.
3. Loading a map where a `LightSource` entity has `shadows: "yes"` (not a recognised boolean string) returns `Err(MapLoadError::ValidationError(...))` before any spawning occurs.
4. Loading a map where a `LightSource` entity has `color: "red"` (not three comma-separated floats) returns `Err(MapLoadError::ValidationError(...))` before any spawning occurs.
5. Loading a map where an `Npc` entity has `radius: "hello"` returns `Err(MapLoadError::ValidationError(...))` before any spawning occurs.
6. Loading a valid map with correct or absent properties succeeds without error.
7. Unknown keys on any entity type are silently accepted (forward-compatibility).
8. Unit tests are added to `validation.rs` covering each invalid property type above, plus a test confirming valid properties pass.
9. `cargo test` passes with no failures and `cargo clippy` reports no new errors.

---

## Non-Functional Requirements

- Validation runs inside `validate_entities()` in `validation.rs`; no changes to the spawner logic or error fallback behaviour.
- No new `MapLoadError` variants required; `ValidationError(String)` is sufficient.
- Error messages must name the entity type and the offending key so the author can find the entry in the RON file.
- Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings.

---

## Tasks

1. Add a `validate_entity_properties(entity: &EntityData) -> MapResult<()>` helper in `src/systems/game/map/validation.rs` that validates `LightSource` and `Npc` property strings.
2. Call `validate_entity_properties(&entity)?` inside the `validate_entities()` loop, after the existing position bounds check.
3. Add unit tests in the `#[cfg(test)]` block of `validation.rs`: invalid intensity, negative range, unrecognised shadows value, malformed color, invalid NPC radius, and a passing case with valid properties.
4. Run `cargo test` and `cargo clippy`; fix any failures or new warnings.
