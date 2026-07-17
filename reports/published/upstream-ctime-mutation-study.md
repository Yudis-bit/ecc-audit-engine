# Upstream ctime mutation study (public summary)

## Decision

**VERIFIED_TEST_COVERAGE_GAP** established for:

`secp256k1_schnorrsig_sign_custom`

A minimal test-only patch is justified (draft PR).

This is **not** a production vulnerability claim against unmodified libsecp256k1.

No vulnerability in upstream libsecp256k1 has been confirmed by the
published experiments.

Synthetic corrupted and leaky targets are calibration fixtures used
to validate the engine.

A negative bounded trace result is not a universal proof of
constant-time behavior.

## Evidence (compact)

| Setup | Official ctime (Valgrind) |
|-------|---------------------------|
| Clean upstream | pass |
| Mut A (secret branch in `sign_custom` only) + old test | pass (miss) |
| Mut B (secret table load in `sign_custom` only) + old test | pass (miss) |
| Mut C (control) + old test | pass |
| Clean + new test | pass (2 runs) |
| Mut A + new test | fail (detected) |
| Mut B + new test | fail (detected) |
| Mut C + new test | pass |

Full lab notes: `reports/upstream-readiness/schnorrsig-sign-custom-mutation.md`

## Scope of proposed patch

- File: `src/ctime_tests.c` only
- Add direct `secp256k1_schnorrsig_sign_custom(..., NULL)` with CHECKMEM-undefined secrets
- No production crypto changes
- No mutation code in the PR

## Remaining gaps (not claimed verified)

- Custom nonce callback path for `sign_custom`
- Effective-address class detection is complementary (engine Lackey), not a substitute for upstream CHECKMEM
