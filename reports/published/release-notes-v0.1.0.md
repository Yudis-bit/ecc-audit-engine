# v0.1.0 — Verified Prototype

First public release of **ecc-audit-engine**: a reproducible secp256k1 differential-testing and dynamic-trace research engine.

## Highlights

- BigUint reference model and structured corpus
- Correct / corrupted / leaky C harnesses + minimizer
- Valgrind Lackey dynamic tracing with region markers
- Synthetic calibration of branch and table-address detection
- Pinned libsecp256k1 baseline (no production vulnerability found)

## Install

See README for bootstrap, tests, and experiment commands.

## Security

Synthetic keys only. See SECURITY.md.
