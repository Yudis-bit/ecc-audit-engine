# Mutation comparison (bounded laboratory)

## Method

Full disposable upstream worktrees with planted production mutations were **not**
executed end-to-end in this publication run because host lacks CMake and a full
Autotools bootstrap for a clean `ctime_tests` binary rebuild.

Instead:

1. **Engine calibration fixtures** (`calib-branch`, `calib-table`, `calib-control`) under Valgrind Lackey demonstrate detector capability.
2. **Source audit** of upstream `src/ctime_tests.c` maps which secret APIs are already exercised.
3. Decision: no **VERIFIED_TEST_COVERAGE_GAP** without a planted regression that *passes* upstream ctime and *fails* only a new test.

## Engine detector results (synthetic)

| Fixture | Upstream ctime N/A | Engine control-flow | Engine effective-address | Control clean |
|---------|-------------------|--------------------|--------------------------|---------------|
| Planted branch | n/a | **detected** (insn/SB diverge) | n/a | yes |
| Planted table | n/a | insn seq may match | **detected** (load set jaccard 0.75) | yes |
| Control | n/a | no false diverge | no false diverge | yes |

## Upstream expectation (source-level)

Upstream `ctime_tests` uses Valgrind memcheck + `SECP256K1_CHECKMEM_*` to treat
secret buffers as uninitialized and flag secret-dependent branches/addresses in
covered APIs. A naive secret branch inside a covered path is expected to fail
upstream ctime when built with Valgrind support.

Without running a planted mutation under official `ctime_tests`, we **do not**
claim an upstream miss.

## Classification

**UPSTREAM_PATCH_NOT_JUSTIFIED**
