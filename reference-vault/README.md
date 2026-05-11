# Your Memcrate Vault

> Edit this file to describe your vault. The text below is the Memcrate-shipped default — keep what's useful, change what isn't.

## What's in here

```
.
├── README.md              # This file
├── .memcrate              # Marker file — lets tools find this vault from any subdirectory
└── Core/
    ├── Context/
    │   ├── Profile.md         # Stable facts about you: tools, preferences, anti-goals
    │   ├── Projects.md        # Every project, with status, stack, decisions
    │   └── Current State.md   # This week's focus, deadlines, recent decisions
    └── Sessions/
        └── _README.md         # Session log schema (filled in by /save)
```

The four files inside `Core/` are the verbs' world — `/save`, `/pin`, and `/load` only read and write here. Everything outside `Core/` is yours.

## Growing into the personal-OS scope

When you want more than just verbs, add optional folders alongside `Core/`:

```
.
├── Core/                  # (already here)
├── Projects/              # Per-project thinking layer: specs, decisions, devlog
├── Daily/                 # Daily notes
├── Tasks/                 # Short-term work queue
└── Inbox/                 # Unprocessed capture
```

These are conventions, not requirements. `mkdir Projects` and you're done — the verbs won't notice, because they only look inside `Core/`.

## Conventions

- Frontmatter is YAML, always at the top of each file. The verbs read `last_updated` (Profile / Projects / Current State) and `projects` / `topics` arrays (Sessions).
- Wikilinks (`[[Profile]]`) work in Obsidian and most modern markdown editors; degrade gracefully.
- Session log filenames: `YYYY-MM-DD-<short-slug>.md`. Multiple same day: append `-2`, `-3`.
- ISO 8601 dates everywhere (`2026-05-09`).

## How to use

End every AI-tool session with `/save`. Start every session with `/load`. When something graduates from "useful for this session" to "true forever," run `/pin <insight>`.

Edit any file by hand whenever you want — the verbs are conveniences, not gates. The vault is yours.

## Read more

See the [Memcrate format spec](../docs/) for the full verb contracts, scoping rules, and integration details across AI tools.
