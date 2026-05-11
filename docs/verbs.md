---
title: Verbs
---

# Verbs

The three verbs are the user-facing surface of Memcrate. They map to specific file operations on the vault, and the contracts described here are what every tool integration is expected to honor.

## `/save [label]`

**When:** End of session, or whenever you want to checkpoint progress.

**Behavior:**

1. Look back through the conversation. Extract: topics, projects touched, decisions made, key learnings, files touched, pending actions, open questions, outcome.
2. If `[label]` is passed, scope the curation to that label only. (Useful for splitting a multi-topic session into focused logs.)
3. Write `Sessions/YYYY-MM-DD-<slug>.md` with the standard schema.
4. If any decision affects weekly operations, also append to `Current State.md` under "Recent Decisions / Changes."

**Output file format** (`Sessions/2026-05-09-auth-rewrite-chapter-1.md`):

```markdown
---
type: session
date: 2026-05-09
time: "14:30"
projects: [myapp]
topics: [auth, oauth, jwt-refresh]
tool: claude-code
outcome: OAuth flow refactored; refresh-token logic in place; ready for review.
---

# Session: 2026-05-09 — Auth rewrite, part 1

## Quick Reference
**Outcome:** OAuth flow refactored; refresh-token logic landed.

## Decisions Made
- Use short-lived access tokens (15min) with sliding refresh windows.
- Token rotation on every refresh; revoke previous on issue.

## Key Learnings
- ...

## Files Touched
- `src/auth/oauth.ts` — refactored
- ...

## Pending / Next Actions
- [ ] Wire the refresh endpoint into the client retry logic
- [ ] Add integration test for revoked-token replay

## Open Questions
- ...
```

**Multi-label / split-session usage:** Run `/save` more than once in a single conversation with different `[label]` arguments. Each run curates only the slices relevant to that label. Example: a 3-hour session that touched two projects becomes `/save myapp` then `/save side-project` — two separate logs, each focused, neither bleeding into the other.

## `/pin <insight>`

**When:** A specific decision or fact has graduated from "useful for this session" to "needed forever." Examples: a tooling default (`gh` CLI account defaults to `myuser`), a hard rule (never deploy on Fridays), a project status change (MyApp launched 2026-04-28).

**Behavior:**

1. Decide which canonical file the insight belongs in:
   - **`Profile.md`** — stable identity, preferences, anti-goals, tools
   - **`Projects.md`** — project status, stack, decisions
   - **`Current State.md`** — this week's focus, active deadlines
2. Append the insight to the right section, preserving existing voice and format.
3. Bump `last_updated` in the file's frontmatter.
4. Tell the user where it landed and why.

**Constraint:** `/pin` is the only operation that writes to the canonical context files. `/save` writes to `Sessions/`, `/load` is read-only. This single-write-path discipline keeps the canonical files clean — no skill or automation should touch `Profile.md` / `Projects.md` / `Current State.md` outside of `/pin`.

## `/load [label]`

**When:** Start of a session, when you want oriented before working.

**Behavior:**

1. Read `README.md` (or `CLAUDE.md` if present), `Profile.md`, `Projects.md`, `Current State.md`.
2. Read the 3 most recent files in `Sessions/` (or more, if `[count]` is passed).
3. If `[label]` is passed, switch to **scoped mode**: extract slices, find label-tagged sessions first, fall back to body grep. Topic-style or project-style scoping per the project-match rule below.
4. Output a short oriented summary (~5–8 lines, more if scoped).

The output is *orientation, not execution*. `/load` doesn't start work; it hands the session to the user oriented and waits.

## Project-match rule

Used by both scoped `/load` and scoped `/save` to decide whether a label refers to a project or a freeform topic.

A label matches a project if either:

- (a) it matches a `## <heading>` in `Projects.md`, OR
- (b) it matches a folder anywhere under `Projects/` — at root *or* one level deep inside a bucket — excluding bucket names themselves (`Shelf`, `Ideas`, `Shipped`, `Archived`).

Matching is **normalized**: lowercase both sides and strip whitespace, dots, and hyphens before comparing. So `myapp` ≈ `MyApp.ai`, `repo-triage` ≈ `RepoTriage`, `mission-control` ≈ `Mission Control`. Use substring containment for partial labels.

If the label matches → treat as a project (write `projects: [<canonical-name>]` in the saved log's frontmatter, scope `/load` to project-related files first).

If no match → treat as a topic (write `topics: [<label>]`, scope `/load` to recent sessions tagged with that topic).

A vault folder is **not required** for a project — many projects have entries in `Projects.md` without a `Projects/<name>/` folder of their own. The folder is a thinking layer, not the project's identity.

If a label looks like it should be a project but doesn't match (typo, non-existent), the verb implementation should list the known projects and ask before proceeding rather than silently treating it as a topic.

## Why three verbs and not more

Memcrate's verb count is deliberately small. Each verb maps to a clear vault operation:

- `/save` — write to `Sessions/`
- `/pin` — write to one of the three canonical files
- `/load` — read everything that matters

A `/search` verb has been considered and rejected for now: every modern editor and AI tool already has full-text search, and the canonical files are small enough to read end-to-end. Adding `/search` is an option for the MCP layer (`vault_search` tool), but not a first-class verb.

A `/sync` verb is rejected by design: Memcrate is local-first. Sync is the user's choice — Obsidian Sync, iCloud, Dropbox, git, whatever — and Memcrate stays out of that decision.
