#!/usr/bin/env bash
# Replay minimized reproducers (or produce them from corrupted differential if none exist).
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT="${ECC_REPORT_DIR:-$REPO_ROOT/reports/readiness-run/generated}"
mkdir -p "$OUT/raw/minimizer" "$OUT/reproducers"

if [[ ! -f targets/corrupted-target.so ]]; then
  echo "+ building targets (missing corrupted-target.so)"
  ./scripts/build_targets.sh
fi

if [[ ! -f fixtures/corpus-v1.json ]]; then
  cargo run -q -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
fi

echo "+ differential corrupted + minimize"
cargo run -q -p cli -- differential \
  --target targets/corrupted-target.so \
  --corpus fixtures/corpus-v1.json \
  --minimize \
  --output "$OUT"

# Replay each minimizer reproducer if present
shopt -s nullglob
repros=("$OUT"/reproducers/MIN-*.json)
if [[ ${#repros[@]} -eq 0 ]]; then
  echo "no MIN-*.json reproducers produced; checking if failures were expected zero"
  # Still success if no failures — but corrupted should fail. Fail hard.
  echo "ERROR: expected minimized reproducers from corrupted target" >&2
  exit 1
fi

echo "replaying ${#repros[@]} minimizer reproducers"
REPO_ROOT="$REPO_ROOT" ECC_REPORT_DIR="$OUT" python3 - <<'PY'
import json, glob, os, subprocess, sys, tempfile
from pathlib import Path

out = Path(os.environ.get("ECC_REPORT_DIR", "reports/readiness-run/generated"))
repros = sorted(glob.glob(str(out / "reproducers" / "MIN-*.json")))
ok = 0
fail = 0
replay_dir = out / "raw" / "minimizer_replay_tmp"
replay_dir.mkdir(parents=True, exist_ok=True)

for p in repros:
    with open(p) as f:
        data = json.load(f)
    if "finding_id" not in data or "original_id" not in data or "case" not in data:
        print(f"WARN: unexpected reproducer schema: {p}")
        fail += 1
        continue
    case = data["case"]
    case_id = case.get("id") or data.get("original_id")
    # Write single-case corpus and re-run differential
    single = replay_dir / f"corpus_{data['finding_id']}.json"
    single.write_text(json.dumps([case], indent=2))
    cmd = [
        "cargo", "run", "-q", "-p", "cli", "--", "differential",
        "--target", "targets/corrupted-target.so",
        "--corpus", str(single),
        "--case", case_id,
        "--output", str(replay_dir / data["finding_id"]),
    ]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        print(f"FAIL replay command: {p}\n{r.stderr}")
        fail += 1
        continue
    # Expect the corrupted target still mismatches for this case
    results_path = replay_dir / data["finding_id"] / "raw" / "differential_results.json"
    if not results_path.is_file():
        print(f"FAIL missing results for {p}")
        fail += 1
        continue
    results = json.loads(results_path.read_text())
    still_bad = [x for x in results if x.get("mismatch") is not None or x.get("ok") is False]
    if not still_bad:
        print(f"FAIL reproducer no longer fails: {p}")
        fail += 1
        continue
    print(f"ok replay: {p} case={case_id}")
    ok += 1

summary = {"reproducers": len(repros), "replay_ok": ok, "replay_fail": fail}
(out / "minimizer_replay_summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
if fail or ok == 0:
    sys.exit(1)
PY

echo "run_minimizer_replay complete"
