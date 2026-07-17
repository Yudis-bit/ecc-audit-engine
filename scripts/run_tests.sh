#!/usr/bin/env bash
# Format, lint, and test the Rust workspace with --locked.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "+ cargo fmt --all --check"
cargo fmt --all --check

echo "+ cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "+ cargo test --workspace --all-features --locked -- --nocapture"
cargo test --workspace --all-features --locked -- --nocapture

echo "+ cargo run -q -p cli -- model self-test"
cargo run -q -p cli -- model self-test

echo "run_tests complete"
