# Upstream constant-time coverage matrix

Source: `src/ctime_tests.c` at pin `11dad6d0` (also matches current origin/master tip at fetch time).

| Operation | In ctime_tests | Classification | Note |
|----------|----------------|----------------|------|
| `secp256k1_ec_pubkey_create` | True | NO_GAP | ctime_tests run_tests |
| `secp256k1_ecdsa_sign` | True | NO_GAP | ctime_tests |
| `secp256k1_ec_seckey_tweak_add` | True | NO_GAP | ctime_tests |
| `secp256k1_ec_seckey_tweak_mul` | True | NO_GAP | ctime_tests |
| `secp256k1_keypair_create` | True | NO_GAP | ctime_tests + extrakeys |
| `secp256k1_keypair_xonly_tweak_add` | True | NO_GAP | ctime_tests |
| `secp256k1_schnorrsig_sign32` | True | NO_GAP | ctime_tests |
| `secp256k1_schnorrsig_sign_custom` | False | POSSIBLE_GAP | not called in ctime_tests.c |
| `secp256k1_ecdh` | True | NO_GAP | ctime_tests |
| `secp256k1_ecdsa_sign_recoverable` | True | NO_GAP | ctime_tests + recovery |
| `secp256k1_ellswift_create` | True | NO_GAP | ctime_tests |
| `secp256k1_ellswift_xdh` | True | NO_GAP | ctime_tests |
| `secp256k1_musig_partial_sign` | True | NO_GAP | ctime_tests + musig |

## Decision

Core secret-bearing public APIs used in production (pubkey_create, ECDSA sign, ECDH, seckey tweaks,
keypair, schnorrsig_sign32, musig partial_sign, ellswift) are already exercised under Valgrind memcheck
uninitialized-secret instrumentation in upstream `ctime_tests`.

`schnorrsig_sign_custom` is not explicitly called; however it is a thin customization path over the same
signing core already covered by `schnorrsig_sign32`. Without a demonstrated planted-regression miss that
survives existing tests while being caught only by a new test, this is **POSSIBLE_GAP** at most — not
**VERIFIED_TEST_COVERAGE_GAP**.

**UPSTREAM_PATCH_NOT_JUSTIFIED** for a public PR at this time.

Standalone engine remains valuable for: effective-address/table-class comparison, differential arithmetic,
and multi-build public-API oracles — complementary, not a replacement for upstream ctime_tests.