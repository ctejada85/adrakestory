---
name: composed-documents-architecture-write
description: Creates architecture change documents for new feature implementations. Covers current architecture analysis, target architecture design, component diagrams, sequence diagrams, and appendices. Use when the user needs to document how a new feature integrates into an existing codebase, or wants to create a before-and-after architecture reference.
---

# Creating Architecture Change Documents

## When to use this skill

- User wants to document how a new feature fits into the existing codebase
- User needs an architecture reference covering current state and target state
- User asks to create a technical design document for a feature implementation
- User wants to plan integration points, new components, and modified components for a feature

## Inputs required

Before starting, gather or locate these inputs:

1. **Requirements document** — the feature's functional and non-functional requirements (see `basic-documents-requirements-write` skill)
2. **Product vision canvas** — the feature's purpose, audience, and success metrics (see `composed-documents-productvisioncanvas-write` skill)
3. **Codebase access** — the repository where the feature will be implemented
4. **Domain knowledge sources** — meeting transcripts, data exports, API docs, schema dumps, or other reference materials that inform the design

If any of these are missing, ask the user before proceeding.

## Workflow

```
Architecture Document:
- [ ] Step 1: Analyze the existing codebase
- [ ] Step 2: Scaffold the document
- [ ] Step 3: Write Section 1 — Current Architecture
- [ ] Step 4: Write Section 2 — Target Architecture
- [ ] Step 5: Write Appendices
- [ ] Step 6: Review and cross-reference
```

### Step 1: Analyze the existing codebase

Before writing anything, explore the codebase to understand:

- **Solution/project structure** — how the code is organized (projects, modules, packages)
- **Pipeline/workflow** — the processing flow that the feature plugs into
- **Extension points** — interfaces, base classes, DI patterns, plugin mechanisms
- **Existing patterns** — how similar features are implemented (retrievers, services, handlers)
- **Data flow** — how data moves through the system (request → processing → response)

Read key files. Trace the execution path. Understand the abstractions before proposing changes.

### Step 2: Scaffold the document

**Location:** Place the document alongside the requirements and product vision canvas in the feature's `requirements/` directory.

**Filename:** `current-architecture.md` (it covers both current and target state).

**Template:** Use [assets/architecture-template.md](assets/architecture-template.md) as the starting structure. Copy it and fill in each section.

### Step 3: Write Section 1 — Current Architecture

Document the existing system as it is today. This section should be accurate enough that a new engineer could understand the system without reading the code.

**Subsections to include:**

| Subsection | Content |
|------------|---------|
| **Solution Structure** | Mermaid diagram of project dependencies; brief description of each project's purpose |
| **Pipeline/Workflow Overview** | Linear flowchart of the main processing pipeline with numbered steps |
| **Pipeline Steps — Detail** | Table with step number, name, class, and purpose for each step |
| **Extension Architecture** | Class diagram showing the interface → abstract → concrete hierarchy for the pluggable component type (retrievers, handlers, processors, etc.) |
| **Query Planning / Routing** | How the system decides which component handles a request (e.g., vector similarity, rule-based, LLM-based). Describe the *current* role clearly — if the mechanism has evolved (e.g., routing moved from component X to LLM-based selection), document the current state, not the historical one |
| **Data Flow** | End-to-end Mermaid flowchart from HTTP request to response, showing external system interactions |

**Guidelines:**

- Use Mermaid diagrams extensively — they render in markdown and convey structure better than prose
- Every class/component mentioned should include its actual class name from the codebase
- Note existing flags, enums, and configuration patterns — the target architecture must extend these
- Highlight empty interfaces, placeholder patterns, or TODOs that signal anticipated extensibility
- **Describe each component's role once, in the right section.** If a concept (e.g., a routing mechanism, a matching process) is explained in §1.5, don't repeat the full explanation in §2.3, §2.4, and Appendix A. Instead, reference the canonical section. Redundant explanations drift apart as the document evolves and create maintenance burden

### Step 4: Write Section 2 — Target Architecture

Design how the feature integrates into the existing system. This is the core of the document.

**Subsections to include:**

| Subsection | Content |
|------------|---------|
| **Design Principles** | Numbered list of architectural decisions (e.g., "Fit into existing pipeline", "No breaking changes", "Reuse pattern X"). Keep each principle concise — 1-2 sentences. Avoid embedding implementation details that belong in other sections |
| **New Components** | Mermaid diagram + table listing each new class/service, its project, and its purpose |
| **Modified Components** | Table of existing components that need changes, with description of each change |
| **Data Population** | If the feature requires new data (DB records, graph nodes, config), describe what must be created and how. Keep each sub-item focused — avoid repeating routing/planning details that are already covered in §1.5 or §2.4 |
| **Pipeline Flow** | Updated flowchart showing where the new feature fits in the existing pipeline, marking existing components (✅) and new ones (🆕) |
| **Internal Flow** | Detailed flowchart of the new feature's internal logic (what happens inside the new component) |
| **Class Diagram** | Full class diagram with properties and methods for all new types |
| **Sequence Diagram — Happy Path** | Step-by-step sequence diagram for the primary success scenario |
| **Clarifying/Error Flow** | Sequence diagram for edge cases (missing data, errors, disambiguation) |
| **Configuration & Registration** | YAML/JSON config examples and DI registration patterns |
| **Phase Boundaries** | Table clarifying what is in scope (MVP) vs. future phases |

**Guidelines:**

- Design principles should trace back to requirements and the product vision canvas
- New components should follow existing patterns in the codebase (naming, DI, config)
- Modified components should describe the minimum necessary change — avoid unnecessary refactoring
- Include a "Design evolution" note if the architecture went through iterations (documents decision history)
- Mark phase boundaries clearly — what is MVP vs. Phase 2 vs. Future

### Step 5: Write Appendices

Include appendices for reference material that supports the design but would clutter the main sections:

| Appendix | Content |
|----------|---------|
| **Data Schema** | Actual schema of data the feature depends on (DB tables, graph nodes, API responses). Keep concise — reference §1.5 or §2.3 for detailed mechanism explanations rather than repeating them here |
| **Open Questions & Decisions** | Separated into **Resolved** and **Open** tables (see format below) |
| **Key File Locations** | Table mapping component names to file paths in the codebase |
| **Query/Code Templates** | Proposed SQL, Cypher, or code snippets the new components will use |

**Open Questions & Decisions format:**

The appendix should have two clearly separated tables with consistent numbering across both:

```markdown
### Resolved

| # | Question | Resolution |
|---|----------|------------|
| 1 | [Question] | [How it was resolved] |

### Open

| # | Question | Impact | Notes |
|---|----------|--------|-------|
| 4 | [Question] | [What it blocks] | [Actionable context] |
```

Rules for managing questions:
- **Number questions sequentially across both tables** — when a question moves from Open to Resolved, keep its original number so references elsewhere in the document stay valid
- **Move decided items promptly** — if a question in the Open table has been decided (e.g., "Decided: X"), move it to Resolved on the next edit pass
- **Add a Notes column to Open questions** — include who to ask, what blocks resolution, or proposed approaches. This makes the list actionable
- **Don't nest "Still Open" under "Resolved"** — use two peer-level sections at the same heading depth

### Step 6: Review and cross-reference

Before finalizing:

1. Verify every class name, file path, and interface mentioned exists in the codebase
2. Confirm design principles align with the requirements document
3. Ensure phase boundaries match the requirements' phase scoping
4. Check that open questions from the requirements are addressed (resolved or explicitly still open)
5. Add a footer with creation date and companion document links (version is tracked in the changelog)
6. Add a companion link in the requirements and product vision canvas documents
7. **Scan for redundancy** — search for key terms (component names, mechanism names) and verify each concept is explained in one canonical location with references elsewhere. If the same mechanism is described in 3+ sections, consolidate to one and reference from the others

## Document structure conventions

### Changelog table

Every architecture document must include a **changelog table** immediately after the header metadata and before the Table of Contents. This is the single source of truth for version history.

```markdown
## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| v1 | YYYY-MM-DD | [Author] | Initial draft — [brief description of approach] |
| v2 | YYYY-MM-DD | [Author] | [What changed and why] |
```

Rules:
- **Add a new row for every meaningful update** — not typo fixes, but structural changes, feedback incorporation, codebase validation findings, etc.
- **Summarize what changed and why**, not just "updated document"
- **Bold the latest version row** for quick scanning
- The footer should reference the changelog (`See [Changelog](#changelog) for version history`) instead of containing inline version history
- When the document is first created, start with `v1`. Increment on each substantive revision

### Reducing redundancy across versions

As a document evolves through multiple versions, information tends to accumulate and repeat. Each revision should actively check for this:

- **Describe each mechanism once.** If GENIE query planning is explained in §1.5, §2.3.1 should reference §1.5, not re-explain the Cypher query, node types, and graph patterns
- **Callout blocks are for exceptions, not re-summaries.** Use `> **Note:**` blocks for genuinely new information (e.g., "this mechanism has changed since v1"). Don't use them to re-summarize what the section already says
- **Tables should not contain prose paragraphs.** If a table cell (e.g., pipeline step description) grows beyond 2-3 sentences, the detail probably belongs in a dedicated subsection with the table referencing it
- **When updating, delete outdated framing.** If a component's role has changed (e.g., from "routing" to "template matching"), update all references — don't add qualifiers like "originally X, now Y" in every section. State the current truth once, note the evolution in the changelog

### Table of Contents

Include a Table of Contents after the changelog. Keep it updated when sections are added, removed, or renamed. Use markdown anchor links.

## Key instructions

- **Mermaid diagrams are mandatory** — use `flowchart`, `sequenceDiagram`, `classDiagram`, and `graph` types as appropriate
- **Use actual class names from the codebase** — never invent names that don't follow the project's naming conventions
- **Document what exists before proposing changes** — the Current Architecture section should be purely descriptive
- **Trace every design decision to a requirement** — if a design choice can't be justified by a requirement, question whether it belongs
- **Keep the document self-contained** — a reader should be able to understand both the current and target architecture without reading the codebase
- **Include a Table of Contents** — the document will be long; navigation is essential
- **Validate claims against the codebase** — when documenting how a mechanism works (routing, selection, matching), read the actual code to confirm. Don't rely on assumptions or outdated documentation. Note the source files in the changelog entry when codebase investigation informs a revision
- **State the current truth, not the history** — the main document body should describe how things work *now*. Historical context belongs in the changelog or a brief "Design evolution" note, not scattered as caveats throughout every section
