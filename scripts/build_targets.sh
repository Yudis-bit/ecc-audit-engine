#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p targets
CC="${CC:-cc}"
COMMON="$ROOT/harnesses/common/secp_mini.c"
MARKERS="$ROOT/harnesses/trace-driver/trace_markers.c"
CFLAGS_BASE="-std=c11 -Wall -Wextra -fPIC -I$ROOT/crates/target-api/include -I$ROOT/harnesses/common -I$ROOT/harnesses/trace-driver"

build_one() {
  local name="$1"
  local src="$2"
  local extra_cflags="$3"
  local opt="$4"
  local extra_src="${5:-}"
  local out="targets/${name}-${opt// /_}.so"
  # shellcheck disable=SC2086
  $CC $CFLAGS_BASE $opt $extra_cflags -shared -o "$out" "$src" "$COMMON" $extra_src
  sha256sum "$out" | tee "targets/${name}-${opt// /_}.sha256"
  echo "built $out"
}

OPTS=("-O0" "-O2" "-O3")

for opt in "${OPTS[@]}"; do
  build_one "correct-target" "$ROOT/harnesses/correct-target/target.c" "" "$opt"
  build_one "corrupted-target" "$ROOT/harnesses/corrupted-target/target.c" \
    "-DCORRUPT_FE_MUL -DCORRUPT_INFINITY_ADD -DCORRUPT_SCALAR_BOUNDARY" "$opt"
  build_one "leaky-branch" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=1" "$opt" "$MARKERS"
  build_one "leaky-table" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=2" "$opt" "$MARKERS"
  build_one "leaky-control" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=0" "$opt" "$MARKERS"
done

# LTO if supported
if $CC -flto -O3 -c -x c /dev/null -o /tmp/ecc_lto_test.o 2>/dev/null; then
  rm -f /tmp/ecc_lto_test.o
  build_one "correct-target" "$ROOT/harnesses/correct-target/target.c" "" "-O3 -flto"
  build_one "corrupted-target" "$ROOT/harnesses/corrupted-target/target.c" \
    "-DCORRUPT_FE_MUL -DCORRUPT_INFINITY_ADD -DCORRUPT_SCALAR_BOUNDARY" "-O3 -flto"
  build_one "leaky-branch" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=1" "-O3 -flto" "$MARKERS"
  build_one "leaky-table" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=2" "-O3 -flto" "$MARKERS"
  build_one "leaky-control" "$ROOT/harnesses/leaky-target/target.c" "-DLEAK_MODE=0" "-O3 -flto" "$MARKERS"
else
  echo "LTO not supported by $CC; skipping -flto builds"
fi

# Fast dynamic-trace calibration gadgets (no full scalar mul)
for mode in 0 1 2; do
  name=calib-control
  [ "$mode" -eq 1 ] && name=calib-branch
  [ "$mode" -eq 2 ] && name=calib-table
  $CC $CFLAGS_BASE -O2 -g -shared -DCALIB_MODE=$mode \
    -o "targets/${name}.so" \
    "$ROOT/harnesses/trace-driver/calib_targets.c" "$MARKERS"
  sha256sum "targets/${name}.so" | tee "targets/${name}.sha256"
done

$CC -std=c11 -O2 -g -o targets/ecc-trace-driver \
  "$ROOT/harnesses/trace-driver/driver.c" "$MARKERS" -ldl

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
