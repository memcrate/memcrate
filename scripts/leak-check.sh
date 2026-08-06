#!/bin/sh
# Blocks private context from reaching this public repo.
#
# The term list is derived at runtime from the maintainer's local vault, so no
# private name is ever committed here. Machines without that vault (CI, other
# contributors) skip the check entirely.
#
# Configure locally with untracked files at the repo root (or env vars):
#   .leakcheck-vault    path to the private vault, one line
#   .leakcheck-ignore   terms to skip, one per line
#   .leakcheck-extra    additional terms to block, one per line
#
# The vault path is never hardcoded here: naming it would itself be a leak.
set -eu

VAULT="${LEAKCHECK_VAULT:-}"
if [ -z "$VAULT" ] && [ -f .leakcheck-vault ]; then
    VAULT=$(head -1 .leakcheck-vault)
fi

PROJECTS="$VAULT/Core/Context/Projects.md"
if [ -z "$VAULT" ] || [ ! -f "$PROJECTS" ]; then
    echo "leak-check: no local vault configured, skipping"
    exit 0
fi

MIN_LEN=5
terms=$(mktemp)
ignore=$(mktemp)
trap 'rm -f "$terms" "$ignore"' EXIT

lower() { tr 'A-Z' 'a-z'; }

# Each project heading becomes several spellings: the literal name, a kebab
# slug, a squashed form, and for domain-style names the part before the dot,
# since a name like Example.io leaks in prose as plain "example".
sed -n 's/^## //p' "$PROJECTS" | lower | while IFS= read -r name; do
    [ -n "$name" ] || continue
    printf '%s\n' "$name"
    printf '%s\n' "$name" | tr ' .' '--'
    printf '%s\n' "$name" | tr -d ' .'
    case "$name" in
        *.*) printf '%s\n' "${name%%.*}" ;;
    esac
done > "$terms"

# The vault folder name and this machine's hostname are private context too.
basename "$VAULT" | lower >> "$terms"
hostname 2>/dev/null | lower >> "$terms" || true
[ -f .leakcheck-extra ] && lower < .leakcheck-extra >> "$terms"

sort -u -o "$terms" "$terms"

# This project's own name is not a leak.
printf 'memcrate\n' > "$ignore"
[ -f .leakcheck-ignore ] && lower < .leakcheck-ignore >> "$ignore"

hits=0
while IFS= read -r term; do
    [ -n "$term" ] || continue
    [ "${#term}" -ge "$MIN_LEN" ] || continue
    grep -qxF "$term" "$ignore" && continue
    found=$(git grep -n -i -I -w -F -- "$term" 2>/dev/null || true)
    if [ -n "$found" ]; then
        echo "leak-check: private term '$term' found in tracked files:"
        printf '%s\n' "$found" | sed 's/^/    /'
        hits=$((hits + 1))
    fi
done < "$terms"

# Absolute home paths are a leak regardless of the vault.
paths=$(git grep -n -I -E -- '/home/[a-z]|/Users/[a-z]|C:\\Users\\[a-z]' 2>/dev/null || true)
if [ -n "$paths" ]; then
    echo "leak-check: absolute home path in tracked files:"
    printf '%s\n' "$paths" | sed 's/^/    /'
    hits=$((hits + 1))
fi

if [ "$hits" -gt 0 ]; then
    echo
    echo "leak-check: FAILED with $hits finding(s)."
    echo "Remove the private reference, or if it is a false positive add the term to"
    echo ".leakcheck-ignore (untracked, one term per line) and re-run."
    exit 1
fi

echo "leak-check: OK"
