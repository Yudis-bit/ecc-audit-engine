#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
./scripts/build_targets.sh
cargo run -q -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
cargo run -q -p cli -- differential \
  --target targets/correct-target.so \
  --corpus fixtures/corpus-v1.json \
  --output reports/latest
cargo run -q -p cli -- differential \
  --target targets/corrupted-target.so \
  --corpus fixtures/corpus-v1.json \
  --minimize \
  --output reports/latest
