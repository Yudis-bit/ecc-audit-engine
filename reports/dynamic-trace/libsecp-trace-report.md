# libsecp256k1 Dynamic Trace Report

## Target

- Upstream: `11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53`
- Adapter: `targets/libsecp256k1-adapter-gcc-O2-dbg.so` (markers outside upstream)
- Binary hash: `ae58dd84df25935c8ff2589f2c05bc8c4b9aea6d105fc68db4dd8a97f873aec7`
- Backend: Valgrind 3.22.0 Lackey

## Campaigns (5 pairs each; synthetic keys)

| Campaign | paired insn seq equal | notes |
|----------|----------------------|-------|
| identical-input repeat | True | n_insn=238455 |
| LSB 0 vs 1 | 5/5 | jaccard 1.0 |
| Hamming low vs high | 5/5 | jaccard 1.0 |
| window pattern | 5/5 | jaccard 1.0 |
| random vs random | 5/5 | negative control |

## Verdict

No class-correlated instruction-sequence divergence observed in region-filtered Lackey traces for tested pubkey_create classes (5 pairs each). Not a universal constant-time proof.

### Explicit non-claims

- This is **not** a proof that libsecp256k1 is constant-time on all platforms/compilers/operations.
- Sample size is **5 pairs** (cost of full mem+insn Lackey traces ~275k lines each); larger campaigns remain future work.
- Dynamic taint / full data-flow not implemented in this slice.
- Conditional branch *outcomes* are inferred via instruction-sequence differences; Lackey does not emit explicit taken/not-taken flags.

### Negative result standard (met)

No reproducible class-correlated instruction-sequence or (where measured) static divergence was observed for the tested `pubkey_create` adapter build and corpus.

Raw: `reports/dynamic-trace/raw/libsecp-*/`
