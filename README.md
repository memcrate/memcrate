# Memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One vault. Any tool.

[![Crates.io](https://img.shields.io/crates/v/memcrate.svg)](https://crates.io/crates/memcrate) [![Downloads](https://img.shields.io/crates/d/memcrate.svg)](https://crates.io/crates/memcrate) [![Code: MIT](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE) [![Spec: CC0](https://img.shields.io/badge/spec-CC0-lightgrey.svg)](LICENSE-spec)

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
│   └── agent/          # /save, /load, /pin as SKILL.md (Claude Code + Codex)
└── docs/               # Format spec: overview, verbs, vault structure, skills, CLI
```

## Getting started

A full first-time setup is five steps.

### 1. Install the CLI

Pick one path:

**Linux / macOS (Apple Silicon)** — curl one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Drops the `memcrate` binary into `/usr/local/bin` (uses `sudo` if needed).

**Windows** — PowerShell one-liner:

```powershell
irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex
```

Drops `memcrate.exe` into `%LOCALAPPDATA%\Programs\memcrate\` and adds it to your user PATH.

**npm** (downloads the same prebuilt binary, no dependencies):

```bash
npm install -g memcrate
```

**Any platform with Rust** (also the path for Intel Macs):

```bash
cargo install memcrate
```

Builds from source via crates.io. Requires a Rust toolchain (`rustup`).

Verify:

```bash
memcrate --version
```

### 2. Scaffold your vault

```bash
memcrate init ~/vault
```

You can use any path — `~/vault` is just the default convention. The command creates `~/vault/Core/Context/{Profile,Projects,Current State}.md` and `~/vault/Core/Sessions/` for session logs, plus a `.memcrate` marker file so tools can find the vault from any subdirectory.

### 3. Populate your vault (optional but recommended)

```bash
memcrate setup
```

Asks four short questions — your name, what you do, tools you always use, active projects (one per line, blank to finish) — and writes the answers into `Profile.md` and `Projects.md`. Day-one `/load` then has real context to read instead of an empty scaffold.

`setup` finds your vault automatically. Resolution order:

1. Explicit path: `memcrate setup /path/to/vault`
2. Current working directory if it has a `.memcrate` marker
3. Any parent directory with a `.memcrate` marker (git-style upward walk)
4. A single vault in `$HOME` (scanned at depth 1)
5. `~/vault` as a final fallback

Skip this step if you'd rather populate everything via `/pin` once you're in an AI tool. You can also hand-edit `Profile.md` and `Projects.md` directly — they include section guidance inline — but you shouldn't need to.

### 4. Install skills for your AI tool

```bash
memcrate install
```

Asks which tool you want, then drops `/load`, `/save`, and `/pin` into place. Claude Code and Codex both read the same `SKILL.md` format, so the same three skills work in either.

```bash
memcrate install claude-code   # ~/.claude/skills/
memcrate install codex         # ~/.codex/skills/
memcrate install all           # both, no prompt
```

Pass `--target <path>` to install one tool somewhere else, or `--force` to update an existing Memcrate install. `--force` only ever replaces skills Memcrate itself installed. If you already have your own skill named `load`, `save`, or `pin`, it stops and tells you rather than overwriting your work.

Other tools (Claude Desktop, Cursor, Aider) are coming in later v0.x releases; see [docs/skills.md](docs/skills.md) for the planned shape.

### 5. Use the verbs

Start an AI tool session in or near your vault, then:

- **`/load`** — read your vault and reconstruct context. Run this first in any new session.
- **`/pin <insight>`** — promote a fact into `Profile.md`, `Projects.md`, or `Current State.md` so it survives across sessions.
- **`/save`** — write a session log to `Core/Sessions/` when you're done so the next `/load` can pick up where this one left off.

The skills look for your vault by reading `~/vault/Core/Context/Profile.md` first, then the current working directory, then a depth-1 scan of your home directory (so custom paths like `~/myvault` are found automatically). If nothing matches, `/load` asks you in plain English where your vault is on first use.

## Advanced install options

```bash
# Specific version (Linux / macOS)
MEMCRATE_VERSION=v0.3.1 curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh

# Specific version (Windows)
$env:MEMCRATE_VERSION="v0.3.1"; irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex

# Custom install dir on Linux / macOS (no sudo needed)
MEMCRATE_INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Pre-built binaries on the [releases page](https://github.com/memcrate/memcrate/releases): Linux x86_64, macOS Apple Silicon, Windows x86_64. **Intel Macs:** install via `cargo install memcrate` (GitHub retired its free Intel macOS runner image in early 2026, so we no longer ship a pre-built Intel binary).

## About `reference-vault/`

A starter vault scaffold (same shape as the "What your vault looks like" diagram above), shipped inside the CLI binary and extracted by `memcrate init`. Each canonical file has section guidance inline so AI tools know what belongs where when `/pin` writes to it. You can also copy `reference-vault/` directly into any directory if you'd rather skip the CLI.

## Development

`sh scripts/verify.sh` is the quality gate: leak check, fmt check, clippy (warnings deny), tests, build. The same script runs as a pre-push hook (wire it once per clone with `git config core.hooksPath .githooks`) and in CI on every push and pull request.

The leak check exists because this repo's reference vault and skills are generalized from a real private vault. It blocks a push when a private project name, hostname, or absolute home path reaches a tracked file. Terms are read from the maintainer's local vault at runtime and are never committed here, so the check is a silent no-op on CI and on any other contributor's machine.

## License

- Code (`skills/`, future CLI source, install scripts) — [MIT](LICENSE)
- Format spec (`docs/`) — [CC0](LICENSE-spec). Build a Memcrate-compatible tool without legal friction.

## Credit

The verb trio (`/save`, `/pin`, `/load`) generalizes [EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) — a Claude-Code-only session-memory skill pack. Memcrate scales that pattern to multi-tool personal-context-OS, adds `/pin` for the bridge from session memory to permanent memory, and decouples the format from any one tool. Honest lineage.
