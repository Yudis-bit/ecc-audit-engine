#!/usr/bin/env bash
# Synthetic dynamic-trace calibration: branch, table-address, control, identical-input stability.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ -d "$REPO_ROOT/third_party/valgrind/bin" ]]; then
  export PATH="$REPO_ROOT/third_party/valgrind/bin:$PATH"
fi

OUT="${ECC_REPORT_DIR:-$REPO_ROOT/reports/readiness-run/generated}"
TRACE_OUT="$OUT/raw/trace"
mkdir -p "$TRACE_OUT"

if ! command -v valgrind >/dev/null 2>&1; then
  echo "ERROR: valgrind not found (install system package or build third_party/valgrind)" >&2
  echo "TABLE_ADDRESS_DETECTION_INCOMPLETE" > "$OUT/table-address-status.txt"
  exit 1
fi

echo "+ valgrind=$(valgrind --version)"
echo "+ cargo run -q -p cli -- trace-backend verify"
cargo run -q -p cli -- trace-backend verify

if [[ ! -x targets/ecc-trace-driver ]] || [[ ! -f targets/calib-branch.so ]]; then
  ./scripts/build_targets.sh
fi

# Prefer fast calib-*.so fixtures; fall back to full leaky-* targets.
BRANCH_T=targets/calib-branch.so
TABLE_T=targets/calib-table.so
CTRL_T=targets/calib-control.so
[[ -f "$BRANCH_T" ]] || BRANCH_T=targets/leaky-branch--O2.so
[[ -f "$TABLE_T" ]] || TABLE_T=targets/leaky-table--O2.so
[[ -f "$CTRL_T" ]] || CTRL_T=targets/leaky-control--O2.so

# Tiny experiment files for readiness (2 keys/class)
mkdir -p experiments/dynamic-trace
cat > experiments/dynamic-trace/readiness-lsb.toml <<'EOF'
[experiment]
name = "readiness-lsb"
seed = 1337
keys_per_class = 2
class = "lsb"
backend = "valgrind-lackey"
EOF

echo "=== branch calibration ($BRANCH_T) ==="
cargo run -q -p cli -- trace-dynamic \
  --target "$BRANCH_T" \
  --experiment experiments/dynamic-trace/readiness-lsb.toml \
  --output "$TRACE_OUT/calib-branch"

echo "=== table calibration ($TABLE_T) ==="
cargo run -q -p cli -- trace-dynamic \
  --target "$TABLE_T" \
  --experiment experiments/dynamic-trace/readiness-lsb.toml \
  --output "$TRACE_OUT/calib-table"

echo "=== control calibration ($CTRL_T) ==="
cargo run -q -p cli -- trace-dynamic \
  --target "$CTRL_T" \
  --experiment experiments/dynamic-trace/readiness-lsb.toml \
  --output "$TRACE_OUT/calib-control"

python3 - <<'PY'
import json, sys
from pathlib import Path
out = Path(__import__("os").environ.get("ECC_REPORT_DIR", "reports/readiness-run/generated"))
trace = out / "raw" / "trace"

def load(name):
    p = trace / name / "campaign_summary.json"
    with open(p) as f:
        return json.load(f)

branch = load("calib-branch")
table = load("calib-table")
control = load("calib-control")

# Branch: expect instruction-set divergence between classes
branch_detected = (
    branch.get("paired_insn_set_equal", 999) < branch.get("pairs", 0)
    or branch.get("aggregate_insn_set_jaccard", 1.0) < 0.999
)
# Table: expect static mem / cache line divergence (NOT only insn equality)
table_insn_equal = table.get("aggregate_insn_set_jaccard", 0) >= 0.999
table_mem_diverge = table.get("aggregate_static_mem_jaccard", 1.0) < 0.999
table_cache_diverge = table.get("aggregate_static_cache_jaccard", 1.0) < 0.999
table_detected = table_mem_diverge or table_cache_diverge
# Control: expect clean
control_clean = (
    control.get("paired_insn_set_equal", 0) == control.get("pairs", -1)
    and control.get("aggregate_static_mem_jaccard", 0) >= 0.999
    and control.get("aggregate_insn_set_jaccard", 0) >= 0.999
)

status = {
    "schema_version": "trace-calibration-v1",
    "branch_detected": branch_detected,
    "branch_summary": {
        "paired_insn_set_equal": branch.get("paired_insn_set_equal"),
        "pairs": branch.get("pairs"),
        "aggregate_insn_set_jaccard": branch.get("aggregate_insn_set_jaccard"),
        "target_sha256": branch.get("target_sha256"),
    },
    "table_detected": table_detected,
    "table_summary": {
        "aggregate_insn_set_jaccard": table.get("aggregate_insn_set_jaccard"),
        "aggregate_static_mem_jaccard": table.get("aggregate_static_mem_jaccard"),
        "aggregate_static_cache_jaccard": table.get("aggregate_static_cache_jaccard"),
        "target_sha256": table.get("target_sha256"),
        "insn_sequences_equal_enough": table_insn_equal,
        "static_mem_diverges": table_mem_diverge,
        "static_cache_diverges": table_cache_diverge,
    },
    "control_clean": control_clean,
    "control_summary": {
        "paired_insn_set_equal": control.get("paired_insn_set_equal"),
        "pairs": control.get("pairs"),
        "aggregate_insn_set_jaccard": control.get("aggregate_insn_set_jaccard"),
        "aggregate_static_mem_jaccard": control.get("aggregate_static_mem_jaccard"),
        "target_sha256": control.get("target_sha256"),
    },
    "notes": [
        "Table-address detection requires static_mem or cache-line divergence under Lackey --trace-mem=yes.",
        "Instruction-sequence equality alone is not accepted as table-address proof.",
        "Synthetic calibration fixtures only; not upstream vulnerabilities.",
    ],
}

if table_detected and table_insn_equal:
    status["table_address_status"] = "TABLE_ADDRESS_DETECTION_VERIFIED"
elif table_detected:
    status["table_address_status"] = "TABLE_ADDRESS_DETECTION_VERIFIED"
    status["notes"].append("Table divergence observed; insn sets may also differ depending on codegen.")
else:
    status["table_address_status"] = "TABLE_ADDRESS_DETECTION_INCOMPLETE"

(out / "trace-calibration.json").write_text(json.dumps(status, indent=2) + "\n")
(out / "table-address-status.txt").write_text(status["table_address_status"] + "\n")
print(json.dumps(status, indent=2))

if not branch_detected:
    print("FAIL: branch fixture not detected", file=sys.stderr)
    sys.exit(1)
if not table_detected:
    print("FAIL: table-address fixture not detected", file=sys.stderr)
    sys.exit(1)
if not control_clean:
    print("FAIL: constant-control fixture not clean", file=sys.stderr)
    sys.exit(1)
print("trace calibration PASSED")
PY

echo "run_trace_calibration complete"
