---
name: basic-documents-userstory-write
description: Creates user story ticket files with a structured format including story, description, acceptance criteria, non-functional requirements, and tasks. Use when the user asks to write a user story, create a ticket, or document a feature or bug fix as a story for implementation.
---

# Writing User Story Files

## When to use

- User asks to "write a user story", "create a ticket", or "write a story for this"
- User wants to document a bug fix, feature, or change as an implementable ticket
- User needs a structured breakdown of work with acceptance criteria and tasks
- User asks to add a `ticket.md` to a feature or bug folder

> **Multiple stories?** If the work spans several independent implementable units, use the `basic-documents-epic-write` skill instead. An epic groups multiple child stories under a shared goal and adds Epic Acceptance Criteria, a Dependencies & Risks table, and a child story index.

## Workflow

```
- [ ] Step 1: Gather context
- [ ] Step 2: Write the story file
- [ ] Step 3: Validate
```

### Step 1: Gather context

Read the relevant source material before writing. Check for:

- Bug reports, investigation docs, or feature descriptions in the same folder
- Requirements and architecture documents if present
- Open questions that are unresolved — do not include them as AC; note them as blockers

### Step 2: Write the story file

Save to the location the user specifies, or default to a `ticket.md` file in the same folder as the related documents.

Use the template in [`assets/userstory-template.md`](assets/userstory-template.md).

**Story sentence rules:**
- Format: `As a <persona>, I want <capability> so that <benefit>.`
- Use a real end-user persona (player, developer, level designer) — not "the system"
- One sentence only

**Description rules:**
- 2–4 sentences: what is broken or missing, what the fix/feature does, what is explicitly out of scope
- Technical enough to orient an implementer, not a full spec

**Acceptance criteria rules:**
- Numbered list of plain-English sentences
- Each AC must be independently verifiable (testable by a reviewer)
- Cover: happy path, edge cases (first-frame, empty state), invariants that must not break
- Include an AC for unit tests when the implementation has testable logic
- Do not include performance targets that require profiling tooling to verify unless a simple proxy exists (e.g., FPS counter)

**Non-functional requirements rules:**
- Bulleted list of sentences
- Cover: performance, memory, API surface, platform scope, and privacy where relevant
- Each bullet must be a constraint, not a wish ("must not", "must remain", "must not affect")

**Tasks rules:**
- Numbered list of short, imperative titles (start with a verb)
- Ordered from lowest to highest dependency (structs before systems, implementation before tests)
- Include a manual verification task as the final step
- Each task should map to roughly one PR commit

### Step 3: Validate

- Every AC is verifiable without ambiguity
- Every task, when completed, contributes to satisfying at least one AC
- No open questions remain unaddressed (either resolved inline or listed as blockers)
- The story sentence names a real persona

## File locations

| File | Purpose |
|------|---------|
| [`assets/userstory-template.md`](assets/userstory-template.md) | Blank template to fill in |
