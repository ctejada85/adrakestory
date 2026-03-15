# Pattern: Web Scraping with Playwright + Docker + SQLite

Reference implementation: `.claude/skills/finance-exchange-fetch`

Use this pattern when a skill needs to read data from a website that:
- Is a **JavaScript-rendered SPA** (React, Vue, Nuxt, etc.) — plain HTTP requests won't work
- Requires **user interaction** to expose data (clicking tabs, drawers, menus)
- Should **cache results locally** to avoid fetching the same data multiple times per day

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Agent                              │
│  1. run bsc-get → cached? → done                       │
│  2. run bsc-fetch → reads raw content                  │
│  3. extracts values from content                       │
│  4. run bsc-save ← pipes extracted JSON via stdin      │
└─────────────────────────────────────────────────────────┘
         │                          │
   [Docker / Playwright]      [SQLite on host]
```

**Key principle:** The scripts handle I/O (browser + database). The **agent handles interpretation** (reading raw text and extracting meaning). This keeps scripts simple and extraction flexible.

---

## Directory structure

```
my-skill/
├── SKILL.md
├── docker-compose.yml        ← references shared agent-browser image
├── data/
│   ├── .gitignore            ← contains *.db
│   └── my-data.db            ← SQLite (gitignored, host-mounted)
└── scripts/
    ├── fetch_content.py      ← Playwright: open page, interact, return raw text
    ├── save_data.py          ← read JSON from stdin, insert into SQLite
    └── get_data.py           ← read today's data from SQLite (no network)
```

---

## Shared Docker image

All skills that need a browser use the same base image at `.claude/docker/agent-browser/`.

```dockerfile
# .claude/docker/agent-browser/Dockerfile
FROM mcr.microsoft.com/playwright/python:v1.50.0-jammy
WORKDIR /app
RUN pip install --quiet playwright && playwright install chromium
```

**Scripts are NOT baked into the image.** They are mounted at runtime via a volume. This means editing a script never requires rebuilding the image.

---

## docker-compose.yml template

```yaml
x-browser: &browser
  build:
    context: ../../docker/agent-browser
  image: agent-browser
  environment:
    - PYTHONUNBUFFERED=1
  volumes:
    - ./scripts:/app/scripts:ro     # scripts mounted read-only
    - ./data:/app/data              # SQLite DB persists on host

services:
  my-fetch:
    <<: *browser
    command: python3 scripts/fetch_content.py

  my-save:
    <<: *browser
    stdin_open: true
    command: python3 scripts/save_data.py

  my-get:
    <<: *browser
    command: python3 scripts/get_data.py
```

The `x-browser` YAML anchor avoids repeating config. `stdin_open: true` is required for the save service to receive piped input.

---

## Script 1: fetch_content.py

Opens the page, performs any needed interactions, returns raw text per section. **No parsing here.**

```python
#!/usr/bin/env python3
import json, sys
from playwright.sync_api import sync_playwright, TimeoutError as PlaywrightTimeoutError

TARGET_URL = "https://example.com/"

def run() -> int:
    result = {"url": TARGET_URL, "sections": {}, "error": None}

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            args=["--no-sandbox", "--disable-dev-shm-usage"],
        )
        ctx = browser.new_context(viewport={"width": 1440, "height": 900}, locale="es-DO")
        page = ctx.new_page()

        try:
            page.goto(TARGET_URL, wait_until="domcontentloaded", timeout=30_000)

            # Wait for a known element that signals the page is ready
            page.wait_for_selector('[data-testid="main-content"]', timeout=20_000)

            # Dismiss any modal/banner if present
            try:
                page.locator("button:has-text('Aceptar')").first.click(timeout=3_000)
            except Exception:
                pass

            # Click to expose the data you need
            page.locator('[aria-label="Open data panel"]').first.click()
            page.wait_for_timeout(2_000)

            # Capture raw text — let the agent read and extract
            result["sections"]["main"] = page.inner_text("body", timeout=5_000)

            # If data is behind tabs, iterate them:
            for label, key in [("Tab A", "a"), ("Tab B", "b")]:
                try:
                    page.locator(f"text={label}").first.click(timeout=4_000)
                    page.wait_for_function(
                        f"document.body.innerText.includes('{label}')",
                        timeout=6_000,
                    )
                except Exception:
                    result["sections"][key] = {"error": "tab_click_failed"}
                    continue
                result["sections"][key] = page.inner_text("body", timeout=5_000)

        except PlaywrightTimeoutError as exc:
            result["error"] = f"Timeout: {exc}"
        except Exception as exc:
            result["error"] = f"Unexpected error: {exc}"
        finally:
            ctx.close()
            browser.close()

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["sections"] and not result["error"] else 1

if __name__ == "__main__":
    sys.exit(run())
```

**Tips:**
- Use `wait_for_selector` (not just `domcontentloaded`) for SPAs — wait for a meaningful element
- Scope tab clicks to a container (drawer/panel) to avoid clicking wrong elements: `page.locator(".drawer").locator("text=Tab A")`
- Use `wait_for_function` with `document.body.innerText.includes(...)` to confirm tab switch completed
- Always `try/except` around optional interactions (cookie banners, tooltips)

---

## Script 2: save_data.py

Reads JSON from stdin (agent-provided), inserts into SQLite.

```python
#!/usr/bin/env python3
import json, os, sqlite3, sys
from datetime import date, datetime, timezone

DB_PATH = os.environ.get("DB_PATH", "/app/data/my-data.db")

def init_db(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    conn = sqlite3.connect(path)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS my_table (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            date      TEXT NOT NULL,
            key       TEXT NOT NULL,
            value     REAL,
            label     TEXT,
            saved_at  TEXT NOT NULL,
            UNIQUE(date, key)
        )
    """)
    conn.commit()
    return conn

def run() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception as exc:
        print(json.dumps({"error": f"Invalid JSON input: {exc}"}))
        return 1

    data = payload.get("data")
    if not data:
        print(json.dumps({"error": "Missing 'data' key in input"}))
        return 1

    today = payload.get("date") or date.today().isoformat()
    saved_at = datetime.now(timezone.utc).isoformat()

    conn = init_db(DB_PATH)
    saved = []

    for key, item in data.items():
        value = item.get("value")
        if value is None:
            continue
        conn.execute(
            "INSERT OR REPLACE INTO my_table (date, key, value, label, saved_at) VALUES (?,?,?,?,?)",
            (today, key, value, item.get("label", key), saved_at),
        )
        saved.append(key)

    conn.commit()
    conn.close()

    print(json.dumps({"status": "ok", "date": today, "saved": saved}, indent=2))
    return 0

if __name__ == "__main__":
    sys.exit(run())
```

**Key decisions:**
- `UNIQUE(date, key)` + `INSERT OR REPLACE` — idempotent; re-running never creates duplicates
- `date` defaults to today if not provided — agent can override for backdating
- Validate `data` key exists before touching the DB

---

## Script 3: get_data.py

Read-only. Returns today's cached data or tells the agent to fetch first.

```python
#!/usr/bin/env python3
import json, os, sqlite3, sys
from datetime import date

DB_PATH = os.environ.get("DB_PATH", "/app/data/my-data.db")

def run() -> int:
    today = date.today().isoformat()

    if not os.path.exists(DB_PATH):
        print(json.dumps({"error": "No database found. Run my-fetch first."}))
        return 1

    conn = sqlite3.connect(DB_PATH)
    rows = conn.execute(
        "SELECT key, label, value FROM my_table WHERE date = ?", (today,)
    ).fetchall()
    conn.close()

    if not rows:
        print(json.dumps({"error": f"No data for {today}. Run my-fetch first."}))
        return 1

    print(json.dumps({
        "date": today,
        "from_cache": True,
        "data": {key: {"label": label, "value": value} for key, label, value in rows},
    }, ensure_ascii=False, indent=2))
    return 0

if __name__ == "__main__":
    sys.exit(run())
```

---

## SKILL.md workflow section template

Document the three commands clearly so the agent knows when to use each:

```markdown
## Workflow

1. **`my-get`** — check DB cache for today's data. If found, return immediately.
2. **`my-fetch`** — open browser, interact with page, return raw content.
3. Agent reads content and extracts structured values.
4. **`my-save`** — agent pipes extracted JSON via stdin; script saves to SQLite.

## Commands

\`\`\`bash
cd .claude/skills/my-skill

# Check cache
docker compose run --rm my-get

# Fetch raw content
docker compose run --rm my-fetch

# Save extracted data (pipe JSON)
echo '{"data": {"key": {"label": "Thing", "value": 42.0}}}' \
  | docker compose run --rm -T my-save
\`\`\`
```

---

## Common SPA interaction patterns

### Wait for page readiness (avoid race conditions)
```python
# Bad: fires before JS has rendered
page.goto(url, wait_until="domcontentloaded")

# Better: wait for a real element
page.wait_for_selector('[data-section="prices"]', timeout=20_000)
```

### Clicking a tab that's inside a panel
```python
# Scope to the open panel — avoids clicking identical text in the nav
panel = page.locator(".v-navigation-drawer--active, [role='dialog']").first
panel.locator(f"text={tab_name}").first.click(timeout=5_000)
```

### Confirm tab switch completed
```python
page.wait_for_function(
    f"document.body.innerText.includes('{expected_text}')",
    timeout=8_000,
)
```

### Fallback when drawer selector is uncertain
```python
try:
    page.wait_for_selector(".drawer--active", timeout=8_000)
except PlaywrightTimeoutError:
    page.wait_for_timeout(3_000)  # blind wait as last resort
```

---

## data/.gitignore

Always exclude the SQLite file from git:

```
*.db
```

The `data/` folder itself should be committed (with only `.gitignore` in it) so the volume mount path exists on fresh clones.

---

## Reference implementation

See `.claude/skills/finance-exchange-fetch/` for a complete working example fetching 5 currencies from a Nuxt SPA (Banco Santa Cruz), iterating tabs, and caching to SQLite.
