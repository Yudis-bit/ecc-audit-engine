# Independent Prototype Verification Report

**Auditor role:** Principal Cryptographic Software Auditor  
**Subject commit:** `a82e8a406673974c166df2a9d50b552b0fbd243c` (`a82e8a4`)  
**Date:** verification run under `reports/verification-run/`  
**Scope:** synthetic keys, local builds only

## Method

1. Inspected commit tree and sources (did not trust prior report text).
2. `cargo clean`; deleted prior `.so`; rebuilt with `./scripts/build_targets.sh`.
3. Re-ran fmt, clippy `-D warnings`, full workspace tests.
4. Regenerated corpus seed 1337; compared SHA-256 to committed fixture.
5. Differential correct/corrupted; isolation builds per corruption flag.
6. Minimizer replay against correct vs corrupted.
7. Disassembly of leaky targets; multi-seed timing methodology check.
8. Independent Python ECDLP-group arithmetic for 1G/2G/3G/7G/(n−1)G.

## Claim verdicts

| Prior claim | Verdict | Evidence |
|-------------|---------|----------|
| Commit `a82e8a4` exists with prototype tree | **CONFIRMED** | `git rev-parse HEAD` = `a82e8a4…` |
| Workspace builds; fmt/clippy/tests pass | **CONFIRMED** | `raw/fmt.log`, `clippy.log`, `cargo_test.log` exit 0 |
| Reference model tests + known vectors | **CONFIRMED** | model tests + independent Python matches 1G/2G/3G/7G/(n−1)G prefixes |
| Corpus seed 1337 = 148 cases, stable hash | **CONFIRMED** | both SHA-256 `4af661f6597633433abfa26ca7df3add0c847eecebc85ddac7fd72ea8251bbee` |
| Correct target: 148 cases, 0 failures | **CONFIRMED** | `raw/diff_correct.log`; clean rebuild hash `7a570315…` |
| Corrupted target: 14 failures detected | **CONFIRMED** | `raw/diff_corrupted.log`; isolation map 11+1+2=14 |
| Failures map to planted corruptions | **CONFIRMED** | `raw/isolation/summary.txt` |
| All corruptions disabled → 0 failures | **CONFIRMED** | isolation flags `''` |
| Minimizer reproducers replay | **CONFIRMED** | `raw/minimizer_replay.json` 14/14 |
| Synthetic branch detected via counter | **CONFIRMED_WITH_LIMITATIONS** | callback only → `INSTRUMENTED_SYNTHETIC_CALIBRATION` |
| Synthetic table index detected | **CONFIRMED_WITH_LIMITATIONS** | same classification |
| Control does not trip same detectors | **CONFIRMED** | `control_clean=true` |
| Secret branch survives -O0/-O2/-O3 | **CONFIRMED** | `raw/disasm_leaky.txt`: `test BYTE PTR [rdi+0x1f],0x1; je` |
| “Binary taint analysis” | **FALSE** (if claimed) | code is callback/counter only; not ABI taint |
| Timing t≈−2.55 / d≈−0.26 as stable leakage | **MISLEADING** | 5 seeds × 500/class: signs `[-1,-1,1,1,-1]` not stable |
| Control shows no timing signal | **CONFIRMED_WITH_LIMITATIONS** | also unstable noise; not a strong CT proof |
| ~8–10 ms per pubkey is measurement bug / sleep | **FALSE** | no sleep; reject path ~2 µs; full mul ~6–12 ms (naive C double-and-add) |
| Reports generated from real runs | **CONFIRMED** | re-generated under verification-run |
| No real secp256k1 vulnerability claimed | **CONFIRMED** | lab fixtures only |

## Finding-to-feature map (corrupted)

| Flag | Failures (corpus-v1) |
|------|----------------------|
| (none) | 0 |
| `CORRUPT_FE_MUL` | 11 × `ArithmeticMismatch` (carry-class `fe_mul/*`) |
| `CORRUPT_INFINITY_ADD` | 1 × `InfinityMismatch` (`point_add/G+(-G)`) |
| `CORRUPT_SCALAR_BOUNDARY` | 2 × `InfinityMismatch` (`scalar/n`, `point_mul/n*G`) |
| all three | 14 |

Note: `CORRUPT_NEGATIVE_POINT_ADD` is an alias of the infinity path in source (`#if defined(CORRUPT_INFINITY_ADD) \|\| defined(CORRUPT_NEGATIVE_POINT_ADD)`), not a fourth independent defect.

## Timing methodology findings

1. **Dominant cost** is `point_mul` (256 double/add chain in `secp_mini.c`), not FFI (~µs) or JSON.
2. Prior ~ms timings are **plausible wall times**, not fabricated delays (grep found no sleep/busy).
3. **Branch-only** secret path is a single `if (sk[31]&1) counter++` — effect on wall time is **below noise** on this host with 500–5000 samples.
4. Prior single-run `|t|≈2.55` with n=200 is **not robust**; multi-seed direction inconsistent.
5. Honest classification: Level-3 statistical leakage for branch-only **NOT established**. Callback divergence remains Level-2 instrumented calibration.

## Source audit notes

- `mod_sub` handles `a < b` without BigUint underflow — OK.
- Model is pure BigUint; does not call C targets — OK.
- Known vectors are published hex constants in tests — OK.
- FFI validates lengths; `unsafe` limited to libloading + RDTSC with comments — OK.
- Leaky control scans all table lines and uses branchless `lsb * 0` no-op — OK for calibration control (not a production CT claim).
- In-process dylib load remains a limitation (crash isolation incomplete).

## Status after Part A

Prototype functional claims for differential/corruption/minimizer/callback calibration: **independently verified**.

Timing “leakage” statistical claim: **misleading / not reproduced as stable**.

Proceeding to Part B only with corrected interpretation.

---

# Part B — libsecp256k1 baseline

## Upstream pin

- remote: https://github.com/bitcoin-core/secp256k1.git
- commit: `11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53`
- date: 2026-07-16 17:34:50 +0200
- build: direct `cc -shared` of `secp256k1.c` + precomputed tables (no cmake on host)

## Build matrix (GCC only; clang absent)

| Tag | Notes | lib SHA-256 (prefix) |
|-----|-------|----------------------|
| gcc-O2 | baseline | see raw/libsecp_hashes.txt |
| gcc-O3 | | |
| gcc-O2-asm | `-DUSE_ASM_X86_64` | |
| gcc-O2-noasm | default | |
| gcc-O2-verify | `-DVERIFY` | |

Adapter: `harnesses/libsecp256k1-adapter/adapter.c` — public API only.

## Real-target differential (honest classification)

Full corpus 148 vs model: **41 failures**, all explained as **API policy / surface mismatch**, not arithmetic bugs:

- `fe_mul/*` → adapter returns REJECT (no public field mul)
- `scalar/n`, `n+1`, `2^256-1` → seckey_verify REJECT (model reduces mod n for point_mul)
- `point_mul/n*G` → REJECT (not reduce-to-infinity)

**Public-API campaign:**

- 10,000 valid scalars: adapter vs mini C **0 mismatches**
- 205 valid + metamorphic `pubkey(n-k)=−pubkey(k)`: **0 fails**
- build matrix 55 keys × 5 builds: **0 output disagreements**
- policy zero/n/n+1/max: **reject as expected**
- malformed SEC1: **reject**
- 10,000 random malformed: **10,000 reject**

## Trace / timing

- Dynamic address-trace backends (QEMU/DynamoRIO/Valgrind): **not available** → blocked
- Static screen `secp256k1_ec_pubkey_create` @ `0x21aa0`: 8 branches, 8 calls → **Level 1 only**
- Timing LSB classes, 5 seeds × 1000/class: means ~25–26 µs, **direction not stable** → **negative result**

## Real libsecp256k1 findings

**None.** Negative results documented.

