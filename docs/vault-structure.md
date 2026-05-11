---
title: Vault Structure
---

# Vault Structure

The vault is a directory of markdown files with conventions. There is one canonical shape — `Core/` holds everything the verbs care about, and a small set of optional human-only folders sit alongside `Core/` for the personal-OS scope.

## The shape

```
<vault>/
├── README.md                  # Vault overview (what's here, conventions)
├── .memcrate                  # Marker file (lets the upward-walk resolver find this vault)
├── Core/
│   ├── Context/
│   │   ├── Profile.md         # Stable: who you are, preferences, tools
│   │   ├── Projects.md        # All your projects with status
│   │   └── Current State.md   # Living: this week's focus, deadlines
│   └── Sessions/              # Session logs from /save
│       └── _README.md         # Session log schema
│
├── Projects/                  # (optional) per-project thinking layer
│   ├── README.md              # Bucket convention
│   ├── <Project>/             # Specs, decisions, devlog
│   ├── Ideas/
│   ├── Shelf/
│   ├── Shipped/
│   └── Archived/
├── Daily/                     # (optional) daily notes
├── Tasks/                     # (optional) short-term work queue
└── Inbox/                     # (optional) unprocessed capture
```

**Everything inside `Core/` is the verbs' world.** Profile, Projects, Current State, and Sessions — these are what `/save`, `/pin`, and `/load` read and write. The structure is fixed; the verbs depend on it.

**Everything outside `Core/` is for humans.** The optional folders (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) are conventions Memcrate ships because they're useful, not requirements the verbs check for. You can add, rename, or skip them freely.

## Why `Core/` exists

Three reasons:

1. **The verbs need a stable, known location.** Skills, the CLI, and the MCP server all look in `Core/Context/` and `Core/Sessions/`. No fallback logic, no resolution table — one place.
2. **It separates "stuff the verbs care about" from "stuff humans care about."** Without that separator, the canonical `Projects.md` file would collide with the optional `Projects/` bucket folder at the root level. With `Core/`, no ambiguity.
3. **It future-proofs.** Adding a new optional folder (say, `Inbox/`) never affects the verbs because they never look outside `Core/`. The personal-OS scope can grow without touching the verb contract.

## Minimum vs. full

The "minimum" install is just `Core/` + a `README.md`. No optional folders. The verbs work the same. The CLI calls this `memcrate init` (the default).

The "full" install is `Core/` + every optional folder pre-scaffolded as empty buckets. Useful when you know you want the personal-OS scope from day one. The CLI calls this `memcrate init --full`.

Both are the same vault. The optional folders are just there or not. You can add them later by running `mkdir Projects` (or `Daily`, etc.) — the verbs won't notice, and they don't need to.

## Vault layout discovery

Tools need to locate the vault regardless of where they're invoked from. Resolution order:

1. **`$MEMCRATE_VAULT_PATH`** environment variable — explicit override.
2. **`./` (current working directory)** — if it contains a `.memcrate` marker file (or a `Core/Context/` folder), use it.
3. **`~/vault/`** — Memcrate's default install path.
4. **Walk parent directories** for a `.memcrate` marker file (similar to `.git` upward-walk).

Inside the located vault, canonical files always live at:

- `<vault>/Core/Context/Profile.md`
- `<vault>/Core/Context/Projects.md`
- `<vault>/Core/Context/Current State.md`
- `<vault>/Core/Sessions/`

No alternates. No fallbacks. If they're not there, the vault is malformed and `memcrate doctor` flags it.

## Schema highlights

- **Frontmatter** is YAML, always at the top. Required fields per file type are documented in each section's `_README.md`.
- **Wikilinks** (`[[Profile]]`, `[[Projects]]`) work in Obsidian and most modern markdown editors; degrade gracefully to plain text in others.
- **Filenames** for session logs: `YYYY-MM-DD-<slug>.md`. Multiple sessions same day: append `-2`, `-3`.
- **Dates in frontmatter** are ISO 8601 (`2026-05-09`).

## Frontmatter contracts

The verbs depend on a small number of frontmatter fields. These are the load-bearing ones; everything else is optional.

**`Profile.md`**, **`Projects.md`**, **`Current State.md`:**

```yaml
---
title: <human-readable name>
last_updated: YYYY-MM-DD          # bumped by /pin
---
```

**Session logs in `Core/Sessions/`:**

```yaml
---
type: session
date: YYYY-MM-DD
time: "HH:MM"
projects: [<list>]                # populated by /save's project-match rule
topics: [<list>]                  # populated for non-project sessions
tool: claude-code | claude-desktop | cursor | aider | other
outcome: <one-line summary>
---
```

The `projects` and `topics` arrays are what scoped `/load <label>` matches against. Anything else in frontmatter is ignored by the verbs but free for you to add.

## What lives where

This is a guide for deciding where a given piece of information belongs. It maps directly to `/pin`'s file-selection step.

| File | Cadence | Examples |
|---|---|---|
| `Core/Context/Profile.md` | Quarterly | Identity, preferences, tools, anti-goals, audience definitions, hard rules |
| `Core/Context/Projects.md` | Monthly | Project status, stack, equity, milestones, decision summaries |
| `Core/Context/Current State.md` | Weekly | This-week focus, active deadlines, recent decisions log, open questions |
| `Core/Sessions/<...>.md` | Per session | Decisions made, files touched, pending actions, outcomes |
| `Daily/YYYY-MM-DD.md` | Daily (optional) | Free-form daily notes; not consumed by verbs |
| `Tasks/<...>.md` | Continuous (optional) | Short-term work queue |
| `Inbox/<...>.md` | Ad-hoc (optional) | Unprocessed capture before sorting |

The four rows under `Core/` are the verbs' world. Everything below is for humans and isn't read by `/load` unless explicitly asked.

## Sync and storage

Memcrate is local-first and storage-agnostic. The vault is just a folder; sync is your call:

- **Obsidian Sync** — paid, end-to-end encrypted, works across desktop + mobile. Recommended for users already on Obsidian.
- **iCloud Drive / Google Drive / Dropbox** — works fine; watch for editor lock-file conflicts.
- **Git** — strong choice if you're comfortable with the workflow; gives diff history for free.
- **Syncthing / `rsync`** — for self-hosted setups.

Memcrate doesn't bundle a sync solution. The CLI's `memcrate init` may prompt for a recommendation but you always pick.

## What's not in the vault

Some things deliberately don't belong:

- **Secrets.** No passwords, API keys, tokens, recovery codes. The vault may end up shared, public-exported, or backed up; treat it as read-only-by-AI in that sense.
- **Code.** The vault is for *thinking about* code, not the code itself. Code lives in repos.
- **Binary blobs.** Images and PDFs are fine if essential; the vault should stay readable in plain text.
- **Mutable runtime state.** Live operational data (cron output, heartbeats, real-time service health) belongs in a system tool, not a markdown file.
