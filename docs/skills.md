---
title: Skills
---

# Skills

The verbs are implemented as skills (or slash-commands, or natural-language patterns) per AI tool. Each integration ships as a small artifact that the tool reads on session start.

The skills are *ergonomic surfaces*, not the system itself. The system is the markdown vault and the verb contracts in [verbs.md](verbs.md). Any tool that can read and write files in a directory can use Memcrate without any of these integrations.

## Claude Code

- **Location:** `~/.claude/skills/<verb>/SKILL.md`
- **Format:** YAML frontmatter (`name`, `description`) + markdown instructions
- **Install:** `memcrate install claude-code` symlinks `~/.claude/skills` to the vault's `Skills/Claude/` directory, or copies the three skill folders if symlink isn't supported.
- **Invocation:** Type `/save`, `/pin`, `/load` in Claude Code.
- **Behavior:** Claude Code auto-discovers skills in `~/.claude/skills/` on session start. No restart required when skills are added or updated (symlink path means vault edits propagate live).

## Claude Desktop (Cowork)

- **Location:** `~/.config/Claude/local-agent-mode-sessions/skills-plugin/<plugin-id>/<install-id>/skills/<verb>/SKILL.md`
- **Install:** Via Claude Desktop UI. The skill is uploaded as a `.skill` zip archive (a zip of the SKILL.md folder) or shared as a `computer://` link to the SKILL.md file. Click "Save Skill" to commit.
- **Caveat:** Claude Desktop keeps an internal cached copy of each registered skill. Direct edits to the on-disk SKILL.md don't take effect — Claude Desktop reads from its internal store. Vault updates require a manual re-register via the UI.
- **CLI integration:** `memcrate install claude-desktop` builds the three `.skill` zips into a known directory and prints the next steps for UI install. The CLI can't programmatically register a skill in Claude Desktop without UI interaction, so its role is staging the zips, not installing them.

## Cursor

- **Location:** `.cursor/rules/` per repo, or `.cursorrules` (legacy single-file format)
- **Approach:** Cursor doesn't expose arbitrary slash commands. The integration writes a `.cursorrules` file (or rule files in `.cursor/rules/`) instructing the agent to read the vault's canonical files at session start and to follow specific patterns when the user types natural-language equivalents:
  - "save this session" / "wrap up" / "compress this" → execute the `/save` workflow
  - "pin this" / "remember this permanently" → execute the `/pin` workflow
  - "load context" / "what are we working on" / "catch me up" → execute the `/load` workflow
- **Install:** `memcrate install cursor` writes the rules file at the repo root (or vault root, depending on your preference).
- **Caveat:** Cursor's rules are read-only context for the agent; the verb behaviors depend on the agent following the rules. Less reliable than slash-command tools where the verb is a hard-coded entry point.

## Aider

- **Location:** `.aider.conf.yml` per repo, or global config at `~/.aider.conf.yml`
- **Approach:** Use Aider's `--read` flag (configured via `read:` in YAML) to auto-load the canonical vault files at session start. Verbs become user-typed natural language similar to the Cursor approach.
- **Install:** `memcrate install aider` appends `read:` entries pointing to the vault's `Profile.md`, `Projects.md`, `Current State.md`, and the most recent N session logs.
- **Caveat:** Aider's session model is code-edit-focused; the `/save` and `/pin` verbs are awkward fits because Aider doesn't naturally write to non-code files. The integration is best-effort — the natural-language patterns work, but expect more friction than Claude Code or Claude Desktop.

## MCP layer (later phase)

For tools with MCP support (Claude Desktop's MCP slot, Cline, and future MCP-aware tools), Memcrate ships an `mcp-memcrate` server exposing tools:

| MCP tool | Maps to | Notes |
|---|---|---|
| `vault_save` | `/save` | Writes a session log; the AI can call this directly at session end without user typing. |
| `vault_pin` | `/pin` | Writes to a canonical file. Same single-write-path discipline as the verb. |
| `vault_load` | `/load` | Returns oriented summary; AI calls at session start. |
| `vault_search` | (no verb equivalent) | Full-text search across vault. Pure read. |

This enables write-back automation — the AI can call `vault_save` directly without the user typing the verb. The MCP layer is *additive*. Markdown remains the source of truth. MCP just removes a step.

## Other agent frameworks

Memcrate's verb contracts are intended to generalize. As long as a tool can:

1. Read files at known vault paths
2. Write files at known vault paths
3. Be told to do those operations on user trigger

…it can support the verbs. The implementation is whatever the tool's skill/rule/config format allows. Reference integrations ship for Claude Code, Claude Desktop, Cursor, and Aider; others are open to community contributions.

## Skill source of truth

The canonical SKILL.md files for each tool live in the public Memcrate repo's `skills/` directory. The CLI bundles them and `memcrate update` pulls latest from the repo. Users with custom modifications opt out of auto-update via a `.memcrate-skills-pinned` flag (a marker file at vault root).
