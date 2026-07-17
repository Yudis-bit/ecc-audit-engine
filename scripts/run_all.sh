#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p reports/latest/raw reports/latest/reproducers
{
  echo "date=$(date -Iseconds)"
  uname -a
  rustc --version
  cargo --version
  cc --version | head -1
  lscpu | head -20
} | tee reports/latest/environment.txt

cargo build --workspace
./scripts/build_targets.sh
cp targets/build-info.txt reports/latest/build-matrix.txt 2>/dev/null || true
sha256sum targets/*.so 2>/dev/null | tee reports/latest/build-matrix.json.hashes || true

cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features -- --nocapture

cargo run -q -p cli -- model self-test
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

cargo run -q -p cli -- trace --target targets/leaky-branch.so --output reports/latest
cargo run -q -p cli -- trace --target targets/leaky-table.so --output reports/latest
cargo run -q -p cli -- trace --target targets/leaky-control.so --output reports/latest
cargo run -q -p cli -- timing --target targets/leaky-branch.so --samples 200 --warmup 50 --output reports/latest
cargo run -q -p cli -- timing --target targets/leaky-control.so --samples 200 --warmup 50 --output reports/latest

echo "run_all complete"
