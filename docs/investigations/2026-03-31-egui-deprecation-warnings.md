# Investigation: bevy_egui Deprecation Warnings

**Date:** 2026-03-31
**Status:** Complete
**Component:** Editor UI / bevy_egui

## Summary

`cargo clippy` emits 37 `#[warn(deprecated)]` warnings across 4 editor source
files. All warnings originate from a single upstream cause: the `bevy_egui`
dependency upgraded to an egui version that renamed or replaced several APIs.
No new egui features are required; each warning has a mechanical one-for-one
replacement. The warnings are cosmetic (compile and run fine) but will become
hard errors if `#[deny(deprecated)]` is ever added, and they obscure other
warning output.

## Environment

- Rust toolchain: stable (cargo clippy, `adrakestory` lib)
- `bevy_egui` version: current in `Cargo.lock`
- Affected files: 4 editor source files
- Warning count: 37 deprecation warnings across 6 distinct deprecated APIs

## Investigation Method

`cargo clippy --lib` run in full; output filtered to `#[warn(deprecated)]`
lines only. Warnings grouped by deprecated symbol and mapped to replacement
API from the compiler hint.

## Findings

### Finding 1 — `egui::menu::bar` renamed (p3 Medium)

**File:** `src/editor/ui/toolbar/mod.rs:52`

```rust
egui::menu::bar(ui, |ui| { ... });
```

**Replacement:** `egui::MenuBar::new().ui(ui, |ui| { ... });`

Occurs 1 time. The free function `egui::menu::bar` has been replaced with a
builder-style `MenuBar` struct.

---

### Finding 2 — `Context::screen_rect` split into two methods (p3 Medium)

**Files:**
- `src/editor/controller/palette.rs:30` — `ui.ctx().screen_rect()`
- `src/editor/controller/palette.rs:165` — `ctx.screen_rect()`
- `src/editor/ui/viewport.rs:48` — `ctx.screen_rect()`

```rust
let screen_rect = ui.ctx().screen_rect();
```

**Replacement:** `ctx.content_rect()` (for content layout) or `ctx.viewport_rect()` (for the full window). The compiler hint recommends `content_rect()` for the typical use case. Review each call site to confirm the correct variant.

Occurs 3 times across 2 files.

---

### Finding 3 — `Frame::rounding` renamed to `corner_radius` (p3 Medium)

**Files:**
- `src/editor/controller/palette.rs:84, 127, 197, 216, 259`
- `src/editor/ui/viewport.rs:108, 175, 227, 305`

```rust
egui::Frame::new().rounding(4.0)
```

**Replacement:** `.corner_radius(4.0)`

Pure rename. Occurs 9 times across 2 files.

---

### Finding 4 — `Frame::none` renamed to `Frame::NONE` / `Frame::new()` (p3 Medium)

**File:** `src/editor/ui/viewport.rs:105, 172, 224, 302`

```rust
egui::Frame::none()
```

**Replacement:** `egui::Frame::NONE` (constant) or `egui::Frame::new()` (mutable builder). Use `Frame::NONE` when the frame is used directly; use `Frame::new()` when builder methods are chained.

Occurs 4 times in 1 file.

---

### Finding 5 — `Ui::close_menu` replaced by `Ui::close` (p3 Medium)

**Files:**
- `src/editor/ui/outliner.rs:274`
- `src/editor/ui/toolbar/menus.rs:30, 40, 60, 68, 77, 82, 94, 120, 134, 175, 181, 248, 263, 275, 289, 298, 308, 313`

```rust
ui.close_menu();
```

**Replacement:** `ui.close()` or `ui.close_kind(UiKind::Menu)`. Use `ui.close()` for the common case; `close_kind` only needed if a non-menu UI kind must be explicitly targeted.

Occurs 19 times across 2 files. This is the highest-volume individual warning.

---

### Finding 6 — `Memory::toggle_popup` replaced by `Popup::toggle_id` (p3 Medium)

**File:** `src/editor/ui/outliner.rs:306`

```rust
ui.memory_mut(|mem| mem.toggle_popup(ui.make_persistent_id("add_entity_popup")));
```

**Replacement:** `egui::Popup::toggle_id(ui.ctx(), ui.make_persistent_id("add_entity_popup"))` (API shape; verify exact signature against egui docs for the installed version).

Occurs 1 time.

---

## Root Cause Summary

| # | Root Cause | Location | Count | Priority |
|---|-----------|----------|-------|----------|
| 1 | `egui::menu::bar` → `MenuBar::new().ui()` | `toolbar/mod.rs` | 1 | p3 Medium |
| 2 | `Context::screen_rect` → `content_rect()` / `viewport_rect()` | `palette.rs`, `viewport.rs` | 3 | p3 Medium |
| 3 | `Frame::rounding` → `corner_radius` | `palette.rs`, `viewport.rs` | 9 | p3 Medium |
| 4 | `Frame::none()` → `Frame::NONE` / `Frame::new()` | `viewport.rs` | 4 | p3 Medium |
| 5 | `Ui::close_menu` → `ui.close()` | `outliner.rs`, `menus.rs` | 19 | p3 Medium |
| 6 | `Memory::toggle_popup` → `Popup::toggle_id` | `outliner.rs` | 1 | p3 Medium |

All 6 root causes are API renames/restructures introduced by an egui version
upgrade. No logic changes are required — only mechanical call-site updates.

## Recommended Fixes

Fix all 6 in a single pass; they are independent and can be batched into one
commit. Suggested order (most mechanical first):

1. **Finding 3** — global rename `.rounding(` → `.corner_radius(` (9 sites, 2 files, pure rename)
2. **Finding 4** — `Frame::none()` → `Frame::NONE` or `Frame::new()` (4 sites, 1 file)
3. **Finding 5** — `close_menu()` → `close()` (19 sites, 2 files, pure rename)
4. **Finding 1** — `egui::menu::bar(ui, |ui| {` → `egui::MenuBar::new().ui(ui, |ui| {` (1 site)
5. **Finding 2** — `screen_rect()` → `content_rect()` with per-site review (3 sites)
6. **Finding 6** — `toggle_popup` → `Popup::toggle_id` with API verification (1 site)

After fixes: `cargo clippy --lib` should show zero `deprecated` warnings.

## Related Bugs

None filed — all findings are straightforward renames with compiler-provided
replacement hints.
