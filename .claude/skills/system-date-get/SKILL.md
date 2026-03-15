---
name: system-date-get
description: Gets the current system date and time. Use when you need to know today's date — for changelog entries, file naming, meeting notes, or any time-sensitive task.
---

# Get System Date

## Command

```bash
date
```

## When to use

- Before writing dates in documents (changelogs, meeting notes, file names)
- When the user asks "what's today's date?"
- When naming files with the `YYYYMMDD` convention
- Any time you need the current date and are unsure

## Notes

- Always use this command rather than guessing or relying on stale context
- The system timezone may differ from the user's local timezone — if precision matters, confirm with the user
