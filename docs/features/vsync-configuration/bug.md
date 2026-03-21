# Bug: VSync frame cap runs ~17% low due to non-self-correcting sleep timing

**Date:** 2026-03-21
**Priority:** p2
**Severity:** High
**Status:** Open
**Component:** Settings / VSync frame limiter

---

## Description

When VSync is enabled at 1× multiplier on a 120 Hz display, the actual frame rate caps at ~100 fps instead of the expected 120 fps. The `apply_vsync_system` records `last_frame_end` **after** the sleep, so `elapsed` on each subsequent frame only measures game-logic time (not the full frame duration including the previous sleep). Combined with macOS's `std::thread::sleep` overshooting short durations by ~1–2 ms, the error accumulates with no self-correction.

---

## Actual Behavior

- VSync enabled, multiplier = 1×, 120 Hz ProMotion display
- Target frame time = 8.333 ms
- `elapsed` measures only game-logic time (~3 ms), so the sleep targets ~5.333 ms
- macOS scheduler rounds the short sleep up to ~6.5–7.5 ms
- Actual frame time ≈ 3 ms + 7 ms = 10 ms → ~100 fps
- Frame rate oscillates slightly above 100 fps due to variable sleep overshoot
- No frame compensates for the previous frame's overshoot

---

## Expected Behavior

- VSync enabled, multiplier = 1×, 120 Hz display → ~120 fps
- Short sleep overshoots in one frame are compensated by a shorter sleep in the next frame
- Frame rate oscillates tightly around 120 fps, not ~17% below it

---

## Root Cause Analysis

**File:** `src/systems/settings/vsync.rs`
**Function:** `apply_vsync_system()`
**Lines:** 151–157

```rust
// elapsed only covers game-logic time, not the full frame
if let (Some(target), Some(last)) = (limiter.target_frame_time, limiter.last_frame_end) {
    let elapsed = last.elapsed();
    if elapsed < target {
        std::thread::sleep(target - elapsed);  // overshoots on macOS
    }
}
limiter.last_frame_end = Some(Instant::now());  // ← captured AFTER sleep
```

Because `last_frame_end` is set after the sleep, `elapsed` on the next call equals only the game-logic portion of the frame (~3 ms). The sleep always targets the same short remainder (~5.333 ms at 120 fps), macOS consistently overshoots that short sleep by ~1–2 ms, and the resulting ~10 ms frame time is never fed back into the calculation.

**Contributing factor:** `std::thread::sleep` on macOS is imprecise for sub-10 ms durations. The scheduler can delay wakeup by ~1–2 ms, which is a ~17–24% error for a 8.333 ms target.

---

## Steps to Reproduce

1. `cargo run --release`
2. Open Settings → enable VSync, set multiplier to 1×
3. Enable the FPS overlay (`F3`)
4. Observe: FPS reads ~100 instead of the monitor's refresh rate (120 Hz)
5. Toggle VSync off → FPS becomes uncapped, confirming the limiter is active

---

## Suggested Fix

Two changes required:

**1. Capture `last_frame_end` before the sleep (self-correcting timestamps)**

Record the ideal frame start time before sleeping. This causes `elapsed` on the next frame to include the full previous frame duration (work + sleep + overshoot), so the next sleep is automatically shortened to compensate.

```rust
pub fn apply_vsync_system(...) {
    let now = Instant::now();

    if let (Some(target), Some(last)) = (limiter.target_frame_time, limiter.last_frame_end) {
        let elapsed = now.duration_since(last);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }

    // Record BEFORE sleep so next frame's elapsed covers the full frame duration.
    limiter.last_frame_end = Some(now);

    // ... rest of dirty-flag logic unchanged
}
```

**2. Spin-sleep for the final ~2 ms (precision)**

For the last ~2 ms of the remaining wait, busy-spin instead of calling the OS. This avoids scheduler wakeup latency entirely for short remainders.

```rust
fn precise_sleep(duration: Duration) {
    const SPIN_THRESHOLD: Duration = Duration::from_millis(2);
    if duration > SPIN_THRESHOLD {
        std::thread::sleep(duration - SPIN_THRESHOLD);
    }
    // Busy-wait for the final portion
    let deadline = Instant::now() + duration.min(SPIN_THRESHOLD);
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}
```

Alternatively, add the `spin_sleep` crate (`spin_sleep = "1"`) which implements this pattern with adaptive calibration.

Fix 1 alone reduces the error significantly (from consistent ~100 fps to oscillating around 120 fps). Fix 2 tightens the oscillation window further.

---

## Related

- Feature ticket: `docs/features/vsync-configuration/ticket.md`
- Feature architecture: `docs/features/vsync-configuration/references/architecture.md`
