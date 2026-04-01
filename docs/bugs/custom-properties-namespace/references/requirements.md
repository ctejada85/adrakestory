# Requirements — custom_properties Namespace Convention

**Source:** Map Format Analysis Investigation — 2026-03-22
**Bug:** `docs/bugs/custom-properties-namespace/ticket.md`
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

Both `MapData.custom_properties` (`src/systems/game/map/format/mod.rs:52`) and
`EntityData.properties` (`src/systems/game/map/format/entities.rs:15`) are
`HashMap<String, String>`. There is no documented convention separating keys
owned by the engine from keys owned by the map author. Any future engine feature
that adds a new well-known key — say `adrakestory:spawn_music` — risks silently
colliding with an author key already named the same way. Likewise, two
independent tools writing to the same map have no reliable way to avoid key
collisions without prior coordination.

The fix establishes `adrakestory:` as a reserved prefix for all engine-defined
keys, documents this convention in the format spec and in the struct
doc-comments, and adds a validator warning (not an error) when an unknown
`adrakestory:`-prefixed key is encountered. The map format version is bumped
from `1.0.0` to `1.1.0` in `MapData::empty_map()` to record when the convention
was introduced. No type-level changes to `MapData` or `EntityData` are needed;
the convention is enforced through documentation and soft validation only.

---

## 2. Functional Requirements

### 2.1 Namespace Convention — Documentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The doc-comment on `MapData::custom_properties` must state that keys beginning with `adrakestory:` are reserved for engine use and must not be written by map authors. | Phase 1 |
| FR-2.1.2 | The doc-comment on `EntityData::properties` must state the same `adrakestory:` reservation. | Phase 1 |
| FR-2.1.3 | `docs/api/map-format-spec.md` must gain a "Namespace Convention" subsection under the "Custom Properties" section documenting: the reserved prefix, the currently empty engine key table, and a worked example showing valid author key naming. | Phase 1 |

### 2.2 Namespace Convention — Validation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | A new `validate_custom_property_namespaces()` function must be added to `src/systems/game/map/validation.rs`. It must iterate `MapData::custom_properties` and, for each key that begins with `adrakestory:`, emit a `warn!()` log entry if the key is not in the known engine key list. The map must still load (no error returned). | Phase 1 |
| FR-2.2.2 | The same prefix check must be applied to `EntityData::properties` for each entity in `map.entities`, emitting a `warn!()` per unknown `adrakestory:` key. | Phase 1 |
| FR-2.2.3 | The warning message must include: the offending key name, the string `"map"` followed by the map metadata name, and the word `"reserved"` so that authors searching their logs can identify the source. | Phase 1 |
| FR-2.2.4 | The reserved prefix must be defined as a named constant `ENGINE_KEY_PREFIX: &str = "adrakestory:"` in `validation.rs` so it can be changed in one place. | Phase 1 |
| FR-2.2.5 | `validate_map()` must call `validate_custom_property_namespaces()` after `validate_entities()`. | Phase 1 |

### 2.3 Known Engine Key Lists

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | A `KNOWN_MAP_ENGINE_KEYS: &[&str]` constant (or equivalent) must be defined in `validation.rs` listing all currently engine-owned `MapData::custom_properties` keys. As of this fix the list is empty (`&[]`). | Phase 1 |
| FR-2.3.2 | A `KNOWN_ENTITY_ENGINE_KEYS: &[&str]` constant (or equivalent) must be defined listing all engine-owned `EntityData::properties` keys prefixed with `adrakestory:`. As of this fix the list is empty (`&[]`). | Phase 1 |
| FR-2.3.3 | Both constants must carry a doc-comment instructing maintainers to add new engine keys here before writing them to map files. | Phase 1 |

### 2.4 Version Bump

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `MapData::empty_map()` in `src/systems/game/map/format/mod.rs` must initialise `metadata.version` to `"1.1.0"` (previously `"1.0.0"`). | Phase 1 |
| FR-2.4.2 | The validator's version check must continue to accept all `1.x.x` versions (i.e. the existing `starts_with("1.")` check is sufficient and must not be tightened). | Phase 1 |

### 2.5 Unit Tests

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | A test `author_key_without_prefix_is_accepted` must confirm that a `custom_properties` key with no `adrakestory:` prefix produces no error and does not trigger the warning path. | Phase 1 |
| FR-2.5.2 | A test `unknown_engine_key_triggers_warning` must confirm that a `custom_properties` key beginning with `adrakestory:` that is not in `KNOWN_MAP_ENGINE_KEYS` causes `validate_custom_property_namespaces()` to return a non-empty warning list (or equivalent observable signal). | Phase 1 |
| FR-2.5.3 | A test `known_engine_key_is_accepted_silently` must confirm that a key present in `KNOWN_MAP_ENGINE_KEYS` does not appear in the warning list. (This test will require at least one entry to be added to the constant for the duration of the test, or the constant to be parameterised in the test.) | Phase 1 |
| FR-2.5.4 | The existing `unknown_property_key_is_accepted` test in `validation.rs` must continue to pass unchanged. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The namespace check must not be a hard error. Maps with unknown `adrakestory:` keys must still load so that forward-compatibility is preserved when an older engine reads a map written by a newer one. | Phase 1 |
| NFR-3.2 | No changes to `MapData`, `EntityData`, or their serde `Serialize`/`Deserialize` implementations are required. The convention is enforced through documentation and soft validation only. | Phase 1 |
| NFR-3.3 | `ENGINE_KEY_PREFIX`, `KNOWN_MAP_ENGINE_KEYS`, and `KNOWN_ENTITY_ENGINE_KEYS` must be defined in `validation.rs` (not in a separate module) to keep the validation logic self-contained and easy to locate. | Phase 1 |
| NFR-3.4 | Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings. | Phase 1 |
| NFR-3.5 | The version bump to `"1.1.0"` must only affect `empty_map()`. Existing map files at `"1.0.0"` must continue to load without error or new warnings. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (this fix)

- `adrakestory:` prefix reserved and documented in struct doc-comments.
- `docs/api/map-format-spec.md` updated with namespace convention section.
- `validate_custom_property_namespaces()` added; integrated into `validate_map()`.
- `ENGINE_KEY_PREFIX`, `KNOWN_MAP_ENGINE_KEYS`, `KNOWN_ENTITY_ENGINE_KEYS` constants defined.
- Unit tests added.
- `empty_map()` version bumped to `"1.1.0"`.
- Both binaries compile cleanly.

### Phase 2 — Future (out of scope)

- Add engine-owned keys to the known-keys lists as actual engine features
  begin writing to `custom_properties` (e.g. `adrakestory:spawn_music`,
  `adrakestory:ambient_sound`).
- Consider a typed wrapper (`EngineProperties`) that provides typed accessors
  for known engine keys while still accepting arbitrary author keys, eliminating
  the runtime string parsing currently done in the spawner.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `adrakestory:` (lowercase, colon suffix) is chosen as the prefix because it matches the crate/binary name, is unambiguous in a RON string context, and is unlikely to appear in author-defined keys by accident. |
| 2 | The known-engine-key lists are empty at the time of this fix. No engine code currently writes `adrakestory:`-prefixed keys. The convention is established proactively for future use. |
| 3 | Warn-only (not error) is the correct behaviour for unknown engine keys. An older engine reading a map produced by a newer engine that added new engine keys must still load the map. |
| 4 | The version bump to `"1.1.0"` is cosmetic — it records the introduction of the convention in new maps without invalidating existing `"1.0.0"` files. The validator's `starts_with("1.")` check already accepts both. |
| 5 | `EntityData.properties` keys are validated in the same pass as `MapData.custom_properties`. No additional call site is required in `validate_entities()` beyond the new prefix check. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should the check be a hard error or a warning? | **Warning only.** A hard error would break forward-compatibility — a map written by engine 1.1 with a new key would fail to load in engine 1.0. |
| 2 | What prefix string? | **`adrakestory:`** — matches the binary/crate name, colon-separated to avoid false matches like `adrakestory_custom`. |
| 3 | Where should the constants live? | **`validation.rs`** — keeps the validation logic self-contained. A separate constants module adds navigation overhead for no benefit at this scale. |
| 4 | Should the version bump be a minor or patch increment? | **Minor** (`1.0.0` → `1.1.0`). Adding a new convention (even a documentation-only one) is a minor, backward-compatible change per SemVer. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Finding 5 — entity property validation | **Done** (commit `a978d23`) | Team |

No further blockers. This is an independent documentation and validation change.

---

## 8. Reference: Affected Files

| File | Change |
|------|--------|
| `src/systems/game/map/format/mod.rs` | **Modified** — update `custom_properties` doc-comment; bump `empty_map()` version |
| `src/systems/game/map/format/entities.rs` | **Modified** — update `properties` doc-comment |
| `src/systems/game/map/validation.rs` | **Modified** — add `ENGINE_KEY_PREFIX`, `KNOWN_MAP_ENGINE_KEYS`, `KNOWN_ENTITY_ENGINE_KEYS`; add `validate_custom_property_namespaces()`; call from `validate_map()` |
| `docs/api/map-format-spec.md` | **Modified** — add namespace convention subsection under "Custom Properties" |

---

*Created: 2026-03-31*
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 9*
