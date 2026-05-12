# Memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One vault. Any tool.

**Status:** Phase 2 in progress. CLI v0.2 ships `memcrate init` and `memcrate install` (more commands — `update`, `status`, `doctor` — coming in subsequent v0.x releases). MCP server (Phase 3) comes later. Not yet announced — being dogfooded by the author.

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
│   └── claude-code/    # /save, /load, /pin (installed via `memcrate install claude-code`)
└── docs/               # Format spec: overview, verbs, vault structure, skills, CLI
```

## Install

**Linux and macOS** (one-liner):

```bash
curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Drops the `memcrate` binary into `/usr/local/bin` (with `sudo` if needed).

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex
```

Drops `memcrate.exe` into `%LOCALAPPDATA%\Programs\memcrate\` and adds it to your user PATH.

Then on any platform:

```bash
memcrate init ~/vault          # scaffold the vault
memcrate setup                 # optional: seed Profile.md + Projects.md from a few prompts
```

You should end up with `~/vault/Core/Context/Profile.md`, `~/vault/Core/Sessions/`, and so on. The optional `setup` wizard asks you four questions (your name, what you do, tools you use, active projects) and writes the answers into the canonical files so `/load` has real context on day one. Skip it if you'd rather populate everything via `/pin` once you're in an AI tool.

### Other install paths

```bash
# From source (also the path for Intel Macs — see note below)
cargo install memcrate

# Specific version (Linux/macOS)
MEMCRATE_VERSION=v0.2.0 curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh

# Specific version (Windows)
$env:MEMCRATE_VERSION="v0.2.0"; irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex

# Custom install dir (no sudo needed)
MEMCRATE_INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Pre-built binaries on the [releases page](https://github.com/memcrate/memcrate/releases): Linux x86_64, macOS Apple Silicon, Windows x86_64. **Intel Macs:** install via `cargo install memcrate` (GitHub retired its free Intel macOS runner image in early 2026, so we no longer ship a pre-built Intel binary).

### Connecting to your AI tool

Install the skills for Claude Code:

```bash
memcrate install claude-code
```

That drops `/save`, `/load`, `/pin` into `~/.claude/skills/`. Pass `--target <path>` to install elsewhere, or `--force` to overwrite an existing install.

The skills look for your vault at `~/vault` by default. If your vault lives elsewhere, `cd` into it before launching Claude Code — the skills also check the current working directory. If neither matches, `/load` asks you in plain English where your vault is on first use.

In a Claude Code session, type `/load` to get oriented, then start working. End the session with `/save`. When something becomes worth remembering forever, `/pin` it.

Other tools (Claude Desktop, Cursor, Aider) — coming in later v0.x releases; see [docs/skills.md](docs/skills.md) for the planned shape.

## About `reference-vault/`

A starter vault scaffold (same shape as the "What your vault looks like" diagram above). Each canonical file has section guidance inline so you know what belongs where. Copy it anywhere, edit by hand, and start running the verbs against it.

## License

- Code (`skills/`, future CLI source, install scripts) — [MIT](LICENSE)
- Format spec (`docs/`) — [CC0](LICENSE-spec). Build a Memcrate-compatible tool without legal friction.

## Credit

The verb trio (`/save`, `/pin`, `/load`) generalizes [EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) — a Claude-Code-only session-memory skill pack. Memcrate scales that pattern to multi-tool personal-context-OS, adds `/pin` for the bridge from session memory to permanent memory, and decouples the format from any one tool. Honest lineage.
