---
title: Sessions Folder README
purpose: Session log format spec — any AI tool can read and write these files consistently
---

# Sessions/

This folder holds structured logs of AI-assisted work sessions. One file per session. Each file captures what happened so future sessions can pick up where the previous one left off.

## File naming

```
YYYY-MM-DD-<short-slug>.md
```

Examples:

- `2026-04-08-cpr-setup.md`
- `2026-04-09-vidpipe-homepage-rewrite.md`

Multiple sessions same day: append `-2`, `-3`.

## File format

```markdown
---
type: session
date: YYYY-MM-DD
time: "HH:MM"
projects: [list, of, related, projects]
topics: [list, of, key, topics]
tool: claude-code | cowork | cursor | aider | other
outcome: one-line summary of what the session accomplished
---

# Session: YYYY-MM-DD — short slug

## Quick Reference
**Topics**: comma-separated key topics
**Projects**: which projects were touched
**Outcome**: one-line summary

## Decisions Made
- Decision 1 (with reasoning if non-obvious)

## Key Learnings
- Thing now understood that wasn't before

## Files Touched
- path/to/file.md — what changed

## Pending / Next Actions
- [ ] Thing to do next session

## Open Questions
- Thing not yet decided

## Raw Log (optional)
Full conversation or detailed notes, for later searchability.
```

## How tools should use this folder

**On session start (`/load`):** read the 3 most recent files (sort by filename, descending). Combine Quick Reference + Decisions + Pending sections to reconstruct recent context.

**On session end (`/save`):** write a new file in this format. Curate, don't dump. Quick Reference / Decisions / Learnings / Files Touched / Pending should stay short and scannable.

**For label-scoped operations:** the `projects` and `topics` arrays in frontmatter are what `/load <label>` and `/save <label>` match against.

## Notes

- Aim to keep files under ~500 lines. If a session log is longer, the curation step probably dumped too much.
- Don't manually edit `type: session` — verbs use it to distinguish session logs from other notes.
- Consider archiving anything older than 6 months into `Sessions/Archive/YYYY-MM/`.
