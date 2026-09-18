# Reproducibility

## Fresh machine (host)

```bash
git clone https://github.com/Yudis-bit/ecc-audit-engine.git
cd ecc-audit-engine
./scripts/bootstrap.sh
./scripts/verify.sh
```

`verify.sh` returns nonzero if a mandatory gate fails.

## Prerequisites

Required:

- Rust (see `rust-toolchain.toml`, currently 1.97.0)
- `cargo`, `cc`/`gcc`, `git`, `python3`, `pkg-config`

Optional:

- Valgrind 3.22+ (dynamic trace)
- CMake / Autotools (official upstream build paths)
- Clang (compiler matrix)
- Docker (clean environment)

## Clean container (non-trace base)

```bash
docker build -t ecc-audit-engine:readiness .
docker run --rm ecc-audit-engine:readiness ./scripts/verify.sh
```

For Valgrind calibration inside a container:

```bash
docker build -f Dockerfile.trace -t ecc-audit-engine:trace .
docker run --rm ecc-audit-engine:trace ./scripts/verify.sh
```

Note: some hosts restrict ptrace; if Valgrind fails in-container, run trace campaigns on native Linux.

## What must not be required

- Prebuilt `targets/*.so`
- Absolute paths under a developer home directory
- Vendored Valgrind or full upstream source trees in Git
- Manually prepared hidden state

## Pinning

- Rust: `rust-toolchain.toml`
- Crates: `Cargo.lock` (`--locked`)
- libsecp256k1: `targets-src/SECP256K1_PIN.txt`

## Step-by-Step Walkthrough: From Fixture to Output

### 1. Deterministic Corpus Generation
The engine uses a seeded pseudo-random generator to produce structured secp256k1 scalar and point test cases (including boundary cases like `0`, `1`, `n-1`, off-curve points, invalid prefixes, and random field elements).

```bash
cargo run -p cli -- corpus generate --seed 1337 --output fixtures/corpus-v1.json
```
- **Provenance & Determinism**: Seed `1337` generates exactly 148 test cases with deterministic SHA256 checksum:
  `4af661f6597633433abfa26ca7df3add0c847eecebc85ddac7fd72ea8251bbee`

### 2. Differential Verification Against Target
The engine executes test vectors in parallel against the pure-Rust reference mathematical model (`crates/model`) and target implementations (`.so` / C ABI conforming to `crates/target-api`).

```bash
cargo run -p cli -- differential \
  --target targets/corrupted-target.so \
  --corpus fixtures/corpus-v1.json \
  --case 42 \
  --minimize \
  --output reports/latest
```
- Mismatches are classified as arithmetic discrepancies (`ArithmeticMismatch`), invalid encoding handling, or policy differences.

### 3. Failure Minimization & Reproducer Generation
When a divergence occurs, `crates/minimizer` reduces the scalar bit-width or structure while preserving the failure condition, writing a minimal reproducible test case under `reports/latest/reproducers/`.

### 4. Dynamic Trace & Side-Channel Calibration
Side-channel detectors are validated against three synthetic calibration fixtures:
- `leaky-branch`: Secret-dependent control flow (detects superblock/instruction divergence).
- `leaky-table`: Secret-dependent table lookups (detects static data address/cache-line divergence via Lackey `--trace-mem=yes`).
- `leaky-control`: Constant-time baseline (must produce zero divergence across secret classes).

```bash
# Synthetic calibration suite
cargo run -p cli -- trace --target targets/leaky-branch.so --experiment experiments/leaky-branch.toml
```

### 5. Report Generation & Schema Validation
All execution outputs conform to versioned JSON schemas in `schemas/`:
```bash
python3 scripts/validate_schemas.py
```
Outputs validate against `finding-v1.schema.json`, `experiment-v1.schema.json`, and `trace-v1.schema.json`.

## Methodological Boundaries & Limitations

1. **Synthetic Keys Only**: All experiments use synthetic or publicly known test keys. No production keys or real-world wallet states are involved.
2. **Bounded Negative Trace $\neq$ Universal Constant-Time Proof**: Passing bounded dynamic traces confirms the absence of instruction and cache-line divergence on the specific corpus, compiler, and architecture tested; it does not constitute a mathematical proof of timing invariance across all hardware microarchitectures.
3. **No Upstream Vulnerability Claim**: Published experiments on upstream `libsecp256k1` evaluate test coverage gaps in testing harnesses (such as `secp256k1_schnorrsig_sign_custom`), not exploitable production vulnerabilities.
4. **Environment Isolation**: In-process dylib tests provide limited memory isolation; CI environments run in sandboxed containers.

