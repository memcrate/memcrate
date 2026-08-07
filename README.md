# Memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One vault. Any tool.

[![Crates.io](https://img.shields.io/crates/v/memcrate.svg)](https://crates.io/crates/memcrate) [![Downloads](https://img.shields.io/crates/d/memcrate.svg)](https://crates.io/crates/memcrate) [![Code: MIT](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE) [![Spec: CC0](https://img.shields.io/badge/spec-CC0-lightgrey.svg)](LICENSE-spec)

## What this is

Every AI coding tool eventually loses context. Sessions end. Tools change. You start fresh in Cursor on Monday, then jump to Claude Code on Tuesday, then a Claude Desktop window for planning on Wednesday - and each one needs to be told from scratch what you're working on, what's been decided, what's broken, what's next.

Memcrate is a local markdown vault plus three verbs (`/save`, `/pin`, `/load`) that any AI tool can read and write to. Your context lives in plain `.md` files you own. The verbs make the rituals consistent across tools.

## What your vault looks like

```
~/memcrate-vault/
├── README.md
├── .memcrate                  # marker file (lets tools find this vault)
└── Core/
    ├── Context/
    │   ├── Profile.md         # stable: who you are, tools, anti-goals
    │   ├── Projects.md        # every project with status, stack
    │   └── Current State.md   # living: this week's focus, deadlines
    └── Sessions/              # /save writes session logs here
```

Four files inside `Core/`. That's the whole verbs surface - `/save`, `/pin`, and `/load` only read and write within `Core/`. Add optional folders alongside `Core/` (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) whenever you want the personal-OS scope.

Read [docs/overview.md](docs/overview.md) for the full pitch, [docs/verbs.md](docs/verbs.md) for the verb contracts, and [docs/vault-structure.md](docs/vault-structure.md) for the structural details.

## Why not [other thing]

- **Static `CLAUDE.md` / `AGENTS.md` / `.cursorrules`** capture project facts but not session state.
- **Cloud-backed AI memory** (mem0, Letta, supermemory) is vendor-locked and not repo-scoped.
- **Session-memory packs** (CPR - the verb-trio inspiration here) work great for one tool but don't span tools or carry full personal context.
- **MCP memory servers** are binary; humans can't read them.

Memcrate is a personal context OS, not a memory tool. Your project catalog, daily state, decisions, and infrastructure all draw from the same vault.

## Repo layout

```
memcrate/
├── reference-vault/    # Starter vault scaffold (Core/ + .memcrate marker) - copy anywhere
├── skills/             # Canonical SKILL.md files for each AI tool
│   └── agent/          # /save, /load, /pin as SKILL.md (Claude Code + Codex)
└── docs/               # Format spec: overview, verbs, vault structure, skills, CLI
```

## Getting started

Install the CLI, then run `memcrate`. One question, then it does the rest.

```
$ memcrate

Where should your vault live? [~/memcrate-vault]:
>

Created your vault at ~/memcrate-vault.

Installed 3 skills for Claude Code to ~/.claude/skills
Installed 3 skills for Codex to ~/.codex/skills

You now have three verbs in Claude Code and Codex:
  /load   read your vault and get oriented. Run this first.
  /pin    promote a fact into your permanent context files.
  /save   write a session log before you finish.
```

Re-running is safe. It reuses an existing vault and refreshes the skills.

Optionally, tell your tools who you are:

```bash
memcrate profile
```

Four questions (name, what you build, tools, projects) that fill in `Profile.md` and `Projects.md` so your first `/load` has something to read. Everything works without it; you can also just edit those two files by hand, or let `/pin` fill them in as you work.

### Install the CLI

Pick one path:

**Linux / macOS (Apple Silicon)** - curl one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Drops the `memcrate` binary into `/usr/local/bin` (uses `sudo` if needed).

**Windows** - PowerShell one-liner:

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

### Options

```bash
memcrate --vault ~/notes    # skip the location prompt
memcrate --yes              # take every default, ask nothing (implied without a terminal)
memcrate --full             # also create Projects/, Daily/, Tasks/, Inbox/
```

`--yes` plus `--vault` is the scripted form for dotfiles and container images.

### Use the verbs

Start Claude Code or Codex, then:

- **`/load`** reads your vault and reconstructs context. Run it first in any new session.
- **`/pin <insight>`** promotes a fact into `Profile.md`, `Projects.md`, or `Current State.md` so it survives across sessions.
- **`/save`** writes a session log to `Core/Sessions/` so the next `/load` picks up where you left off.

The skills find your vault by reading `~/memcrate-vault/Core/Context/Profile.md`, then the current directory, then a depth-1 scan of your home directory. If nothing matches, `/load` asks you where it is.

### Skills you already own

Memcrate never replaces a skill it did not install. If you already have your own `load`, `save`, or `pin`, it sets up everything else and tells you which one it skipped.

## Advanced install options

```bash
# Specific version (Linux / macOS)
MEMCRATE_VERSION=v0.4.0 curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh

# Specific version (Windows)
$env:MEMCRATE_VERSION="v0.4.0"; irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex

# Custom install dir on Linux / macOS (no sudo needed)
MEMCRATE_INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh
```

Pre-built binaries on the [releases page](https://github.com/memcrate/memcrate/releases): Linux x86_64, macOS Apple Silicon, Windows x86_64. **Intel Macs:** install via `cargo install memcrate` (GitHub retired its free Intel macOS runner image in early 2026, so we no longer ship a pre-built Intel binary).

## About `reference-vault/`

A starter vault scaffold (same shape as the "What your vault looks like" diagram above), shipped inside the CLI binary and extracted when you run `memcrate`. Each canonical file has section guidance inline so AI tools know what belongs where when `/pin` writes to it. You can also copy `reference-vault/` directly into any directory if you'd rather skip the CLI.

## Development

`sh scripts/verify.sh` is the quality gate: leak check, fmt check, clippy (warnings deny), tests, build. The same script runs as a pre-push hook (wire it once per clone with `git config core.hooksPath .githooks`) and in CI on every push and pull request.

The leak check exists because this repo's reference vault and skills are generalized from a real private vault. It blocks a push when a private project name, hostname, or absolute home path reaches a tracked file. Terms are read from the maintainer's local vault at runtime and are never committed here, so the check is a silent no-op on CI and on any other contributor's machine.

## License

- Code (`skills/`, future CLI source, install scripts) - [MIT](LICENSE)
- Format spec (`docs/`) - [CC0](LICENSE-spec). Build a Memcrate-compatible tool without legal friction.

## Credit

The verb trio (`/save`, `/pin`, `/load`) generalizes [EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) - a Claude-Code-only session-memory skill pack. Memcrate scales that pattern to multi-tool personal-context-OS, adds `/pin` for the bridge from session memory to permanent memory, and decouples the format from any one tool. Honest lineage.
