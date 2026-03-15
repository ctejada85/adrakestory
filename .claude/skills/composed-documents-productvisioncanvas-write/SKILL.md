---
name: composed-documents-productvisioncanvas-write
description: Creates product vision canvas documents for new features. Synthesizes problem statement, target audience, needs, features, value proposition, success metrics, and experience principles into a structured one-page canvas. Use when the user wants to articulate the product vision for a feature, or needs a canvas-format summary from requirements and meeting notes.
---

# Creating Product Vision Canvas Documents

## When to use this skill

- User wants to create a product vision canvas for a new feature
- User needs to synthesize requirements into a strategic product view
- User asks for a one-pager or vision summary for a feature
- User wants to articulate the "why" and "for whom" behind a feature before diving into technical design

## Inputs required

Before starting, gather or locate these inputs:

1. **Requirements document** — the feature's functional and non-functional requirements (see `basic-documents-requirements-write` skill)
2. **Source material** — meeting transcripts, kickoff call notes, or product briefs that explain the feature's purpose and target users

Optional but helpful:

- Existing product vision canvases (to maintain style consistency)
- User research or personas
- Competitive analysis or market context

If the requirements document is missing, create it first using the `basic-documents-requirements-write` skill.

## Workflow

```
Product Vision Canvas:
- [ ] Step 1: Synthesize the source material
- [ ] Step 2: Scaffold the document
- [ ] Step 3: Write the problem statement
- [ ] Step 4: Define target audience and needs
- [ ] Step 5: List features by phase
- [ ] Step 6: Articulate unique value proposition
- [ ] Step 7: Define success metrics
- [ ] Step 8: Write experience principles and vision statement
- [ ] Step 9: Build the summary canvas table
- [ ] Step 10: Review and cross-reference
```

### Step 1: Synthesize the source material

Read the requirements document and source material to extract:

- **The core problem** — what pain point or gap does this feature address?
- **Who feels this pain** — which user types are affected and how?
- **What they need** — expressed in the user's own words (first person)
- **What makes this different** — why is this approach better than alternatives?
- **How we'll know it works** — measurable outcomes

Focus on the user perspective, not the technical implementation.

### Step 2: Scaffold the document

**Location:** Same directory as the requirements document.

**Filename:** `[YYYYMMDD] - [feature-name]-product-vision-canvas.md`

**Template:** Use [assets/product-vision-canvas-template.md](assets/product-vision-canvas-template.md) as the starting structure.

### Step 3: Write the problem statement

One paragraph that captures:

1. **Who** is affected (role/persona)
2. **What** they currently have to do (the pain)
3. **Why** that's a problem (the consequence)
4. **Because** of what gap (what's missing)

**Pattern:** "[Role] who need [outcome] currently must [painful workaround] — because [the system] only [current limitation]."

Keep it to 2-3 sentences. Avoid technical jargon — this should be understandable by a product manager or executive.

### Step 4: Define target audience and needs

**Target Audience:** Define 2-3 audience segments:

- **Primary:** The main user group — who uses this most and gets the most value
- **Secondary:** Occasional or indirect users who still benefit

For each, include: role description, what they know, what they don't know, and how they interact with the system.

**Audience Needs:** Write 4-6 needs in first person, as direct quotes from the user's perspective:

- "I want to [action] and get [result] — I shouldn't need to [painful alternative]."
- "When I say [natural input], I mean [natural input]. Don't ask me for [technical artifact]."
- "If I'm not specific enough, [help me] instead of [failing]."

These should feel authentic — drawn from the source material, not generic.

### Step 5: List features by phase

Organize features into three tiers, matching the requirements document's phase scoping:

**Phase 1 — MVP:** Table with Feature and Description columns. These are the minimum capabilities for the feature to deliver value.

**Phase 2 — Enhanced:** Features that extend the MVP. Less detail needed.

**Future:** Aspirational features. One-line descriptions are sufficient.

Each feature should map to one or more functional requirements from the requirements document.

### Step 6: Articulate unique value proposition

Write 2-3 numbered value propositions. Each should:

1. **State the benefit** in bold
2. **Explain how it's delivered** in 1-2 sentences
3. **Contrast with the alternative** (what happens without this feature)

Focus on what makes this feature's approach uniquely valuable — not just "it does X" but "it does X in a way that [differentiation]."

### Step 7: Define success metrics

Create a metrics table:

| Metric | Description | Target |
|--------|-------------|--------|
| [Name] | [What it measures] | [Specific target] |

Include 5-7 metrics covering:

- **Accuracy** — does the system route/match/select correctly?
- **Completion** — do requests succeed end-to-end?
- **Performance** — is it fast enough?
- **User satisfaction** — do users like the results?
- **Adoption** — are users choosing this over alternatives?

Targets should be specific and measurable ("> 90%", "< 15s", "positive trend").

### Step 8: Write experience principles and vision statement

**Experience Principles:** 3 principles that guide design decisions. Each should:

- Have a **memorable short name** in bold (e.g., "Trust the data, not the model")
- Include a 1-2 sentence explanation
- Be actionable — a developer should be able to use it to resolve a design trade-off

**Happy Users Say...:** 4-5 quotes that illustrate what success looks like from the user's perspective. These should feel like real feedback, covering different aspects of the feature (ease, accuracy, error handling, edge cases, redirect to better tools).

**Product Vision Statement:** One sentence that captures the feature's purpose. Pattern: "Enable [audience] to [outcome] through [mechanism] — [bridging what gap]."

### Step 9: Build the summary canvas table

At the bottom of the document, create a two-part summary table that captures the entire canvas on one visual "page":

**Top row (wide):** Problem Statement, Experience Principles, Product Vision — stacked in a two-column layout.

**Bottom row (5 columns):** Target Audience | Audience Needs | Features (Phase 1) | Unique Value Proposition | Success Metrics — with key points from each section.

This table is the "canvas" — it should be scannable by an executive in under 2 minutes.

### Step 10: Review and cross-reference

Before finalizing:

1. Verify every Phase 1 feature maps to functional requirements in the requirements document
2. Ensure success metrics are measurable and aligned with requirements
3. Confirm audience needs are drawn from source material, not invented
4. Check that experience principles don't contradict each other
5. Verify the vision statement is concise (one sentence) and non-technical
6. Add a footer with creation date and source references
7. Add companion document links to the requirements and architecture documents

## Key instructions

- **User perspective, not system perspective** — the canvas describes what users need and experience, not how the system works internally
- **First person for needs, third person for everything else** — audience needs are quotes; the rest is descriptive
- **Trace features to requirements** — every feature listed should map to one or more `FR-*` IDs
- **Keep the vision statement non-technical** — an executive should understand it without knowing the tech stack
- **Happy Users Say quotes should feel real** — specific scenarios, not generic praise
- **Experience principles should resolve trade-offs** — "Trust the data" tells a developer to prioritize API data over LLM generation
- **The canvas table is the deliverable** — the sections above it are the supporting detail; the table at the bottom is what gets presented
