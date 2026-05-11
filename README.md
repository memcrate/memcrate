# Memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One vault. Any tool.

**Status:** Phase 2 in progress. CLI v0.1 ships `memcrate init` (more commands — `install`, `update`, `status`, `doctor` — coming in subsequent v0.x releases). MCP server (Phase 3) comes later. Not yet announced — being dogfooded by the author.

## What this is

Every AI coding tool eventually loses context. Sessions end. Tools change. You start fresh in Cursor on Monday, then jump to Claude Code on Tuesday, then a Claude Desktop window for planning on Wednesday — and each one needs to be told from scratch what you're working on, what's been decided, what's broken, what's next.

Memcrate is a local markdown vault plus three verbs (`/save`, `/pin`, `/load`) that any AI tool can read and write to. Your context lives in plain `.md` files you own. The verbs make the rituals consistent across tools.

## What your vault looks like

```
~/vault/
├── README.md
├── .memcrate                  # marker file (lets tools find this vault)
└── Core/
    ├── Context/
    │   ├── Profile.md         # stable: who you are, tools, anti-goals
    │   ├── Projects.md        # every project with status, stack
    │   └── Current State.md   # living: this week's focus, deadlines
    └── Sessions/              # /save writes session logs here
```

Four files inside `Core/`. That's the whole verbs surface — `/save`, `/pin`, and `/load` only read and write within `Core/`. Add optional folders alongside `Core/` (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) whenever you want the personal-OS scope.

Read [docs/overview.md](docs/overview.md) for the full pitch, [docs/verbs.md](docs/verbs.md) for the verb contracts, and [docs/vault-structure.md](docs/vault-structure.md) for the structural details.

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

## Install

One-liner for Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/bradtraversy/memcrate/main/install.sh | sh
```

Drops the `memcrate` binary into `/usr/local/bin` (with `sudo` if needed). Then:

```bash
memcrate init ~/vault
```

You should end up with `~/vault/Core/Context/Profile.md`, `~/vault/Core/Sessions/`, and so on.

### Other install paths

```bash
# From source
cargo install memcrate

# Specific version
MEMCRATE_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/bradtraversy/memcrate/main/install.sh | sh

# Custom install dir (no sudo needed)
MEMCRATE_INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/bradtraversy/memcrate/main/install.sh | sh
```

Pre-built binaries are also available on the [releases page](https://github.com/bradtraversy/memcrate/releases).

### Connecting to your AI tool

The CLI ships `init` today; the per-tool `install` command lands in a later v0.x release. Until then, link the skills manually for Claude Code:

```bash
git clone https://github.com/bradtraversy/memcrate /tmp/memcrate-src
ln -s /tmp/memcrate-src/skills/claude-code/save ~/.claude/skills/save
ln -s /tmp/memcrate-src/skills/claude-code/load ~/.claude/skills/load
ln -s /tmp/memcrate-src/skills/claude-code/pin  ~/.claude/skills/pin
echo 'export MEMCRATE_VAULT_PATH=~/vault' >> ~/.zshrc
```

In a Claude Code session, type `/load` to get oriented, then start working. End the session with `/save`. When something becomes worth remembering forever, `/pin` it.

For other tools (Claude Desktop, Cursor, Aider) — see [docs/skills.md](docs/skills.md) for per-tool install steps.

## About `reference-vault/`

A starter vault scaffold (same shape as the "What your vault looks like" diagram above). Each canonical file has section guidance inline so you know what belongs where. Copy it anywhere, edit by hand, and start running the verbs against it.

## License

- Code (`skills/`, future CLI source, install scripts) — [MIT](LICENSE)
- Format spec (`docs/`) — [CC0](LICENSE-spec). Build a Memcrate-compatible tool without legal friction.

## Credit

The verb trio (`/save`, `/pin`, `/load`) generalizes [EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) — a Claude-Code-only session-memory skill pack. Memcrate scales that pattern to multi-tool personal-context-OS, adds `/pin` for the bridge from session memory to permanent memory, and decouples the format from any one tool. Honest lineage.
