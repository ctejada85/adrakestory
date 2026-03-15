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
- [ ] Step 1: Read all three documents
- [ ] Step 2: Load ticket tasks into SQL todos
- [ ] Step 3: Implement code changes (task by task)
- [ ] Step 4: Write unit tests
- [ ] Step 5: Validate (build + lint + tests)
- [ ] Step 6: Commit
```

---

### Step 1: Read all three documents

Read `requirements.md`, `architecture.md`, and `ticket.md` in full before writing any code.

Extract from `ticket.md`:
- The numbered **task list** → each task becomes a SQL todo
- The **acceptance criteria** → used to verify completeness at Step 5
- The **non-functional requirements** → inform code quality constraints

If any open questions remain in the requirements or architecture, stop and resolve them with `ask_user` before proceeding.

---

### Step 2: Load ticket tasks into SQL todos

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

### Step 3: Implement code changes

Execute tasks in dependency order. For each task:

1. `UPDATE todos SET status = 'in_progress' WHERE id = 'task-N'`
2. Make the change — follow the code template in `architecture.md` Appendix D
3. `UPDATE todos SET status = 'done' WHERE id = 'task-N'`

**Bevy / Rust specifics for this repo:**
- Systems must be added to the correct `GameSystemSet` (Input → Movement → Physics → Visual → Camera)
- All map mutations in the editor must go through `EditorHistory`
- Never call `Assets::get_mut()` unconditionally — guard with change detection or a dirty check
- Use `SpatialGrid` for collision queries; never iterate all `SubVoxel` entities directly
- Derive `Debug` on any struct used in `assert_eq!` / `assert_ne!` tests
- Private types used in `pub` function signatures need `pub(super)` to satisfy Clippy

---

### Step 4: Write unit tests

Tests live in an inline `#[cfg(test)]` module at the bottom of the file under test.

Write tests for every test task listed in `ticket.md`. Cover at minimum:
- **Cache / no-op path** — identical input produces no observable change
- **Change path** — mutated input produces a different result
- **Independence** — changing concern A does not affect concern B
- **Cold-start / first-frame** — empty state always triggers the action
- **Assembly correctness** — output struct maps every field from inputs

Rules:
- Prefer pure helper functions — avoid full Bevy `World` setup unless unavoidable
- One logical claim per `assert_*`; use descriptive values, not magic numbers
- Keep tests in the same file as the production code

---

### Step 5: Validate

Run in order. Fix all failures before proceeding to Step 6.

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

### Step 6: Commit

Stage all changed source files and commit with a conventional message.

```
<type>(<scope>): <short imperative summary>

- One bullet per logical change (struct added, system rewritten, tests added)
- Reference the ticket or requirements doc if helpful

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

Group all changes from the ticket (code + tests) in a single commit unless the user asks to split them.
