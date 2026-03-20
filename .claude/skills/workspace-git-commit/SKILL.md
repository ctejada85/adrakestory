---
name: workspace-git-commit
description: Creates a git commit with conventional commits format. Use when the user asks to commit changes, create a commit message, or needs version control tracking after completing code work.
---

# Git Commit Skill

## When to use

- User requests "commit" or "create a commit"
- After completing code changes that need version control tracking
- When creating commits for features, fixes, documentation, or refactoring
- When amending previous commits (only unpushed ones)

## Workflow Checklist

```
Git Commit:
- [ ] Step 1: Review staged and unstaged changes
- [ ] Step 2: Stage files with `git add` if needed
- [ ] Step 3: Choose conventional commit type
- [ ] Step 4: Write concise subject line (<50 chars)
- [ ] Step 5: Add detailed body (optional, for complex changes)
- [ ] Step 6: Execute `git commit -m "..."`
```

## Commit Types (Conventional Commits)

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat: add player jump mechanic` |
| `fix` | Bug fix | `fix: resolve collision detection issue` |
| `perf` | Performance improvement | `perf: optimize chunk meshing algorithm` |
| `docs` | Documentation changes | `docs: update README with installation steps` |
| `refactor` | Code refactoring (no behavior change) | `refactor: extract player movement logic` |
| `chore` | Maintenance tasks | `chore: add .gitignore for node_modules` |
| `style` | Formatting changes | `style: format code with rustfmt` |

## Commit Message Format

- **Subject line**: Keep under 50 characters, imperative mood, lowercase (except proper nouns)
- **Body** (optional): Wrap at 72 characters, explain what and why not how
- **References**: Include helpful context like files/functions when applicable

## Commit Types (Conventional Commits)

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat: add player jump mechanic` |
| `fix` | Bug fix | `fix: resolve collision detection issue` |
| `perf` | Performance improvement | `perf: optimize chunk meshing algorithm` |
| `docs` | Documentation changes | `docs: update README with installation steps` |
| `refactor` | Code refactoring (no behavior change) | `refactor: extract player movement logic` |
| `chore` | Maintenance tasks | `chore: add .gitignore for node_modules` |
| `style` | Formatting changes | `style: format code with rustfmt` |

## Commit Message Format

- **Subject line**: Keep under 50 characters, lowercase (except proper nouns)
- **Body** (optional): Wrap at 72 characters, explain what and why not how
- **Reference files/functions**: Include helpful context when applicable

## Examples

```bash
# Check current status before committing
git status

# Stage specific files (optional if already staged)
git add src/systems/game/collision.rs

# Simple feature commit
git commit -m "feat: add new voxel placement tool"

# Commit with detailed body (use quotes for multi-line)
git commit -m "fix: resolve cylinder collider collision issue

- Fixed cylinder collider math for player entity in check_sub_voxel_collision()
- Updated collision bounds calculation to use radius for horizontal checks
- Added unit tests for edge cases at chunk boundaries"

# Amend previous commit (use only for unpushed commits)
git commit --amend -m "chore: update .roo/skills symlink to .claude/skills"

# Add more changes to last commit without amending
git add <new-changes> && git commit --amend --no-edit
```

## Notes

- Use `--amend` only when the last commit needs correction or adding more changes
- Never amend commits that have been pushed to shared/remote branches
- Reference related files or functions in commit messages for traceability
- Check `git status` before committing to see what's staged vs unstaged
