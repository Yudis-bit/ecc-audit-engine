# Limitations

- Mini-C field arithmetic is a lab fixture, not production crypto.
- In-process dylib loading provides limited crash isolation.
- Timing harness is host-noise limited; large \|t\| is not key recovery.
- Lackey is not a complete architectural side-channel oracle.
- Bounded samples cannot prove universal constant-time behavior.
- Instruction-sequence equality is not memory-address leakage proof.
- API rejection-policy differences are not arithmetic vulnerabilities.
- Synthetic mutations are not real upstream vulnerabilities.
- CI does not run multi-hour Valgrind campaigns by default.
- Windows/macOS are not mandatory platforms.
