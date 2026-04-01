# Fix: custom_properties Has No Namespace Convention

**Date:** 2026-03-31
**Severity:** Low (p3)
**Component:** Map format — `src/systems/game/map/format/mod.rs`, entity format — `src/systems/game/map/format/entities.rs`, validation — `src/systems/game/map/validation.rs`

---

## Story

As a level designer, I want engine-defined keys in `custom_properties` and
`EntityData.properties` to use a reserved prefix so that my own
game-specific metadata never silently conflicts with keys the engine reads.

---

## Description

Both `MapData.custom_properties` and `EntityData.properties` are
`HashMap<String, String>` with no reserved key prefix. There is no documented
convention separating engine-owned keys from author-owned keys. Two independent
systems writing to the same map — or a future engine feature adding a new key —
can silently collide with author-defined keys that happen to share the same
name. The fix establishes `adrakestory:` as the reserved namespace prefix for
all engine-written keys, documents this convention in the format spec and in the
field doc-comments, and adds a validator warning for any key that uses the
prefix without being a known engine key. The map format version is bumped to
`1.1.0` to reflect the new convention. Author keys using other prefixes (or no
prefix) are unaffected.

---

## Acceptance Criteria

1. The field doc-comments on `MapData::custom_properties`
   (`src/systems/game/map/format/mod.rs:51`) and `EntityData::properties`
   (`src/systems/game/map/format/entities.rs:15`) document the `adrakestory:`
   prefix convention and state that keys beginning with `adrakestory:` are
   reserved for engine use.
2. `docs/api/map-format-spec.md` documents the namespace convention, the full
   list of currently reserved keys (initially none for `custom_properties`; none
   for `EntityData.properties` — the convention is established for future use),
   and an example showing correct author key naming.
3. `validate_map()` emits a `warn!()` log entry (not an error) for any
   `custom_properties` key that begins with `adrakestory:` but is not in the
   known engine key list. The map still loads successfully.
4. The same warning is emitted for any `EntityData.properties` key that begins
   with `adrakestory:` but is not in the known entity engine key list.
5. The warning message includes the offending key name and the map metadata name
   so authors can locate the source map.
6. Existing map files with no `adrakestory:`-prefixed keys load without any
   new warning or error.
7. A new `validate_custom_property_namespaces()` function (or equivalent) in
   `validation.rs` contains the prefix check and is covered by unit tests.
8. Unit tests cover: known engine key accepted silently, unknown `adrakestory:`
   key emits a warning path (tested via return value or side-effect), author key
   with no prefix accepted silently.
9. `MapMetadata::version` default value in `MapData::empty_map()` is updated
   from `"1.0.0"` to `"1.1.0"`.
10. `cargo build` succeeds for both `adrakestory` and `map_editor` with zero new
    errors or warnings.
11. `cargo test` passes with no new failures.
12. `cargo clippy` reports zero new errors.

---

## Non-Functional Requirements

- The namespace check must not be a hard error — maps with unknown
  `adrakestory:` keys must still load so that forward-compatibility is
  preserved when an older engine reads a map written by a newer one.
- No changes to `MapData`, `EntityData`, or their serde serialisation are
  required. The convention is enforced through documentation and validation
  warnings only, not through type-level changes.
- The reserved prefix `adrakestory:` must be defined as a named constant
  (`ENGINE_KEY_PREFIX`) in `validation.rs` (or a shared constants module) so it
  can be updated in one place.
- Both binaries must compile without new warnings after the change.
- The version bump to `"1.1.0"` must only be applied to `empty_map()` and to
  the spec's default example. Existing map files remain at `"1.0.0"` and must
  still load correctly — the validator accepts all `1.x.x` versions.

---

## Tasks

1. Define `const ENGINE_KEY_PREFIX: &str = "adrakestory:";` in
   `src/systems/game/map/validation.rs`.
2. Add `validate_custom_property_namespaces()` to `validation.rs` that iterates
   `map.custom_properties` keys and warns on unknown `adrakestory:` keys.
3. Add an analogous check inside `validate_entities()` for
   `EntityData.properties` keys.
4. Call both checks from `validate_map()`.
5. Update the doc-comment on `MapData::custom_properties` in
   `src/systems/game/map/format/mod.rs` to document the prefix convention.
6. Update the doc-comment on `EntityData::properties` in
   `src/systems/game/map/format/entities.rs` to document the prefix convention.
7. Update `docs/api/map-format-spec.md`: add a "Namespace Convention" subsection
   under "Custom Properties" listing the reserved prefix, the currently empty
   engine key table, and an authoring example.
8. Bump `MapData::empty_map()` default version from `"1.0.0"` to `"1.1.0"` in
   `src/systems/game/map/format/mod.rs`.
9. Write unit tests in `validation.rs` covering AC 7–8.
10. Run `cargo build`, `cargo test`, `cargo clippy`; fix any failures or new
    warnings.
