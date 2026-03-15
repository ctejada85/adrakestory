---
name: basic-agents-skills-create
description: Creates, updates, and deletes Agent Skills following the open specification (agentskills.io). Use when the user asks to create, scaffold, modify, rename, consolidate, or remove a skill, or when building reusable agent capabilities with SKILL.md files.
---

# Writing Agent Skills

## When to use this skill

Use this skill when the user wants to:

- Create a new agent skill from scratch
- Scaffold a skill directory structure
- Write or improve a SKILL.md file
- Convert existing instructions into the Agent Skills format
- Rename, consolidate, or delete existing skills

## Workflow

Copy this checklist and track progress:

```
Skill Write:
- [ ] Step 1: Determine scope and naming
- [ ] Step 2: Create directory structure
- [ ] Step 3: Write SKILL.md frontmatter
- [ ] Step 4: Write SKILL.md body
- [ ] Step 5: Add reference files (if needed)
- [ ] Step 5b: Add extension guide (if skill has scripts or API integrations)
- [ ] Step 6: Validate the skill
- [ ] Step 7: Update AGENTS.md
```

### Step 1: Determine scope and naming

Ask or infer:

- **What does the skill do?** One focused capability per skill.
- **When should it activate?** Specific triggers/keywords.
- **How complex is it?** Simple (SKILL.md only) vs. complex (with references/scripts/assets).

Choose a name using the **`{scope}-{subject}-{action}`** convention used in this workspace:

- **Scope** — the top-level domain or system the skill belongs to (e.g., `documents`, `workspace`, `jira`, `ado`, `agents`)
- **Subject** — what the skill operates on (e.g., `architecture`, `git`, `rust`, `nodejs`, `dotnet`, `bugs`, `skills`)
- **Action** — the verb describing what the skill does (e.g., `create`, `build`, `commit`, `push`, `login`, `write`, `analyze`, `track`, `generate`, `plan`, `review`, `trace`)

Examples:

| Skill name | Scope | Subject | Action |
|------------|-------|---------|--------|
| `basic-finance-expenses-log` | `finance` | `expenses` | `log` |
| `basic-family-math-generate` | `family` | `math` | `generate` |
| `basic-plans-weekly-write` | `plans` | `weekly` | `write` |
| `basic-health-exercise-track` | `health` | `exercise` | `track` |

Formatting rules:
- Lowercase letters, numbers, hyphens only. Max 64 chars.
- No leading/trailing hyphens, no consecutive hyphens (`--`).
- No reserved words: "anthropic", "claude".
- Scope can be compound (e.g., `repos-inference`) when scoping to a specific repository.
- **Do not use gerund form** (`processing-pdfs`) — use the bare verb (`process`) as the action segment.

### Step 2: Create directory structure

Minimal skill:

```
skill-name/
└── SKILL.md
```

Complex skill:

```
skill-name/
├── SKILL.md           # Required: instructions + metadata
├── references/        # Optional: detailed docs loaded on demand
├── scripts/           # Optional: executable code
└── assets/            # Optional: templates, schemas, static resources
```

**Script preference:** Prefer **Python** for all skill automation scripts. Run them with `uv` or `uvx` — never install packages globally.

- `uv run script.py` — runs a script; declare inline dependencies at the top so `uv` installs them automatically into an isolated environment:
  ```python
  # /// script
  # dependencies = ["httpx", "beautifulsoup4"]
  # ///
  ```
- `uvx tool-name` — runs a one-off CLI tool without installing it (e.g., `uvx ruff check .`)
- **Browser automation** (Playwright) runs inside Docker using the shared `agent-browser` image at `.claude/docker/agent-browser/`. See [references/pattern-web-scraping.md](references/pattern-web-scraping.md).

**One script per action (atomic scripts):** When a skill supports multiple actions, give each action its own script file. Do **not** combine actions into a single script with mode flags.

```
scripts/
├── gmail_list.py      # lists messages (gmail.readonly scope)
├── gmail_archive.py   # archives messages (gmail.modify scope)
└── gmail_read.py      # reads body of a message (future)
```

Benefits:
- Each script has a single responsibility and can evolve independently
- Scopes, dependencies, and token files can differ between actions without coupling
- Agent invocation is unambiguous — one script name = one operation
- Scripts are easier to test and debug in isolation

Naming: `{subject}_{action}.py` — lowercase, underscore-separated.

Place the skill directory at `.claude/skills/` in this workspace.

### Step 3: Write SKILL.md frontmatter

Required fields only:

```yaml
---
name: skill-name
description: Does X and Y. Use when the user asks about Z or mentions W.
---
```

With optional fields (workspace defaults):

```yaml
---
name: skill-name
description: Does X and Y. Use when the user asks about Z or mentions W.
---
```

**Description rules:**

- Write in **third person** ("Processes files..." not "I can process..." or "You can use...").
- Include **what it does** and **when to use it** in one statement.
- Be specific — include keywords that help the agent match tasks to this skill.
- Max 1024 characters.

### Step 4: Write SKILL.md body

**Core principle: Be concise.** Claude is already smart — only provide context it doesn't already have. Every token competes with conversation history.

Structure the body with these recommended sections:

1. **When to use** — bullet list of triggers
2. **Workflow** — numbered steps or checklist for complex tasks
3. **Key instructions** — the essential how-to content
4. **References** — links to bundled files for advanced/detailed content

**Degrees of freedom** — match specificity to the task:

| Task type | Freedom level | Instruction style |
|-----------|--------------|-------------------|
| Fragile operations (migrations, deployments) | Low | Exact commands, no deviation |
| Preferred patterns exist | Medium | Pseudocode, parameterized scripts |
| Multiple valid approaches | High | Text guidelines, heuristics |

**Guidelines:**

- Keep SKILL.md body under **500 lines**.
- Move detailed reference material to separate files in `references/`.
- Use **one level of reference depth** — all referenced files should link directly from SKILL.md.
- Include **feedback loops** for complex tasks: run → validate → fix → repeat.
- Avoid time-sensitive information (no "before August 2025, use..." statements).
- Use **consistent terminology** throughout — pick one term per concept.

See [references/examples.md](references/examples.md) for full annotated examples.

### Step 5: Add reference files (if needed)

Only create reference files when SKILL.md exceeds ~300 lines or has distinct sub-domains.

- `references/*.md` — detailed docs, API references, domain-specific guides
- `scripts/*.py|*.sh` — executable code the agent can run
- `assets/*` — templates, schemas, lookup tables

For reference files over 100 lines, add a **table of contents** at the top.

### Step 5b: Add an extension guide (if the skill has scripts or external integrations)

When a skill includes scripts or integrates with an external API, create `references/adding-capabilities.md`. This guide lets the skill extend itself consistently without re-deriving patterns from scratch.

**When to create it:**
- The skill has one or more scripts in `scripts/`
- The skill calls an external API or service
- There are clear Phase 2 / future capabilities planned
- Another developer (or agent) could reasonably add a new action

**What to include:**

```
references/adding-capabilities.md:
- [ ] Implementation patterns (how scripts are structured, invoked, communicate)
- [ ] New script checklist + script template to copy
- [ ] Shared code patterns (auth, API calls, data parsing) — copy-paste ready
- [ ] API/service reference (query syntax, available endpoints, response shapes)
- [ ] Planned capabilities (Phase 2+) mapped to script names and args
- [ ] Scope / permission reference (what the current credentials allow + how to upgrade)
```

**After writing it**, add a link from SKILL.md's "File locations" section:
```markdown
| Extension guide | [`references/adding-capabilities.md`](references/adding-capabilities.md) |
```

**Example:** see `.claude/skills/basic-inbox-gmail-manage/references/adding-capabilities.md`.

---

### Step 6: Validate the skill

Verify manually:

1. `name` matches the directory name
2. `name` follows `{scope}-{subject}-{action}` convention (lowercase, hyphens, no reserved words)
3. `description` is non-empty, under 1024 chars, third person
4. SKILL.md body is under 500 lines
5. All referenced files exist at the paths specified
6. No deeply nested reference chains (max one level from SKILL.md)

**Common SKILL.md authoring mistakes to avoid:**

| Mistake | Symptom | Fix |
|---------|---------|-----|
| File starts with ` ````skill ` code fence | `missing or malformed YAML frontmatter` | Remove the wrapper — `---` must be the very first line of the file |
| `metadata:` block in frontmatter | `unknown field ignored: metadata` | Remove the `metadata:` block entirely; only `name:` and `description:` are supported |
| Unquoted colon-space (`: `) inside a description value (e.g., `type(scope): desc`) | `Nested mappings are not allowed in compact mappings` | Wrap the description value in double quotes |
| Windows CRLF line endings (`\r\n`) | Malformed frontmatter / parse errors | Save as LF-only (`\n`); run `sed -i 's/\r//' SKILL.md` to fix |

### Step 7: Update AGENTS.md

**This step is mandatory after every skill create, update, rename, or delete.**

Read `AGENTS.md` and update the **Agent Skills** section to reflect the change:

- **Created a skill** — add a row to the relevant table with the skill name, directory, and "When to Use" description
- **Renamed a skill** — update the skill name and directory in the table row
- **Consolidated skills** — remove the old rows and add the new consolidated row
- **Deleted a skill** — remove the row from the table
- **Updated a skill's scope or triggers** — update the "When to Use" column

Also check for any other references to the old skill name elsewhere in `AGENTS.md` (e.g., inline mentions, workflow descriptions) and update them.

## Anti-patterns to avoid

- **Over-explaining** — Don't teach Claude what PDFs are or how libraries work.
- **Vague descriptions** — "Helps with files" won't trigger correctly. Be specific.
- **Deeply nested references** — SKILL.md → advanced.md → details.md causes partial reads.
- **Time-sensitive content** — Dates and version cutoffs go stale.
- **Inconsistent terminology** — Pick one word per concept and stick with it.
- **Giant monolithic SKILL.md** — Split into references at ~300 lines.
- **First/second person descriptions** — Always use third person.
- **Forgetting AGENTS.md** — Every skill change must be reflected in AGENTS.md.
