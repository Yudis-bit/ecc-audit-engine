# Dynamic Trace Calibration Report

## Backend

- **Valgrind 3.22.0** Lackey (`third_party/valgrind`, built from source)
- Events: superblocks, guest instructions, loads/stores (`--trace-superblocks=yes --trace-mem=yes`)
- Region markers: `ecc_trace_region_begin` / `ecc_trace_region_end` (outside crypto)
- **Not** target leak callbacks

## Results

| Fixture | insn seq equal | load set equal | Detected? |
|---------|----------------|----------------|-----------|
| planted branch | False | n/a | **True** (insn/sb diverge) |
| planted table | True | False | **True** (load set diverge) |
| control | True | True | clean |

### Branch detail
- first_insn_div index: 26
- n_insn even/odd: 29/33
- target sha256: `9990b1b298abd549c77605d35855f6f2ddf060c16cba339fd6f7997466dd3035`

### Table detail
- load_jaccard: 0.75
- target sha256: `5e9ee8a0480caff5322a2fe57cc8e3c8c5b4d67bf7828ea97d8e8812d3546dbc`

### Control detail
- full insn sequence equal and load sets equal under LSB 0 vs 1

## Classification

Synthetic findings: **Level 2** dynamic divergence (calibration fixtures only).

Raw: `reports/dynamic-trace/raw/calib-{branch,table,control}/`
