# Experiment methodology

## Differential testing

1. Generate corpus with fixed seed (`1337` for published runs).
2. Load target `.so` via public ABI.
3. Compare target outputs to `crates/model`.
4. Classify mismatches (`ArithmeticMismatch`, policy accept/reject, etc.).
5. Minimize failing inputs when requested.

## Dynamic trace (Valgrind Lackey)

Backend flags:

- `--tool=lackey`
- `--trace-superblocks=yes`
- `--trace-mem=yes` (required for table-address evidence)

Region markers (`ecc_trace_region_begin` / `end`) delimit the measured window.

Normalization:

- module-relative instruction offsets
- static-data load offsets (stack/heap classified out of primary sets)
- 64-byte cache-line IDs for static loads

### Branch calibration

Expect **instruction / superblock** divergence between secret classes.

### Table-address calibration

Expect:

- same or highly similar instruction sequences (when possible)
- **different** static effective addresses / cache lines controlled by synthetic secret index

Instruction-sequence equality alone is **not** accepted as table-address proof.

### Control calibration

Expect no class-correlated instruction or static-load divergence.

## libsecp256k1 baseline

- Pin recorded in `targets-src/SECP256K1_PIN.txt`
- Build unmodified sources
- Public-API adapter only
- Bounded corpus differential
- Policy rejections are not arithmetic bugs

## Required public language

No vulnerability in upstream libsecp256k1 has been confirmed by the
published experiments.

Synthetic corrupted and leaky targets are calibration fixtures used
to validate the engine.

A negative bounded trace result is not a universal proof of
constant-time behavior.
