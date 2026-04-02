# Requirements — NPC Display Names

**Source:** `docs/bugs/npc-display-names/ticket.md` — 2026-04-01
**Status:** Draft

---

## 1. Overview

NPCs in the game world carry a `name: String` field (populated from map data at spawn time) but no system currently reads or renders it. Players have no way to identify individual characters they encounter. This feature adds a world-space text label that floats above each NPC and shows/hides based on the player's proximity, making NPC identification possible without any new map format changes or editor UI work.

The change is additive: it introduces two new systems and removes the `#[allow(dead_code)]` suppression on `Npc::name`. No existing gameplay mechanics, collision behaviour, or map data structures are modified.

---

## 2. Functional Requirements

### 2.1 Label Spawning

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | When an `Npc` entity is spawned, a child text-label entity must be created above the NPC's origin. | Phase 1 |
| FR-2.1.2 | The label entity must be a child of the NPC entity (parented via `ChildOf`) so it moves with the NPC if the NPC is ever repositioned. | Phase 1 |
| FR-2.1.3 | The label must use `Text2d` (Bevy's built-in world-space text component) — no third-party crates. | Phase 1 |
| FR-2.1.4 | If `Npc::name` is the empty string `""` or the exact string `"NPC"` (case-sensitive), no label entity is spawned. | Phase 1 |

### 2.2 Label Visibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The label is only visible when the player is within a configurable interaction range of the NPC. | Phase 1 |
| FR-2.2.2 | The label is hidden (`Visibility::Hidden`) when the player moves outside interaction range. | Phase 1 |
| FR-2.2.3 | The default interaction range is 3.0 world units (horizontal distance). | Phase 1 |
| FR-2.2.4 | The visibility check must run every frame while `GameState::InGame` or `GameState::Paused` is active (labels remain visible during the pause menu). | Phase 1 |

### 2.3 Label Appearance

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | The label uses Bevy's embedded default font — no font asset file is required. | Phase 1 |
| FR-2.3.2 | The label is positioned at a fixed Y offset above the NPC origin (approximately `+1.2` world units, above the character model). | Phase 1 |
| FR-2.3.3 | The label text is the `Npc::name` string exactly as stored. | Phase 1 |
| FR-2.3.4 | The label must scale with camera distance so it remains legible at close range but does not dominate the screen. | Phase 1 |

### 2.4 System Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | The spawn system must run in `GameSystemSet::Visual` under `GameState::InGame`. | Phase 1 |
| FR-2.4.2 | The visibility update system must run in `GameSystemSet::Visual` under both `GameState::InGame` and `GameState::Paused`. | Phase 1 |
| FR-2.4.3 | Existing `Npc::radius` collision behaviour must not be affected. | Phase 1 |
| FR-2.4.4 | The feature must work correctly in the game binary; the editor binary may exclude it (editor does not run `GameState::InGame` systems). | Phase 1 |

### 2.5 Testing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | A unit or integration test must verify that a label child entity is present after `spawn_npc` is called with a non-default NPC name. | Phase 1 |
| FR-2.5.2 | A test must verify that no label entity is spawned when `Npc::name` is `"NPC"` (the default) or empty. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | Label rendering must not cause a noticeable FPS drop when 10 or more NPCs are on screen simultaneously. The visibility system must iterate only NPC entities (not all SubVoxel entities) and must not use O(n²) queries. | Phase 1 |
| NFR-3.2 | Must use Bevy's existing text/UI pipeline (`Text2d`) — no third-party crates are permitted. | Phase 1 |
| NFR-3.3 | No changes to the RON map format or `EntityData` struct are needed — `Npc::name` is already populated from map properties at spawn time. | Phase 1 |
| NFR-3.4 | The `#[allow(dead_code)]` attribute on `Npc::name` must be removed once the field is read by the new system. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- `spawn_npc_label` system: create a `Text2d` child entity for each NPC with a non-default name at spawn time
- `update_npc_label_visibility` system: show/hide labels based on player proximity (range: 3.0 world units)
- Both systems registered under `GameSystemSet::Visual` in `src/main.rs`
- Unit tests for spawn (name present) and no-spawn (default name / empty)
- `#[allow(dead_code)]` removal from `Npc::name`

### Phase 2 — Enhanced

- Configurable interaction range per NPC (via map properties, e.g. `"label_range": "5.0"`)
- Custom font support (load a `.ttf` asset for label text)
- Smooth fade-in / fade-out instead of instant show/hide

### Future Phases

- Dialogue HUD: show NPC name in a screen-space interaction prompt
- NPC name input in the map editor UI
- Localisation / display-name overrides

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `Npc::name` is populated from map data before the spawn systems run — no change to `spawn_npc()` or the RON format is required. |
| 2 | Bevy 0.18 `Text2d` is available as a world-space text component without additional Cargo features beyond what is already enabled. |
| 3 | The editor binary does not run `GameState::InGame` systems, so the label systems are automatically excluded from the editor without an explicit guard. |
| 4 | No font files exist in `assets/` — the default Bevy embedded font is used. |
| 5 | NPC entities are not stored in `SpatialGrid`; the visibility system must query NPC entities directly (the set is expected to be small, so O(n) direct iteration is acceptable). |
| 6 | The `Text2d` label Y offset of `+1.2` world units is the initial best-guess value (character model top ≈ `+0.7`); it will be tuned after implementation and in-game review. |

---

## 6. Open Questions

All questions resolved — no open items.

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `Npc::name` populated at spawn time from map data | Done — `spawn_npc()` in `src/systems/game/map/spawner/entities.rs` | — |
| 2 | Bevy 0.18 `Text2d` API available | Done — Bevy 0.18 in `Cargo.toml` | — |

---

*Created: 2026-04-02*
*Source: `docs/bugs/npc-display-names/ticket.md`*
