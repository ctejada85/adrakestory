# NPC Display Names

**Date:** 2026-04-01
**Component:** NPC / UI

---

## Story

As a player, I want to see an NPC's name when I approach or interact with them so that I can identify characters in the world.

---

## Description

`Npc` (`src/systems/game/components.rs`) has a `name: String` field that is populated at spawn time from map data, but no system currently reads or displays it. Players have no way to identify individual NPCs in the world. This ticket covers rendering the NPC name as a world-space label (billboard text) above the entity, and optionally also in a dialogue/interaction HUD element. Out of scope: voice acting, full dialogue trees, name input from the level editor UI.

---

## Acceptance Criteria

1. When a player is within interaction range of an NPC, the NPC's name is visible above their position as a world-space text label.
2. The label scales appropriately with camera distance so it remains legible but not intrusive.
3. If `Npc::name` is the empty string or the default `"NPC"`, a sensible fallback (or no label) is shown.
4. The label disappears when the player moves out of range.
5. Existing `Npc::radius` collision behaviour is unaffected.
6. A unit or integration test verifies that the label entity is spawned when an `Npc` component is present.

---

## Non-Functional Requirements

- Must not cause a noticeable FPS drop when 10 or more NPCs are on screen simultaneously.
- Label rendering must use Bevy's existing text/UI pipeline — no third-party crates.
- Must work on both game and editor binaries (or be explicitly excluded from the editor binary).

---

## Tasks

1. Decide on world-space label approach (Bevy `Text2d` billboard or egui overlay).
2. Add a `spawn_npc_label` system that creates a child text entity for each `Npc` at spawn time.
3. Add a `update_npc_label_visibility` system that shows/hides the label based on player proximity.
4. Wire both systems into the game plugin under `GameSystemSet::Visual`.
5. Write a unit test that spawns an `Npc` entity and asserts the label child entity exists.
6. Manually verify label appears and disappears at correct distances in the game.
