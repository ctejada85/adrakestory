---
name: basic-bugs-investigate
description: Investigates bugs, performance issues, and unexpected behavior to identify root causes. Use when the user asks to investigate why something is broken, slow, or behaving unexpectedly, or asks for a root cause analysis (RCA).
---

# Investigating Bugs and Root Causes

## When to use

- User reports something is broken, slow, or behaving unexpectedly
- User asks "why does X happen" or "what causes Y"
- User asks for a root cause analysis, RCA, or investigation
- One component/binary/path works and another doesn't — need to find the difference

## Workflow

```
RCA:
- [ ] Step 1: Establish the comparison baseline
- [ ] Step 2: Trace the hot path / data flow
- [ ] Step 3: Verify suspects with targeted searches
- [ ] Step 4: Rank findings by severity
- [ ] Step 5: Write investigation report
- [ ] Step 6: File individual bug reports
```

### Step 1: Establish the comparison baseline

If one thing works and another doesn't, always diff them first:
- Compare entry points, plugin/module registrations, config defaults
- List what the broken path has that the working path doesn't
- Check for systems, plugins, or resources present in one but absent in the other

### Step 2: Trace the hot path

For **performance issues:**
- Identify all systems/functions that run every frame or tick (no throttling)
- Check for expensive operations done unconditionally: asset mutations, full-collection iteration, BFS/DFS, GPU uploads
- Verify delta-time usage (frame-rate independence)
- Look for periodic spikes — these point to throttled work with too-short intervals

For **correctness issues:**
- Trace data flow from trigger to symptom
- Identify where state is written vs. read
- Look for stale data, missing change detection, incorrect ordering

### Step 3: Verify suspects with targeted searches

- `grep` for all usages of the suspect function/resource/component
- Check recent git history on the affected files
- Look at default values for config structs — they often expose the intent vs. actual behavior gap

### Step 4: Rank findings by severity

| Priority | Severity | Criteria |
|----------|----------|----------|
| **p1** | Critical | Reproduces every run, core path affected |
| **p2** | High | Frequent or under common conditions |
| **p3** | Medium | Visible to users, non-blocking |
| **p4** | Low | Only measurable under profiling |

### Step 5: Write investigation report

Save to `docs/investigations/[YYYY-MM-DD-HHmm]-[short-description].md`.

```markdown
# Investigation: [Title]
**Date:** YYYY-MM-DD HH:MM
**Status:** Complete
**Component:** [system]

## Summary
## Environment
## Investigation Method
## Findings
### Finding N — [Title] (p1 Critical | p2 High | p3 Medium | p4 Low)
File, function, line. What the code does. Why it's wrong. Code snippet.
## Root Cause Summary
Table: # | Root Cause | Location | Priority | Severity | Notes
## Recommended Fixes
## Related Bugs
```

### Step 6: File individual bug reports

For each distinct root cause found, create a bug report using the `basic-bugs-report` skill.

## Key heuristics (Bevy / Rust)

- `Assets::get_mut()` always marks the asset dirty — even writing the same value forces GPU re-upload
- `iter_mut()` alone does NOT mark components dirty; only writing through `DerefMut` does
- Frame-rate-dependent lerp: `lerp(a, b, speed * delta)` is NOT frame-rate-independent — use `1.0 - exp(-speed * delta)`
- Periodic spikes ≠ constant lag — check throttle intervals on recurring systems
