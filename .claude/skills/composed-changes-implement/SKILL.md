---
name: composed-changes-implement
description: "Implements a change from a completed user story ticket. Pre-requisites: requirements.md, architecture.md, and ticket.md must all exist. Use when the user says 'implement this ticket' or points at a ticket.md file to execute."
---

# Implementing from a User Story Ticket

## Pre-requisites (must exist before this skill runs)

| Document | Typical location |
|----------|-----------------|
| `requirements.md` | `docs/bugs/<slug>/` or `docs/features/<slug>/` |
| `architecture.md` | same folder |
| `ticket.md` | same folder |

If any of these are missing, stop and use the appropriate upstream skill first:
- Missing requirements → `basic-documents-requirements-write`
- Missing architecture → `composed-documents-architecture-write`
- Missing ticket → `basic-documents-userstory-write`

## When to use

- User says "implement this ticket" or "implement `ticket.md`"
- User points at a `ticket.md` and says "get to work"
- All planning documents are already complete and code changes are ready to start

## Workflow

```
Implementation:
- [ ] Step 1: Prime — read developer guides
- [ ] Step 2: Read all three ticket documents
- [ ] Step 3: Load ticket tasks into SQL todos
- [ ] Step 4: Implement code changes (task by task)
- [ ] Step 5: Write unit tests
- [ ] Step 6: Validate (build + lint + tests)
- [ ] Step 7: Update architecture.md
- [ ] Step 8: Commit
```

---

### Step 1: Prime — read developer guides

Read these three files in full before touching any code. They define non-negotiable rules and patterns that all implementation must follow.

| File | What it covers |
|------|---------------|
| `docs/developer-guide/coding-guardrails.md` | 10 hard rules — things that cause real bugs if violated (unconditional `get_mut`, missing `SpatialGrid`, delta clamping, etc.) |
| `docs/developer-guide/coding-style.md` | Naming, derives, system signatures, module layout, logging format, test structure |
| `docs/developer-guide/architecture.md` | High-level system architecture, ECS patterns, state machine, two-binary structure |

Read all three before proceeding. Do not skip this step even if the task seems small.

---

### Step 2: Read all three ticket documents

Read `requirements.md`, `architecture.md`, and `ticket.md` in full before writing any code.

Extract from `ticket.md`:
- The numbered **task list** → each task becomes a SQL todo
- The **acceptance criteria** → used to verify completeness at Step 6
- The **non-functional requirements** → inform code quality constraints

If any open questions remain in the requirements or architecture, stop and resolve them with `ask_user` before proceeding.

---

### Step 3: Load ticket tasks into SQL todos

Insert one row per task. Use descriptive IDs.

```sql
INSERT INTO todos (id, title, description) VALUES
  ('task-1', 'Task title from ticket', 'Implementation detail from architecture Appendix D');
```

Add dependencies where tasks must be sequential:

```sql
INSERT INTO todo_deps (todo_id, depends_on) VALUES ('task-3', 'task-2');
```

Query ready tasks before starting each one:

```sql
SELECT t.* FROM todos t
WHERE t.status = 'pending'
AND NOT EXISTS (
  SELECT 1 FROM todo_deps td
  JOIN todos dep ON td.depends_on = dep.id
  WHERE td.todo_id = t.id AND dep.status != 'done'
);
```

---

### Step 4: Implement code changes

Execute tasks in dependency order. For each task:

1. `UPDATE todos SET status = 'in_progress' WHERE id = 'task-N'`
2. Make the change — follow the code template in `architecture.md` Appendix D
3. `UPDATE todos SET status = 'done' WHERE id = 'task-N'`

Refer back to `coding-guardrails.md` (read in Step 1) if you are unsure whether a pattern is safe.

---

### Step 5: Write unit tests

Tests live in a sibling `tests.rs` file (see `coding-style.md` §7). Write tests for every test task in `ticket.md`. Cover at minimum:
- **Cache / no-op path** — identical input produces no observable change
- **Change path** — mutated input produces a different result
- **Independence** — changing concern A does not affect concern B
- **Cold-start / first-frame** — empty state always triggers the action
- **Assembly correctness** — output struct maps every field from inputs

Rules:
- Prefer pure helper functions — avoid full Bevy `World` setup unless unavoidable
- One logical claim per `assert_*`; use descriptive values, not magic numbers

---

### Step 6: Validate

Run in order. Fix all failures before proceeding to Step 7.

```bash
cargo test --lib                  # unit tests
cargo clippy --lib                # lint — zero errors allowed
cargo build --release             # confirm clean release build
```

For each acceptance criterion in `ticket.md`, verify it is satisfied:
- Automated ACs → confirmed by a passing test
- Manual ACs → note them for the user; do not block the commit

Pre-existing test failures in unrelated files: confirm with `git stash` + rerun, then document but do not fix.

---

### Step 7: Update architecture.md

After validation passes, update `docs/developer-guide/architecture.md` to reflect any structural changes introduced by this ticket.

Review what changed and update the relevant sections:

| What changed | Section to update |
|---|---|
| New component, resource, or system added | Component/Resource/System inventory or ECS overview |
| New module or file added | File structure / module organization section |
| Data flow or system ordering changed | System set ordering or data flow diagrams |
| New state or state transition added | State machine section |
| New external dependency or Bevy plugin added | Dependencies or architecture notes |
| Performance-critical pattern introduced | Performance-critical patterns section |

Rules:
- Only update sections that are materially affected — do not rewrite unrelated content
- Keep descriptions concise; this is a reference doc, not a tutorial
- If a section does not exist yet but the change warrants one, add it
- Do not add speculative or future content — only document what was actually implemented

---

### Step 8: Commit

Stage all changed source files and commit with a conventional message.

```
<type>(<scope>): <short imperative summary>

- One bullet per logical change (struct added, system rewritten, tests added)
- Reference the ticket or requirements doc if helpful

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

Group all changes from the ticket (code + tests + architecture.md) in a single commit unless the user asks to split them.
