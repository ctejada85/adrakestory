---
name: composed-changes-implement
description: "Orchestrates the full end-to-end workflow for implementing a change: investigate → requirements → architecture → user story → code → tests → validate → commit. Use when the user asks to implement a feature, fix a bug, or work a ticket from scratch through to a committed, tested result."
---

# Implementing Changes End-to-End

## When to use

- User asks to implement a feature, fix a bug, or work a ticket end-to-end
- User says "implement this", "fix this", "build this" with a description or linked document
- Starting from a bug report, requirements doc, or verbal description
- Any change that warrants documentation before coding

## Workflow

Track each step in the SQL `todos` table before starting.

```
Implementation:
- [ ] Step 1: Clarify scope
- [ ] Step 2: Investigate (if bug or unknown code area)
- [ ] Step 3: Write requirements doc
- [ ] Step 4: Write architecture doc
- [ ] Step 5: Write user story / ticket
- [ ] Step 6: Implement code changes
- [ ] Step 7: Write unit tests
- [ ] Step 8: Validate (build + lint + tests)
- [ ] Step 9: Commit
```

---

### Step 1: Clarify scope

Before doing anything else, confirm:
- Is this a **bug fix** or a **new feature**?
- Is there an existing bug report, requirements doc, or ticket to work from?
- What is explicitly **out of scope**?

Use `ask_user` if ambiguous. Do not start Step 2 until scope is clear.

---

### Step 2: Investigate (bugs and unknown areas)

Use the `basic-bugs-investigate` skill.

Skip this step if:
- Requirements are already written and the code area is well understood
- The user provides a clear enough description to proceed directly to Step 3

Output: investigation report (in `docs/bugs/` or session state), discrete list of root causes.

---

### Step 3: Write requirements document

Use the `basic-documents-requirements-write` skill.

- Output goes in `docs/bugs/<slug>/requirements.md` (bugs) or `docs/features/<slug>/requirements.md` (features)
- Resolve all open questions before proceeding — do not leave `TBD` in scope-critical fields
- Phase boundaries must be explicit; confirm with user if phasing is unclear

---

### Step 4: Write architecture document

Use the `composed-documents-architecture-write` skill.

- Output goes alongside requirements at `docs/…/<slug>/architecture.md`
- Must include: current state, target state, sequence diagrams, code template (Appendix D)
- Verify all Mermaid diagrams render (no `~`, `&`, `||`/`&&` in node labels; quote complex edge labels)
- Align with requirements: every FR should map to a code change in the architecture

---

### Step 5: Write user story / ticket

Use the `basic-documents-userstory-write` skill.

- Output: `docs/…/<slug>/ticket.md`
- Required sections: story, description, acceptance criteria (numbered), non-functional requirements (bulleted), tasks (numbered)
- Always include unit test tasks — one task per test case group
- Tasks must be granular enough to track individually in SQL todos

---

### Step 6: Implement code changes

Load the ticket tasks into the `todos` SQL table, then execute them in order.

```sql
INSERT INTO todos (id, title, description) VALUES
  ('task-1', 'Task title', 'Detail from ticket');
```

**Per task:**
1. Set `status = 'in_progress'` before starting
2. Make surgical, focused changes — one concern per edit
3. Follow project code style (see AGENTS.md)
4. Set `status = 'done'` when complete

**Bevy / Rust specifics for this repo:**
- Systems must be added to the correct `GameSystemSet` (see AGENTS.md ordering)
- All map mutations in editor must go through `EditorHistory`
- Never use `Assets::get_mut()` unconditionally — always guard with change detection or dirty check
- Use `SpatialGrid` for collision queries, never iterate all `SubVoxel` entities directly

---

### Step 7: Write unit tests

Tests live in an inline `#[cfg(test)]` module at the bottom of the file being tested.

**What to test:**
- Happy path (cache hit, expected output)
- Miss/change path (changed input → different result)
- Independence of concerns (changing A does not affect B)
- First-frame / cold-start (empty cache → always executes)
- Assembly correctness (output struct maps all fields correctly)

**Rules:**
- Pure helper functions only — no Bevy `World` setup required unless unavoidable
- One `assert_*` per logical claim; prefer `assert_eq!` with descriptive values
- Both structs under test must derive `Debug` (required by `assert_eq!` / `assert_ne!`)
- Keep tests in the same file as the code under test

---

### Step 8: Validate

Run in order; fix any failures before proceeding to Step 9.

```bash
cargo test --lib                  # all unit tests
cargo clippy --lib                # lint; zero errors allowed
cargo build --release             # confirm release build is clean
```

**Interpreting results:**
- Test failures in unrelated files: check `git stash` + rerun to confirm pre-existing; document but do not fix
- Clippy `private type in public interface`: add `pub(super)` to the private type
- Clippy `unused`: remove or add `#[allow(dead_code)]` only if intentional

---

### Step 9: Commit

Stage all changed files and commit with a conventional commit message.

```
<type>(<scope>): <short imperative summary>

- Bullet list of what changed and why
- One bullet per logical change (struct added, system rewritten, tests added, docs created)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

Group related changes (code + docs + skill) in a single commit unless the user asks to split them.

---

## Decision table

| Situation | Action |
|-----------|--------|
| Bug with unknown root cause | Start at Step 2 (investigate) |
| Bug with known root cause | Start at Step 3 (requirements) |
| New feature from scratch | Start at Step 1 (clarify), then Step 3 |
| Existing ticket to implement | Start at Step 6 (load todos from ticket tasks) |
| Requirements exist, no architecture | Start at Step 4 |
| Architecture exists, no ticket | Start at Step 5 |
