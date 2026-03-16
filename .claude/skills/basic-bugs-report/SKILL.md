---
name: basic-bugs-report
description: Writes structured bug reports documenting description, actual behavior, expected behavior, root cause analysis, steps to reproduce, and suggested fix. Use when the user asks to write, file, or document a bug, or after a root cause investigation identifies discrete issues.
---

# Writing Bug Reports

## When to use

- User asks to write, file, or document a bug
- A root cause investigation (`basic-bugs-investigate`) has identified discrete issues
- User reports an issue and wants it formally documented

## Workflow

```
Bug Report:
- [ ] Step 1: Gather required information
- [ ] Step 2: Determine severity
- [ ] Step 3: Choose file name
- [ ] Step 4: Write the report
- [ ] Step 5: Link to related reports
```

### Step 1: Gather required information

Before writing, have:
- Observable symptom (what the user/player sees)
- Expected behavior (what should happen instead)
- Root cause (specific file, function, line, and mechanism)
- Reliable reproduction steps
- Suggested fix (even if approximate)

### Step 2: Determine severity

| Priority | Severity | Criteria |
|----------|----------|----------|
| **p1** | Critical | Data loss, crash on startup, core feature completely broken |
| **p2** | High | Significant performance degradation, feature broken under common conditions |
| **p3** | Medium | Feature works but feels wrong, visible but non-blocking |
| **p4** | Low | Minor overhead, edge case, only measurable under profiling |

Severity reflects player/user impact, not code complexity.

### Step 3: Choose file name

Format: `[YYYY-MM-DD-HHmm]-[p1|p2|p3|p4]-[kebab-case-title].md`

The priority prefix (`p1`–`p4`) encodes severity so bug files sort by importance.  
The title should describe **the cause**, not the symptom:
- ✅ `2026-03-15-2141-p1-occlusion-material-gpu-reupload-every-frame`
- ❌ `2026-03-15-2141-game-runs-slowly`

Save to `docs/bugs/`.

### Step 4: Write the report

```markdown
# Bug: [Clear title — describe the cause, not the symptom]

**Date:** YYYY-MM-DD
**Priority:** p1 | p2 | p3 | p4
**Severity:** Critical | High | Medium | Low
**Status:** Open
**Component:** [subsystem]

---

## Description
One to three sentences. What is wrong, where it happens, why it matters.

---

## Actual Behavior
- Bullet list of what actually happens (specific system, action, observable effect)

---

## Expected Behavior
- Bullet list of what should happen instead

---

## Root Cause Analysis

**File:** `path/to/file.rs`
**Function:** `function_name()`
**Approximate line:** N

Explanation of the mechanism. Code snippet showing the problem.

---

## Steps to Reproduce

1. Build: `cargo build --release`
2. Run: `cargo run --release`
3. ...

---

## Suggested Fix

Concrete fix with code snippet. List options with tradeoffs if multiple exist.

---

## Related
- Links to investigation reports and related bugs
```

### Step 5: Link to related reports

- Add links in the bug report to the investigation that found it
- Add links in the investigation report's "Related Bugs" section to this file
- Link related bugs to each other when they share a root system

## Rules

- One bug per file — do not bundle multiple issues
- Root cause section is mandatory — always include file + line
- Suggested fix must be actionable code, not vague advice
- Always use third person in descriptions
