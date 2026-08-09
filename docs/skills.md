---
title: Skills
---

# Skills

The verbs are implemented as skills (or slash-commands, or natural-language patterns) per AI tool. Each integration ships as a small artifact that the tool reads on session start.

The skills are *ergonomic surfaces*, not the system itself. The system is the markdown vault and the verb contracts in [verbs.md](verbs.md). Any tool that can read and write files in a directory can use Memcrate without any of these integrations.

## Claude Code

- **Location:** `~/.claude/skills/<verb>/SKILL.md`
- **Format:** YAML frontmatter (`name`, `description`) + markdown instructions
- **Install:** `memcrate` writes the three skill folders, each embedded in the CLI binary at compile time (so it works offline).
- **Invocation:** Type `/save`, `/pin`, `/load` in Claude Code.
- **Behavior:** Claude Code auto-discovers skills in `~/.claude/skills/` on session start.
- **Desktop too:** Claude Desktop's local agent mode (Cowork) runs the same Claude Code binary against the same `~/.claude/skills/` directory, so one `memcrate` run covers both. See [Claude Desktop](#claude-desktop-cowork) below.

## Codex

- **Location:** `~/.codex/skills/<verb>/SKILL.md`
- **Format:** identical to Claude Code. Codex reads the same `SKILL.md` shape, so Memcrate ships one canonical set of skills from `skills/agent/` and installs it to whichever tools you pick.
- **Install:** `memcrate`, which installs for both tools at once.
- **Invocation:** Type `$save`, `$pin`, `$load` in Codex. Codex invokes skills with `$`, not `/`.
- **Note:** the vault-discovery step names a concrete call rather than an abstract goal, because an explicit tool call is followed more reliably. It branches on capability (Glob if the tool has it, `ls -d` otherwise) rather than on tool name, so an agent that doesn't recognize itself by name still picks the right call.

### Skill ownership

Installed skills get a `.memcrate-skill` marker file. Re-running `memcrate` replaces only marked skills, so a skill you wrote yourself named `load`, `save`, or `pin` is never overwritten. Skills installed before the marker existed are still recognized by their content.

## Claude Desktop (Cowork)

- **Location:** `~/.claude/skills/<verb>/SKILL.md` - the same directory Claude Code uses.
- **Install:** nothing extra. Claude Desktop's local agent mode runs the Claude Code binary with user-level settings enabled, so it reads `~/.claude/skills/` directly. The `memcrate` run that set up Claude Code already set up Cowork.
- **Invocation:** Type `/save`, `/pin`, `/load`, same as Claude Code.

### Skills synced from your Claude account

Claude Desktop also shows skills you uploaded through the Claude web UI. Those are a **separate surface**: they live in your account, sync down into a session-scoped cache under `~/.config/Claude/local-agent-mode-sessions/skills-plugin/`, and that cache is regenerated from the server. Writing files into it does nothing.

You only need this route if you want the verbs in Claude web sessions, which have no access to your local vault and so can't run them meaningfully. For local work, the filesystem install above is the whole story.

There is no programmatic path to register an account-level skill. The `/v1/skills` endpoint on the Claude Developer Platform is org-scoped and API-key authenticated; it does not touch personal account skills. Uploading a `.skill` zip through the UI is the only way, which is why Memcrate doesn't try to automate it.

## Cursor

- **Location:** `.cursor/rules/` per repo, or `.cursorrules` (legacy single-file format)
- **Approach:** Cursor doesn't expose arbitrary slash commands. The integration writes a `.cursorrules` file (or rule files in `.cursor/rules/`) instructing the agent to read the vault's canonical files at session start and to follow specific patterns when the user types natural-language equivalents:
  - "save this session" / "wrap up" / "compress this" → execute the `/save` workflow
  - "pin this" / "remember this permanently" → execute the `/pin` workflow
  - "load context" / "what are we working on" / "catch me up" → execute the `/load` workflow
- **Install:** Memcrate would write the rules file at the repo root (or vault root, depending on your preference).
- **Caveat:** Cursor's rules are read-only context for the agent; the verb behaviors depend on the agent following the rules. Less reliable than slash-command tools where the verb is a hard-coded entry point.

## Aider

- **Location:** `.aider.conf.yml` per repo, or global config at `~/.aider.conf.yml`
- **Approach:** Use Aider's `--read` flag (configured via `read:` in YAML) to auto-load the canonical vault files at session start. Verbs become user-typed natural language similar to the Cursor approach.
- **Install:** Memcrate would append `read:` entries pointing to the vault's `Profile.md`, `Projects.md`, `Current State.md`, and the most recent N session logs.
- **Caveat:** Aider's session model is code-edit-focused; the `/save` and `/pin` verbs are awkward fits because Aider doesn't naturally write to non-code files. The integration is best-effort - the natural-language patterns work, but expect more friction than Claude Code or Claude Desktop.

## MCP layer (later phase)

For tools with MCP support (Claude Desktop's MCP slot, Cline, and future MCP-aware tools), Memcrate ships an `mcp-memcrate` server exposing tools:

| MCP tool | Maps to | Notes |
|---|---|---|
| `vault_save` | `/save` | Writes a session log; the AI can call this directly at session end without user typing. |
| `vault_pin` | `/pin` | Writes to a canonical file. Same single-write-path discipline as the verb. |
| `vault_load` | `/load` | Returns oriented summary; AI calls at session start. |
| `vault_search` | (no verb equivalent) | Full-text search across vault. Pure read. |

This enables write-back automation - the AI can call `vault_save` directly without the user typing the verb. The MCP layer is *additive*. Markdown remains the source of truth. MCP just removes a step.

## Other agent frameworks

Memcrate's verb contracts are intended to generalize. As long as a tool can:

1. Read files at known vault paths
2. Write files at known vault paths
3. Be told to do those operations on user trigger

…it can support the verbs. The implementation is whatever the tool's skill/rule/config format allows. Reference integrations ship today for Claude Code, Claude Desktop, and Codex; Cursor and Aider are designed but not built. Others are open to community contributions.

## Skill source of truth

The canonical SKILL.md files for each tool live in the public Memcrate repo's `skills/` directory. The CLI bundles them at compile time; a future `update` command would pull the latest from the repo. Users with custom modifications opt out of auto-update via a `.memcrate-skills-pinned` flag (a marker file at vault root).
