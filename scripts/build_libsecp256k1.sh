#!/usr/bin/env bash
# Build unmodified upstream libsecp256k1 as shared libraries (no cmake required).
# Adapter code is separate; upstream sources are not patched.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/targets-src/secp256k1"
OUT="$ROOT/targets/libsecp256k1"
mkdir -p "$OUT"

if [[ ! -d "$SRC/.git" ]]; then
  echo "missing $SRC — clone bitcoin-core/secp256k1 first" >&2
  exit 1
fi

COMMIT="$(git -C "$SRC" rev-parse HEAD)"
DATE="$(git -C "$SRC" log -1 --format=%ci)"
REMOTE="$(git -C "$SRC" remote get-url origin)"
echo "upstream=$REMOTE"
echo "commit=$COMMIT"
echo "date=$DATE"

# Common defines matching a typical production-ish default without modules.
# ECMULT_WINDOW_SIZE defaults in headers if unset.
COMMON_DEFS=(
  -DSECP256K1_BUILD
  -DECMULT_WINDOW_SIZE=15
  -DECMULT_GEN_PREC_BITS=4
)
# Prefer 128-bit wide mul on x86_64 GCC.
if [[ "$(uname -m)" == "x86_64" ]]; then
  COMMON_DEFS+=(-DSECP256K1_WIDEMUL_INT128)
else
  COMMON_DEFS+=(-DSECP256K1_WIDEMUL_INT64)
fi

CFLAGS_BASE=(
  -std=c89
  -O2
  -fPIC
  -Wall
  -Wextra
  -Wno-unused-function
  -I"$SRC"
  -I"$SRC/include"
  -I"$SRC/src"
)

# Sources: amalgamation unit + precomputed tables (upstream layout).
SOURCES=(
  "$SRC/src/secp256k1.c"
  "$SRC/src/precomputed_ecmult.c"
  "$SRC/src/precomputed_ecmult_gen.c"
)

build_one() {
  local cc="$1"
  local opt="$2"
  local tag="$3"
  local extra=("${@:4}")
  local out="$OUT/libsecp256k1-${tag}.so"
  # shellcheck disable=SC2086
  "$cc" "${CFLAGS_BASE[@]}" $opt "${COMMON_DEFS[@]}" "${extra[@]}" \
    -shared -o "$out" "${SOURCES[@]}" -lm
  sha256sum "$out" | tee "$OUT/libsecp256k1-${tag}.sha256"
  echo "built $out with $cc $opt ${extra[*]:-}"
}

# GCC matrix
build_one cc "-O2" "gcc-O2"
build_one cc "-O3" "gcc-O3"

# Assembly: only if src/asm exists and we enable USE_ASM_X86_64 — check header support
if grep -q "USE_ASM_X86_64" -r "$SRC/src" 2>/dev/null; then
  if [[ -d "$SRC/src/asm" ]]; then
    build_one cc "-O2" "gcc-O2-asm" -DUSE_ASM_X86_64 || echo "asm build skipped"
  fi
fi

# no-asm is default when USE_ASM not set
build_one cc "-O2" "gcc-O2-noasm"  # explicit noasm = default

# VERIFY (debug checks) build
build_one cc "-O2" "gcc-O2-verify" -DVERIFY

# Clang if present
if command -v clang >/dev/null 2>&1; then
  build_one clang "-O2" "clang-O2"
  build_one clang "-O3" "clang-O3"
  if clang -flto -O3 -c -x c /dev/null -o /tmp/secp_lto.o 2>/dev/null; then
    rm -f /tmp/secp_lto.o
    build_one clang "-O3 -flto" "clang-O3-flto"
  fi
else
  echo "clang not installed; skipping clang matrix"
fi

{
  echo "remote=$REMOTE"
  echo "commit=$COMMIT"
  echo "commit_date=$DATE"
  echo "builder_cc=$(cc --version | head -1)"
  echo "arch=$(uname -m)"
  echo "defs=${COMMON_DEFS[*]}"
  date -Iseconds
} | tee "$OUT/build-info.txt"

ln -sfn "libsecp256k1/libsecp256k1-gcc-O2.so" "$ROOT/targets/libsecp256k1.so" 2>/dev/null || \
  ln -sfn "libsecp256k1-gcc-O2.so" "$OUT/default.so"

echo "libsecp256k1 builds complete under $OUT"
ls -la "$OUT"
