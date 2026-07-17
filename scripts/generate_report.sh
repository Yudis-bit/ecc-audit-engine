#!/usr/bin/env bash
# Aggregate readiness verification into compact JSON + Markdown reports.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT="${ECC_REPORT_DIR:-$REPO_ROOT/reports/readiness-run/generated}"
mkdir -p "$OUT" "$REPO_ROOT/reports/readiness-run"

export REPO_ROOT OUT
python3 - <<'PY'
import hashlib, json, os, subprocess, datetime
from pathlib import Path

root = Path(os.environ["REPO_ROOT"])
out = Path(os.environ["OUT"])

def sha_file(p: Path):
    if not p.is_file():
        return None
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()

def git(*args):
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()

def read_json(p: Path):
    if p.is_file():
        return json.loads(p.read_text())
    return None

corpus = root / "fixtures" / "corpus-v1.json"
table_status = (out / "table-address-status.txt").read_text().strip() if (out / "table-address-status.txt").is_file() else "UNKNOWN"
trace = read_json(out / "trace-calibration.json") or {}
libsecp = read_json(out / "libsecp-baseline.json") or {}

doc = {
    "schema_version": "experiment-v1",
    "kind": "readiness-verification",
    "generated_at_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "engine_commit": git("rev-parse", "HEAD"),
    "engine_branch": git("rev-parse", "--abbrev-ref", "HEAD"),
    "repository_url": "https://github.com/Yudis-bit/ecc-audit-engine",
    "corpus_hash": sha_file(corpus),
    "experiment_seed": 1337,
    "operation": "readiness_suite",
    "sample_count": None,
    "backend": {
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip(),
        "cc": subprocess.check_output(["cc", "--version"], text=True).splitlines()[0],
        "valgrind": subprocess.check_output(["valgrind", "--version"], text=True).strip() if subprocess.call(["bash","-lc","command -v valgrind"], stdout=subprocess.DEVNULL)==0 else "absent",
    },
    "result": "see_gates",
    "table_address_status": table_status,
    "trace_calibration": trace,
    "libsecp_baseline": libsecp,
    "limitations": [
        "A negative bounded trace result is not a universal proof of constant-time behavior.",
        "Synthetic corrupted and leaky targets are calibration fixtures used to validate the engine.",
        "No vulnerability in upstream libsecp256k1 has been confirmed by the published experiments.",
    ],
    "reproduction_command": "./scripts/verify.sh",
}

# Schema validation (lightweight structural)
required = ["schema_version", "engine_commit", "corpus_hash", "reproduction_command", "limitations"]
missing = [k for k in required if not doc.get(k)]
doc["schema_validation"] = {"ok": not missing, "missing": missing}

(out / "readiness-report.json").write_text(json.dumps(doc, indent=2) + "\n")

md = []
md.append("# ecc-audit-engine readiness report\n")
md.append(f"Generated (UTC): {doc['generated_at_utc']}\n")
md.append(f"Engine commit: `{doc['engine_commit']}`\n")
md.append(f"Corpus SHA-256: `{doc['corpus_hash']}`\n")
md.append("\n## Required language\n")
md.append("- No vulnerability in upstream libsecp256k1 has been confirmed by the published experiments.\n")
md.append("- Synthetic corrupted and leaky targets are calibration fixtures used to validate the engine.\n")
md.append("- A negative bounded trace result is not a universal proof of constant-time behavior.\n")
md.append(f"\n## Table-address status\n\n`{table_status}`\n")
if trace:
    md.append("\n## Trace calibration (summary)\n\n")
    md.append(f"- branch_detected: {trace.get('branch_detected')}\n")
    md.append(f"- table_detected: {trace.get('table_detected')}\n")
    md.append(f"- control_clean: {trace.get('control_clean')}\n")
if libsecp:
    md.append("\n## libsecp baseline (summary)\n\n")
    md.append(f"- total: {libsecp.get('total')}\n")
    md.append(f"- arithmetic_suspect: {libsecp.get('arithmetic_suspect')}\n")
    md.append(f"- policy_or_boundary: {libsecp.get('policy_or_boundary')}\n")
md.append("\n## Reproduction\n\n```bash\n./scripts/verify.sh\n```\n")
(out / "readiness-report.md").write_text("".join(md))

# Also publish compact copy under reports/readiness-run/
(out.parent / "readiness-report.json").write_text(json.dumps(doc, indent=2) + "\n")
(out.parent / "readiness-report.md").write_text("".join(md))
print("wrote", out / "readiness-report.json")
print("wrote", out / "readiness-report.md")
if missing:
    raise SystemExit(f"schema missing fields: {missing}")
print("generate_report complete")
PY
