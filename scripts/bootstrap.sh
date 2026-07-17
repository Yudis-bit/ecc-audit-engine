#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "ecc-audit-engine bootstrap"
rustc --version
cargo --version
cc --version | head -1
cargo fetch
./scripts/build_targets.sh
echo "bootstrap complete"
