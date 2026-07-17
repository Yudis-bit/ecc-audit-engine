#!/usr/bin/env bash
# Build pinned libsecp256k1 (if needed), build public-API adapter, run bounded differential baseline.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ -d "$REPO_ROOT/third_party/sysroot/usr/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/sysroot/usr/bin:$PATH"
fi
if [[ -d "$HOME/.local/bin" ]]; then
  export PATH="$HOME/.local/bin:$PATH"
fi

OUT="${ECC_REPORT_DIR:-$REPO_ROOT/reports/readiness-run/generated}"
mkdir -p "$OUT/raw/baseline" "$OUT/libsecp"

PIN_FILE="$REPO_ROOT/targets-src/SECP256K1_PIN.txt"
if [[ ! -f "$PIN_FILE" ]]; then
  echo "ERROR: missing $PIN_FILE" >&2
  exit 1
fi
PIN_COMMIT="$(grep '^commit=' "$PIN_FILE" | cut -d= -f2)"
PIN_REMOTE="$(grep '^remote=' "$PIN_FILE" | cut -d= -f2)"
echo "pinned_upstream_remote=$PIN_REMOTE"
echo "pinned_upstream_commit=$PIN_COMMIT"

SRC="$REPO_ROOT/targets-src/secp256k1"
if [[ ! -d "$SRC/.git" ]]; then
  echo "+ cloning pinned libsecp256k1 into targets-src/secp256k1 (not vendored in Git)"
  mkdir -p "$REPO_ROOT/targets-src"
  git clone --no-checkout "$PIN_REMOTE" "$SRC"
  git -C "$SRC" checkout "$PIN_COMMIT"
else
  CUR="$(git -C "$SRC" rev-parse HEAD)"
  if [[ "$CUR" != "$PIN_COMMIT" ]]; then
    echo "WARNING: local checkout $CUR != pin $PIN_COMMIT; checking out pin"
    git -C "$SRC" fetch --tags origin 2>/dev/null || git -C "$SRC" fetch --tags
    git -C "$SRC" checkout "$PIN_COMMIT"
  fi
fi

echo "+ ./scripts/build_libsecp256k1.sh"
./scripts/build_libsecp256k1.sh | tee "$OUT/raw/baseline/build_libsecp256k1.log"

# Build public-API adapter by compiling adapter + unmodified upstream amalgamation sources.
# Adapter does not patch upstream crypto; it only calls public headers/APIs.
ADAPTER_SRC="$REPO_ROOT/harnesses/libsecp256k1-adapter/adapter.c"
MARKERS="$REPO_ROOT/harnesses/trace-driver/trace_markers.c"
LIBDIR="$REPO_ROOT/targets/libsecp256k1"
LIBSO="$LIBDIR/libsecp256k1-gcc-O2.so"
if [[ ! -f "$LIBSO" ]]; then
  echo "ERROR: missing $LIBSO after build" >&2
  exit 1
fi

CC="${CC:-cc}"
ADAPTER_OUT="targets/libsecp256k1-adapter-gcc-O2.so"
echo "+ building adapter $ADAPTER_OUT (adapter + markers + unmodified upstream sources)"
# shellcheck disable=SC2086
$CC -std=c11 -O2 -g -fPIC -shared \
  -I"$REPO_ROOT/crates/target-api/include" \
  -I"$REPO_ROOT/harnesses/trace-driver" \
  -I"$SRC/include" \
  -I"$SRC" \
  -I"$SRC/src" \
  -o "$ADAPTER_OUT" \
  "$ADAPTER_SRC" \
  "$MARKERS" \
  "$SRC/src/secp256k1.c" \
  "$SRC/src/precomputed_ecmult.c" \
  "$SRC/src/precomputed_ecmult_gen.c" \
  -DSECP256K1_BUILD \
  -DECMULT_WINDOW_SIZE=15 \
  -DECMULT_GEN_PREC_BITS=4 \
  -DSECP256K1_WIDEMUL_INT128 \
  -lm 2>&1 | tee "$OUT/raw/baseline/build_adapter.log"

ln -sfn "libsecp256k1-adapter-gcc-O2.so" "targets/libsecp256k1-adapter.so"

sha256sum "$ADAPTER_OUT" | tee "$OUT/raw/baseline/adapter.sha256"
sha256sum "$LIBSO" | tee "$OUT/raw/baseline/libsecp.sha256"

if [[ ! -f fixtures/corpus-v1.json ]]; then
  cargo run -q -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
fi

echo "+ differential against libsecp adapter (bounded corpus)"
cargo run -q -p cli -- differential \
  --target "$ADAPTER_OUT" \
  --corpus fixtures/corpus-v1.json \
  --output "$OUT/raw/baseline" \
  2>&1 | tee "$OUT/raw/baseline/differential.log"

# Summarize: policy rejects are OK; arithmetic mismatches are not
python3 - <<'PY'
import json, sys
from pathlib import Path
out = Path(__import__("os").environ.get("ECC_REPORT_DIR", "reports/readiness-run/generated"))
candidates = [
    out / "raw" / "baseline" / "raw" / "differential_results.json",
    out / "raw" / "baseline" / "differential_results.json",
]
path = next((p for p in candidates if p.is_file()), None)
if path is None:
    print("ERROR: no differential results for libsecp baseline", file=sys.stderr)
    sys.exit(1)
results = json.loads(path.read_text())
mismatches = [r for r in results if r.get("mismatch") is not None or r.get("ok") is False]

policy = []
arith = []
for r in mismatches:
    kind = str(r.get("mismatch") or "")
    # DiffResult serializes enum variants as strings like "UnexpectedReject" / "ArithmeticMismatch"
    if "Arithmetic" in kind:
        arith.append(r)
    else:
        policy.append(r)

summary = {
    "schema_version": "libsecp-baseline-v1",
    "results_path": str(path),
    "total": len(results),
    "mismatch_count": len(mismatches),
    "policy_or_boundary": len(policy),
    "arithmetic_suspect": len(arith),
    "notes": [
        "API rejection-policy differences are not classified as arithmetic vulnerabilities.",
        "No vulnerability in upstream libsecp256k1 has been confirmed by the published experiments.",
    ],
}
(out / "libsecp-baseline.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
if arith:
    print("ERROR: unexpected arithmetic mismatches against model", file=sys.stderr)
    for r in arith[:10]:
        print(r.get("case_id"), r.get("mismatch"), file=sys.stderr)
    sys.exit(1)
print("libsecp baseline PASSED (no arithmetic mismatch claimed)")
PY

echo "run_libsecp_baseline complete"
