#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
./scripts/build_targets.sh
cargo run -q -p cli -- trace --target targets/leaky-branch.so --output reports/latest
cargo run -q -p cli -- trace --target targets/leaky-table.so --output reports/latest
cargo run -q -p cli -- trace --target targets/leaky-control.so --output reports/latest
cargo run -q -p cli -- timing --target targets/leaky-branch.so --samples 200 --warmup 50 --output reports/latest
cargo run -q -p cli -- timing --target targets/leaky-control.so --samples 200 --warmup 50 --output reports/latest
