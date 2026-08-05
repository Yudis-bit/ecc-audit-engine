# Changelog

## [0.2.1] — 2026-08-06

### Changed

- Dependency maintenance: rand 0.8.7 → 0.10.2, thiserror 1.0.69 → 2.0.19, toml 0.8.23 → 1.1.4, num-bigint 0.4.8 → 0.5.1, libloading 0.8.9 → 0.9.0
- Workspace version aligned with the release line (was still 0.1.0 at the v0.2.0 tag)

### Fixed

- `crates/corpus`: migrated to the rand 0.10 API (`RngCore` is no longer exported at the crate root; `fill_bytes` resolves via `rand::Rng`)

### Verified

- `./scripts/verify.sh` full gate suite green on Linux host (12/12 gates, including `trace_calibration` with valgrind 3.27.1 provisioned under `third_party/valgrind`)
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features --locked`

## [0.2.0] — 2026-07-17

### Added

- Portable readiness scripts: `bootstrap.sh`, `check_prerequisites.sh`, `verify.sh`, focused runners
- JSON schemas: `finding-v1`, `experiment-v1`, `trace-v1` with `scripts/validate_schemas.py`
- Dockerfiles for clean-environment and Valgrind-extended verification
- Documentation suite under `docs/`
- CI: locked tests, target build smoke, differential/minimizer smoke, schema + shell syntax, compiler matrix
- Manual/scheduled `extended-trace.yml` and tag `release.yml`
- Dependabot for Cargo and GitHub Actions
- Upstream mutation-lab evidence for `schnorrsig_sign_custom` (verified test-coverage gap)
- Published readiness and upstream study summaries

### Changed

- `.gitignore` excludes generated raw logs, binaries, Valgrind builds, and upstream checkouts
- Untracked historical verification raw logs that embedded machine-local absolute paths
- README rewritten for independent reproduction
- Coverage matrix upgraded after mutation lab

### Verified

- `TABLE_ADDRESS_DETECTION_VERIFIED` (synthetic Lackey static load / cache-line divergence with equal insn sets)
- `./scripts/verify.sh` full gate suite on Linux host
- Draft upstream test PR for `sign_custom` ctime coverage

## [0.1.0] — 2026-07-17

### Added

- BigUint reference model and structured corpus
- Correct / corrupted / leaky C harnesses + minimizer
- Valgrind Lackey dynamic tracing with region markers
- Synthetic calibration of branch and table-address detection
- Pinned libsecp256k1 baseline (no production vulnerability found)
