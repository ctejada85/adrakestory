# Improvement: Remove Dead Code and Tighten Public API Surface

**Date:** 2026-04-09
**Severity:** Low
**Component:** Editor & Game — Various (`src/`)

---

## Story

As a developer working on the codebase, I want dead code and vestigial APIs removed so that I am not misled by functions that appear callable but are never invoked.

---

## Description

Four distinct dead-code issues exist across the codebase:

1. **`load_map_system`** (`src/systems/game/map/loader/mod.rs:195`) — A Bevy system decorated with `#[allow(dead_code)]` that hardcodes the path `"assets/maps/default.ron"`. It is never registered in any plugin or schedule. The actual load path is handled by `src/main.rs` via `MapLoader::load_from_file`.

2. **`render_viewport_controls`** (`src/editor/ui/viewport/mod.rs`) — Described in a comment as "deprecated" but carries neither a `#[deprecated]` attribute nor a scheduled removal milestone, making it invisible to Rust tooling and future contributors.

3. **`EditorCamera::calculate_position`** (`src/editor/camera/mod.rs:83`) — A public method that unconditionally returns `self.position`. It is never called anywhere in the codebase. Callers that need the position read `editor_cam.position` directly.

4. **`save_map_to_file`** (`src/editor/file_io/mod.rs:278`) — The function signature is `fn save_map_to_file(map: &MapData, path: &PathBuf)`. `PathBuf` dereferences to `Path`, so the idiomatic Rust signature is `path: impl AsRef<Path>` (or `path: &Path`). The current signature forces callers to pass a `&PathBuf` where a `&str` or `&Path` would work just as well.

---

## Acceptance Criteria

1. `load_map_system` is deleted from `src/systems/game/map/loader/mod.rs`; the `#[cfg(test)] mod tests;` declaration at the bottom of that file is preserved.
2. `render_viewport_controls` is either deleted (if no longer needed) or annotated with `#[deprecated(since = "...", note = "...")]` and scheduled for removal.
3. `EditorCamera::calculate_position` is deleted; any hypothetical callers (none exist as of this writing) are updated to use `editor_cam.position` directly.
4. `save_map_to_file` signature is changed to `path: &Path` (or `impl AsRef<Path>`); all call sites are updated.
5. `cargo clippy` reports zero warnings after the changes.
6. `cargo build` and `cargo test` succeed with no errors.

---

## Non-Functional Requirements

- No behavior change; all changes are purely structural.
- Public API items that are part of the inter-module contract must not be removed without checking all callers across both binaries (`adrakestory` and `map_editor`).
- Scope: `src/systems/game/map/loader/mod.rs`, `src/editor/ui/viewport/mod.rs`, `src/editor/camera/mod.rs`, `src/editor/file_io/mod.rs`.

---

## Tasks

1. Delete `load_map_system` from `src/systems/game/map/loader/mod.rs`.
2. Investigate `render_viewport_controls` in `src/editor/ui/viewport/mod.rs`; delete it or add `#[deprecated]`.
3. Delete `EditorCamera::calculate_position` from `src/editor/camera/mod.rs`.
4. Change `save_map_to_file` signature to `path: &Path` and update call sites.
5. Run `cargo clippy` and resolve any remaining warnings surfaced by the removals.
6. Run `cargo test` to confirm no tests broke.
