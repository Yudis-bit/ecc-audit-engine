# Upstream constant-time coverage matrix

Source: `src/ctime_tests.c` at pin `11dad6d0` (matches upstream/master HEAD at study time).

| Operation | In ctime_tests (direct) | Classification | Note |
|----------|-------------------------|----------------|------|
| `secp256k1_ec_pubkey_create` | True | NO_GAP | ctime_tests run_tests |
| `secp256k1_ecdsa_sign` | True | NO_GAP | ctime_tests |
| `secp256k1_ec_seckey_tweak_add` | True | NO_GAP | ctime_tests |
| `secp256k1_ec_seckey_tweak_mul` | True | NO_GAP | ctime_tests |
| `secp256k1_keypair_create` | True | NO_GAP | ctime_tests + extrakeys |
| `secp256k1_keypair_xonly_tweak_add` | True | NO_GAP | ctime_tests |
| `secp256k1_schnorrsig_sign32` | True | NO_GAP | ctime_tests |
| `secp256k1_schnorrsig_sign_custom` | False (pre-patch) | **VERIFIED_TEST_COVERAGE_GAP** | Mutation lab: planted branch/table in unique path survived old ctime; detected by minimal new call |
| `secp256k1_ecdh` | True | NO_GAP | ctime_tests |
| `secp256k1_ecdsa_sign_recoverable` | True | NO_GAP | ctime_tests + recovery |
| `secp256k1_ellswift_create` | True | NO_GAP | ctime_tests |
| `secp256k1_ellswift_xdh` | True | NO_GAP | ctime_tests |
| `secp256k1_musig_partial_sign` | True | NO_GAP | ctime_tests + musig |

## Decision

Core secret-bearing APIs exercised by current `ctime_tests` remain **NO_GAP** for the CHECKMEM properties those tests target.

`schnorrsig_sign_custom` is **not** merely an unused alias: it is a public entry with extraparams/custom nonce wiring. Direct CHECKMEM coverage was missing; synthetic mutations unique to that entry survived existing ctime tests and were caught by a one-call test addition.

Status for contribution: draft PR justified (test-only).

Custom nonce-callback coverage remains incomplete after the minimal patch (future POSSIBLE_GAP).
