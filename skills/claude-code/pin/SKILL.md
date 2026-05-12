---
name: pin
description: "Promote a specific insight, decision, or fact from the current conversation into one of the user's permanent context files (Profile, Projects, or Current State). Invoke whenever the user runs /pin or tells you to pin, remember permanently, or add something to their profile. Figures out which file the insight belongs in, updates it, and bumps the last_updated timestamp."
---

# pin — Promote Insight to Permanent Memory

Your job: take a specific insight, decision, or fact and write it into the right permanent context file in the user's Memcrate vault. This is the bridge from *session memory* (the Sessions folder) to *permanent memory* (`Profile.md` / `Projects.md` / `Current State.md`).

## Narrate before acting

**Before any tool call, output a one-line status message to the user** so they understand what's about to happen — especially the first time `/pin` runs in a fresh install, when each new tool triggers a permission prompt.

Open with something like:

> Pinning this — figuring out which context file it belongs in (Profile / Projects / Current State), then updating it.

After deciding the destination, restate which file you're about to edit before the Edit call.

## Vault Location

**Use the Read tool, not Bash.** Read handles `~` expansion and produces clean, recognizable permission prompts (one file at a time, full path visible).

Try in order:

1. **Read `~/vault/Core/Context/Profile.md`** — the default install location. If Read returns content, the vault is `~/vault`.
2. **Read `<cwd>/Core/Context/Profile.md`** — handles users who `cd`'d into their vault before launching Claude Code. Use the absolute working-directory path from your session context.

If both reads fail, ask the user in plain English:

> I couldn't find a Memcrate vault at `~/vault` or in this directory. Where is your vault? (Paste the absolute path. If you haven't set one up yet, run `memcrate init ~/vault` and try again.)

Then `Read <answer>/Core/Context/Profile.md` to confirm before proceeding.

The canonical context files always live at:

- `<vault>/Core/Context/Profile.md`
- `<vault>/Core/Context/Projects.md`
- `<vault>/Core/Context/Current State.md`

If they're not there, the vault is malformed — surface that to the user rather than writing to an alternate location.

## When to Use This

Session logs are ephemeral — they capture what happened in one conversation. Permanent context files hold things that should be true across *every* session. Examples of things worth pinning:

- "My core audience is junior developers" → `Core/Context/Profile.md`
- "Year-1 MRR target for my SaaS is $50K" → `Core/Context/Projects.md`
- "This week's #3 priority is shipping the auth rewrite" → `Core/Context/Current State.md`
- "I don't do live streams, talks, or podcasts" → `Core/Context/Profile.md` (anti-goals section)
- "New collaborator on a 2-month trial" → `Core/Context/Projects.md` (the relevant project's section)

Examples of things NOT to pin (these belong in `Core/Sessions/` via `/save`, not in the canonical files):

- "Today we decided the button should be blue"
- "The homepage copy is stuck"
- "I wrote 200 words of the book today"

The test: does this matter *next week*? If yes → pin. If it's only relevant to this session → save it via `/save` instead.

## Step 1: Identify the Insight

If the user says "pin this" after a specific statement, use that statement. If they say "/pin" without context, ask:

```
What specifically should I pin? Paste or describe the insight, decision, or fact you want moved into permanent memory.
```

## Step 2: Decide Which File

Use this decision tree:

- **`Core/Context/Profile.md`** — stable facts about the user, how they work, tools, preferences, goals, constraints, background. Changes quarterly.
- **`Core/Context/Projects.md`** — anything about a specific project (status, stack, equity, milestones, success markers, blockers). Changes monthly or on project events.
- **`Core/Context/Current State.md`** — this-week stuff (focus, deadlines, decisions log, open questions). Changes weekly.

If you're not sure, tell the user your guess and ask for confirmation:

```
This sounds like it belongs in **Core/Context/Profile.md** (Tools & Stack section). Does that match your thinking, or should it go elsewhere?
```

Let the user override. They know their own brain better than you do.

## Step 3: Find the Right Section

Within the target file, find the correct section. Examples:

- Tool change → `Core/Context/Profile.md` → `## Tools & Stack` section
- New project status → `Core/Context/Projects.md` → that project's block
- This week's priority → `Core/Context/Current State.md` → `## This Week's Focus` section
- New anti-goal → `Core/Context/Profile.md` → `## Anti-goals` (or wherever the user keeps them)
- New risk tolerance note → `Core/Context/Profile.md` → `## Personal Context` → Risk tolerance subsection

If the right section doesn't exist yet, ask the user whether to create it or where to put the content.

## Step 4: Write the Update

Use the Edit tool (not Write) to make surgical edits to the target file. Keep:

- The surrounding text intact
- The file's existing tone and format
- Bullet style and indentation consistent with neighbors

## Step 5: Bump `last_updated`

In the target file's YAML frontmatter, update the `last_updated` field to today's date. Check today's date via `date +%Y-%m-%d` if unsure.

## Step 6: Confirm With the User

```
Pinned.
**File**: `Core/Context/Profile.md` (Tools & Stack → Code & AI)
**Added**: "Experimenting with [new tool] for [purpose]"
**last_updated** bumped to YYYY-MM-DD.
```

## Important

- **One insight per `/pin` call.** If the user wants to pin multiple things, prompt them to run it once per insight so each lands in the right place.
- **Don't duplicate.** Before writing, check whether the insight is already captured somewhere else in the file. If it is, update the existing line instead of adding a new one.
- **Respect the file's purpose.** Don't cram weekly stuff into `Profile.md`, and don't put stable facts in `Current State.md`. When in doubt, ask.
- **If `Profile.md` or `Projects.md` exceeds ~500 lines**, flag it to the user so they can decide whether to split or archive. Don't silently let them balloon.
- **If the insight contradicts something already in the file**, flag the contradiction to the user and ask which version is correct before overwriting.
- **`/pin` is the only verb that writes to the canonical context files** (with one exception: `/save` may append a one-line dated entry to `Core/Context/Current State.md`'s "Recent Decisions / Changes" section). Treat the canonical files as `/pin`'s territory.
