#!/usr/bin/env bash
# Remove generated build artifacts and raw experiment outputs (keeps source and published summaries).
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "cleaning generated artifacts under $REPO_ROOT"

rm -rf target/
rm -f targets/*.so targets/*.sha256 targets/ecc-trace-driver targets/iso_*.so
rm -rf targets/libsecp256k1/
# Keep targets/build-info.txt only if regenerating; remove for clean state.
rm -f targets/build-info.txt

rm -rf reports/latest/raw/* reports/latest/reproducers/*
rm -f reports/latest/report.json reports/latest/report.md \
  reports/latest/environment.json reports/latest/environment.txt \
  reports/latest/build-matrix* 2>/dev/null || true

rm -rf reports/readiness-run/generated \
  reports/readiness-run/raw/differential \
  reports/readiness-run/raw/trace \
  reports/readiness-run/raw/baseline \
  reports/readiness-run/raw/minimizer 2>/dev/null || true

# Do not delete third_party/valgrind or targets-src/secp256k1 (expensive rebuilds).
# Do not delete published reports or fixtures.

echo "clean_generated complete"
