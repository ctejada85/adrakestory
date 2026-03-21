# Requirements — VSync Configuration

**Source:** Product specification — 2026-03-21
**Status:** Draft

---

## 1. Overview

The VSync Configuration feature adds vertical synchronization support to the game's display settings. Currently, the engine runs with `PresentMode::AutoNoVsync`, rendering frames as fast as possible without tearing prevention or frame rate capping.

VSync synchronizes frame presentation to the monitor's refresh rate, eliminating screen tearing. The feature introduces an additional **multiplier** setting that caps the target frame rate to a user-defined multiple of the display's refresh rate (e.g., 1× = 60 fps on a 60 Hz monitor, 0.5× = 30 fps). This gives players precise control over performance and power consumption without needing an external frame limiter.

---

## 2. Data Domains

### 2.1 VSync Mode vs. Frame Rate Cap

| Domain | Description | Typical User Signal |
|--------|-------------|---------------------|
| **VSync Toggle** | Whether frames are synchronized to the monitor's refresh rate (eliminates tearing) | User enables/disables VSync in settings |
| **Multiplier** | The fractional or whole multiple of the refresh rate used as the target frame rate cap | User selects 1×, 0.5×, 2×, etc. |

**Requirement:** Both settings are stored together and applied atomically; changing either one must update the window presentation mode in the same frame.

---

## 3. Functional Requirements

### 3.1 VSync Toggle

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | The game MUST expose a boolean `vsync_enabled` setting that enables or disables vertical synchronization. | Phase 1 |
| FR-3.1.2 | When `vsync_enabled = false`, the engine MUST use `PresentMode::AutoNoVsync` (current behavior). | Phase 1 |
| FR-3.1.3 | When `vsync_enabled = true`, the engine MUST use `PresentMode::Fifo` (GPU-synchronized presentation). | Phase 1 |

### 3.2 Refresh Rate Multiplier

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | The game MUST expose a `vsync_multiplier: f32` setting representing the target frame rate as a multiple of the monitor's refresh rate. | Phase 1 |
| FR-3.2.2 | The default value of `vsync_multiplier` MUST be `1.0` (target = 1 × refresh rate). | Phase 1 |
| FR-3.2.3 | When `vsync_enabled = true` and `vsync_multiplier = 1.0`, the engine MUST run at the monitor's native refresh rate with no additional frame cap. | Phase 1 |
| FR-3.2.4 | When `vsync_multiplier < 1.0` (e.g., 0.5), the engine MUST apply a software frame limiter that caps the frame rate to `multiplier × refresh_rate` fps. | Phase 1 |
| FR-3.2.5 | When `vsync_multiplier > 1.0` (e.g., 2.0) and `vsync_enabled = true`, the engine MUST disable the software frame limiter; the GPU VSync already caps at the refresh rate. | Phase 2 |
| FR-3.2.6 | The multiplier MUST be clamped to the range `[0.25, 4.0]`. Values outside this range MUST be rejected with a logged warning and snapped to the nearest bound. | Phase 1 |

### 3.3 Settings Persistence

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | Both `vsync_enabled` and `vsync_multiplier` MUST be persisted in `settings.ron` alongside existing `OcclusionConfig` fields. | Phase 1 |
| FR-3.3.2 | Settings MUST be loaded at startup and applied before the first frame is rendered. | Phase 1 |
| FR-3.3.3 | Settings MUST be saved when the application exits (consistent with existing `save_settings` behavior). | Phase 1 |

### 3.4 Runtime Settings UI

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | The in-game settings screen (`SettingsPlugin`) MUST expose a VSync toggle control. | Phase 1 |
| FR-3.4.2 | The in-game settings screen MUST expose a multiplier selector (e.g., dropdown or slider) visible only when VSync is enabled. | Phase 1 |
| FR-3.4.3 | Changes made in the settings UI MUST take effect immediately without requiring a restart. | Phase 1 |

### 3.5 Monitor Refresh Rate Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | The system MUST read the primary monitor's refresh rate at startup via Bevy's `MonitorSelection` / `VideoModeSelection` APIs. | Phase 1 |
| FR-3.5.2 | If the refresh rate cannot be detected, the system MUST fall back to 60 Hz and log a warning. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | VSync enable/disable MUST apply within 1 frame; no restart or map reload required. | Phase 1 |
| NFR-4.2 | The software frame limiter MUST introduce no more than 1 ms of additional frame-time jitter compared to VSync alone. | Phase 1 |
| NFR-4.3 | The settings change MUST NOT trigger a full render pipeline recreation (no `PipelineCache` invalidation). | Phase 1 |
| NFR-4.4 | Serialization of `VsyncConfig` MUST be backward-compatible: `settings.ron` files without VSync fields MUST load using defaults (`vsync_enabled = false`, `vsync_multiplier = 1.0`). | Phase 1 |
| NFR-4.5 | The feature MUST work on macOS, Windows, and Linux. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- VSync toggle (`vsync_enabled`)
- Multiplier setting with sub-1× software frame cap
- Monitor refresh rate detection (with 60 Hz fallback)
- Persistence in `settings.ron` with backward-compatible deserialization
- In-game settings UI controls

### Phase 2 — Enhanced

- Multiplier > 1× support (useful for high-refresh monitors without tearing)
- Per-monitor refresh rate detection (multi-monitor setups)
- Preset labels in UI (e.g., "Half Rate", "Native", "Double")

### Future Phases

- Adaptive VSync (`PresentMode::FifoRelaxed`) as an alternative mode
- Dynamic multiplier adjustment based on GPU frame time

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Bevy 0.18 is the target engine version; `PresentMode` and `Window` mutation APIs are stable. |
| 2 | The software frame limiter is implemented using `bevy_framepace` or Bevy's built-in `FramepacePlugin` (if available) rather than a custom busy-wait loop. |
| 3 | VSync state is stored inside `OcclusionConfig` or alongside it in a new `DisplayConfig` resource — both approaches are acceptable; architecture doc resolves this. |
| 4 | The multiplier is treated as a floating-point divisor: `target_fps = refresh_rate × multiplier`. |
| 5 | Multiplier values are discrete in the UI (0.25, 0.5, 1.0, 2.0) but stored as `f32` for flexibility. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | Should VSync config live inside `OcclusionConfig` or a new `DisplayConfig` struct? | Architecture |
| 2 | Should `vsync_enabled` default to `true` or `false`? (Current behavior is `false`.) | Product |
| 3 | Is `bevy_framepace` already a dependency, or does it need to be added? | Engineering |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Bevy 0.18 `Window` resource mutation (mutable access to `PresentMode` at runtime) | Resolved — available in 0.18 | — |
| 2 | Frame rate limiter crate (`bevy_framepace` or equivalent) | Pending evaluation | Engineering |

---

## 9. Reference: Sample Scenarios

| Type | Example | Complexity |
|------|---------|------------|
| Default | VSync off, unlimited FPS (current behavior) | Low |
| Standard VSync | Enable VSync, multiplier = 1.0 → 60 fps on 60 Hz monitor | Low |
| Half-rate | Enable VSync, multiplier = 0.5 → 30 fps on 60 Hz (battery-saving mode) | Medium |
| Quarter-rate | Enable VSync, multiplier = 0.25 → 15 fps (low-power / CI test mode) | Medium |
| High-refresh | Enable VSync, multiplier = 1.0 → 144 fps on 144 Hz monitor | Low |

---

*Created: 2026-03-21*
*Source: Product specification*
