# ecc-audit-engine

Authorized **local laboratory** prototype for verifying secp256k1 implementation correctness and calibrating synthetic side-channel detectors.

## Scope (non-negotiable)

- Synthetic private keys only
- Locally compiled targets only
- No live wallets, exchanges, nodes, hardware wallets owned by others, or production services
- Synthetic leakage detection does **not** imply a real secp256k1 vulnerability

## Architecture

| Crate | Role |
|-------|------|
| `model` | BigUint reference field/group math (primary oracle) |
| `corpus` | Deterministic structured test cases |
| `target-api` | Shared metadata / finding enums |
| `runner` | `libloading` FFI to C `.so` targets |
| `differential` | Target vs reference comparison |
| `minimizer` | Delta-debug failing inputs |
| `trace` | Synthetic branch/table leak calibration |
| `timing` | LSB-class timing harness + Welch stats |
| `report` | JSON + Markdown reports |
| `cli` | `ecc-audit` command line |

C harnesses under `harnesses/`:

- `correct-target` — mini secp256k1 C implementation
- `corrupted-target` — controlled defects (`CORRUPT_*`)
- `leaky-target` — branch / table / control calibration modes

## Prerequisites

- Rust (see `rust-toolchain.toml`)
- `cc` (GCC)
- Linux x86_64 recommended for RDTSC timing

## Bootstrap

```bash
./scripts/bootstrap.sh
```

## Tests

```bash
./scripts/run_unit_tests.sh
# or
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features -- --nocapture
```

## Experiments

```bash
./scripts/run_all.sh
# or piecemeal:
cargo run -p cli -- model self-test
cargo run -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
cargo run -p cli -- differential --target targets/correct-target.so --corpus fixtures/corpus-v1.json
cargo run -p cli -- differential --target targets/corrupted-target.so --corpus fixtures/corpus-v1.json --minimize
cargo run -p cli -- trace --target targets/leaky-branch.so
cargo run -p cli -- timing --target targets/leaky-branch.so --samples 200 --warmup 50
```

## Target ABI

See `crates/target-api/include/ecc_target.h`.

Return codes: `0` OK, `1` reject, `-1` internal error.

## Report schema

`reports/latest/report.json` and `report.md` contain findings with levels:

- Level 0 Noise
- Level 1 Static suspicion
- Level 2 Dynamic divergence
- Level 3 Statistical leakage
- Level 4 Synthetic information-bearing leakage

## Interpretation

- Differential failures on **corrupted-target** are expected planted defects.
- Branch/table hits on **leaky-*** targets are calibration, not production CVEs.
- Timing `welch_t` alone is not key recovery.
- ASan/UBSan clean builds do not prove constant-time behavior.

## Known limitations

- No full binary taint / QEMU / symbolic execution in this slice
- C mini implementation is for lab harnesses, not production crypto
- Timing counters (`instructions`, cache, etc.) are `null` unless extended
- Subprocess crash isolation not fully implemented (dylib in-process)

## License

MIT (research prototype)
