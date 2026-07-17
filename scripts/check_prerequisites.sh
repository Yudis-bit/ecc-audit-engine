#!/usr/bin/env bash
# Verify host tools required for ecc-audit-engine readiness verification.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Optional project-local tool prefixes (never required to be present in Git).
if [[ -d "$REPO_ROOT/third_party/sysroot/usr/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/sysroot/usr/bin:$PATH"
fi
if [[ -d "$HOME/.local/bin" ]]; then
  export PATH="$HOME/.local/bin:$PATH"
fi
if [[ -d "$REPO_ROOT/third_party/valgrind/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/valgrind/bin:$PATH"
fi

fail=0
warn=0

need() {
  local name="$1"
  if command -v "$name" >/dev/null 2>&1; then
    echo "[ok] required: $name -> $(command -v "$name")"
  else
    echo "[MISSING] required: $name" >&2
    fail=1
  fi
}

optional() {
  local name="$1"
  local note="${2:-}"
  if command -v "$name" >/dev/null 2>&1; then
    echo "[ok] optional: $name -> $(command -v "$name") ${note}"
  else
    echo "[warn] optional missing: $name ${note}"
    warn=1
  fi
}

echo "ecc-audit-engine prerequisite check"
echo "repo_root=$REPO_ROOT"
echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

need rustc
need cargo
need cc
need git
need python3
need sha256sum
need pkg-config

optional cmake "(needed for official libsecp256k1 CMake builds)"
optional autoconf "(needed for official libsecp256k1 Autotools builds)"
optional automake "(needed for official libsecp256k1 Autotools builds)"
optional libtoolize "(needed for official libsecp256k1 Autotools builds)"
optional clang "(Clang matrix builds)"
optional valgrind "(dynamic-trace calibration; project-local third_party/valgrind also accepted)"
optional docker "(clean-environment container path)"

if command -v rustc >/dev/null 2>&1; then
  echo "rustc=$(rustc --version)"
fi
if command -v cargo >/dev/null 2>&1; then
  echo "cargo=$(cargo --version)"
fi
if command -v cc >/dev/null 2>&1; then
  echo "cc=$(cc --version | head -1)"
fi
if command -v valgrind >/dev/null 2>&1; then
  echo "valgrind=$(valgrind --version)"
fi
if command -v cmake >/dev/null 2>&1; then
  echo "cmake=$(cmake --version | head -1)"
fi

if [[ ! -f "$REPO_ROOT/Cargo.lock" ]]; then
  echo "[MISSING] Cargo.lock (required for --locked builds)" >&2
  fail=1
else
  echo "[ok] Cargo.lock present"
fi

if [[ ! -f "$REPO_ROOT/rust-toolchain.toml" ]]; then
  echo "[MISSING] rust-toolchain.toml" >&2
  fail=1
else
  echo "[ok] rust-toolchain.toml present"
fi

if [[ "$fail" -ne 0 ]]; then
  echo "prerequisite check FAILED" >&2
  exit 1
fi
echo "prerequisite check PASSED (warnings=$warn)"
exit 0
