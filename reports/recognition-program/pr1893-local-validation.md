# PR #1893 local validation (2026-07-17)

## Identity

- Author: Yudistira Putra <85178972+Yudis-bit@users.noreply.github.com>
- Commit: c64c477b095d9b4c9acdf625875d6f9d81def57e
- Base: upstream/master 11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53 (not behind)
- Diff: src/ctime_tests.c +11 only

## Official tests (GCC 13.3, CMake VALGRIND=ON)

| Suite | Result |
|-------|--------|
| `tests` | PASS (~77s) |
| `ctime_tests` under Valgrind run 1 | PASS (ERROR SUMMARY 0) |
| `ctime_tests` under Valgrind run 2 | PASS (ERROR SUMMARY 0) |

## Mutation re-proof (local lab only, not on PR branch)

| Setup | Result |
|-------|--------|
| Mut A + old ctime | miss (ERROR SUMMARY 0) |
| Mut A + new ctime (same patch as PR) | detect (exit 42; conditional jump on uninit) |

## Ready-for-review gate

- [x] current upstream base
- [x] clean test-only diff
- [x] author identity correct
- [x] local official tests green
- [x] mutation calibration reproduced
- [x] PR body accurate (no promotional claims)
- [ ] upstream CI fully green (in progress / unstable while queued)
- [ ] no maintainer review feedback yet

**Decision:** keep **draft** until full CI completes without failure.
