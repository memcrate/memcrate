---
title: Overview
---

# Overview

## The problem

Every AI coding tool eventually loses context. Sessions end. Tools change. You start fresh in Cursor on Monday, then jump to Claude Code on Tuesday, then a Claude Desktop window for planning on Wednesday — and each one needs to be told from scratch what you're working on, what's been decided, what's broken, and what's next.

Existing solutions cover slices, not the whole picture:

- **Static project rules** (`CLAUDE.md`, `AGENTS.md`, `.cursorrules`) capture *project facts* but not *session state*. They don't change between sessions.
- **Managed AI memory** (mem0, Letta, supermemory, built-in tool memory) is cloud-backed, vendor-locked, and not repo-scoped. You don't own your context.
- **Session-memory skill packs** like [CPR](https://github.com/EliaAlberti/cpr-compress-preserve-resume) save and restore session context for one tool. Useful, but they don't span tools and don't carry full personal context (projects, decisions, week-to-week state).
- **MCP memory servers** are binary and AI-only — humans can't read them; only MCP-aware clients can.

The gap: a portable, markdown-native, locally-owned **personal context vault** that any AI tool can read and write to, plus a small set of operations that make it easy to maintain.

## The solution

A markdown directory + three verbs + per-tool skill packs.

**Philosophy:**

1. **Markdown-first.** Every artifact is a `.md` file you can read in any editor. The vault works without any tool installed.
2. **Local-first.** It lives on your filesystem. Sync via Obsidian Sync, iCloud, Dropbox, git — your call. No cloud account required.
3. **Tool-agnostic.** Any AI assistant that reads files in a directory can use the vault. Skill packs make it ergonomic for specific tools (Claude Code, Claude Desktop, Cursor, Aider) but they're optional.
4. **Human and AI both first-class.** The vault is your second brain *and* your AI's continuity layer. Same files serve both.
5. **Vault = source of truth.** CLI, skills, MCP — all interfaces *to* the vault. None of them own the data.

**Three core operations** that every supported tool exposes:

| Verb | When | What it does |
|---|---|---|
| `/save` | At session end | Writes a structured session log capturing decisions, files touched, pending actions, outcome. Optional `<label>` arg scopes the log to a project or topic. |
| `/pin` | When an insight emerges mid-session | Promotes a single fact, decision, or learning from session memory to permanent context (Profile / Projects / Current State, depending on type). |
| `/load` | At session start | Reads canonical context files + recent saves and gives a short oriented summary. Optional `<label>` arg switches to scoped mode. |

The verbs are *conveniences* — you can manually edit any vault file at any time. The verbs make the rituals consistent across tools. See [verbs.md](verbs.md) for the full spec on each.

## Differentiation

| | CPR | mem0 / Letta / supermemory | Static `CLAUDE.md` / `AGENTS.md` | MCP-only memory servers | **Memcrate** |
|---|---|---|---|---|---|
| Storage | `CC-Session-Logs/` per project | Cloud, vendor DB | Single static file | Server-managed | Local markdown vault |
| Scope | Session memory | Conversation memory | Project rules | Conversation memory | Personal context (projects, sessions, decisions, weekly state) |
| Tools | Claude Code | Multi-tool via SDK/API | Tool-specific | MCP-aware tools | Any tool that reads files; ergonomic skills for major tools |
| Human-readable | Yes (markdown) | API / dashboard | Yes | No (binary) | Yes (markdown, native to any editor) |
| Vendor lock-in | None | Yes (cloud) | None | None | None |
| Philosophy | One-purpose pack | Memory-as-a-service | Static prompt | Live API | Vault as personal OS |

The positioning: **Memcrate is a personal context OS, not a memory tool.** Memory is one feature; the vault also holds your project catalog, daily state, decisions, and infrastructure. Future capabilities (morning briefing, scheduled tasks, project state buckets, etc.) all draw from the same substrate without changing the verbs.

## Prior art and credit

[EliaAlberti/cpr-compress-preserve-resume](https://github.com/EliaAlberti/cpr-compress-preserve-resume) is the verb-trio inspiration. CPR is a Claude Code session-memory skill pack — focused, useful, and the conceptual ancestor of Memcrate's `/save` and `/load`. Memcrate scales that pattern from session-memory-for-one-tool to personal-context-OS-across-tools, adds `/pin` for the session-memory-to-permanent-memory bridge, and decouples the format from any one tool.
