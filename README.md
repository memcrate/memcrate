# Memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One folder. Any tool.

**Status:** Phase 1. Skills, reference vault, and format spec are written. Manual install works. CLI (Phase 2) and MCP server (Phase 3) come later. Not yet announced — being dogfooded by the author.

## What this is

Every AI coding tool eventually loses context. Sessions end. Tools change. You start fresh in Cursor on Monday, then jump to Claude Code on Tuesday, then a Claude Desktop window for planning on Wednesday — and each one needs to be told from scratch what you're working on, what's been decided, what's broken, what's next.

Memcrate is a local markdown directory plus three verbs (`/save`, `/pin`, `/load`) that any AI tool can read and write to. Your context lives in plain `.md` files you own. The verbs make the rituals consistent across tools.

Read [docs/overview.md](docs/overview.md) for the full pitch and [docs/verbs.md](docs/verbs.md) for the verb contracts.

## Why not [other thing]

- **Static `CLAUDE.md` / `AGENTS.md` / `.cursorrules`** capture project facts but not session state.
- **Cloud-backed AI memory** (mem0, Letta, supermemory) is vendor-locked and not repo-scoped.
- **Session-memory packs** (CPR — the verb-trio inspiration here) work great for one tool but don't span tools or carry full personal context.
- **MCP memory servers** are binary; humans can't read them.

Memcrate is a personal context OS, not a memory tool. Your project catalog, daily state, decisions, and infrastructure all draw from the same vault.

## Repo layout

```
memcrate/
├── reference-vault/    # Starter vault scaffold (Core/ + .memcrate marker) — copy anywhere
├── skills/             # Canonical SKILL.md files for each AI tool
│   └── claude-code/    # /save, /load, /pin (working; symlink and go)
└── docs/               # Format spec: overview, verbs, vault structure, skills, CLI
```

## Manual install (Phase 1)

For Claude Code:

```bash
git clone https://github.com/bradtraversy/memcrate
cd memcrate

# Pick a vault location and copy the starter files
cp -r reference-vault ~/vault

# Symlink the Claude Code skills
ln -s "$(pwd)/skills/claude-code/save" ~/.claude/skills/save
ln -s "$(pwd)/skills/claude-code/load" ~/.claude/skills/load
ln -s "$(pwd)/skills/claude-code/pin" ~/.claude/skills/pin

# Optional — set the vault path env var so the verbs find your vault from anywhere
echo 'export MEMCRATE_VAULT_PATH=~/vault' >> ~/.zshrc
```

In a Claude Code session, type `/load` to get oriented, then start working. End the session with `/save`. When something becomes worth remembering forever, `/pin` it.

For other tools (Claude Desktop, Cursor, Aider) — see [docs/skills.md](docs/skills.md) for per-tool install steps. Phase 2 will collapse all of this into `memcrate install --all`.

## What's in `reference-vault/`

A starter vault. Everything the verbs need is inside `Core/`; the rest is yours to grow.

```
reference-vault/
├── README.md
├── .memcrate              # marker file (lets tools find the vault)
└── Core/
    ├── Context/
    │   ├── Profile.md         # who you are, your tools, anti-goals, hard rules
    │   ├── Projects.md        # every project with status, stack, milestones
    │   └── Current State.md   # this week's focus, deadlines, recent decisions
    └── Sessions/              # /save writes session logs here
```

Each canonical file has section guidance inline so you know what belongs where. Add optional folders (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) alongside `Core/` whenever you want the personal-OS scope — see [docs/vault-structure.md](docs/vault-structure.md).

## License

- Code (`skills/`, future CLI source, install scripts) — [MIT](LICENSE)
- Format spec (`docs/`) — [CC0](LICENSE-spec). Build a Memcrate-compatible tool without legal friction.

## Credit

The verb trio (`/save`, `/pin`, `/load`) generalizes [EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) — a Claude-Code-only session-memory skill pack. Memcrate scales that pattern to multi-tool personal-context-OS, adds `/pin` for the bridge from session memory to permanent memory, and decouples the format from any one tool. Honest lineage.
