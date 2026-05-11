---
title: CLI
---

# CLI

The CLI is Memcrate's install layer — it scaffolds vaults and distributes skills. The CLI is *not* the system; the vault is. The CLI exists to make adopting and maintaining the vault easy.

> **Status:** the CLI lands in Phase 2. Phase 1 is manual install (clone the repo, symlink the skill folder, copy the reference vault). This page is the design target.

## Commands

```bash
memcrate init [path]               # Scaffold a vault at path (default: ~/vault)
memcrate install <tool>            # Install skills for a tool (claude-code, claude-desktop, cursor, aider)
memcrate install --all             # Install for all detected tools
memcrate update                    # Refresh skills from canonical source
memcrate status                    # Show vault health: structure, skills installed, last save
memcrate doctor                    # Diagnose problems (missing files, stale skills, broken symlinks)
```

### `memcrate init [path]`

Scaffolds a new vault.

- Default path: `~/vault`
- Default shape: `Core/` + `README.md` + `.memcrate` marker — nothing else (per [vault-structure.md](vault-structure.md)).
- `--full` flag also scaffolds the optional folders (`Projects/`, `Daily/`, `Tasks/`, `Inbox/`) as empty buckets, useful when you know you want the personal-OS scope from day one.
- The `.memcrate` marker file lets the upward-walk vault-resolution rule locate the vault from any subdirectory.
- Prompts for: vault name, primary tool (claude-code / claude-desktop / cursor / aider / none), sync recommendation (Obsidian Sync / iCloud / git / skip).
- `--no-prompt` flag accepts all defaults for scripted use.

After `init`, the vault is empty (no project entries, no sessions). Run `/load` once to confirm the wiring works, then write your first `/pin` entries.

### `memcrate install <tool>`

Installs the skill bundle for a single tool.

- `claude-code` — symlinks `~/.claude/skills` to the vault's `Skills/Claude/` directory. Prefers symlink, falls back to copy if the OS doesn't support it (some Windows configs).
- `claude-desktop` — builds `.skill` zips into the vault's `Skills/ClaudeDesktop/dist/` directory and prints UI install steps. Claude Desktop skills can't be programmatically registered.
- `cursor` — writes `.cursorrules` at the vault root (or repo root if `--repo <path>` is passed).
- `aider` — appends `read:` entries to `~/.aider.conf.yml` for the canonical files.

Re-running `install <tool>` is idempotent — existing skills are updated in place, not duplicated.

### `memcrate install --all`

Detects which AI tools are present on the machine (by checking for known config paths, binaries, or environment markers) and runs `install` for each. Reports skipped tools so you know what didn't apply.

### `memcrate update`

Refreshes skills from the canonical source (the public Memcrate repo). Replaces the vault's `Skills/` directory contents with the latest. Users with local modifications opt out of auto-update via `.memcrate-skills-pinned` (a marker file at vault root).

`update` does *not* touch your `Profile.md`, `Projects.md`, `Current State.md`, or `Sessions/`. You own all content.

### `memcrate status`

Quick vault health check. Output:

```
Vault: ~/vault (full shape)
Last /save: 2026-05-10 14:30 (auth-rewrite-part-1)
Skills installed: claude-code (symlinked), claude-desktop (dist staged, not registered)
Profile.md: 247 lines, last_updated 2026-04-22
Projects.md: 312 lines, last_updated 2026-05-08
Current State.md: 198 lines, last_updated 2026-05-10
Sessions: 142 logs (oldest 2026-04-12, most recent 2026-05-10)
```

### `memcrate doctor`

Deeper diagnostic. Checks for:

- Missing canonical files (`Profile.md`, `Projects.md`, `Current State.md`).
- Stale `last_updated` (>90 days on `Profile.md` may be intentional; on `Current State.md` is a smell).
- Broken symlinks in `~/.claude/skills/`.
- Claude Desktop dist zips older than the canonical SKILL.md sources (prompt to rebuild).
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
- Cross-platform compilation (Linux / macOS / Windows) via `cross` or GitHub Actions.

Considered and rejected:

- **Go** — equally good distribution, slightly faster to ship, but less brand signal.
- **Node** — lowest barrier to contribute, but startup cost (~100ms+ for a CLI) is rough for a verb users may run dozens of times a day. Also creates a "you need Node installed" gate that the binary distribution avoids.
- **Bash** — initial impulse for a "config installer" CLI. Fails as soon as we want JSON/YAML parsing or cross-platform installer behavior.

### Distribution

- **Cargo:** `cargo install memcrate` — for Rust devs.
- **Homebrew:** `brew install memcrate` — primary path for Mac users.
- **curl-pipe-bash:** `curl -sSL memcrate.dev/install | sh` — primary path for Linux users and the docs-site headline install line.
- **GitHub Releases:** prebuilt binaries for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. Linked from the docs site.
- **Scoop / WinGet:** for Windows. Lower priority; ship after Phase 2 if there's demand.

### Source of truth for skills

The CLI bundles the canonical SKILL.md files into the binary at compile time. `memcrate update` fetches the latest skill files from the public repo (HTTPS, no auth) and replaces the vault's local copies.

This means:

- Users get fresh skills via `update` without recompiling the CLI.
- The CLI's bundled skills are the *fallback* when offline or when the repo is unreachable.
- Pin via `.memcrate-skills-pinned` opts out of `update` entirely for users running custom skill modifications.

### Auto-update vs. manual

The CLI does not auto-update itself. Run `memcrate update` (skills) or upgrade via your package manager (CLI binary) on your own cadence. The dev tool aesthetic is "the CLI does what you tell it, nothing else."

A `--check-update` flag on `memcrate status` will report whether a newer skill or CLI version is available. No action taken without explicit command.

## What the CLI explicitly doesn't do

- **No telemetry.** No usage analytics, no error reporting back to anyone, no version pings.
- **No cloud account.** The CLI works fully offline once installed.
- **No vault hosting.** The vault is a local directory; sync is your call (Obsidian Sync, iCloud, git, etc.). Memcrate doesn't host anything.
- **No write to canonical files.** The CLI never edits `Profile.md` / `Projects.md` / `Current State.md` directly. Those changes go through `/pin` (or your own edits). The CLI scaffolds them, then steps back.
- **No skill execution.** The CLI installs skills; the AI tool runs them. The CLI is plumbing, not runtime.
