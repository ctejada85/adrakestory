# Improvement: Eliminate Duplicate Map Path Resources

**Date:** 2026-04-09
**Severity:** Low
**Component:** Game — Startup (`src/main.rs`, `src/systems/game/hot_reload/`)

---

## Story

As a developer, I want the CLI map path stored in exactly one resource so that there is a single source of truth and no risk of the two copies diverging.

---

## Description

`src/main.rs` (lines 152–155) inserts two separate resources that both store the same `Option<PathBuf>`:

```rust
.insert_resource(CommandLineMapPath { path: args.map_path })
.insert_resource(MapPathForHotReload(hot_reload_path))
```

`hot_reload_path` is assigned on line 133 as `args.map_path.clone()`. The two resources are initialised from the same value and are never mutated independently. `CommandLineMapPath` is consumed by the map-loading system; `MapPathForHotReload` is consumed by the hot-reload system. Both can be served by a single resource.

Keeping two resources creates maintenance risk: a future change that updates one resource but not the other would introduce a subtle inconsistency (e.g. hot-reload reloads a different file than the one originally loaded).

---

## Acceptance Criteria

1. Only one resource stores the CLI map path after the change.
2. The map-loading system and the hot-reload system both read from the same resource.
3. `CommandLineMapPath` and `MapPathForHotReload` are deduplicated to a single type with a clear name (e.g. keep `MapPathForHotReload` since the hot-reload module already defines it, or introduce a new canonical type).
4. `cargo build --bin adrakestory` and `cargo build --bin map_editor` succeed.
5. Hot reload (Ctrl+R / F5) still reloads the correct map after the change.

---

## Non-Functional Requirements

- No new resources may be introduced; the result must be exactly one fewer resource than currently exists.
- The change must not affect the map-editor binary, which does not use either of these resources.

---

## Tasks

1. Choose a canonical resource type (keep `MapPathForHotReload` or introduce a shared `CliMapPath`).
2. Remove the redundant resource insertion from `src/main.rs`.
3. Update the map-loading system to read from the chosen resource instead of `CommandLineMapPath`.
4. Delete the now-unused resource struct (`CommandLineMapPath` or the deprecated type).
5. Run `cargo build` for both binaries and `cargo test` to confirm nothing broke.
6. Verify hot reload still works by running the game with `--map` and pressing Ctrl+R.
