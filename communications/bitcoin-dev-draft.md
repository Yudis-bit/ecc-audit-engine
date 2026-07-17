# Draft: Bitcoin development mailing list

**Status:** `USER_REVIEW_REQUIRED_BEFORE_SEND`  
**Recommendation:** **Do not send** for PR #1893 alone.

## Why not justified yet

PR #1893 is a small, test-only CHECKMEM coverage improvement. That discussion belongs in the GitHub PR review thread, not on the Bitcoin development mailing list.

A mailing-list post would only become appropriate after one of:

- a broader methodology result with reproducible public artifact;
- a coordinated security disclosure post-embargo;
- a design proposal that needs protocol-wide input.

## Placeholder structure (if justified later)

Subject: [libsecp256k1] Reproducible constant-time test coverage methodology (not a vulnerability report)

Body outline:

1. Problem (entry-point coverage under CHECKMEM)
2. Method (mutation calibration, not production claims)
3. Link to engine + pin
4. Bounded results and limitations
5. Concrete question for reviewers

No promotional language. No “legendary” claims. No maintainer tagging.

## Publication gate

`USER_REVIEW_REQUIRED_BEFORE_SEND`
