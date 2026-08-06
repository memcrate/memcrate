#!/bin/sh
# Single verify gate. Run by .githooks/pre-push and .github/workflows/ci.yml.
set -e

echo "verify: leak-check"
sh scripts/leak-check.sh

echo "verify: cargo fmt --check"
cargo fmt --check

echo "verify: cargo clippy"
cargo clippy --all-targets --locked -- -D warnings

echo "verify: cargo test"
cargo test --locked

echo "verify: cargo build"
cargo build --locked

echo "verify: OK"
