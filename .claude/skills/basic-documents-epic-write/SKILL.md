---
name: basic-documents-epic-write
description: Creates epic ticket files containing multiple child user stories. Each story follows the user story template (story, description, acceptance criteria, NFRs, tasks). Use when the user asks to write an epic, create a multi-story ticket, or document a large feature that spans several implementable units of work.
---

# Writing Epic Files

## When to use

- User asks to "write an epic", "create an epic ticket", or "document this as an epic"
- A feature or migration is too large for a single user story and needs to be broken into child stories
- User wants a single document that groups related user stories under a shared goal
- User needs a ticket that spans multiple components, phases, or team members

## Relationship to user stories

An epic is a collection of user stories under a shared strategic goal. Each child story follows the same format as a standalone user story (see `basic-documents-userstory-write`). The epic adds:

- An **Epic Story** — a single high-level story sentence that describes the overall goal
- A **Child Stories table** — index of all stories with status tracking
- **Epic Acceptance Criteria** — outcomes verified only when all stories are complete
- **Epic Non-Functional Requirements** — constraints that apply across the whole epic
- **Dependencies & Risks** — blockers and risks that span multiple stories

## Workflow

```
- [ ] Step 1: Gather context
- [ ] Step 2: Define the epic boundary
- [ ] Step 3: Break into child stories
- [ ] Step 4: Write the epic file
- [ ] Step 5: Validate
```

### Step 1: Gather context

Read the relevant source material. Check for:

- Requirements and architecture documents in the same folder
- Existing bug reports, investigation docs, or feature descriptions
- Open questions — do not include them as AC; surface them in Dependencies & Risks

### Step 2: Define the epic boundary

Decide what the epic delivers end-to-end:

- What is the single user-facing (or developer-facing) outcome when the epic is done?
- What is explicitly out of scope?
- Is there a natural Phase 1 / Phase 2 split, or is the epic itself one phase?

### Step 3: Break into child stories

Decompose the epic into stories that are:

- **Independent** — each story can be started without waiting for another story (or dependencies are explicit)
- **Testable** — each story has its own acceptance criteria
- **Sized for a single sprint** — if a story itself feels too large, break it further

Common decomposition patterns:

| Pattern | Example split |
|---------|--------------|
| **By layer** | Dependency update → Rust migration → Shader migration |
| **By component** | Auth service → API routes → UI |
| **By phase** | MVP → Enhanced → Edge cases |
| **By risk** | Highest-risk story first, lower-risk stories after |

### Step 4: Write the epic file

Save to the location the user specifies, defaulting to `ticket.md` in the feature or bug folder.

Use the template in [`assets/epic-template.md`](assets/epic-template.md).

**Epic Story rules:**
- Format: `As a <persona>, I want <high-level capability> so that <strategic benefit>.`
- Should describe the end-to-end outcome, not a single implementation step
- One sentence only

**Child story rules (same as `basic-documents-userstory-write`):**
- Each story has its own Story sentence, Description, Acceptance Criteria, NFRs, and Tasks
- Story sentences must name a real persona
- AC items must be independently verifiable
- Tasks ordered from lowest to highest dependency

**Epic Acceptance Criteria rules:**
- Should be things only verifiable after ALL child stories are done
- At minimum: "all child stories complete and verified", integration test, and no regressions
- Do not duplicate individual story AC here

**Dependencies & Risks table:**
- Dependencies: external things that must be true before the epic starts
- Risks: things that could cause delay or failure; include a mitigation note

### Step 5: Validate

- Every child story maps to at least one epic AC
- Every epic AC is verifiable without ambiguity
- No story duplicates work from another story
- The epic story sentence describes the sum of all child stories
- Dependencies & Risks are actionable (owner or mitigation noted)

## File locations

| File | Purpose |
|------|---------|
| [`assets/epic-template.md`](assets/epic-template.md) | Blank template to fill in |
| `basic-documents-userstory-write` skill | Reference for individual story formatting rules |
