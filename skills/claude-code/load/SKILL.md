---
name: load
description: "Load the user's operating context at the start of a session by reading their Memcrate vault (Profile, Projects, Current State, and recent session logs). Output a short oriented summary, then wait for instructions. Invoke proactively on the first user message of any fresh session before substantive work — even if the message seems self-contained — so you have full context from turn 1. Also invoke whenever the user runs /load or asks you to load context. Skip only if context has already been loaded this session or the message is purely conversational with no task implication."
---

# load — Load Operating Context

Your job: load the user's persistent operating context from their Memcrate vault so you can work with full awareness of who they are, what they're building, and what happened in recent sessions. End with a short oriented summary and then wait for instructions.

## Narrate before acting

**Before any tool call, output a one-line status message to the user** so they understand what's about to happen. This matters because the first tool call (locating the vault) will trigger a permission prompt on a fresh install, and an unexplained bash prompt is jarring.

Open with something like:

> Loading your Memcrate context — checking for your vault, then reading Profile, Projects, Current State, and recent sessions.

Then proceed. If you discover something mid-load (e.g. "scoped mode for project X" or "no session logs yet, falling back to general mode"), say that in one line before the next tool call. End with the formatted summary (see Output Format below).

## Vault Location

**Use the Read tool, not Bash.** Read handles `~` expansion and produces clean, recognizable permission prompts (one file at a time, full path visible). Bash with shell expansion triggers harder-to-read permission prompts.

Try in order:

1. **Read `~/vault/Core/Context/Profile.md`** — the default install location. If Read returns content, the vault is `~/vault`.
2. **Read `<cwd>/Core/Context/Profile.md`** — handles users who `cd`'d into their vault before launching Claude Code. Use the absolute working-directory path from your session context.

If both reads fail, ask the user in plain English:

> I couldn't find a Memcrate vault at `~/vault` or in this directory. Where is your vault? (Paste the absolute path. If you haven't set one up yet, run `memcrate init ~/vault` and try again.)

Then `Read <answer>/Core/Context/Profile.md` to confirm before proceeding.

Canonical files always live at:

- `<vault>/Core/Context/Profile.md`
- `<vault>/Core/Context/Projects.md`
- `<vault>/Core/Context/Current State.md`
- `<vault>/Core/Sessions/`

If `Profile.md` exists but the others don't, the vault is malformed — surface that to the user rather than guessing alternate locations.

## What to Read

Read these files in this order. If any are missing, note it and continue — don't stop.

1. **`<vault>/README.md`** (or `<vault>/CLAUDE.md` if present) — vault overview, conventions
2. **`Core/Context/Profile.md`** — who the user is, how they work, tools, goals, anti-goals, preferences
3. **`Core/Context/Projects.md`** — every project with status, stack, milestones
4. **`Core/Context/Current State.md`** — this week's focus, active deadlines, recent decisions, open questions
5. **The 3 most recent files in `Core/Sessions/`** — session logs from recent work. List the folder, sort filenames descending, skip `_README.md` and anything in `Archive/`, read the top 3.

For the session logs, focus on the Quick Reference, Decisions Made, Key Learnings, and Pending Next Actions sections. You don't need to read the Raw Log section unless asked.

## Optional Arguments

The user can pass arguments to narrow what you load:

- `/load` — default general mode: last 3 sessions + all canonical context files
- `/load 10` — general mode with last 10 sessions (deeper history)
- `/load <label>` — **scoped mode**: lean context focused on one label (project or topic). See below.
- `/load 5 myapp` — scoped mode with the session count bumped to 5

**How to decide which mode:**

1. If the arg is a number → general mode with adjusted session count.
2. If the arg is any non-number string → **scoped mode** (see next section). Whether the label is a "project" or a "topic" is auto-detected via the project-match rule below; if it's a project, project rules apply on top of the common scoped behavior.

**Project-match rule (used by both `/load` and `/save`):**

A label matches a project if either:
- (a) it matches a `## <heading>` in `Core/Context/Projects.md`, OR
- (b) it matches a folder anywhere under `Projects/` — at root *or* one level deep inside a bucket — excluding the bucket names themselves (`Shelf`, `Ideas`, `Shipped`, `Archived`).

Matching is **normalized**: lowercase both sides and strip whitespace, dots, and hyphens before comparing. So `myapp` ≈ `MyApp.ai`, `repo-triage` ≈ `RepoTriage`, `mission-control` ≈ `Mission Control`. Use substring containment for partial labels (`myapp` matches `myappai`).

If multiple projects match, ask the user to disambiguate. If nothing matches, treat as a topic.

## Scoped Mode

When the user passes any label — a project name, a topic, a concept, anything — switch to a focused load. Skip unrelated context and zoom in on that label. This is for tactical work (coding a feature, writing about one topic, picking up a thread) where the full strategic load is noise.

**Common rules for every scoped load:**

1. **Extract slices, not whole files** (skip the generic parts):
   - From `Core/Context/Projects.md` — grep for the label, read matching sections only
   - From `Core/Context/Current State.md` — grep for lines mentioning the label, include those
   - **Skip `Core/Context/Profile.md` entirely** — the user's voice and anti-goals stay internalized, but don't load the full file
2. **Find label-tagged sessions first** — in `Core/Sessions/` (live folder, not `Archive/`), look for session logs whose frontmatter `projects:` or `topics:` array contains this label. Take the 2-3 most recent matches. Focus on Decisions Made and Pending Next Actions.
3. **Fallback — grep session bodies** in the live folder for the label; take the 2-3 most recent hits.
4. **Archive fallback** — if still no matches, grep the same way inside `Core/Sessions/Archive/`. If you find hits, surface them and note they're from archive.
5. **Last resort** — if nothing matches anywhere, say so explicitly, fall back to the last 3 sessions overall, and ask the user if this label is new or if they meant something else.
6. **Case-insensitive** matching throughout.

**If the label matches a project** (project-style scoping):

- Read the matching `## <name>` section from `Core/Context/Projects.md` (the strategic prose layer for the project — always the canonical source of project state).
- If a vault folder exists for the project (at `Projects/<name>/` *or* `Projects/<bucket>/<name>/`), read its `*.md` files — Changelog, Ideas, active work docs. **The vault folder is optional** — many projects have entries in `Projects.md` without a dedicated folder. Skip silently if absent; do not flag it as missing.
- If `Projects.md` mentions a repo URL, surface the likely local clone path but **don't read repo files automatically** — Claude Code loads the repo's own `CLAUDE.md` when the user `cd`s into it.
- Title the summary with the project name.

**If the label doesn't match a project** (topic-style scoping):

- Treat it as a free-form topic (e.g., "auth", "sync", "memory", "skills").
- No project folder or `Projects.md` section to read — the sliced context + matching sessions are enough.
- Title the summary with the topic label.

## Output Format

After reading everything, give the user a short oriented summary. **Do not dump the file contents.**

### General mode (no arg or numeric session count)

Aim for ~5-8 lines:

```
**Loaded context.**
- **Who**: [1 sentence — identity, role, what they do]
- **This week**: [1-2 lines — from Current State's "This Week's Focus" section]
- **Active deadlines**: [1 line — flagging anything urgent]
- **Last session (YYYY-MM-DD)**: [1 line — outcome from the most recent session log, if any]
- **Pending from last session**: [1 line — top 1-2 pending items, if any]
What are we working on today?
```

### Scoped mode

Aim for ~5-7 lines, focused entirely on the label:

```
**Loaded <Label> context.**
- **What this is**: [1 line — from Projects.md entry if project, else the most on-point Current State.md line or session summary]
- **Recent activity**: [1-2 lines — from the most recent matching session, or note if none]
- **Pending**: [1 line — top 1-2 pending items tied to this label]
- **Local repo** (only if label is a project + repo is cloned locally): [path]
What are we working on?
```

If no session logs exist yet, say so and skip that line. If the vault is empty or canonical files are missing, say what's missing and ask the user to run `/save` first or create the missing files.

## Important

- **Don't start doing work until the user tells you what we're working on.** The purpose of `/load` is orientation, not execution.
- **Don't paraphrase the user's preferences back at them.** Internalize them silently and act on them throughout the session.
- **Respect anti-goals.** If you see anti-goals in `Core/Context/Profile.md` (things the user has explicitly opted out of), don't forget them later in the session.
- **Match the user's voice and tone** as you can infer it from their context files: terse if they're terse, expansive if they're expansive, plain if they're plain.
