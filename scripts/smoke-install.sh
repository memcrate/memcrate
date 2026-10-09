#!/bin/sh
# Runs install.sh from this checkout in a scratch HOME, then setup, and checks
# what it wrote. The installer fetches the latest release, so this tests the
# installer, not this checkout's binary.
#
#   sh scripts/smoke-install.sh
set -eu

REPO="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
trap 'exit 1' INT TERM

BIN_DIR="$SCRATCH/bin"
FAKE_HOME="$SCRATCH/home"
mkdir -p "$FAKE_HOME"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# run() turns a failed skill install into a "Skipped" line and still exits 0.
run_setup() {
    out=$(HOME="$FAKE_HOME" "$EXE" --yes) || fail "memcrate --yes exited $?"
    echo "$out"
    case "$out" in
        *Skipped*) fail "setup skipped a tool" ;;
    esac
}

check_files() {
    vault="$FAKE_HOME/memcrate-vault"
    for f in .memcrate Core/Context/Profile.md Core/Context/Projects.md "Core/Context/Current State.md"; do
        [ -f "$vault/$f" ] || fail "vault is missing $f"
    done
    for tool in .claude .codex; do
        for skill in load save pin; do
            dir="$FAKE_HOME/$tool/skills/$skill"
            [ -f "$dir/SKILL.md" ] || fail "missing $tool/skills/$skill/SKILL.md"
            [ -f "$dir/.memcrate-skill" ] || fail "missing ownership marker in $tool/skills/$skill"
        done
    done
}

echo "==> install.sh"
MEMCRATE_INSTALL_DIR="$BIN_DIR" sh "$REPO/install.sh"
EXE="$BIN_DIR/memcrate"
[ -x "$EXE" ] || fail "no executable at $EXE"

echo "==> memcrate --yes"
run_setup
check_files

echo "==> memcrate --yes again (must reuse the vault and refresh the skills)"
run_setup
case "$out" in
    *"Using the existing vault"*) ;;
    *) fail "second run did not reuse the vault" ;;
esac
check_files

echo "smoke-install: OK"
