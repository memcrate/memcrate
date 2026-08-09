#!/usr/bin/env bash
# Live agent test: does /load actually find and read a vault in a real session?
#
# Everything else in this repo checks files on disk. This starts an agent,
# points it at a vault it has never seen, and checks that a fact only present
# inside that vault comes back in the answer.
#
#   scripts/live-test.sh            test Codex   (needs OPENAI_API_KEY)
#   scripts/live-test.sh claude     test Claude Code (needs ANTHROPIC_API_KEY)
#
# Not part of verify.sh or the pre-push hook on purpose: it costs money, needs
# network, and an agent's wording varies between runs. Run it before tagging a
# release, or from the live-test GitHub workflow.
set -euo pipefail

TOOL="${1:-codex}"
CANARY="PERSIMMON-7F3A"
REPO="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$REPO/target/debug/memcrate"

case "$TOOL" in
    codex | claude) ;;
    *)
        echo "unknown tool '$TOOL'. Use: codex | claude" >&2
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
    echo "Do not reuse an interactive login for this: the key is written to a" >&2
    echo "scratch home that this script deletes on exit." >&2
    exit 2
fi

# Always build. Skills are embedded at compile time, so a stale binary would
# silently test the previous version of the skill files.
echo "==> building memcrate"
(cd "$REPO" && cargo build --locked)

# A scratch HOME keeps the test off your real skills and vault. It holds a
# credential for the duration, so it is removed on every exit path.
SCRATCH="$(mktemp -d)"
cleanup() { rm -rf "$SCRATCH"; }
trap cleanup EXIT INT TERM

echo "==> installing memcrate into a scratch home"
HOME="$SCRATCH" "$BIN" --yes >/dev/null

VAULT="$SCRATCH/memcrate-vault"
cat > "$VAULT/Core/Context/Profile.md" <<EOF
---
title: Profile
---

# Profile

## Who I am

Test user. I build command-line tools in Rust.

## Canary

My current side project is codenamed $CANARY.
EOF

cat > "$VAULT/Core/Context/Current State.md" <<EOF
---
title: Current State
---

# Current State

## This week's focus

Shipping the $CANARY installer to three channels.
EOF

OUT="$SCRATCH/agent-output.txt"

echo "==> running $TOOL against the vault"
set +e
case "$TOOL" in
    codex)
        # Codex 0.146 ignores OPENAI_API_KEY and wants a stored login. Piping
        # it in keeps the key out of the process list.
        printenv OPENAI_API_KEY \
            | HOME="$SCRATCH" codex login --with-api-key >/dev/null 2>&1
        HOME="$SCRATCH" codex exec \
            --sandbox read-only \
            --skip-git-repo-check \
            -C "$SCRATCH" \
            '$load' > "$OUT" 2>&1
        ;;
    claude)
        # --bare is what makes an API key work at all: it reads ANTHROPIC_API_KEY
        # and never touches OAuth or the keychain. The tradeoff is that slash
        # commands do not register, so this exercises the skill's description
        # (the agent deciding to use it) rather than an explicit /load. That is
        # the harder half anyway; a human typing /load is the easy case.
        # cd into the scratch home first. Claude Code fences file access to the
        # working directory, so running from the repo leaves the seeded vault
        # unreadable and the canary check fails for a harness reason rather than
        # a skill one. This mirrors the codex branch's -C.
        (
            cd "$SCRATCH" &&
                HOME="$SCRATCH" claude --bare \
                    --add-dir "$SCRATCH" \
                    -p 'Load my memcrate context and get me oriented.' \
                    --permission-mode acceptEdits
        ) > "$OUT" 2>&1
        ;;
esac
RC=$?
set -e

if [ "$RC" -ne 0 ]; then
    echo "FAIL: $TOOL exited $RC" >&2
    tail -20 "$OUT" >&2
    exit 1
fi

# The canary is in the vault and nowhere in the prompt, so seeing it back means
# the agent found the vault on its own and read it. Anything less (the skill
# firing, a plausible-sounding summary) proves nothing.
if ! grep -q "$CANARY" "$OUT"; then
    echo "FAIL: $TOOL answered without the canary ($CANARY)." >&2
    echo "The skill did not find or did not read the vault." >&2
    tail -20 "$OUT" >&2
    exit 1
fi

echo "live-test: OK ($TOOL found the vault and read it)"
