# Dynamic-trace summary (public)

| Field | Value |
|-------|--------|
| Backend | Valgrind 3.22.0 Lackey |
| Markers | `ecc_trace_region_begin` / `ecc_trace_region_end` |
| Classification of synthetic hits | Level 2 dynamic divergence (calibration only) |

## Calibration

| Fixture | Result |
|---------|--------|
| Planted secret branch | Instruction/SB sequence divergence detected |
| Planted secret table | **Load-set / effective-address difference detected** (insn seq may match) |
| Control | No class-correlated insn or load-set divergence |

Table detection is **address-based**, not instruction-sequence equality.

## libsecp256k1 adapter (bounded)

No reproducible input-class-correlated instruction-sequence divergence was observed for the tested adapter build, backend, operation, and sample corpus (5 pairs × LSB / Hamming / window / random-vs-random; identical-input repeat equal).

**Not** a universal constant-time proof.

## Commands

```bash
export PATH="$PWD/third_party/valgrind/bin:$PATH"
cargo run -p cli -- trace-backend verify
# see scripts/run_dynamic_trace.sh and harnesses/trace-driver/
```
