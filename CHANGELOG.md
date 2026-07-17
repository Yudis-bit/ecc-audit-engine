# Changelog

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
