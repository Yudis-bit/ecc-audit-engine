# Upstream patch validation: cover `schnorrsig_sign_custom` in ctime_tests

## Patch

- Branch: `yudis/ctime-cover-schnorrsig-sign-custom`
- File: `src/ctime_tests.c` only (+11 lines)
- No production crypto changes
- No mutation code

## Required proof

| Scenario | Result |
|----------|--------|
| Clean upstream + old test | pass |
| Planted mut A + old test | not detected |
| Planted mut B + old test | not detected |
| Clean upstream + new test | pass (2 runs) |
| Planted mut A + new test | detected (exit 42) |
| Planted mut B + new test | detected (exit 42) |
| Control mut C + new test | pass |
| Full `tests` binary (clean CMake build) | pass (pre-patch baseline; post-patch ctime validated under Valgrind) |

## Commands (CMake)

```bash
cmake -B build -DSECP256K1_VALGRIND=ON -DSECP256K1_BUILD_CTIME_TESTS=ON \
  -DSECP256K1_ENABLE_MODULE_SCHNORRSIG=ON -DSECP256K1_ENABLE_MODULE_EXTRAKEYS=ON \
  -DSECP256K1_ENABLE_MODULE_ECDH=ON -DSECP256K1_ENABLE_MODULE_RECOVERY=ON \
  -DSECP256K1_ENABLE_MODULE_ELLSWIFT=ON -DSECP256K1_ENABLE_MODULE_MUSIG=ON
cmake --build build --target ctime_tests
valgrind --error-exitcode=42 ./build/bin/ctime_tests
```

Host note: libc debug symbols (`libc6-dbg`) required for Valgrind on this Ubuntu/Pop host.

## Classification

VERIFIED_TEST_COVERAGE_GAP — test improvement only.
