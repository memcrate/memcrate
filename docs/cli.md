---
title: CLI
---

# CLI

The CLI is Memcrate's install layer - it scaffolds vaults and distributes skills. The CLI is *not* the system; the vault is. The CLI exists to make adopting and maintaining the vault easy.

> **Status:** shipped. `cargo install memcrate` or the install one-liners in the [README](../README.md) get you the current release. The commands below are split into what exists today and what's planned; planned commands are design targets, not promises of syntax.

## Shipped commands

```bash
memcrate init [path]               # Scaffold a vault at path (default: ~/vault)
memcrate setup [path]              # Populate Profile.md and Projects.md from 4 prompts
memcrate install claude-code       # Install /save, /pin, /load skills for Claude Code
```

### `memcrate init [path]`

Scaffolds a new vault.

- Default path: `~/vault`
- Default shape: `Core/` + `README.md` + `.memcrate` marker, nothing else (per [vault-structure.md](vault-structure.md)).
- `--full` flag also scaffolds the optional folders (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) as empty buckets, useful when you know you want the personal-OS scope from day one.
- `--force` flag overwrites an existing vault (or scaffolds into a non-empty directory).
- The `.memcrate` marker file lets the upward-walk vault-resolution rule locate the vault from any subdirectory.
- No prompts: `init` only scaffolds. Seeding content is `setup`'s job.

After `init`, the vault is scaffolded but empty of personal content (no project entries, no sessions). Run `memcrate setup` to seed it, or run `/load` once to confirm the wiring works and write your first `/pin` entries.

### `memcrate setup [path]`

Interactive wizard that populates `Profile.md` and `Projects.md` from four short questions: your name, what you do, tools you always use, and active projects (one per line). Press Enter to skip any question.

- Finds the vault automatically when `path` is omitted. Resolution order: current directory with a `.memcrate` marker, then an upward walk to any parent with a marker, then a single marked vault at depth 1 in your home directory, then `~/vault`.
- Refuses to overwrite hand-edited files; `--force` overrides.
- Only touches `Profile.md` and `Projects.md`. `Current State.md` and `Sessions/` are yours.

### `memcrate install claude-code`

Installs the bundled `/load`, `/save`, and `/pin` SKILL.md files to `~/.claude/skills/`.

- `--target <path>` installs somewhere else.
- `--force` overwrites an existing install. The skill folders are replaced wholesale, so local edits to installed skills are lost; keep custom behavior in your own fork of the skills instead.
- The skills are embedded in the binary at compile time, so `install` works offline.

## Planned commands

These are design targets for future releases. Names and shapes may change.

```bash
memcrate install <tool>            # More tools: claude-desktop, cursor, aider
memcrate install --all             # Install for all detected tools
memcrate update                    # Refresh skills from canonical source
memcrate status                    # Show vault health: structure, skills installed, last save
memcrate doctor                    # Diagnose problems (missing files, stale skills, broken symlinks)
```

### `memcrate install <tool>` (more tools)

- `claude-desktop` - builds `.skill` zips and prints UI install steps. Claude Desktop skills can't be programmatically registered.
- `cursor` - writes `.cursorrules` at the vault root (or repo root if `--repo <path>` is passed).
- `aider` - appends `read:` entries to `~/.aider.conf.yml` for the canonical files.

### `memcrate install --all`

Detects which AI tools are present on the machine (by checking for known config paths, binaries, or environment markers) and runs `install` for each. Reports skipped tools so you know what didn't apply.

### `memcrate update`

Refreshes installed skills from the canonical source (the public Memcrate repo) without recompiling the CLI. Users with local modifications opt out via `.memcrate-skills-pinned` (a marker file at vault root).

`update` does *not* touch your `Profile.md`, `Projects.md`, `Current State.md`, or `Sessions/`. You own all content.

### `memcrate status`

Quick vault health check. Sketch:

```
Vault: ~/vault (full shape)
Last /save: 2026-05-10 14:30 (auth-rewrite-part-1)
Skills installed: claude-code
Profile.md: 247 lines, last_updated 2026-04-22
Projects.md: 312 lines, last_updated 2026-05-08
Current State.md: 198 lines, last_updated 2026-05-10
Sessions: 142 logs (oldest 2026-04-12, most recent 2026-05-10)
```

### `memcrate doctor`

Deeper diagnostic. Checks for:

- Missing canonical files (`Profile.md`, `Projects.md`, `Current State.md`).
- Stale `last_updated` (>90 days on `Profile.md` may be intentional; on `Current State.md` is a smell).
- Skills in `~/.claude/skills/` older than the CLI's bundled versions.
- Sessions folder size (warns if >500 logs without an archive policy).
- Frontmatter parsing errors (file with malformed YAML).
- Sessions older than 6 months without archive structure.

Each check returns OK / WARN / FAIL with a one-line remediation hint.

## Implementation

### Language

**Rust.** Rationale:

- Single static binary, ~5MB, no runtime dependencies.
- Cargo distribution path doubles as the crate-name claim on `crates.io`.
- "Built in Rust" signals craftsmanship to the dev audience this targets.
- Markdown parsing, YAML parsing, file-watching, and HTTP fetching all have mature crates.
- Cross-platform compilation (Linux / macOS / Windows) via GitHub Actions.

Considered and rejected:

- **Go** - equally good distribution, slightly faster to ship, but less brand signal.
- **Node** - lowest barrier to contribute, but startup cost (~100ms+ for a CLI) is rough for a verb users may run dozens of times a day. Also creates a "you need Node installed" gate that the binary distribution avoids.
- **Bash** - initial impulse for a "config installer" CLI. Fails as soon as we want JSON/YAML parsing or cross-platform installer behavior.

### Distribution

Shipped today:

- **Cargo:** `cargo install memcrate` - for anyone with a Rust toolchain (also the path for Intel Macs).
- **curl-pipe-sh (Linux / Apple Silicon macOS):** `curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh`
- **PowerShell (Windows):** `irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex`
- **GitHub Releases:** prebuilt binaries for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, each with a `.sha256` alongside. Tag push builds and publishes automatically (GitHub Releases + crates.io).

Planned:

- **Homebrew:** `brew install memcrate` - primary path for Mac users once a tap or core formula lands.
- **Scoop / WinGet:** for Windows. Lower priority; ship if there's demand.

### Source of truth for skills

The CLI bundles the canonical SKILL.md files into the binary at compile time, so `install` works offline and the installed skills always match the CLI version. The planned `memcrate update` will fetch the latest skill files from the public repo (HTTPS, no auth) without requiring a CLI upgrade.

### Auto-update vs. manual

The CLI does not auto-update itself. Upgrade via your package manager (or re-run the install one-liner) on your own cadence. The dev tool aesthetic is "the CLI does what you tell it, nothing else."

A `--check-update` flag on the planned `memcrate status` will report whether a newer skill or CLI version is available. No action taken without explicit command.

## What the CLI explicitly doesn't do

- **No telemetry.** No usage analytics, no error reporting back to anyone, no version pings.
- **No cloud account.** The CLI works fully offline once installed.
- **No vault hosting.** The vault is a local directory; sync is your call (Obsidian Sync, iCloud, git, etc.). Memcrate doesn't host anything.
- **No ongoing writes to canonical files.** `setup` seeds `Profile.md` and `Projects.md` once, from the pristine scaffold (and refuses to touch hand-edited files without `--force`). After that, changes go through `/pin` or your own edits; the CLI steps back.
- **No skill execution.** The CLI installs skills; the AI tool runs them. The CLI is plumbing, not runtime.
