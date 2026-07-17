# ecc-audit-engine

Reproducible **secp256k1** correctness, differential-testing, failure-minimization, and dynamic-trace **research** engine.

[![CI](https://github.com/Yudis-bit/ecc-audit-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/Yudis-bit/ecc-audit-engine/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Yudis-bit/ecc-audit-engine)](https://github.com/Yudis-bit/ecc-audit-engine/releases)

## What this is

- BigUint reference mathematics for secp256k1 laboratory oracles
- Deterministic structured corpus generation
- Differential testing against local C ABI targets
- Failure minimization and reproducer replay
- Valgrind Lackey dynamic instruction / memory-address tracing
- Synthetic calibration fixtures (branch, table-address, control)
- Bounded public-API baseline against pinned upstream libsecp256k1

## What this is not

- Not a wallet, exchange, or production crypto library
- Not a private-key cracker, “Bitcoin breaker”, or exploit engine
- Not a universal proof of constant-time behavior
- Not authorization to test third-party systems

## Authorized scope

- Synthetic private keys only
- Locally compiled binaries and authorized laboratory targets only
- No live wallets, exchanges, nodes, browser extensions, hardware wallets, or production services

## Required language

No vulnerability in upstream libsecp256k1 has been confirmed by the
published experiments.

Synthetic corrupted and leaky targets are calibration fixtures used
to validate the engine.

A negative bounded trace result is not a universal proof of
constant-time behavior.

## Quick start (fresh machine)

```bash
git clone https://github.com/Yudis-bit/ecc-audit-engine.git
cd ecc-audit-engine
./scripts/bootstrap.sh
./scripts/verify.sh
```

`verify.sh` checks prerequisites, builds, lints, tests, runs differential and minimizer gates, synthetic dynamic-trace calibration (when Valgrind is available), pinned libsecp baseline, schema validation, and writes a compact report under `reports/readiness-run/`.

## Fast test suite

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked -- --nocapture
./scripts/build_targets.sh
./scripts/run_differential.sh
./scripts/run_minimizer_replay.sh
```

## Extended trace suite (optional Valgrind)

```bash
export PATH="$PWD/third_party/valgrind/bin:$PATH"   # if using project-local Valgrind
./scripts/run_trace_calibration.sh
# full campaigns: scripts/run_dynamic_trace.sh
```

Manual CI workflow: `.github/workflows/extended-trace.yml`

## Dependencies

| Dependency | Required? |
|------------|-----------|
| Rust (see `rust-toolchain.toml`) | yes |
| GCC/`cc`, git, python3 | yes |
| Valgrind 3.22+ | optional (dynamic trace) |
| CMake / Autotools | optional (official upstream builds) |
| Clang | optional (compiler matrix) |
| Docker | optional (clean environment) |

See [docs/dependencies.md](docs/dependencies.md).

## What was verified (prototype / readiness)

| Capability | Status |
|------------|--------|
| Reference model + known vectors | Verified by tests |
| Corpus seed 1337 deterministic | Gate in `verify.sh` |
| Correct mini-C differential | Expect zero unexpected mismatches |
| Corrupted fixtures + minimizer replay | Expect mapped defects + MIN reproducers |
| Lackey branch calibration | Synthetic detection |
| Lackey table-address calibration | Synthetic static load / cache-line divergence |
| Constant-time control fixture | Must stay clean |
| Pinned libsecp256k1 public-API baseline | Policy-consistent; no arithmetic bug claimed |

**Pinned upstream commit (historical v0.1.0 / reproducibility):**  
`11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53` (bitcoin-core/secp256k1)

## Synthetic vs unmodified upstream

| Result class | Meaning |
|--------------|---------|
| Synthetic calibration | Intentionally leaky fixtures validate detectors |
| Controlled corrupted fixtures | Planted arithmetic defects validate differential + minimizer |
| Unmodified upstream baseline | Bounded adapter experiments only |
| Negative bounded trace | No class-correlated divergence **on the tested corpus/build** |

## Upstream commit tested

See `targets-src/SECP256K1_PIN.txt` and [docs/upstream-research.md](docs/upstream-research.md).

## Reports

- `reports/published/` — compact public summaries
- `reports/readiness-run/` — verification outputs (raw/generated gitignored)
- `reports/dynamic-trace/` — backend metadata and calibration summaries
- Schemas: `schemas/*-v1.schema.json`

## Add a target

1. Implement the ABI in `crates/target-api/include/ecc_target.h`
2. Add sources under `harnesses/`
3. Extend `scripts/build_targets.sh`
4. Add corpus cases / experiments as needed
5. Run `./scripts/verify.sh`

## Reproduce one finding

```bash
cargo run -p cli -- differential \
  --target targets/corrupted-target.so \
  --corpus fixtures/corpus-v1.json \
  --case <case_id> \
  --minimize \
  --output reports/latest
```

For dynamic-trace campaigns, see `experiments/dynamic-trace/` and `scripts/run_trace_calibration.sh`.

## Real security findings

See [SECURITY.md](SECURITY.md) and [docs/threat-model.md](docs/threat-model.md).
Do not publish exploitable detail for unmodified upstream without coordinated disclosure.

## Documentation

- [Architecture](docs/architecture.md)
- [Threat model](docs/threat-model.md)
- [Experiment methodology](docs/experiment-methodology.md)
- [Report schema](docs/report-schema.md)
- [Reproducibility](docs/reproducibility.md)
- [Upstream research](docs/upstream-research.md)
- [Limitations](docs/limitations.md)
- [Dependencies](docs/dependencies.md)
- [Release process](docs/release-process.md)

## License

MIT — see [LICENSE](LICENSE).
