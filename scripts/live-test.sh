#!/usr/bin/env bash
# Live agent test: do load, save, and pin work in a real agent session?
#
# Everything else in this repo checks files on disk. This installs memcrate
# into a scratch home, then runs headless agent sessions against one vault.
# Each step must show that the installed skill ran, and is then checked by a
# canary or a file on disk, never by wording:
#
#   load   a fact that exists only in the vault comes back in the final answer
#   save   exactly one new session log appears, carrying a fact from the prompt
#   pin    a fact from the prompt lands in a context file and bumps last_updated
#
# It also fails if anything outside the vault was created or removed, apart
# from the agent CLI's own state.
#
#   scripts/live-test.sh codex             test Codex (needs OPENAI_API_KEY)
#   scripts/live-test.sh claude            test Claude Code (needs ANTHROPIC_API_KEY)
#   scripts/live-test.sh claude prefixed   a foreign save skill is already there,
#                                          so check the memcrate- names and load
#
# LIVE_TEST_CLAUDE_MODEL, when set, is passed to Claude Code as --model.
#
# Not part of verify.sh or the pre-push hook on purpose: it costs money, needs
# network, and an agent's wording varies between runs. Run it before tagging a
# release, or from the live-test GitHub workflow.
set -euo pipefail

TOOL="${1:-codex}"
MODE="${2:-plain}"
LOAD_CANARY="PERSIMMON-7F3A"
SAVE_CANARY="QUINCE-42B1"
PIN_CANARY="TANGELO-5D7C"
# The save skill skips sessions with nothing worth keeping, so this one has a decision, a reason, and a next step.
SAVE_SESSION="We decided to codename the next release $SAVE_CANARY, because the old name clashed with a customer's product. Next step: update the changelog before Friday."
REPO="$(cd "$(dirname "$0")/.." && pwd)"

case "$TOOL" in
    codex | claude) ;;
    *)
        echo "unknown tool '$TOOL'. Use: codex | claude" >&2
        exit 2
        ;;
esac
case "$MODE" in
    plain | prefixed) ;;
    *)
        echo "unknown mode '$MODE'. Use: plain | prefixed" >&2
        exit 2
        ;;
esac

command -v "$TOOL" >/dev/null || {
    echo "$TOOL is not on PATH" >&2
    exit 2
}

# Fail before the expensive work rather than after it.
if [ "$TOOL" = codex ] && [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "OPENAI_API_KEY is unset. Codex needs it to run non-interactively." >&2
    exit 2
fi
if [ "$TOOL" = claude ] && [ -z "${ANTHROPIC_API_KEY:-}" ]; then
    echo "ANTHROPIC_API_KEY is unset. Create one at console.anthropic.com." >&2
    echo "Claude Code runs with --bare here, which never reads an interactive login." >&2
    exit 2
fi

# Each agent sees only its own key.
OTHER_KEY=ANTHROPIC_API_KEY
if [ "$TOOL" = claude ]; then OTHER_KEY=OPENAI_API_KEY; fi

# Git Bash on Windows: native programs need Windows paths and the .exe name.
WINDOWS=0
case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) WINDOWS=1 ;;
esac
native() {
    if [ "$WINDOWS" = 1 ]; then cygpath -w "$1"; else printf '%s\n' "$1"; fi
}
BIN="$REPO/target/debug/memcrate"
if [ "$WINDOWS" = 1 ]; then BIN="$BIN.exe"; fi

# Always build. Skills are embedded at compile time, so a stale binary would
# silently test the previous version of the skill files.
echo "==> building memcrate"
(cd "$REPO" && cargo build --locked)

# A scratch HOME keeps the test off your real skills and vault. Codex stores
# its key there, so it is removed on every exit path.
# Agent output sits beside the scratch home, so a later session cannot read it.
WORK="$(mktemp -d)"
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT
trap 'exit 130' INT TERM
SCRATCH="$WORK/home"
mkdir "$SCRATCH"
AGENT_HOME="$(native "$SCRATCH")"
VAULT="$SCRATCH/memcrate-vault"
STEP=""
OUT=""

# Windows tools read USERPROFILE; a set CLAUDE_CONFIG_DIR or CODEX_HOME would reach your real config.
# Git Bash rewrites any argument starting with / (such as /load) into a Windows
# path; every path passed here already went through native(), so turn that off.
in_home() {
    env -u CLAUDE_CONFIG_DIR -u "$OTHER_KEY" \
        HOME="$AGENT_HOME" USERPROFILE="$AGENT_HOME" \
        CODEX_HOME="$(native "$SCRATCH/.codex")" \
        MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' \
        "$@"
}

# File storage keeps the key in the scratch home, never in an OS keychain.
codex_home() { in_home codex -c cli_auth_credentials_store=file "$@"; }

# The agent's final answer only, so a canary echoed in the prompt or a file read cannot count.
answer() {
    case "$TOOL" in
        codex) cat "$WORK/$STEP-answer.txt" 2>/dev/null || true ;;
        claude)
            grep '"type":"result"' "$OUT" 2>/dev/null |
                sed -nE 's/.*"result":"(([^"\\]|\\.)*)".*/\1/p' |
                awk '{ gsub(/\\n/, "\n"); print }' || true
            ;;
    esac
}

fail() {
    echo "FAIL: $1" >&2
    if [ -n "$OUT" ] && [ -f "$OUT" ]; then
        echo "--- $TOOL's final answer ---" >&2
        answer | tail -20 >&2
        echo "--- last events ---" >&2
        tail -5 "$OUT" | cut -c1-300 >&2
    fi
    exit 1
}

SKILLS="$SCRATCH/.claude/skills"
if [ "$TOOL" = codex ]; then SKILLS="$SCRATCH/.codex/skills"; fi

if [ "$MODE" = prefixed ]; then
    echo "==> planting a foreign save skill for $TOOL"
    mkdir -p "$SKILLS/save"
    cat >"$SKILLS/save/SKILL.md" <<'EOF'
---
name: save
description: Save a timestamped backup copy of a file the user names.
---

# save

Copy the named file to a sibling file with a timestamp suffix.
EOF
fi

echo "==> installing memcrate into a scratch home"
in_home "$BIN" --yes >/dev/null

if [ "$MODE" = prefixed ]; then
    for verb in load pin save; do
        grep "^name: memcrate-$verb" "$SKILLS/memcrate-$verb/SKILL.md" >/dev/null 2>&1 ||
            fail "prefixed: memcrate did not install memcrate-$verb for $TOOL"
    done
    for verb in load pin; do
        [ ! -e "$SKILLS/$verb" ] || fail "prefixed: a plain $verb skill was installed next to the user's own save"
    done
    grep "timestamped backup" "$SKILLS/save/SKILL.md" >/dev/null ||
        fail "prefixed: memcrate changed the user's own save skill"
fi

# Keep the template's last_updated so the pin step can see it change.
cat >"$VAULT/Core/Context/Profile.md" <<EOF
---
title: Profile
last_updated: YYYY-MM-DD
---

# Profile

## Identity

Test user. I build command-line tools in Rust. My current side project is codenamed $LOAD_CANARY.

## Tools & Stack

Rust, cargo, and a terminal.

## How I Work

<!-- Working hours, communication preferences, decision-making style. -->
EOF

cat >"$VAULT/Core/Context/Current State.md" <<EOF
---
title: Current State
last_updated: YYYY-MM-DD
---

# Current State

## This Week's Focus

Shipping the $LOAD_CANARY installer to three channels.

## Recent Decisions / Changes

<!-- Chronological log of meaningful decisions. /pin appends here when relevant. -->
EOF

if [ "$TOOL" = codex ]; then
    # Codex ignores OPENAI_API_KEY and wants a stored login. Piping it in keeps
    # the key out of the process list.
    printenv OPENAI_API_KEY | codex_home login --with-api-key >/dev/null 2>&1 ||
        fail "codex login --with-api-key failed"
fi

CODEX_SANDBOX="--sandbox=workspace-write"
if [ "$WINDOWS" = 1 ] && [ "${CI:-}" = true ]; then
    # workspace-write may be unsupported on native Windows; the CI runner is disposable.
    CODEX_SANDBOX="--dangerously-bypass-approvals-and-sandbox"
fi

# Runs one headless session; a third argument resumes that Claude session.
run_agent() {
    local resume="${3:-}" rc
    STEP="$1"
    OUT="$WORK/$STEP-events.jsonl"
    echo "==> $STEP: running $TOOL"
    set +e
    case "$TOOL" in
        codex)
            codex_home exec "$CODEX_SANDBOX" --json \
                --skip-git-repo-check \
                -C "$AGENT_HOME" \
                --output-last-message "$(native "$WORK/$STEP-answer.txt")" \
                "$2" </dev/null >"$OUT" 2>&1
            ;;
        claude)
            # --bare means API-key auth only and no Skill tool, so prompts call skills as /commands.
            # Claude Code fences file access to its working directory, hence the cd.
            (
                cd "$SCRATCH" &&
                    in_home claude --bare \
                        --add-dir "$AGENT_HOME" \
                        --permission-mode acceptEdits \
                        --output-format stream-json --verbose \
                        ${LIVE_TEST_CLAUDE_MODEL:+--model "$LIVE_TEST_CLAUDE_MODEL"} \
                        ${resume:+--resume "$resume"} \
                        -p "$2"
            ) </dev/null >"$OUT" 2>&1
            ;;
    esac
    rc=$?
    set -e
    [ "$rc" -eq 0 ] || fail "$STEP: $TOOL exited $rc"
}

session_id() {
    awk -F'"session_id":"' '/"subtype":"init"/ { split($2, a, "\""); print a[1]; exit }' "$OUT"
}

# Proof that the installed skill ran, not just that the agent found the vault on its own.
skill_ran() {
    local name="$1" sid transcript
    case "$TOOL" in
        claude)
            # Claude Code writes an invoked skill's directory into the session transcript.
            sid="$(session_id)"
            [ -n "$sid" ] || return 1
            transcript="$(find "$SCRATCH/.claude/projects" -name "$sid.jsonl" 2>/dev/null)"
            [ -n "$transcript" ] && [ -f "$transcript" ] || return 1
            grep -E 'Base directory for this skill: [^"]*skills[/\\]+'"$name"'\\n' "$transcript" >/dev/null
            ;;
        codex)
            # Codex opens a skill by reading its SKILL.md with a shell command.
            grep -oE '"command":"([^"\\]|\\.)*"' "$OUT" |
                grep -E 'skills[/\\]+'"$name"'([^-A-Za-z0-9_]|$)' |
                grep -F SKILL.md >/dev/null
            ;;
    esac
}

# Prints a file's YAML frontmatter; fails if the file does not open with one.
frontmatter() {
    awk '{ sub(/\r$/, "") }
        NR == 1 { if ($0 != "---") exit 1; next }
        $0 == "---" { closed = 1; exit }
        { print }
        END { if (!closed) exit 1 }' "$1"
}

last_updated() { frontmatter "$1" | grep '^last_updated:' || true; }

# The agent CLIs, and Windows app data (PowerShell, which Codex uses there), keep their own state
# here; nothing else outside the vault may appear or vanish.
outside_vault() {
    (cd "$SCRATCH" && find . \
        -path ./memcrate-vault -prune -o \
        -path ./.claude -prune -o \
        -path './.claude.json*' -prune -o \
        -path ./.codex -prune -o \
        -path ./AppData -prune -o \
        -print) | LC_ALL=C sort
}

session_logs() {
    (cd "$VAULT/Core/Sessions" && find . -type f -name '*.md' ! -name _README.md) | LC_ALL=C sort
}

outside_vault >"$WORK/outside-before.txt"

LOAD_SKILL=load
if [ "$MODE" = prefixed ]; then LOAD_SKILL=memcrate-load; fi
if [ "$TOOL" = claude ]; then
    LOAD_PROMPT="/$LOAD_SKILL"
elif [ "$MODE" = prefixed ]; then
    LOAD_PROMPT='Use the memcrate-load skill to load my context and get me oriented.'
else
    LOAD_PROMPT='Load my memcrate context and get me oriented.'
fi

run_agent load "$LOAD_PROMPT"
skill_ran "$LOAD_SKILL" || fail "load: no sign that the $LOAD_SKILL skill ran."
# The canary is in the vault and nowhere in the prompt, so seeing it back means
# the skill found the vault and read it.
answer | grep "$LOAD_CANARY" >/dev/null ||
    fail "load: $TOOL answered without the canary ($LOAD_CANARY), so it did not find or did not read the vault."
PASSED="load"
if [ "$MODE" = prefixed ]; then PASSED="memcrate- names, memcrate-load"; fi

if [ "$MODE" = plain ]; then
    if [ "$TOOL" = claude ]; then
        # /save cannot carry the decision itself, so state it first and save in the same session.
        run_agent save-context "$SAVE_SESSION"
        SID="$(session_id)"
        [ -n "$SID" ] || fail "save: Claude reported no session id to resume."
        session_logs >"$WORK/sessions-before.txt"
        run_agent save "/save" "$SID"
    else
        session_logs >"$WORK/sessions-before.txt"
        run_agent save "$SAVE_SESSION Save this session to my memcrate vault."
    fi
    skill_ran save || fail "save: no sign that the save skill ran."
    session_logs >"$WORK/sessions-after.txt"
    LC_ALL=C comm -13 "$WORK/sessions-before.txt" "$WORK/sessions-after.txt" >"$WORK/sessions-new.txt"
    NEW_LOGS="$(wc -l <"$WORK/sessions-new.txt" | tr -d ' ')"
    [ "$NEW_LOGS" = 1 ] ||
        fail "save: expected one new session log in Core/Sessions, found $NEW_LOGS."
    LOG="$VAULT/Core/Sessions/$(cat "$WORK/sessions-new.txt")"
    frontmatter "$LOG" | grep -E "^type:[[:space:]]*['\"]?session['\"]?[[:space:]]*\$" >/dev/null ||
        fail "save: $(basename "$LOG") does not open with frontmatter containing type: session."
    grep "$SAVE_CANARY" "$LOG" >/dev/null ||
        fail "save: $(basename "$LOG") does not mention $SAVE_CANARY."
    PASSED="$PASSED, save"

    # Save may bump a file's date too, so pin must change the date it finds.
    for name in Profile Projects "Current State"; do
        last_updated "$VAULT/Core/Context/$name.md" >"$WORK/updated-${name// /-}.txt"
    done
    if [ "$TOOL" = claude ]; then
        PIN_PROMPT="/pin I always ship releases on Tuesdays (ref $PIN_CANARY). It belongs in my Profile."
    else
        PIN_PROMPT="Pin this to my Profile: I always ship releases on Tuesdays (ref $PIN_CANARY)."
    fi
    run_agent pin "$PIN_PROMPT"
    skill_ran pin || fail "pin: no sign that the pin skill ran."
    PINNED=""
    UNBUMPED=""
    for name in Profile Projects "Current State"; do
        file="$VAULT/Core/Context/$name.md"
        grep "$PIN_CANARY" "$file" >/dev/null || continue
        before="$(cat "$WORK/updated-${name// /-}.txt")"
        after="$(last_updated "$file")"
        if [ "$after" != "$before" ] &&
            printf '%s\n' "$after" | grep -E '^last_updated:[[:space:]]*"?[0-9]{4}-[0-9]{2}-[0-9]{2}' >/dev/null; then
            PINNED="$name.md"
            break
        fi
        UNBUMPED="$name.md ($before -> $after)"
    done
    if [ -z "$PINNED" ] && [ -n "$UNBUMPED" ]; then
        fail "pin: $PIN_CANARY landed in $UNBUMPED but last_updated did not change to a date."
    fi
    [ -n "$PINNED" ] || fail "pin: no context file mentions $PIN_CANARY."
    PASSED="$PASSED, pin"
fi

outside_vault >"$WORK/outside-after.txt"
if ! diff "$WORK/outside-before.txt" "$WORK/outside-after.txt" >"$WORK/outside-diff.txt"; then
    OUT=""
    echo "FAIL: files outside the vault changed (> new, < removed):" >&2
    grep '^[<>]' "$WORK/outside-diff.txt" >&2 || true
    echo "If a path is the agent CLI's own state, add it to outside_vault()." >&2
    exit 1
fi

echo "live-test: OK ($TOOL, $MODE: $PASSED passed, each through its skill; nothing written outside the vault)"
