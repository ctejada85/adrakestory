---
name: basic-documents-requirements-write
description: Creates requirements documents for new features from meeting transcripts, kickoff calls, and other input sources. Extracts functional requirements, non-functional requirements, phase scoping, assumptions, dependencies, and open questions. Use when the user needs to document feature requirements from a meeting, conversation, or specification.
---

# Creating Requirements Documents

## When to use this skill

- User has a meeting transcript or notes and wants to extract structured requirements
- User wants to create a requirements document for a new feature
- User needs to organize feature specifications into functional/non-functional requirements with phase scoping
- User asks to document what was decided in a kickoff call or planning session

## Inputs required

Before starting, gather or locate these inputs:

1. **Primary source** — meeting transcript, recording notes, email thread, or specification document
2. **Feature context** — which product/system the feature is for, and what team is involved
3. **Participant list** — who was in the discussion and their roles (to attribute requirements correctly)

Optional but helpful:

- Existing codebase or architecture docs (to understand integration points)
- Prior requirements documents (to maintain formatting consistency)
- Product vision canvas (if one exists for this feature)

If the primary source is missing, ask the user before proceeding.

## Workflow

```
Requirements Document:
- [ ] Step 1: Analyze the source material
- [ ] Step 2: Scaffold the document
- [ ] Step 3: Write the overview and data domains
- [ ] Step 4: Extract functional requirements
- [ ] Step 5: Extract non-functional requirements
- [ ] Step 6: Define phase scoping
- [ ] Step 7: Document assumptions, dependencies, and open questions
- [ ] Step 8: Review and cross-reference
```

### Step 1: Analyze the source material

Read through the entire source (transcript, notes, spec) and identify:

- **Feature description** — what is being built and why
- **Data domains** — distinct categories or types of data the feature handles
- **Explicit requirements** — things someone directly stated as needed ("we need X", "it must do Y")
- **Implicit requirements** — things implied by the discussion ("users will provide human-readable names" → system must resolve to IDs)
- **Constraints** — limitations explicitly stated ("no direct SQL", "API One only")
- **Open questions** — things left unresolved or explicitly deferred
- **Dependencies** — things that must happen before development can start
- **Phase signals** — anything marked as "MVP", "Phase 1", "later", "future", "Phase 2"

Attribute statements to specific participants when possible — this helps resolve ambiguity later.

### Step 2: Scaffold the document

**Location:** `requirements/[feature-name]/` from workspace root.

**Filename:** `[YYYYMMDD] - [feature-name]-requirements.md`

**Template:** Use [assets/requirements-template.md](assets/requirements-template.md) as the starting structure. Copy it and fill in each section.

### Step 3: Write the overview and data domains

- **Overview:** 1-2 paragraphs explaining what the feature does and how it differs from existing capabilities
- **Data Domains:** If the feature handles distinct data types or categories, document them in a table with descriptions and user signals (how does the user indicate which domain they mean?)

### Step 4: Extract functional requirements

Organize requirements into logical groups. Common groupings:

| Group | Covers |
|-------|--------|
| **Selection / Routing** | How the system identifies this feature should handle a request |
| **Matching / Classification** | How the system maps user intent to specific data or actions |
| **Data Retrieval / Invocation** | How the system fetches the data or performs the action |
| **Parameter Resolution** | How the system fills in required parameters the user didn't provide explicitly |
| **Disambiguation / Clarification** | How the system handles ambiguity or missing information |
| **Authentication / Authorization** | How access control works |
| **Response Formatting** | How results are presented to the user |

For each requirement:

- **Assign an ID** — use the format `FR-[section].[number]` (e.g., `FR-3.2.1`)
- **Write a clear statement** — what the system must do, in declarative form
- **Assign a priority** — Phase 1, Phase 2, or Future (based on source material signals)

Present requirements as tables within each group:

```markdown
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | [Statement] | Phase 1 |
```

### Step 5: Extract non-functional requirements

These cover quality attributes, constraints, and architectural boundaries:

- Performance limits (response time, throughput)
- Reliability expectations (error handling, graceful degradation)
- Security requirements (auth, data protection)
- Architectural constraints (what NOT to do — "don't duplicate X", "LLMs can't do math")
- Scalability considerations

Use the format `NFR-[number]`:

```markdown
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | [Statement] | Phase 1 |
```

### Step 6: Define phase scoping

Create three phase definitions:

1. **Phase 1 — MVP:** The minimum set of capabilities for the feature to be useful
2. **Phase 2 — Enhanced:** Capabilities that build on the MVP and are planned but not immediate
3. **Future Phases:** Capabilities that are aspirational or require significant additional design

List capabilities as bullet points under each phase. These should map directly to functional requirements.

### Step 7: Document assumptions, dependencies, and open questions

**Assumptions & Constraints:** Things assumed to be true that, if wrong, would change the requirements. Number them for easy reference.

**Dependencies & Blockers:** Things that must happen before development can proceed. Include:

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | [What's needed] | [BLOCKED / Pending / Not started / Unknown] | [Person] |

**Open Questions:** Things that need answers but don't have them yet:

| # | Question | Owner |
|---|----------|-------|
| 1 | [The question] | [Who should answer] |

**Key Contacts:** Table of people to reach out to for specific topics.

### Step 8: Review and cross-reference

Before finalizing:

1. Verify every requirement traces to something said in the source material
2. Ensure phase scoping is consistent between the requirements table and the phase section
3. Check that open questions don't duplicate resolved requirements
4. Confirm all participants from the source are listed where they contributed
5. If a product vision canvas exists, verify alignment between requirements and features listed there
6. Add a footer with creation date and source reference

## Key instructions

- **Attribute requirements to sources** — note who stated or requested each requirement when possible
- **Use declarative language** — "The system must..." or "The retriever shall..." not "We should..."
- **Distinguish explicit from inferred requirements** — if you inferred a requirement from context rather than someone stating it directly, note that
- **Don't over-scope Phase 1** — when in doubt about phasing, mark as Phase 2
- **Include sample/example data** — if the source includes example questions, queries, or scenarios, add them in a reference section at the end
- **Keep open questions actionable** — each should have an owner who can answer it
- **Number everything** — requirements, assumptions, dependencies, and open questions should all have IDs for easy cross-referencing
