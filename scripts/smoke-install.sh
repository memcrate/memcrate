#!/bin/sh
# Installs memcrate through one channel in a scratch HOME, then runs setup and
# checks what it wrote.
#
#   sh scripts/smoke-install.sh [script|npm|cargo]
#
# script (the default) runs this checkout's install.sh, which fetches the latest
# release. npm packs this checkout's wrapper as the latest published version and
# installs that. Neither tests this checkout's binary.
#
# With MEMCRATE_EXPECT_VERSION set (the post-release check), npm and cargo
# install from their registries instead, and every channel must report that version.
set -eu

CHANNEL="${1:-script}"
EXPECT="${MEMCRATE_EXPECT_VERSION:-}"
EXPECT="${EXPECT#v}"

REPO="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
trap 'exit 1' INT TERM

FAKE_HOME="$SCRATCH/home"
mkdir -p "$FAKE_HOME"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

npm_lists_expected() {
    [ "$(npm view memcrate version)" = "$EXPECT" ]
}

crates_lists_expected() {
    curl -fsS https://index.crates.io/me/mc/memcrate | grep -q "\"vers\":\"$EXPECT\""
}

# Registries can lag the release workflow by a minute or two.
wait_until() {
    what=$1
    shift
    tries=0
    until "$@"; do
        tries=$((tries + 1))
        [ "$tries" -lt 20 ] || fail "$what still does not list $EXPECT after 5 minutes"
        echo "waiting for $what to list $EXPECT"
        sleep 15
    done
}

install_script() {
    MEMCRATE_INSTALL_DIR="$SCRATCH/bin" sh "$REPO/install.sh"
    EXE="$SCRATCH/bin/memcrate"
}

install_npm() {
    if [ -n "$EXPECT" ]; then
        wait_until "the npm registry" npm_lists_expected
        pkg=memcrate
    else
        # postinstall downloads the release matching the package version, which
        # this checkout's version may not have yet.
        EXPECT=$(npm view memcrate version)
        cp -R "$REPO/npm" "$SCRATCH/pkg"
        (cd "$SCRATCH/pkg" && npm pkg set "version=$EXPECT" && npm pack --pack-destination "$SCRATCH" >/dev/null)
        pkg="$SCRATCH/memcrate-$EXPECT.tgz"
    fi
    npm install --global --prefix "$SCRATCH/npm" "$pkg"
    EXE="$SCRATCH/npm/bin/memcrate"
}

install_cargo() {
    if [ -n "$EXPECT" ]; then
        wait_until "the crates.io index" crates_lists_expected
    fi
    cargo install memcrate --root "$SCRATCH/cargo"
    EXE="$SCRATCH/cargo/bin/memcrate"
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

case "$CHANNEL" in
    script) echo "==> install.sh"; install_script ;;
    npm) echo "==> npm install"; install_npm ;;
    cargo) echo "==> cargo install"; install_cargo ;;
    *) fail "unknown channel '$CHANNEL' (script, npm, or cargo)" ;;
esac
[ -x "$EXE" ] || fail "no executable at $EXE"

if [ -n "$EXPECT" ]; then
    got=$("$EXE" --version) || fail "memcrate --version exited $?"
    [ "$got" = "memcrate $EXPECT" ] || fail "$CHANNEL installed '$got', expected 'memcrate $EXPECT'"
    echo "==> $CHANNEL installed $got"
fi

if [ "$CHANNEL" = npm ]; then
    echo "==> npm shim passes the exit code through"
    code=0
    "$EXE" --no-such-flag >/dev/null 2>&1 || code=$?
    [ "$code" -eq 2 ] || fail "a bad flag exited $code through the npm shim; the binary exits 2"
fi

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

echo "smoke-install ($CHANNEL): OK"
