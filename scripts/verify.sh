#!/usr/bin/env bash
# One-command readiness verification for independent engineers.
# Returns nonzero if any mandatory gate fails.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ -d "$REPO_ROOT/third_party/sysroot/usr/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/sysroot/usr/bin:$PATH"
fi
if [[ -d "$HOME/.local/bin" ]]; then
  export PATH="$HOME/.local/bin:$PATH"
fi
if [[ -d "$REPO_ROOT/third_party/valgrind/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/valgrind/bin:$PATH"
fi

export ECC_REPORT_DIR="${ECC_REPORT_DIR:-$REPO_ROOT/reports/readiness-run/generated}"
export REPO_ROOT
mkdir -p "$ECC_REPORT_DIR/raw" "$REPO_ROOT/reports/readiness-run/raw"

LOG="$REPO_ROOT/reports/readiness-run/verify.log"
exec > >(tee "$LOG") 2>&1

echo "============================================================"
echo "ecc-audit-engine verify"
echo "repo_root=$REPO_ROOT"
echo "utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "============================================================"

GATES=()
record() {
  local name="$1"
  local ok="$2"
  GATES+=("$name=$ok")
  if [[ "$ok" == "PASS" ]]; then
    echo "[PASS] $name"
  else
    echo "[FAIL] $name" >&2
  fi
}

run_gate() {
  local name="$1"
  shift
  echo "---- gate: $name ----"
  if "$@"; then
    record "$name" "PASS"
    return 0
  else
    record "$name" "FAIL"
    return 1
  fi
}

FAILED=0
./scripts/check_prerequisites.sh || FAILED=1

run_gate "cargo_fmt" cargo fmt --all --check || FAILED=1
run_gate "cargo_clippy" cargo clippy --workspace --all-targets --all-features -- -D warnings || FAILED=1
run_gate "cargo_test_locked" cargo test --workspace --all-features --locked -- --nocapture || FAILED=1
run_gate "model_self_test" cargo run -q -p cli -- model self-test || FAILED=1

run_gate "build_targets" ./scripts/build_targets.sh || FAILED=1

# Deterministic corpus
echo "---- gate: corpus_deterministic ----"
cargo run -q -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
H1=$(sha256sum fixtures/corpus-v1.json | awk '{print $1}')
cargo run -q -p cli -- corpus generate --seed 1337 --output "$ECC_REPORT_DIR/raw/corpus-v1-repeat.json"
H2=$(sha256sum "$ECC_REPORT_DIR/raw/corpus-v1-repeat.json" | awk '{print $1}')
echo "corpus_hash_1=$H1"
echo "corpus_hash_2=$H2"
if [[ "$H1" == "$H2" ]]; then
  record "corpus_deterministic" "PASS"
else
  record "corpus_deterministic" "FAIL"
  FAILED=1
fi
echo "$H1" > "$ECC_REPORT_DIR/corpus.sha256"

# Differential correct
echo "---- gate: differential_correct ----"
mkdir -p "$ECC_REPORT_DIR/raw/diff-correct"
if cargo run -q -p cli -- differential \
  --target targets/correct-target.so \
  --corpus fixtures/corpus-v1.json \
  --output "$ECC_REPORT_DIR/raw/diff-correct"; then
  # check zero unexpected failures via report
  if python3 - <<'PY'
import json, sys
from pathlib import Path
import os
p = Path(os.environ["ECC_REPORT_DIR"]) / "raw" / "diff-correct" / "raw" / "differential_results.json"
if not p.is_file():
    p = Path(os.environ["ECC_REPORT_DIR"]) / "raw" / "diff-correct" / "report.json"
data = json.loads(p.read_text())
if isinstance(data, list):
    fails = [r for r in data if r.get("mismatch") not in (None, "None") or r.get("ok") is False or r.get("passed") is False]
    # DiffResult likely has mismatch: null when ok
    fails = []
    for r in data:
        m = r.get("mismatch")
        if m is not None:
            fails.append(r)
    print(f"failures={len(fails)} total={len(data)}")
    sys.exit(0 if len(fails)==0 else 1)
else:
    ds = data.get("differential_summary", {})
    print("failures", ds.get("failures"))
    sys.exit(0 if ds.get("failures", 1)==0 else 1)
PY
  then
    record "differential_correct" "PASS"
  else
    record "differential_correct" "FAIL"
    FAILED=1
  fi
else
  record "differential_correct" "FAIL"
  FAILED=1
fi

# Differential corrupted + minimizer
echo "---- gate: differential_corrupted_and_minimizer ----"
export ECC_REPORT_DIR
if ./scripts/run_minimizer_replay.sh; then
  record "differential_corrupted_and_minimizer" "PASS"
else
  record "differential_corrupted_and_minimizer" "FAIL"
  FAILED=1
fi

# Trace calibration (optional if no valgrind — but then table status incomplete and fail readiness for full)
echo "---- gate: trace_calibration ----"
if command -v valgrind >/dev/null 2>&1; then
  if ./scripts/run_trace_calibration.sh; then
    record "trace_calibration" "PASS"
  else
    record "trace_calibration" "FAIL"
    FAILED=1
  fi
else
  echo "valgrind absent: marking TABLE_ADDRESS_DETECTION_INCOMPLETE"
  echo "TABLE_ADDRESS_DETECTION_INCOMPLETE" > "$ECC_REPORT_DIR/table-address-status.txt"
  record "trace_calibration" "FAIL"
  FAILED=1
fi

# libsecp baseline
echo "---- gate: libsecp_baseline ----"
if ./scripts/run_libsecp_baseline.sh; then
  record "libsecp_baseline" "PASS"
else
  record "libsecp_baseline" "FAIL"
  FAILED=1
fi

# Schema validation
echo "---- gate: schema_validation ----"
if python3 "$REPO_ROOT/scripts/validate_schemas.py"; then
  record "schema_validation" "PASS"
else
  record "schema_validation" "FAIL"
  FAILED=1
fi

# Generate compact report
./scripts/generate_report.sh || FAILED=1

# Hygiene: no personal absolute paths in tracked source (scripts + crates + docs)
echo "---- gate: no_personal_paths ----"
if git -C "$REPO_ROOT" grep -nE '/home/arkheionx|arkheionx@|github_pat_|ghp_[A-Za-z0-9]{20,}' -- \
  ':!reports/verification-run/raw/**' \
  ':!reports/**/raw/**' \
  2>/dev/null | grep -v '^$' ; then
  record "no_personal_paths" "FAIL"
  FAILED=1
else
  record "no_personal_paths" "PASS"
fi

echo "============================================================"
echo "GATE SUMMARY"
for g in "${GATES[@]}"; do
  echo "  $g"
done
echo "============================================================"

{
  echo "utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "commit=$(git rev-parse HEAD)"
  for g in "${GATES[@]}"; do echo "$g"; done
  echo "failed=$FAILED"
} > "$REPO_ROOT/reports/readiness-run/gates.txt"

if [[ "$FAILED" -ne 0 ]]; then
  echo "verify FAILED"
  exit 1
fi
echo "verify PASSED"
exit 0
