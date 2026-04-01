# Requirements — Entity Property Validation

**Source:** Map Format Analysis Investigation — 2026-03-22  
**Bug:** `docs/bugs/entity-properties-silent-parse-failure/ticket.md`  
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

All entity configuration in the RON map format is stored as `HashMap<String, String>`
in `EntityData.properties` (`src/systems/game/map/format/entities.rs:14–15`). At
spawn time, `spawn_light_source()` and `spawn_npc()` in
`src/systems/game/map/spawner/entities.rs:184–242` parse these strings into typed
values using `.and_then(|s| s.parse::<f32>().ok())`. Any value that fails to
parse silently falls back to a hardcoded default. The map author receives no
error or warning; the entity spawns with the wrong configuration.

The fix adds property validation inside `validate_entities()` in `validation.rs`.
For each entity, a new helper `validate_entity_properties()` checks that
string-valued properties for known entity types (`LightSource`, `Npc`) are
well-formed before any spawning begins. Unknown keys and unknown entity types
pass without error (forward-compatibility). The spawner's fallback logic is not
changed.

---

## 2. Functional Requirements

### 2.1 LightSource Property Validation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | If a `LightSource` entity has an `intensity` key, its value must parse as a non-negative `f32`. A value that fails `str::parse::<f32>()` or is negative returns `Err(ValidationError(...))`. | Phase 1 |
| FR-2.1.2 | If a `LightSource` entity has a `range` key, its value must parse as a strictly positive `f32` (> 0.0). A non-parseable or non-positive value returns `Err(ValidationError(...))`. | Phase 1 |
| FR-2.1.3 | If a `LightSource` entity has a `shadows` key, its value must be one of `"true"`, `"false"`, `"1"`, or `"0"` (case-sensitive). Any other string returns `Err(ValidationError(...))`. | Phase 1 |
| FR-2.1.4 | If a `LightSource` entity has a `color` key, its value must be three comma-separated tokens each parseable as `f32`. A value that does not yield exactly three parseable floats returns `Err(ValidationError(...))`. | Phase 1 |

### 2.2 Npc Property Validation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | If an `Npc` entity has a `radius` key, its value must parse as a strictly positive `f32` (> 0.0). A non-parseable or non-positive value returns `Err(ValidationError(...))`. | Phase 1 |

### 2.3 Forward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | Unknown property keys on any entity type are accepted without error. The validator only checks the keys it knows about. | Phase 1 |
| FR-2.3.2 | Entity types other than `LightSource` and `Npc` are accepted without property validation. | Phase 1 |

### 2.4 Error Messages

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | Each `ValidationError` message must name the entity type and the offending key so the author can find the entry in the RON file. Example: `"LightSource entity has invalid 'intensity': expected non-negative f32, got \"abc\""`. | Phase 1 |

### 2.5 Integration with Existing Validation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | Property validation must be called from within the existing `validate_entities()` function, after the existing position bounds check, for each entity. | Phase 1 |
| FR-2.5.2 | All existing acceptance criteria for `validate_entities()` (player spawn required, position bounds) must be preserved unchanged. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | No new `MapLoadError` variants are required; `ValidationError(String)` is sufficient. | Phase 1 |
| NFR-3.2 | The spawner (`entities.rs`) fallback parse logic must not be modified. The validator and spawner are independent. | Phase 1 |
| NFR-3.3 | Both binaries (`adrakestory` and `map_editor`) must compile without new errors or warnings. | Phase 1 |
| NFR-3.4 | All existing tests must continue to pass. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (this fix)

- `validate_entity_properties()` helper validates `LightSource` and `Npc` properties.
- Called from `validate_entities()` for each entity.
- Unit tests for each invalid property type and a passing valid-property case.
- Both binaries compile cleanly with zero new clippy errors.

### Phase 2 — Future (out of scope)

- Replace `HashMap<String, String>` with per-variant typed config structs using serde adjacency.
- Editor-side property form with type-aware inputs and live validation.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The loader pipeline `migrate_legacy_rotations() → normalise_staircase_variants() → validate_map()` is stable (commits `eda90e3`, `4874885`). |
| 2 | The spawner's fallback values — `intensity: 10000.0`, `range: 10.0`, `shadows: false`, `color: WHITE`, `radius: 0.3` — are not changed by this fix. |
| 3 | The fix does not attempt to collect all invalid properties before returning; fail-fast on the first invalid value is consistent with all other validators. |
| 4 | Color component range (`[0.0, 1.0]`) is **not** validated in Phase 1. Validating that the string is parseable as three floats is sufficient to catch the most common authoring mistakes. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should color component values be range-checked (0.0–1.0)? | **No, Phase 1 only validates parseability.** Bevy clamps or wraps out-of-range values; a range check is Phase 2. |
| 2 | Should `intensity` and `range` clamping limits be validated? | **No.** The spawner clamps them silently; exceeding the clamp is not an authoring error. Only unparseable values are rejected. |
| 3 | Should unknown property keys trigger a warning? | **No.** Silent acceptance is required for forward-compatibility (FR-2.3.1). |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system (`map-format-multi-axis-rotation`) | **Done** (commit `eda90e3`) | Team |
| 2 | Fix 2 — Staircase normalisation (`staircase-double-rotation`) | **Done** (commit `4874885`) | Team |
| 3 | Fix 4 — Duplicate voxel detection (`duplicate-voxel-positions`) | **Done** (commit `9f960d1`) | Team |

---

## 8. Reference: Example Scenarios

| Type | Property | Expected result |
|------|----------|-----------------|
| Valid | `intensity: "5000.0"` on LightSource | `validate_map()` returns `Ok(())` |
| Invalid | `intensity: "bright"` on LightSource | `Err(ValidationError("LightSource entity has invalid 'intensity': ..."))` |
| Invalid | `range: "-1.0"` on LightSource | `Err(ValidationError("LightSource entity has invalid 'range': ..."))` |
| Invalid | `shadows: "yes"` on LightSource | `Err(ValidationError("LightSource entity has invalid 'shadows': ..."))` |
| Invalid | `color: "red"` on LightSource | `Err(ValidationError("LightSource entity has invalid 'color': ..."))` |
| Invalid | `radius: "big"` on Npc | `Err(ValidationError("Npc entity has invalid 'radius': ..."))` |
| Unknown key | `foo: "bar"` on LightSource | `Ok(())` — unknown key silently accepted |
| Absent key | No `intensity` on LightSource | `Ok(())` — absent key uses spawner default |

---

*Created: 2026-03-31*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 5*
