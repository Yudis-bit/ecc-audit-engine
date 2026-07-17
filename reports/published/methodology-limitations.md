# Methodology limitations

1. Mini-C secp implementation is for harnesses only.
2. Timing (wall-clock/RDTSC) on non-isolated hosts is noisy; multi-seed stability required.
3. Callback leak counters are secondary calibration — not binary taint.
4. Valgrind Lackey without `--trace-mem=yes` does not emit guest instruction lines needed for marker-based region filtering.
5. Dynamic address-trace equality on one backend/arch/compiler/corpus ≠ CT proof.
6. Sample sizes for full mem+insn traces are bounded by cost.
7. Absolute paths in local logs must not be published as credentials.
