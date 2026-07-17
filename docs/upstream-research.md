# Upstream research workflow

## Repositories

| Role | URL |
|------|-----|
| Engine | https://github.com/Yudis-bit/ecc-audit-engine |
| Fork | https://github.com/Yudis-bit/secp256k1 |
| Official | https://github.com/bitcoin-core/secp256k1 |

Historical pin used by v0.1.0 experiments:

`11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53`

## Worktree discipline

Keep separate trees:

- `secp256k1-upstream-clean` — unmodified official
- `secp256k1-mutation-lab` — local-only mutations (never push)
- `secp256k1-contribution` — minimal test-only patches

## Gap standard

A **VERIFIED_TEST_COVERAGE_GAP** requires all of:

1. operation processes secret material;
2. current official ctime suite does not test the relevant path/property;
3. planted secret-dependent regression survives current tests;
4. proposed minimal test detects the regression;
5. clean upstream remains green;
6. control mutations remain green;
7. no production behavior changes required;
8. reproducible across ≥2 clean runs;
9. patch is small and reviewable.

Otherwise:

- `NO_GAP`
- `POSSIBLE_GAP`
- `INCONCLUSIVE`
- `UPSTREAM_PATCH_NOT_JUSTIFIED`

## PR policy

Do **not** open an upstream PR without a verified gap.
Do **not** push mutation code to a review branch.
Do **not** claim a real production vulnerability from synthetic mutations.
