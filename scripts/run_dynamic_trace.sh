#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/third_party/valgrind/bin:$PATH"
mkdir -p reports/dynamic-trace/{raw,normalized,comparisons,reproducers}

cargo run -q -p cli -- trace-backend verify

echo "=== calibration: leaky-branch ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/leaky-branch--O2.so \
  --experiment experiments/dynamic-trace/leaky-branch.toml \
  --output reports/dynamic-trace/raw/calib-branch

echo "=== calibration: leaky-table ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/leaky-table--O2.so \
  --experiment experiments/dynamic-trace/leaky-table.toml \
  --output reports/dynamic-trace/raw/calib-table

echo "=== calibration: control ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/leaky-control--O2.so \
  --experiment experiments/dynamic-trace/leaky-control.toml \
  --output reports/dynamic-trace/raw/calib-control

echo "=== libsecp LSB ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/libsecp256k1-adapter-gcc-O2-dbg.so \
  --experiment experiments/dynamic-trace/libsecp-lsb.toml \
  --output reports/dynamic-trace/raw/libsecp-lsb

echo "=== libsecp hamming ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/libsecp256k1-adapter-gcc-O2-dbg.so \
  --experiment experiments/dynamic-trace/libsecp-hamming.toml \
  --output reports/dynamic-trace/raw/libsecp-hamming

echo "=== libsecp window ==="
cargo run -q -p cli -- trace-dynamic \
  --target targets/libsecp256k1-adapter-gcc-O2-dbg.so \
  --experiment experiments/dynamic-trace/libsecp-window.toml \
  --output reports/dynamic-trace/raw/libsecp-window

echo "dynamic trace campaigns complete"
