#!/usr/bin/env bash
# Bootstrap a fresh clone: check tools, fetch crates, build lab targets.
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

echo "ecc-audit-engine bootstrap"
echo "repo_root=$REPO_ROOT"

./scripts/check_prerequisites.sh

echo "+ rustc=$(rustc --version)"
echo "+ cargo=$(cargo --version)"
echo "+ cc=$(cc --version | head -1)"

echo "+ cargo fetch --locked"
cargo fetch --locked

echo "+ cargo build --workspace --locked"
cargo build --workspace --locked

echo "+ ./scripts/build_targets.sh"
./scripts/build_targets.sh

echo "bootstrap complete"
echo "Next: ./scripts/verify.sh"
