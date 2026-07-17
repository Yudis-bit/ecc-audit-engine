# ecc-audit-engine

Standalone **defensive** research engine for secp256k1:

- mathematical reference model (BigUint)
- differential testing against local C targets
- input minimization
- Valgrind Lackey **dynamic** instruction / memory tracing (non-callback)

**No vulnerability in upstream libsecp256k1 was found by the published baseline experiments.**

The synthetic corrupted and leaky targets exist only to calibrate and validate the detection engine.

## Authorized scope

- Synthetic private keys only
- Locally compiled binaries and authorized laboratory targets only
- No live wallets, exchanges, nodes, hardware wallets owned by others, or production services

## Architecture

| Component | Role |
|-----------|------|
| `crates/model` | Auditable field/group math (primary oracle) |
| `crates/corpus` | Deterministic structured cases |
| `crates/runner` | FFI loader for C ABI targets |
| `crates/differential` | Target vs model comparison |
| `crates/minimizer` | Delta-debug failing inputs |
| `crates/dyntrace` | Valgrind Lackey parse / normalize / compare |
| `crates/trace` | Optional callback calibration (secondary) |
| `crates/timing` | Wall-clock / RDTSC harness (noisy; not CT proof) |
| `harnesses/` | correct / corrupted / leaky / libsecp adapter / trace driver |
| `scripts/` | Build and experiment helpers |

## What has been verified (v0.1.0)

| Capability | Status |
|------------|--------|
| Reference model + known vectors | Pass |
| Corpus seed 1337 (148 cases) | Stable hash |
| Correct mini-C target differential | 0 unexpected failures |
| Corrupted fixtures + isolation map | Detected as planted |
| Minimizer replay | Pass |
| Lackey dynamic branch calibration | Detected planted branch |
| Lackey dynamic table-address calibration | Detected load-set difference |
| Constant-time control fixture | No false class correlation |
| libsecp256k1 public-API differential | Policy-consistent; no arithmetic bug claimed |
| libsecp256k1 dynamic-trace (bounded) | No class-correlated insn-seq divergence on tested corpus |

**Pinned upstream commit tested historically:**  
`11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53` (bitcoin-core/secp256k1)

Bounded dynamic-trace language:

> No reproducible input-class-correlated instruction-sequence divergence was observed for the tested adapter build, backend, operation, and sample corpus.

This is **not** a proof that libsecp256k1 is constant-time on all platforms.

Do **not** describe callback counters as binary taint analysis.

## Prerequisites

- Rust (see `rust-toolchain.toml`)
- GCC/`cc`
- Linux x86_64 recommended
- Optional: Valgrind 3.22+ for dynamic tracing (build under `third_party/` — see `third_party/README.md`)

## Bootstrap

```bash
./scripts/bootstrap.sh
# or:
cargo build --workspace
./scripts/build_targets.sh
```

## Tests

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features -- --nocapture
```

## Differential experiments

```bash
cargo run -p cli -- model self-test
cargo run -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
cargo run -p cli -- differential \
  --target targets/correct-target.so \
  --corpus fixtures/corpus-v1.json
cargo run -p cli -- differential \
  --target targets/corrupted-target.so \
  --corpus fixtures/corpus-v1.json \
  --minimize
```

## Dynamic trace (Valgrind Lackey)

```bash
export PATH="$PWD/third_party/valgrind/bin:$PATH"   # if using project-local Valgrind
cargo run -p cli -- trace-backend verify

# minimal calibration targets (fast; markers + planted gadgets only)
./targets/ecc-trace-driver targets/calib-branch.so <64-hex-sk> case_id
# full campaigns: see experiments/dynamic-trace/ and scripts/run_dynamic_trace.sh
```

Table-address detection requires Lackey `--trace-mem=yes` and comparison of **effective load addresses / cache lines**, not instruction-sequence equality alone.

## Report structure

- `reports/published/` — compact public research summaries
- `reports/verification-run/` — independent prototype verification
- `reports/dynamic-trace/` — backend metadata and bounded campaign summaries  
  (large `raw/` traces are gitignored)

## Known limitations

- Mini-C field arithmetic is a lab fixture, not production crypto.
- In-process dylib loading (limited crash isolation).
- Timing harness is host-noise limited; large \|t\| alone is not key recovery.
- Dynamic taint / full ISA semantics not complete.
- CI does not run multi-hour Valgrind campaigns by default.

## License

MIT — see `LICENSE`.

## Security

See `SECURITY.md`.
