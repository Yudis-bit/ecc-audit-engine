# Mutation lab: `secp256k1_schnorrsig_sign_custom`

**Upstream base:** `11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53` (matches `upstream/master` at experiment time)  
**Build:** CMake + `SECP256K1_VALGRIND=ON` + GCC 13.3  
**Official ctime runner:** `valgrind --error-exitcode=42 ./ctime_tests` (memcheck / CHECKMEM)

## Relationship to `sign32`

- `sign32` is a thin wrapper that calls `secp256k1_schnorrsig_sign_internal` with BIP-340 nonce function and `msglen=32`.
- `sign_custom` is a distinct public entry point: optional `extraparams` (magic, custom `noncefp`, `ndata`) and variable message length, then the same internal signer.
- Existing `ctime_tests.c` calls **`sign32` only**, not `sign_custom`.
- Functional tests under `modules/schnorrsig/tests_impl.h` exercise `sign_custom`, but **not** under Valgrind CHECKMEM secret-undefine instrumentation.

## Mutations (local-only, never pushed)

| ID | Class | Location |
|----|-------|----------|
| A | secret-dependent branch | unique path in `sign_custom` after param parse |
| B | secret-dependent static table load | unique path in `sign_custom` |
| C | constant-time control (no secret branch/address) | same site |

Public signature output intentionally unchanged.

## Results matrix

| Mutation | Existing ctime reaches path? | Existing ctime detects? | New test detects? | Control clean with new test? |
|----------|------------------------------|-------------------------|-------------------|------------------------------|
| A (branch) | No | No (ERROR SUMMARY 0) | Yes (exit 42; conditional jump on uninit) | — |
| B (table addr) | No | No (ERROR SUMMARY 0) | Yes (exit 42; use of uninit value) | — |
| C (control) | No | No | No (ERROR SUMMARY 0) | Yes |
| Clean + new test | N/A | N/A | Pass (2 runs) | Yes |

### Engine (standalone) note

Standalone Lackey table/branch calibration remains verified on synthetic fixtures (`TABLE_ADDRESS_DETECTION_VERIFIED`). Engine was not used as the primary oracle for this upstream memcheck mutation lab.

## Conclusion

**VERIFIED_TEST_COVERAGE_GAP** for direct CHECKMEM coverage of `secp256k1_schnorrsig_sign_custom`.

This is a **test-coverage** finding, not a claim of a production vulnerability in unmodified upstream.

Minimal remediation: call `secp256k1_schnorrsig_sign_custom` from `src/ctime_tests.c` with secret key material undefined under CHECKMEM (default nonce path). Custom nonce-callback coverage remains future work (still POSSIBLE_GAP for callback-specific paths).
