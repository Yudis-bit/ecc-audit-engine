#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p targets
CC="${CC:-cc}"
COMMON="$ROOT/harnesses/common/secp_mini.c"
CFLAGS_BASE="-std=c11 -Wall -Wextra -fPIC -I$ROOT/crates/target-api/include -I$ROOT/harnesses/common"

build_one() {
  local name="$1"
  local src="$2"
  local extra_cflags="$3"
  local opt="$4"
  local out="targets/${name}-${opt// /_}.so"
  # shellcheck disable=SC2086
  $CC $CFLAGS_BASE $opt $extra_cflags -shared -o "$out" "$src" "$COMMON"
  sha256sum "$out" | tee "targets/${name}-${opt// /_}.sha256"
  echo "built $out"
}

OPTS=("-O0" "-O2" "-O3")

for opt in "${OPTS[@]}"; do
  build_one "correct-target" "$ROOT/harnesses/correct-target/target.c" "" "$opt"
  build_one "corrupted-target" "$ROOT/harnesses/corrupted-target/target.c" \
    "-DCORRUPT_FE_MUL -DCORRUPT_INFINITY_ADD -DCORRUPT_SCALAR_BOUNDARY" "$opt"
  build_one "leaky-branch" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=1" "$opt"
  build_one "leaky-table" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=2" "$opt"
  build_one "leaky-control" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=0" "$opt"
done

# LTO if supported
if $CC -flto -O3 -c -x c /dev/null -o /tmp/ecc_lto_test.o 2>/dev/null; then
  rm -f /tmp/ecc_lto_test.o
  build_one "correct-target" "$ROOT/harnesses/correct-target/target.c" "" "-O3 -flto"
  build_one "corrupted-target" "$ROOT/harnesses/corrupted-target/target.c" \
    "-DCORRUPT_FE_MUL -DCORRUPT_INFINITY_ADD -DCORRUPT_SCALAR_BOUNDARY" "-O3 -flto"
  build_one "leaky-branch" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=1" "-O3 -flto"
  build_one "leaky-table" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=2" "-O3 -flto"
  build_one "leaky-control" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=0" "-O3 -flto"
else
  echo "LTO not supported by $CC; skipping -flto builds"
fi

# Convenience symlinks for default -O2
ln -sfn correct-target--O2.so targets/correct-target.so
ln -sfn corrupted-target--O2.so targets/corrupted-target.so
ln -sfn leaky-branch--O2.so targets/leaky-target.so
ln -sfn leaky-branch--O2.so targets/leaky-branch.so
ln -sfn leaky-table--O2.so targets/leaky-table.so
ln -sfn leaky-control--O2.so targets/leaky-control.so

# Metadata
{
  echo "compiler=$CC"
  $CC --version | head -1
  echo "arch=$(uname -m)"
  echo "date=$(date -Iseconds)"
} > targets/build-info.txt

echo "All targets built."
ls -la targets/
