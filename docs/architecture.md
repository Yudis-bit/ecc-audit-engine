# Architecture

## Purpose

`ecc-audit-engine` is a **defensive laboratory engine** for:

- auditable secp256k1 reference mathematics;
- deterministic corpus generation;
- differential testing of local C ABI targets;
- failure minimization;
- Valgrind Lackey dynamic instruction and memory-address tracing.

It is not a wallet, not a production crypto library, and not an exploit toolkit.

## Workspace crates

| Crate | Role |
|-------|------|
| `model` | BigUint field/group reference oracle |
| `corpus` | Deterministic structured cases |
| `target-api` | Shared ABI types and finding enums |
| `runner` | `libloading` FFI loader for `.so` targets |
| `differential` | Target vs model comparison |
| `minimizer` | Delta-debug failing inputs |
| `dyntrace` | Lackey parse / normalize / compare |
| `trace` | Optional callback calibration (secondary) |
| `timing` | Noisy wall-clock / RDTSC harness |
| `report` | JSON + Markdown report writers |
| `cli` | `ecc-audit` command entrypoint |

## Harnesses

- `correct-target` — expected-clean mini-C fixture
- `corrupted-target` — planted arithmetic defects
- `leaky-target` / `calib-*` — synthetic branch and table gadgets
- `libsecp256k1-adapter` — public API only, unmodified upstream sources
- `trace-driver` — region markers + driver for Lackey

## Data flow

1. Corpus (seeded) → differential runner → findings / minimizer
2. Calibration target → `ecc-trace-driver` under Lackey → normalized events → class comparison
3. Pinned upstream checkout (local, not vendored) → adapter → bounded baseline

## Safety boundaries

- Synthetic keys only
- Local binaries only
- Upstream production crypto sources are not patched by default
- Large Valgrind trees and upstream checkouts are gitignored
