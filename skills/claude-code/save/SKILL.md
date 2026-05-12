---
name: save
description: "Save the current session as a structured log in the user's Memcrate vault so future sessions can pick up where this one left off. Invoke whenever the user runs /save or tells you to save the session or wrap up. Writes a properly formatted session log following the Memcrate session schema."
---

# save — Write Session Log

Your job: write a structured session log to the user's Memcrate vault capturing what happened, what was decided, what was learned, and what's pending. The goal is that `/load` in a future session can read this file and instantly reconstruct useful context.

## Narrate before acting

**Before any tool call, output a one-line status message to the user** so they understand what's about to happen — especially the first time `/save` runs in a fresh install, when each new tool (Bash, Write) triggers a permission prompt.

Open with something like:

> Saving this session — locating your vault, summarizing what we did, and writing a session log to Core/Sessions/.

If you discover something mid-save (e.g. "writing scoped log for project X" or "no existing Sessions folder, creating one"), say that in one line before the next tool call.

## Optional Arguments

The user can pass a label to scope the session log. The label can be a **project** or a **topic** — anything meaningful.

- `/save` — default: infer projects and topics from the conversation, capture the whole session
- `/save <label>` — scope to one label (e.g., `/save myapp`, `/save auth-redesign`, `/save memory-system`)

**Auto-detect project vs topic** (same rule as `/load`):

A label matches a project if either:
- (a) it matches a `## <heading>` in `Core/Context/Projects.md`, OR
- (b) it matches a folder anywhere under `Projects/` — at root *or* one level deep inside a bucket — excluding bucket names themselves (`Shelf`, `Ideas`, `Shipped`, `Archived`).

Matching is **normalized**: lowercase both sides and strip whitespace, dots, and hyphens before comparing. So `myapp` ≈ `MyApp.ai`, `repo-triage` ≈ `RepoTriage`, `mission-control` ≈ `Mission Control`. Use substring containment for partial labels.

- If matched → treat as a project.
- Otherwise → treat as a topic (free-form label, any kebab-case string is fine).
- A vault folder is **not required** for a project — many projects have entries in `Projects.md` without a `Projects/<name>/` folder of their own. The folder is a thinking layer, not the project's identity.
- If the user passes something that looks like it should be a project but doesn't match (typo, non-existent), list the known projects and ask before proceeding.

**When a label is passed, do all of the following:**

- **Curate the conversation to only label-relevant portions.** This is the whole point of scoping — a session about vault stuff AND a side project AND something else should become separate focused logs, not one sprawling one.
- **Set frontmatter field:**
  - If project → primary entry in `projects:` array (can still add secondaries if legitimately touched)
  - If topic → primary entry in `topics:` array
- **Bias the slug** toward the label (e.g., `myapp-homepage-rework` not `homepage-rework`).
- **Frame the outcome and summary** around the label.

**Splitting one conversation into multiple logs:**

The user can run `/save` multiple times in one session with different labels to cut the conversation into focused logs. Example: after 90 min of vault work + 60 min of project work, run `/save vault` then `/save myproject` — two separate files, each covering just its slice. For each run, curate only the portions relevant to that label and ignore content from the other arcs.

**Why this matters:** `/load <label>` filters sessions by matching the `projects:` / `topics:` frontmatter — scoped saves make future scoped loads surface exactly the right history.

A vault folder is **not required** for a project — many projects have entries in `Projects.md` without a `Projects/<name>/` folder of their own. The folder is a thinking layer, not the project's identity.

## Vault Location

**Use Read and Glob, not Bash.** They produce clean permission prompts with full paths visible.

Try in order:

1. **Read `~/vault/Core/Context/Profile.md`** — the default install location. If Read returns content, `<vault>` = `~/vault`.
2. **Read `<cwd>/Core/Context/Profile.md`** — handles users who `cd`'d into their vault before launching Claude Code. Use the absolute working-directory path from your session context.
3. **Glob with `path: <home>` and `pattern: "*/Core/Context/Profile.md"`** — depth-1 scan of the user's home directory for vaults at custom paths like `~/myvault`. Resolve `<home>` from your session context (typically `/home/<user>` on Linux, `/Users/<user>` on macOS, `C:\Users\<user>` on Windows).
    - **One match** → `<vault>` is the first path segment of that match (strip `/Core/Context/Profile.md`).
    - **Multiple matches** → list them and ask the user which to use.

If all three fail, ask the user in plain English:

> I couldn't find a Memcrate vault at `~/vault`, in this directory, or anywhere one level deep in your home directory. Where is your vault? (Paste the absolute path. If you haven't set one up yet, run `memcrate init ~/vault` and try again.)

Then `Read <answer>/Core/Context/Profile.md` to confirm before proceeding.

Session logs always live at `<vault>/Core/Sessions/`. If that folder doesn't exist, create it (and `Core/Sessions/_README.md` describing the schema) before writing the first log. The canonical context files at `<vault>/Core/Context/{Profile,Projects,Current State}.md` should already exist — if they don't, the vault is malformed and that's worth flagging to the user.

## Step 1: Extract Content From the Session

Look back through the current conversation and pull out:

- **Topics**: 2-5 key topics that came up (e.g., "auth redesign", "vault restructure", "claude memory")
- **Projects touched**: which projects were discussed or worked on
- **Decisions made**: concrete choices with reasoning
- **Key learnings**: things now understood that weren't clear before
- **Files touched**: any vault files or code files that were created, edited, or deleted
- **Pending next actions**: anything that was deferred, flagged for later, or left unfinished
- **Open questions**: things that were raised but not resolved
- **Outcome**: one sentence summarizing what the session accomplished overall

## Step 2: Write the File

Don't ask the user to review or confirm before writing — they trust the session notes. Just write the file directly. (If something is genuinely ambiguous, e.g. "should this be split into two logs?", that's worth a quick check; routine "does this look good?" preview prompts are not.)

Filename format: `YYYY-MM-DD-short-slug.md`

- Use today's date (check via bash `date +%Y-%m-%d` if unsure)
- Slug should be 2-5 kebab-case words describing the session focus
- If a file with that name already exists, append `-2`, `-3`, etc.

File format (follow exactly — this matches the schema in `Core/Sessions/_README.md`):

```markdown
---
type: session
date: YYYY-MM-DD
time: HH:MM
projects: [project1, project2]
topics: [topic1, topic2, topic3]
tool: claude-code
outcome: one-line summary
---

# Session: YYYY-MM-DD — short slug

## Quick Reference
**Topics**: topic1, topic2, topic3
**Projects**: project1, project2
**Outcome**: one-line summary

## Decisions Made
- Decision 1 (with reasoning if it isn't obvious)
- Decision 2
- ...

## Key Learnings
- Thing now understood
- ...

## Files Touched
- `path/to/file.md` — what changed
- ...

## Pending / Next Actions
- [ ] Task to do next session
- [ ] Follow-up
- ...

## Open Questions
- Unresolved thing
- ...

## Raw Log
_(Optional. Include a condensed version of the key conversation moments if they would help reconstruct the thinking later. Skip if the structured sections above are enough.)_
```

## Step 3: Confirm Save

After writing, tell the user:

```
Saved to `Core/Sessions/2026-05-11-short-slug.md`.
Run `/load` in your next session and this will be part of the loaded context.
```

## Important

- **Don't dump the raw conversation.** Curate. Better to lose some detail than to create a 2000-line file nobody will read. Raw Log should be condensed, not verbatim.
- **Pending items should be actionable.** "Think about X" is bad. "Draft the homepage hero copy" is good.
- **If the session didn't produce anything worth saving** (quick lookup, simple Q&A, etc.), tell the user that and skip the write. Not every conversation needs a log.
- **Update `Core/Context/Current State.md`'s "Recent Decisions / Changes" section** if any decisions from this session belong there (anything that affects weekly operations). This is the one place `/save` touches a canonical context file — keep the entries one-line and dated.
