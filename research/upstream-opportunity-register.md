# Upstream opportunity register

Living register of legitimate contribution candidates for `bitcoin-core/secp256k1` and related integration paths.

Statuses: `REJECTED_NO_GAP` | `NEEDS_EVIDENCE` | `VERIFIED_TEST_GAP` | `VERIFIED_CORRECTNESS_GAP` | `VERIFIED_PORTABILITY_GAP` | `VERIFIED_DOCUMENTATION_GAP` | `PRIVATE_SECURITY_CANDIDATE` | `READY_FOR_PATCH`

## Active / completed

### UOP-001 — ctime coverage for `secp256k1_schnorrsig_sign_custom`

| Field | Value |
|-------|--------|
| Repository | bitcoin-core/secp256k1 |
| Subsystem | tests / CHECKMEM |
| File | `src/ctime_tests.c` |
| Current behavior | `sign32` covered; `sign_custom` not called under CHECKMEM |
| Evidence | Mutation lab: secret branch/table unique to `sign_custom` missed by old test; detected by +11-line call |
| Why it matters | Public secret-bearing entry with extraparams path should be exercised under CT harness |
| Proposed change | Direct `sign_custom(..., NULL)` with undefined secrets |
| Production impact | None (test-only) |
| Review complexity | Low |
| Status | **READY_FOR_PATCH** → PR [#1893](https://github.com/bitcoin-core/secp256k1/pull/1893) draft |

### UOP-002 — custom nonce callback CHECKMEM for `sign_custom`

| Field | Value |
|-------|--------|
| Repository | bitcoin-core/secp256k1 |
| Subsystem | tests / schnorrsig |
| Current behavior | Minimal PR covers default nonce only |
| Evidence | Source review of `extraparams.noncefp` path |
| Why it matters | Unique control flow vs `sign32` |
| Production impact | None if test-only |
| Status | **NEEDS_EVIDENCE** (follow-up after #1893) |

### UOP-003 — secret-argument naming consistency (#1191 / open #1875)

| Field | Value |
|-------|--------|
| Repository | bitcoin-core/secp256k1 |
| Subsystem | API docs / headers |
| Status | **NEEDS_EVIDENCE** — open PR #1875 exists; prefer review over competing PR |

### UOP-004 — minimum toolchain documentation (#1874)

| Field | Value |
|-------|--------|
| Repository | bitcoin-core/secp256k1 |
| Status | **NEEDS_EVIDENCE** — open PR by hebasto; review candidate |

### UOP-005 — Bitcoin Core subtree import of #1893

| Field | Value |
|-------|--------|
| Repository | bitcoin/bitcoin |
| Status | **REJECTED_NO_GAP** as self-import; wait for maintainer subtree practice after secp merge |

## Policy

- Prefer test gaps with mutation evidence over cosmetic edits.
- Prefer reviewing existing open PRs over opening competing ones.
- No self-serving Bitcoin Core subtree import PRs.
