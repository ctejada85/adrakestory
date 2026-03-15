# Annotated Skill Examples

## Contents

- [Minimal skill (SKILL.md only)](#minimal-skill)
- [Medium skill (with references)](#medium-skill)
- [Complex skill (with scripts and assets)](#complex-skill)
- [Common patterns](#common-patterns)

---

## Minimal skill

A single SKILL.md for a focused, self-contained task.

**Directory:**

```
formatting-markdown/
└── SKILL.md
```

**SKILL.md:**

```yaml
---
name: formatting-markdown
description: Formats and lints Markdown files following consistent style rules. Use when the user asks to clean up, format, or standardize Markdown documents.
---
```

```markdown
# Formatting Markdown

## When to use
- User asks to format, lint, or clean up .md files
- User wants consistent heading styles, list formatting, or link syntax

## Rules
1. Use ATX-style headings (`#` not underlines)
2. One blank line before and after headings
3. Fenced code blocks with language tags (never indented blocks)
4. Ordered lists use `1.` for all items (auto-numbering)
5. Reference links for URLs used more than once
6. No trailing whitespace
7. Single newline at end of file
```

**Why it works:** Focused scope, clear triggers in description, concise rules Claude can follow without extra context.

---

## Medium skill

SKILL.md with reference files for a multi-domain task.

**Directory:**

```
querying-databases/
├── SKILL.md
└── references/
    ├── postgres.md
    ├── mysql.md
    └── sqlite.md
```

**SKILL.md:**

```yaml
---
name: querying-databases
description: Writes and optimizes SQL queries for PostgreSQL, MySQL, and SQLite. Use when the user needs help writing queries, debugging SQL, or optimizing database performance.
---
```

```markdown
# Querying Databases

## When to use
- User asks for help writing SQL queries
- User needs query optimization or debugging
- User mentions PostgreSQL, MySQL, or SQLite

## Workflow
1. Identify the target database engine
2. Read the relevant reference file for engine-specific syntax
3. Write the query following engine conventions
4. Suggest indexes or optimizations if the query involves joins or large tables

## Database references
- **PostgreSQL**: See [references/postgres.md](references/postgres.md)
- **MySQL**: See [references/mysql.md](references/mysql.md)
- **SQLite**: See [references/sqlite.md](references/sqlite.md)

## General SQL guidelines
- Use explicit `JOIN` syntax (never implicit joins in `WHERE`)
- Alias all tables in multi-table queries
- Use parameterized queries to prevent injection
- Add `LIMIT` during development/debugging
```

**Why it works:** Main SKILL.md stays short. Engine-specific details load on demand. One level of reference depth.

---

## Complex skill

Full directory with scripts, references, and assets.

**Directory:**

```
processing-csv-data/
├── SKILL.md
├── references/
│   ├── encoding-guide.md
│   └── large-files.md
├── scripts/
│   ├── validate.py
│   └── convert.py
└── assets/
    └── output-template.csv
```

**SKILL.md:**

```yaml
---
name: processing-csv-data
description: Parses, validates, transforms, and exports CSV files. Handles encoding issues, large files, and schema validation. Use when the user works with CSV data or tabular text files.
compatibility: Requires python3 with pandas installed
metadata:
  author: data-team
  version: "1.0"
---
```

```markdown
# Processing CSV Data

## When to use
- User asks to parse, transform, or validate CSV files
- User encounters encoding or delimiter issues
- User needs to process large CSV files efficiently

## Quick start

Validate a CSV file:

    python scripts/validate.py input.csv

Convert encoding or delimiters:

    python scripts/convert.py input.csv --encoding utf-8 --delimiter ","

## Workflow

1. **Validate**: Run `scripts/validate.py` to check structure and encoding
2. **Fix issues**: If validation fails, consult [references/encoding-guide.md](references/encoding-guide.md)
3. **Transform**: Apply required transformations using pandas
4. **Export**: Use [assets/output-template.csv](assets/output-template.csv) as the output format reference
5. **Re-validate**: Run validation again on the output

For files over 100MB, see [references/large-files.md](references/large-files.md) for chunked processing.

## Common transforms
- Column renaming / reordering
- Type casting (dates, numbers, booleans)
- Deduplication
- Filtering rows by condition
- Merging multiple CSVs
```

**Why it works:** Layered disclosure — quick start in SKILL.md, details in references, executable validation scripts, template asset for output format.

---

## Common patterns

### Template pattern

Provide exact output format when consistency matters:

```markdown
## Output format

ALWAYS use this structure:

    # [Title]
    ## Summary
    [1-2 sentences]
    ## Findings
    - Finding with evidence
    ## Recommendations
    1. Specific action
```

### Feedback loop pattern

Run, validate, fix, repeat:

```markdown
## Editing workflow
1. Make changes
2. Run `scripts/validate.py`
3. If validation fails, fix issues and go to step 2
4. Only proceed when validation passes
```

### Conditional workflow pattern

Branch based on context:

```markdown
## Determine approach

**Creating from scratch?** → Use the template in `assets/template.md`
**Editing existing file?** → Read the file first, preserve structure
**Bulk operation?** → Use `scripts/batch.py` for multiple files
```

### Examples pattern (input/output pairs)

Show Claude the desired output style:

```markdown
## Naming convention

**Input:** user authentication module
**Output:** `authenticating-users`

**Input:** PDF text extraction tool
**Output:** `extracting-pdf-text`

**Input:** Git commit message helper
**Output:** `generating-commit-messages`
```
