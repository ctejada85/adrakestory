# Fix bevy_egui Deprecation Warnings

**Date:** 2026-03-31
**Severity:** Low
**Component:** Editor UI / bevy_egui

---

## Story

As a developer, I want the editor codebase to use current egui APIs so that
`cargo clippy` produces no deprecation warnings and the editor stays compatible
with future egui upgrades.

---

## Description

An egui version upgrade (pulled in via `bevy_egui`) renamed or replaced six
APIs used across four editor source files, producing 37 `#[warn(deprecated)]`
warnings. All six are mechanical renames with no behaviour change — no new egui
features are required. The warnings are non-blocking today but obscure other
clippy output and will become hard errors if `#[deny(deprecated)]` is ever
enforced. Out of scope: any behaviour changes to the editor UI, and any
non-deprecation clippy warnings.

---

## Acceptance Criteria

1. `cargo clippy --lib` produces zero `#[warn(deprecated)]` warnings.
2. `Frame::rounding` is replaced with `.corner_radius()` at all 9 call sites in
   `palette.rs` and `viewport.rs`.
3. `Frame::none()` is replaced with `egui::Frame::NONE` or `egui::Frame::new()`
   at all 4 call sites in `viewport.rs`.
4. `Ui::close_menu()` is replaced with `ui.close()` at all 19 call sites in
   `outliner.rs` and `menus.rs`.
5. `egui::menu::bar(ui, …)` is replaced with `egui::MenuBar::new().ui(ui, …)`
   at the 1 call site in `toolbar/mod.rs`.
6. `Context::screen_rect()` is replaced with `ctx.content_rect()` at all 3
   call sites in `palette.rs` and `viewport.rs`, with each site verified to
   want content bounds (not raw viewport bounds).
7. `Memory::toggle_popup` is replaced with the current `Popup` API at the 1
   call site in `outliner.rs`.
8. The editor builds successfully (`cargo build --bin map_editor`) after all
   changes.
9. All existing lib tests continue to pass (`cargo test --lib`).

---

## Non-Functional Requirements

- Must not change any visible editor UI behaviour.
- Must not introduce new clippy warnings beyond those pre-existing before this
  change.
- Changes are editor-only; no game runtime code (`src/systems/`) is modified.

---

## Tasks

1. Replace `Frame::rounding` → `.corner_radius()` in `palette.rs` and
   `viewport.rs` (9 sites — pure rename, safe to do with search-and-replace).
2. Replace `Frame::none()` → `egui::Frame::NONE` in `viewport.rs` (4 sites —
   use the constant form; chain `.show()` if needed).
3. Replace `Ui::close_menu()` → `ui.close()` in `outliner.rs` and `menus.rs`
   (19 sites — pure rename).
4. Replace `egui::menu::bar(ui, |ui| {` → `egui::MenuBar::new().ui(ui, |ui| {`
   in `toolbar/mod.rs` (1 site).
5. Replace `ctx.screen_rect()` → `ctx.content_rect()` in `palette.rs` and
   `viewport.rs` (3 sites — verify each returns the correct rect for its use).
6. Replace `mem.toggle_popup(id)` → `egui::Popup::toggle_id(ctx, id)` in
   `outliner.rs` (1 site — verify exact `Popup` API signature for the installed
   egui version).
7. Run `cargo build --bin map_editor` and confirm clean build.
8. Run `cargo clippy --lib` and confirm zero `deprecated` warnings remain.
9. Run `cargo test --lib` and confirm all tests pass.
10. Manually open the editor, use the toolbar menus, outliner add-entity popup,
    and palette panel to confirm no visible regressions.
